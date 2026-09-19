//! Cheap, dependency-free melody-analysis features from music information
//! retrieval (MIR).
//!
//! These are *additive* — they enrich what you can measure about a [`Phrase`]
//! without changing the 32-dim [`MusicEmbedding`] or the jam loop. Each is
//! standard in symbolic-music analysis, computes in O(n), and needs no external
//! crates. They're useful for style discrimination, indexing, and as candidate
//! inputs to a future embedding revision.
//!
//! Grounding (see README "Prior art"):
//! - Parsons contour — Parsons, *The Directory of Tunes and Musical Themes* (1975).
//! - Huron 9-type contour — Huron (1996), *The melodic arch in Western folksongs*.
//! - Pitch-class histogram (chroma) — standard MIR tonal feature (e.g. jSymbolic).
//! - Longuet-Higgins & Lee syncopation — LHL (1984); cf. Sioros & Guedes, ISMIR 2012.
//! - Interval edit distance — melodic similarity via string edit distance.

use crate::Phrase;

/// Huron's nine melodic-contour categories, from the ordering of a phrase's
/// first pitch, the mean of its pitches, and its last pitch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HuronContour {
    /// first < mean < last
    Ascending,
    /// first > mean > last
    Descending,
    /// first < mean > last (an arch)
    ConvexArch,
    /// first > mean < last (a valley)
    ConcaveValley,
    /// first ≈ mean ≈ last
    Horizontal,
    /// first ≈ mean < last
    HorizontalAscending,
    /// first ≈ mean > last
    HorizontalDescending,
    /// first < mean ≈ last
    AscendingHorizontal,
    /// first > mean ≈ last
    DescendingHorizontal,
    /// Fewer than two notes — no contour.
    Undefined,
}

impl Phrase {
    /// Parsons code: melodic contour as a string of `U`/`D`/`R` (up/down/repeat),
    /// prefixed with `*` for the first note (Parsons' convention).
    ///
    /// Octave- and key-invariant — "do-re-mi" and "sol-la-ti" share `*UU`.
    pub fn parsons_code(&self) -> String {
        let mut s = String::new();
        if self.events.is_empty() {
            return s;
        }
        s.push('*');
        for w in self.events.windows(2) {
            let d = w[1].pitch.0 as i16 - w[0].pitch.0 as i16;
            s.push(match d.cmp(&0) {
                std::cmp::Ordering::Greater => 'U',
                std::cmp::Ordering::Less => 'D',
                std::cmp::Ordering::Equal => 'R',
            });
        }
        s
    }

    /// Huron's nine-type contour classification (first vs mean vs last pitch).
    ///
    /// `tol` is the tolerance in semitones for treating two heights as equal
    /// (0.5 is a reasonable default; the mean is fractional).
    pub fn huron_contour(&self) -> HuronContour {
        if self.events.len() < 2 {
            return HuronContour::Undefined;
        }
        let tol = 0.5f32;
        let first = self.events.first().unwrap().pitch.0 as f32;
        let last = self.events.last().unwrap().pitch.0 as f32;
        let mean =
            self.events.iter().map(|e| e.pitch.0 as f32).sum::<f32>() / self.events.len() as f32;

        use std::cmp::Ordering::*;
        let cmp = |a: f32, b: f32| -> std::cmp::Ordering {
            if (a - b).abs() <= tol {
                Equal
            } else if a < b {
                Less
            } else {
                Greater
            }
        };
        match (cmp(first, mean), cmp(mean, last)) {
            (Less, Less) => HuronContour::Ascending,
            (Greater, Greater) => HuronContour::Descending,
            (Less, Greater) => HuronContour::ConvexArch,
            (Greater, Less) => HuronContour::ConcaveValley,
            (Equal, Equal) => HuronContour::Horizontal,
            (Equal, Less) => HuronContour::HorizontalAscending,
            (Equal, Greater) => HuronContour::HorizontalDescending,
            (Less, Equal) => HuronContour::AscendingHorizontal,
            (Greater, Equal) => HuronContour::DescendingHorizontal,
        }
    }

