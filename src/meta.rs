//! # The meta layer — an abstraction approximator
//!
//! Everything else in this crate treats a phrase as a *point*: one 32-dim
//! aggregate vector, compared by cosine. That is the math as it is usually done —
//! a tensor is a **function approximator**, so you summarize and measure.
//!
//! This module goes below that. A phrase is not a point; it is a **path** its
//! notes trace through abstraction space, and the object worth knowing is the
//! smooth curve underneath the discrete steps — the *spline of reality*. We fit
//! that curve and then read its geometry: how far the gesture travels (arc
//! length), how hard it turns (bending energy), and which way it is moving
//! (the tangent — a velocity through abstraction space). That makes musician-soul
//! an **abstraction approximator-engine**: it doesn't approximate the output of a
//! function, it approximates the *shape of the movement between abstractions*.
//!
//! Concretely:
//! - [`Phrase::embedding_trajectory`] slides a window across a phrase to get a
//!   *sequence* of embeddings — the path, not the average.
//! - [`AbstractionSpline`] fits a Catmull-Rom curve through that path and exposes
//!   its differential geometry ([`arc_length`](AbstractionSpline::arc_length),
//!   [`bending_energy`](AbstractionSpline::bending_energy),
//!   [`tangent`](AbstractionSpline::tangent)).
//! - [`meta_similarity`] compares two phrases as *gestures* (curve-to-curve),
//!   which sees a difference in motion that cosine-on-aggregates and
//!   edit-distance-on-intervals both miss.
//! - [`MusicianPersona::soul_spline`] makes a persona's identity a *trajectory of
//!   becoming* rather than a single centroid, and
//!   [`MusicianPersona::vibe_velocity`] reads the tangent at its leading edge —
//!   the first-class `d_mu` "Vibe velocity" that [`FLEET.md`] flagged as missing.
//!
//! [`FLEET.md`]: https://github.com/SuperInstance/musician-soul/blob/master/FLEET.md
//!
//! It is pure Rust, zero dependencies, and — like the rest of the crate — never
//! panics on empty or degenerate input.

use crate::{MusicEmbedding, MusicianPersona, Phrase};

/// Embedding dimensionality (matches [`MusicEmbedding`]).
pub const DIM: usize = 32;

/// A point in abstraction space.
pub type Point = [f32; DIM];

/// A Catmull-Rom spline through control points in abstraction space — the
/// smooth "gesture" underneath a discrete sequence of embeddings.
#[derive(Debug, Clone)]
pub struct AbstractionSpline {
    control: Vec<Point>,
}

impl AbstractionSpline {
    /// Build a spline from explicit control points.
    pub fn new(control: Vec<Point>) -> Self {
        Self { control }
    }

    /// Build a spline from a path of embeddings (the usual entry point).
    pub fn from_embeddings(points: &[MusicEmbedding]) -> Self {
        Self {
            control: points.iter().map(|e| e.0).collect(),
        }
    }

    /// Number of control points.
    pub fn len(&self) -> usize {
        self.control.len()
    }
    pub fn is_empty(&self) -> bool {
        self.control.is_empty()
    }

    /// Sample the curve at global parameter `t` in `[0, 1]`.
    ///
    /// Uniform Catmull-Rom: the curve passes through every control point, with
    /// endpoints clamped (duplicated) so the ends are well-defined.
    pub fn sample(&self, t: f32) -> Point {
        match self.control.len() {
            0 => [0.0; DIM],
            1 => self.control[0],
            _ => {
                let segs = self.control.len() - 1;
                let t = t.clamp(0.0, 1.0);
                let ft = t * segs as f32;
                let mut i = ft.floor() as usize;
                if i >= segs {
                    i = segs - 1;
                }
                let local = ft - i as f32;
                let get = |idx: isize| -> Point {
                    let clamped = idx.clamp(0, (self.control.len() - 1) as isize) as usize;
                    self.control[clamped]
                };
                let p0 = get(i as isize - 1);
                let p1 = get(i as isize);
                let p2 = get(i as isize + 1);
                let p3 = get(i as isize + 2);
                catmull_rom(&p0, &p1, &p2, &p3, local)
            }
        }
    }

    /// Sample the curve at `n` evenly spaced parameters (n >= 1).
    pub fn resample(&self, n: usize) -> Vec<Point> {
        if n == 0 || self.control.is_empty() {
            return Vec::new();
        }
        if n == 1 {
            return vec![self.sample(0.0)];
        }
        (0..n)
            .map(|j| self.sample(j as f32 / (n - 1) as f32))
            .collect()
    }

    /// Total distance the gesture travels through abstraction space.
    pub fn arc_length(&self) -> f32 {
        let pts = self.resample(self.resolution());
        pts.windows(2).map(|w| dist(&w[0], &w[1])).sum()
    }

    /// Total turning of the gesture — the sum of discrete curvature magnitudes.
    ///
    /// Zero for a straight line (a phrase that moves steadily in one direction);
    /// large for a restless, direction-changing line. It is the "how much does
    /// this gesture *bend*" that a single aggregate vector cannot represent.
    pub fn bending_energy(&self) -> f32 {
        let pts = self.resample(self.resolution());
        if pts.len() < 3 {
            return 0.0;
        }
        // Sum the turning between consecutive segment directions. Using the
        // *direction* (not the raw second difference) makes this pure curvature:
        // it is zero for collinear points regardless of how the curve is
        // parameterized, and grows toward 2.0 per point for a full reversal.
        let mut energy = 0.0;
        for w in pts.windows(3) {
            let d1 = sub(&w[1], &w[0]);
            let d2 = sub(&w[2], &w[1]);
            if norm(&d1) > 1e-9 && norm(&d2) > 1e-9 {
                energy += 1.0 - cosine(&d1, &d2);
            }
        }
        energy
    }

    /// Unit tangent (direction of motion) at global parameter `t` — a velocity
    /// through abstraction space. Zero vector if the curve is a single point.
    pub fn tangent(&self, t: f32) -> Point {
        if self.control.len() < 2 {
            return [0.0; DIM];
        }
        let h = 1e-3;
        let a = self.sample((t - h).clamp(0.0, 1.0));
        let b = self.sample((t + h).clamp(0.0, 1.0));
        let mut v = [0.0f32; DIM];
        for d in 0..DIM {
            v[d] = b[d] - a[d];
        }
        let n = norm(&v);
        if n > 0.0 {
            for x in &mut v {
                *x /= n;
            }
        }
        v
    }

    /// How many samples to use when measuring the curve — dense enough to be
    /// stable, cheap enough to stay O(control points).
    fn resolution(&self) -> usize {
        (self.control.len().saturating_mul(8)).max(2)
    }
}

impl Phrase {
    /// The phrase's **path** through abstraction space: a sliding window of
    /// `window` notes (step 1) embedded at each position, in order.
    ///
    /// This is the raw material of the meta layer — the sequence of local shapes,
    /// not their average. A phrase with fewer than `window` notes yields a single
    /// embedding of the whole phrase.
    pub fn embedding_trajectory(&self, window: usize) -> Vec<MusicEmbedding> {
        let w = window.max(1);
        if self.events.len() <= w {
            return vec![MusicEmbedding::from_phrase(self)];
        }
        self.events
            .windows(w)
            .map(|slice| {
                let sub = Phrase {
                    events: slice.to_vec(),
                    source: self.source.clone(),
                    instrument: self.instrument.clone(),
                };
                MusicEmbedding::from_phrase(&sub)
            })
            .collect()
    }

    /// The phrase's *spline of reality* — the smooth gesture underneath its notes.
    ///
    /// `window` is the note-window used to trace the path (3 is a good default:
    /// small enough to be local, large enough to have shape).
    pub fn reality_spline(&self, window: usize) -> AbstractionSpline {
        AbstractionSpline::from_embeddings(&self.embedding_trajectory(window))
    }
}