    /// Pitch-class histogram (chroma): the fraction of notes in each of the 12
    /// pitch classes (C, C#, …, B), summing to 1.0 (or all zeros if empty).
    ///
    /// Captures tonal color / modal bias without any model — a diatonic phrase
    /// concentrates in seven classes, a chromatic run spreads across twelve.
    pub fn pitch_class_histogram(&self) -> [f32; 12] {
        let mut counts = [0f32; 12];
        if self.events.is_empty() {
            return counts;
        }
        for e in &self.events {
            counts[e.pitch.note_class() as usize] += 1.0;
        }
        let total = self.events.len() as f32;
        for c in &mut counts {
            *c /= total;
        }
        counts
    }

    /// A Longuet-Higgins & Lee (1984) style syncopation score.
    ///
    /// Onsets are quantized to a 16th-note grid within 4/4 bars (480 ticks per
    /// quarter, so 1920 per bar, 120 per 16th). Each grid slot has a metric
    /// weight (downbeat strongest). A syncopation is a note whose onset is
    /// followed, before the next onset, by an *empty* slot of *greater* metric
    /// weight; its strength is that weight difference. The return is the summed
    /// strength normalized by note count (0.0 = perfectly on-beat).
    ///
    /// Assumptions (documented, not hidden): monophonic, 4/4, 480 TPQN, onsets
    /// derived from cumulative `tick_offset` + `duration`. It's an index in the
    /// LHL tradition, not a claim of bit-exact LHL on a fully-notated score.
    pub fn syncopation(&self) -> f32 {
        if self.events.len() < 2 {
            return 0.0;
        }
        const BAR: u32 = 1920;
        const STEP: u32 = 120; // 16th note
        const SLOTS: usize = (BAR / STEP) as usize; // 16

        // Metric weight per slot in a bar (higher = stronger beat).
        let weight = |slot: usize| -> i32 {
            if slot.is_multiple_of(16) {
                4 // downbeat
            } else if slot.is_multiple_of(8) {
                3 // half-bar
            } else if slot.is_multiple_of(4) {
                2 // quarter
            } else if slot.is_multiple_of(2) {
                1 // eighth
            } else {
                0 // sixteenth
            }
        };

        // Absolute onset ticks: onset_i = sum_{j<=i} tick_offset_j + sum_{j<i} duration_j.
        let mut onset_slots = Vec::with_capacity(self.events.len());
        let mut cursor: u32 = 0;
        let mut prev_dur: u32 = 0;
        for (i, e) in self.events.iter().enumerate() {
            cursor += e.tick_offset + if i == 0 { 0 } else { prev_dur };
            let slot = ((cursor / STEP) as usize) % SLOTS;
            onset_slots.push(slot);
            prev_dur = e.duration.0;
        }

        // Occupancy of grid slots (mod bar) for "is a stronger slot empty?".
        let mut occupied = [false; SLOTS];
        for &s in &onset_slots {
            occupied[s] = true;
        }

        let mut score = 0.0f32;
        for pair in onset_slots.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            // Look at slots strictly between a and b (wrapping) for a stronger empty one.
            let mut slot = (a + 1) % SLOTS;
            let mut best_gain = 0i32;
            while slot != b {
                if !occupied[slot] {
                    best_gain = best_gain.max(weight(slot) - weight(a));
                }
                slot = (slot + 1) % SLOTS;
            }
            if best_gain > 0 {
                score += best_gain as f32;
            }
        }
        score / self.events.len() as f32
    }

    /// Melodic similarity via normalized edit (Levenshtein) distance on the two
    /// phrases' interval sequences: `1.0 - dist / max(len_a, len_b)`.
    ///
    /// Complements [`MusicEmbedding::similarity`](crate::MusicEmbedding::similarity):
    /// cosine compares aggregate feature vectors, this compares the actual
    /// step-by-step shape, so a transposed copy of a line scores ~1.0 while a
    /// same-register but differently-shaped line scores low. Range 0.0–1.0.
    pub fn interval_edit_similarity(&self, other: &Phrase) -> f32 {
        let a = self.intervals();
        let b = other.intervals();
        if a.is_empty() && b.is_empty() {
            return 1.0;
        }
        let max_len = a.len().max(b.len());
        if max_len == 0 {
            return 1.0;
        }
        let dist = levenshtein(&a, &b);
        1.0 - dist as f32 / max_len as f32
    }
}

/// Levenshtein distance between two slices (two-row DP, O(len_a * len_b) time,
/// O(min) space).
fn levenshtein(a: &[i8], b: &[i8]) -> usize {
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];
    for (i, &ai) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, &bj) in b.iter().enumerate() {
            let cost = if ai == bj { 0 } else { 1 };
            curr[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(curr[j] + 1);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

/// Interval bucket count: intervals clamped to [-12, 12] semitones → 25 classes.
const N_BUCKETS: usize = 25;

fn interval_bucket(i: i8) -> usize {
    (i.clamp(-12, 12) as i32 + 12) as usize
}

/// Information-theoretic novelty: the mean per-transition **surprisal** of a
/// probe phrase's melodic intervals under a transition model learned from a
/// reference corpus.
///
/// Builds a first-order model `P(next_interval | prev_interval)` from `reference`
/// (with Laplace/add-one smoothing over 25 interval buckets), then returns the
/// average of `-log2 P(next | prev)` over the probe's consecutive interval pairs.
/// Higher = the probe's moves are less expected given the corpus. This is the
/// cheap, cognitively-grounded alternative to `1 - cosine` the scout recommended
/// (cf. Sioros/Guedes; expectation-violation models of musical novelty).
///
/// Returns 0.0 for a probe with fewer than two intervals (nothing to predict).
/// With an empty reference the model is uniform, so every transition scores
/// `log2(25) ≈ 4.64` bits.
pub fn transition_surprisal(reference: &[Phrase], probe: &Phrase) -> f32 {
    // counts[prev][next], Laplace-smoothed.
    let mut counts = vec![[1u32; N_BUCKETS]; N_BUCKETS];
    for phrase in reference {
        let iv = phrase.intervals();
        for w in iv.windows(2) {
            counts[interval_bucket(w[0])][interval_bucket(w[1])] += 1;
        }
    }
    let probe_iv = probe.intervals();
    if probe_iv.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0f32;
    let mut n = 0u32;
    for w in probe_iv.windows(2) {
        let prev = interval_bucket(w[0]);
        let next = interval_bucket(w[1]);
        let row_sum: u32 = counts[prev].iter().sum();
        let p = counts[prev][next] as f32 / row_sum as f32;
        total += -p.log2();
        n += 1;
    }
    total / n as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Duration, NoteEvent, Pitch, Velocity};

    fn ph(pitches: &[u8]) -> Phrase {
        let events = pitches
            .iter()
            .map(|&p| NoteEvent {
                pitch: Pitch(p),
                velocity: Velocity(80),
                duration: Duration(240),
                tick_offset: 0,
            })
            .collect();
        Phrase {
            events,
            source: "t".into(),
            instrument: "t".into(),
        }
    }

    #[test]
    fn parsons_basic() {
        assert_eq!(ph(&[60, 62, 64, 62, 62]).parsons_code(), "*UUDR");
        assert_eq!(ph(&[]).parsons_code(), "");
        assert_eq!(ph(&[60]).parsons_code(), "*");
    }

    #[test]
    fn parsons_is_transposition_invariant() {
        assert_eq!(
            ph(&[60, 64, 67]).parsons_code(),
            ph(&[48, 52, 55]).parsons_code()
        );
    }

    #[test]
    fn huron_ascending_and_arch() {
        assert_eq!(
            ph(&[60, 62, 64, 66]).huron_contour(),
            HuronContour::Ascending
        );
        assert_eq!(
            ph(&[60, 66, 72, 66, 60]).huron_contour(),
            HuronContour::ConvexArch
        );
        assert_eq!(ph(&[72, 66, 60]).huron_contour(), HuronContour::Descending);
        assert_eq!(
            ph(&[72, 60, 72]).huron_contour(),
            HuronContour::ConcaveValley
        );
        assert_eq!(ph(&[60]).huron_contour(), HuronContour::Undefined);
    }

    #[test]
    fn chroma_sums_to_one() {
        let h = ph(&[60, 62, 64, 60]).pitch_class_histogram();
        let sum: f32 = h.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
        assert!((h[0] - 0.5).abs() < 1e-6); // two C's out of four
        assert!((h[2] - 0.25).abs() < 1e-6); // one D
        assert_eq!(ph(&[]).pitch_class_histogram(), [0.0; 12]);
    }

    #[test]
    fn edit_similarity_identical_and_transposed() {
        let a = ph(&[60, 62, 64, 65]);
        let b = ph(&[67, 69, 71, 72]); // same interval steps, transposed
        assert!((a.interval_edit_similarity(&b) - 1.0).abs() < 1e-6);
        assert_eq!(a.interval_edit_similarity(&a), 1.0);
        // A very different contour scores lower.
        let c = ph(&[60, 48, 72, 50]);
        assert!(a.interval_edit_similarity(&c) < 0.6);
    }

    #[test]
    fn edit_similarity_empty_cases() {
        assert_eq!(ph(&[]).interval_edit_similarity(&ph(&[])), 1.0);
        // single-note phrases have no intervals → both empty → 1.0
        assert_eq!(ph(&[60]).interval_edit_similarity(&ph(&[72])), 1.0);
    }

    #[test]
    fn syncopation_on_beat_is_low_offbeat_is_higher() {
        // On-beat: quarter notes on the grid.
        let on = Phrase {
            events: (0..4)
                .map(|_| NoteEvent {
                    pitch: Pitch(60),
                    velocity: Velocity(80),
                    duration: Duration(480),
                    tick_offset: 0,
                })
                .collect(),
            source: "on".into(),
            instrument: "t".into(),
        };
        // Off-beat: push the first onset to an eighth, leaving the downbeat empty.
        let off = Phrase {
            events: vec![
                NoteEvent {
                    pitch: Pitch(60),
                    velocity: Velocity(80),
                    duration: Duration(240),
                    tick_offset: 240,
                },
                NoteEvent {
                    pitch: Pitch(62),
                    velocity: Velocity(80),
                    duration: Duration(240),
                    tick_offset: 240,
                },
                NoteEvent {
                    pitch: Pitch(64),
                    velocity: Velocity(80),
                    duration: Duration(240),
                    tick_offset: 240,
                },
            ],
            source: "off".into(),
            instrument: "t".into(),
        };
        assert!(off.syncopation() >= on.syncopation());
        assert_eq!(ph(&[60]).syncopation(), 0.0);
    }

    #[test]
    fn transition_surprisal_rewards_the_expected() {
        // A corpus of steady rising steps: intervals are all +2.
        let corpus = vec![ph(&[60, 62, 64, 66, 68]), ph(&[50, 52, 54, 56, 58])];
        // A probe that continues the +2 habit should be LESS surprising than one
        // that leaps around unexpectedly.
        let expected = ph(&[70, 72, 74, 76]); // +2, +2, +2
        let surprising = ph(&[70, 58, 79, 55]); // wild leaps
        let se = transition_surprisal(&corpus, &expected);
        let ss = transition_surprisal(&corpus, &surprising);
        assert!(ss > se, "unexpected moves score higher: {ss} > {se}");
        // Fewer than two intervals → nothing to predict.
        assert_eq!(transition_surprisal(&corpus, &ph(&[60])), 0.0);
        // Empty reference → uniform model, ~log2(25) bits per transition.
        let uni = transition_surprisal(&[], &expected);
        assert!((uni - (N_BUCKETS as f32).log2()).abs() < 1e-4);
    }

    #[test]
    fn levenshtein_sanity() {
        assert_eq!(levenshtein(&[1, 2, 3], &[1, 2, 3]), 0);
        assert_eq!(levenshtein(&[1, 2, 3], &[1, 9, 3]), 1);
        assert_eq!(levenshtein(&[], &[1, 2]), 2);
    }
}