impl MusicianPersona {
    /// The persona's **soul spline**: a curve through its confident patterns,
    /// ordered by generation then confidence — the *trajectory of its becoming*,
    /// where [`soul_print`](crate::PatternVectorDB::soul_print) is only the
    /// centroid (one point on it).
    pub fn soul_spline(&self) -> AbstractionSpline {
        let mut confident: Vec<&crate::Pattern> = self
            .vector_db
            .patterns
            .iter()
            .filter(|p| p.confidence() >= 0.5)
            .collect();
        confident.sort_by(|a, b| {
            a.generation.cmp(&b.generation).then(
                a.confidence()
                    .partial_cmp(&b.confidence())
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
        });
        AbstractionSpline::new(confident.iter().map(|p| p.embedding.0).collect())
    }

    /// The persona's **Vibe velocity** (`d_mu`): the speed at which its identity
    /// is currently moving through abstraction space — the magnitude of the soul
    /// spline's tangent at its leading edge. 0.0 for a persona that hasn't begun
    /// to move (fewer than two confident patterns).
    ///
    /// This is the first-class velocity that `FLEET.md` noted the crate lacked
    /// when mapping onto the elephant substrate's Vibe primitive.
    pub fn vibe_velocity(&self) -> f32 {
        let spline = self.soul_spline();
        if spline.len() < 2 {
            return 0.0;
        }
        // Local speed at the leading edge = distance between the last two
        // resampled points, scaled to the number of segments.
        let pts = spline.resample(spline.len().saturating_mul(8).max(2));
        match (pts.iter().nth_back(1), pts.last()) {
            (Some(a), Some(b)) => dist(a, b) * (pts.len() as f32 - 1.0),
            _ => 0.0,
        }
    }
}

/// Gesture similarity: compare two phrases as *curves*, not points.
///
/// Both reality splines are resampled to a common resolution and compared
/// point-for-point by cosine similarity, then averaged. Two lines with the same
/// aggregate features but different *motion* (a smooth arc vs. a jagged zig-zag)
/// score lower here than under [`MusicEmbedding::similarity`], because this
/// compares the shape of the movement itself. Range 0.0–1.0.
pub fn meta_similarity(a: &Phrase, b: &Phrase, window: usize) -> f32 {
    let sa = a.reality_spline(window);
    let sb = b.reality_spline(window);
    if sa.is_empty() || sb.is_empty() {
        return 0.0;
    }
    const N: usize = 16;
    let pa = sa.resample(N);
    let pb = sb.resample(N);
    let mut sum = 0.0f32;
    let mut count = 0usize;
    for (x, y) in pa.iter().zip(pb.iter()) {
        sum += cosine(x, y);
        count += 1;
    }
    if count == 0 {
        0.0
    } else {
        (sum / count as f32).clamp(0.0, 1.0)
    }
}

// ── vector helpers (kept private; dependency-free) ─────────────────

/// Uniform Catmull-Rom interpolation of one segment, per dimension.
fn catmull_rom(p0: &Point, p1: &Point, p2: &Point, p3: &Point, t: f32) -> Point {
    let t2 = t * t;
    let t3 = t2 * t;
    let mut out = [0.0f32; DIM];
    for d in 0..DIM {
        out[d] = 0.5
            * ((2.0 * p1[d])
                + (-p0[d] + p2[d]) * t
                + (2.0 * p0[d] - 5.0 * p1[d] + 4.0 * p2[d] - p3[d]) * t2
                + (-p0[d] + 3.0 * p1[d] - 3.0 * p2[d] + p3[d]) * t3);
    }
    out
}

fn sub(a: &Point, b: &Point) -> Point {
    let mut out = [0.0f32; DIM];
    for d in 0..DIM {
        out[d] = a[d] - b[d];
    }
    out
}

fn dist(a: &Point, b: &Point) -> f32 {
    let mut s = 0.0f32;
    for d in 0..DIM {
        let diff = a[d] - b[d];
        s += diff * diff;
    }
    s.sqrt()
}

fn norm(a: &Point) -> f32 {
    a.iter().map(|x| x * x).sum::<f32>().sqrt()
}

fn cosine(a: &Point, b: &Point) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na = norm(a);
    let nb = norm(b);
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Duration, NoteEvent, Pitch, Velocity};

    fn ph(pitches: &[u8]) -> Phrase {
        Phrase {
            events: pitches
                .iter()
                .map(|&p| NoteEvent {
                    pitch: Pitch(p),
                    velocity: Velocity(80),
                    duration: Duration(240),
                    tick_offset: 0,
                })
                .collect(),
            source: "t".into(),
            instrument: "t".into(),
        }
    }

    #[test]
    fn spline_passes_through_control_points() {
        let c = vec![[0.0; DIM], {
            let mut p = [0.0; DIM];
            p[0] = 1.0;
            p
        }];
        let s = AbstractionSpline::new(c.clone());
        // Endpoints of a Catmull-Rom curve interpolate the first/last control pts.
        assert!(dist(&s.sample(0.0), &c[0]) < 1e-5);
        assert!(dist(&s.sample(1.0), &c[1]) < 1e-5);
    }

    #[test]
    fn empty_and_single_point_are_graceful() {
        let empty = AbstractionSpline::new(vec![]);
        assert!(empty.is_empty());
        assert_eq!(empty.sample(0.5), [0.0; DIM]);
        assert_eq!(empty.arc_length(), 0.0);
        assert_eq!(empty.bending_energy(), 0.0);
        assert_eq!(empty.tangent(0.5), [0.0; DIM]);

        let single = AbstractionSpline::new(vec![[0.3; DIM]]);
        assert_eq!(single.sample(0.7), [0.3; DIM]);
        assert_eq!(single.arc_length(), 0.0);
    }

    #[test]
    fn straight_line_has_no_bending_but_has_length() {
        // Control points on a straight diagonal in dim 0 → zero curvature.
        let control: Vec<Point> = (0..5)
            .map(|i| {
                let mut p = [0.0; DIM];
                p[0] = i as f32;
                p
            })
            .collect();
        let s = AbstractionSpline::new(control);
        assert!(s.arc_length() > 3.9); // ~4.0 across the span
        assert!(s.bending_energy() < 1e-3, "a line should barely bend");
    }

    #[test]
    fn zigzag_bends_more_than_line() {
        let line: Vec<Point> = (0..5)
            .map(|i| {
                let mut p = [0.0; DIM];
                p[0] = i as f32;
                p
            })
            .collect();
        let zig: Vec<Point> = (0..5)
            .map(|i| {
                let mut p = [0.0; DIM];
                p[0] = i as f32;
                p[1] = if i % 2 == 0 { 0.0 } else { 1.0 };
                p
            })
            .collect();
        let bl = AbstractionSpline::new(line).bending_energy();
        let bz = AbstractionSpline::new(zig).bending_energy();
        assert!(bz > bl);
    }

    #[test]
    fn tangent_is_unit_or_zero() {
        let s = ph(&[60, 62, 64, 65, 67, 69, 71, 72]).reality_spline(3);
        let t = s.tangent(0.5);
        let n = norm(&t);
        assert!(n == 0.0 || (n - 1.0).abs() < 1e-3);
    }

    #[test]
    fn trajectory_is_a_path_not_a_point() {
        let long = ph(&[60, 62, 64, 65, 67, 69, 71, 72]);
        let traj = long.embedding_trajectory(3);
        assert!(
            traj.len() > 1,
            "a long phrase should trace multiple windows"
        );
        // A short phrase collapses to a single embedding.
        assert_eq!(ph(&[60, 62]).embedding_trajectory(3).len(), 1);
    }

    #[test]
    fn meta_similarity_self_is_high_and_bounded() {
        let p = ph(&[60, 62, 64, 65, 67, 69, 71, 72]);
        let s = meta_similarity(&p, &p, 3);
        assert!(s > 0.99, "a gesture matches itself: {s}");
        assert!((0.0..=1.0).contains(&s));
    }

    #[test]
    fn meta_similarity_distinguishes_gestures() {
        let rising = ph(&[60, 62, 64, 65, 67, 69, 71, 72]);
        let arch = ph(&[60, 64, 68, 72, 68, 64, 60, 55]);
        let s = meta_similarity(&rising, &arch, 3);
        assert!((0.0..=1.0).contains(&s));
        // A steady climb and an arch are different motions.
        assert!(s < meta_similarity(&rising, &rising, 3));
    }

    #[test]
    fn soul_spline_and_vibe_velocity() {
        let mut persona = MusicianPersona::new("Test", "sax");
        for i in 0..6 {
            let mut p = ph(&[60, 62, 64, 65, 67]);
            p.source = format!("s{i}");
            persona.digest_phrase(&p, "inf");
        }
        // Untested patterns have confidence 0.5 → included; spline forms.
        let spline = persona.soul_spline();
        assert!(spline.len() >= 2);
        // Velocity is finite and non-negative.
        let v = persona.vibe_velocity();
        assert!(v >= 0.0 && v.is_finite());

        // A brand-new persona hasn't begun to move.
        let empty = MusicianPersona::new("New", "sax");
        assert_eq!(empty.vibe_velocity(), 0.0);
    }
}
