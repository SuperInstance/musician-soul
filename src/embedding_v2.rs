//! Embedding v2 — principled comparison in a standardized space.
//!
//! The v1 [`MusicEmbedding`] packs 32 features of very different scales into one
//! vector: a normalized register (~0–1) sits next to raw intervals and a
//! variance term scaled by 100. Plain cosine over that vector is **dominated by
//! whichever dimensions happen to vary most**, so two phrases can read as
//! "similar" mostly because they agree on one loud dimension. That is the
//! standard heterogeneous-feature pitfall flagged in the README's *Prior art &
//! design notes*.
//!
//! v2 fixes it without changing the 32-dim container (so the [`meta`](crate::meta)
//! layer and everything else keep working): fit a [`Standardizer`] over a corpus
//! of phrases, then compare in the z-scored space where every dimension
//! contributes on equal footing. It is opt-in and additive — v1 similarity is
//! unchanged — but it is the recommended way to compare, and the reason this
//! crate is now 0.2.
//!
//! ```
//! use musician_soul::{MusicEmbedding, Phrase, embedding_v2::Standardizer};
//! # fn corpus() -> Vec<Phrase> { vec![] }
//! let phrases: Vec<Phrase> = corpus();
//! let z = Standardizer::fit_from_phrases(&phrases); // fit once over your material
//! # let (a, b) = (MusicEmbedding::zero(), MusicEmbedding::zero());
//! let sim = z.similarity(&a, &b); // cosine in the whitened space
//! # let _ = sim;
//! ```

use crate::{MusicEmbedding, Phrase};

/// The embedding width v2 standardizes over (matches [`MusicEmbedding`]).
pub const DIM: usize = 32;

/// Per-dimension mean/standard-deviation, fit from a corpus, used to z-score
/// embeddings so no single high-variance dimension dominates cosine.
#[derive(Debug, Clone)]
pub struct Standardizer {
    pub mean: [f32; DIM],
    /// Standard deviation per dimension. Floored away from zero so a constant
    /// dimension neither divides-by-zero nor amplifies noise.
    pub std: [f32; DIM],
}

impl Standardizer {
    /// The no-op standardizer (mean 0, std 1): `apply` returns the input
    /// unchanged, and `similarity` equals plain v1 cosine.
    pub fn identity() -> Self {
        Self {
            mean: [0.0; DIM],
            std: [1.0; DIM],
        }
    }

    /// Fit per-dimension statistics from a set of embeddings (population std).
    ///
    /// An empty corpus yields [`identity`](Self::identity). Dimensions whose
    /// spread is negligible get std = 1.0, so they pass through as a centered
    /// constant rather than being blown up.
    pub fn fit(samples: &[MusicEmbedding]) -> Self {
        if samples.is_empty() {
            return Self::identity();
        }
        let n = samples.len() as f32;
        let mut mean = [0.0f32; DIM];
        for s in samples {
            for (m, &v) in mean.iter_mut().zip(s.0.iter()) {
                *m += v;
            }
        }
        for m in &mut mean {
            *m /= n;
        }
        let mut std = [0.0f32; DIM];
        for s in samples {
            for d in 0..DIM {
                let x = s.0[d] - mean[d];
                std[d] += x * x;
            }
        }
        for sd in &mut std {
            let v = (*sd / n).sqrt();
            *sd = if v < 1e-6 { 1.0 } else { v };
        }
        Self { mean, std }
    }

    /// Fit from phrases directly (embeds each with [`MusicEmbedding::from_phrase`]).
    pub fn fit_from_phrases(phrases: &[Phrase]) -> Self {
        let embeddings: Vec<MusicEmbedding> =
            phrases.iter().map(MusicEmbedding::from_phrase).collect();
        Self::fit(&embeddings)
    }

    /// Z-score an embedding into the standardized space: `(x - mean) / std`.
    pub fn apply(&self, e: &MusicEmbedding) -> MusicEmbedding {
        let mut out = [0.0f32; DIM];
        for (o, ((&x, &m), &sd)) in out
            .iter_mut()
            .zip(e.0.iter().zip(self.mean.iter()).zip(self.std.iter()))
        {
            *o = (x - m) / sd;
        }
        MusicEmbedding(out)
    }

    /// Cosine similarity of two embeddings **in the standardized space** — the
    /// recommended v2 comparison. Every dimension contributes on equal footing.
    pub fn similarity(&self, a: &MusicEmbedding, b: &MusicEmbedding) -> f32 {
        self.apply(a).similarity(&self.apply(b))
    }
}

impl MusicEmbedding {
    /// Return this embedding z-scored by `z` (see [`Standardizer`]). Convenience
    /// for `z.apply(self)`.
    pub fn standardized(&self, z: &Standardizer) -> MusicEmbedding {
        z.apply(self)
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
    fn identity_matches_plain_cosine() {
        let a = MusicEmbedding::from_phrase(&ph(&[60, 62, 64, 65]));
        let b = MusicEmbedding::from_phrase(&ph(&[67, 69, 71, 72]));
        let z = Standardizer::identity();
        assert!((z.similarity(&a, &b) - a.similarity(&b)).abs() < 1e-6);
    }

    #[test]
    fn empty_corpus_is_identity() {
        let z = Standardizer::fit(&[]);
        assert_eq!(z.mean, [0.0; DIM]);
        assert_eq!(z.std, [1.0; DIM]);
    }

    #[test]
    fn constant_dimension_gets_unit_std() {
        // Two embeddings differing only in dim 5; every other dim is constant.
        let mut e1 = [0.0f32; DIM];
        let mut e2 = [0.0f32; DIM];
        e1[0] = 1.0;
        e2[0] = 1.0; // dim 0 constant
        e1[5] = 0.0;
        e2[5] = 2.0; // dim 5 varies
        let z = Standardizer::fit(&[MusicEmbedding(e1), MusicEmbedding(e2)]);
        assert_eq!(z.std[0], 1.0); // constant dim floored, not zero
        assert!(z.std[5] > 0.0);
    }

    #[test]
    fn standardizing_surfaces_a_dominated_dimension() {
        // Craft a corpus where dim 0 is huge-scale and dims 1..3 are the real
        // signal. Under raw cosine, two vectors that agree on the loud dim 0 but
        // differ on the quiet dims look nearly identical; standardization exposes
        // the difference.
        let mk = |big: f32, a: f32, b: f32, c: f32| {
            let mut v = [0.0f32; DIM];
            v[0] = big;
            v[1] = a;
            v[2] = b;
            v[3] = c;
            MusicEmbedding(v)
        };
        // Corpus establishes that dim 0 varies a lot; dims 1..3 vary a little.
        let corpus = vec![
            mk(100.0, 0.0, 0.0, 0.0),
            mk(120.0, 0.1, 0.0, 0.1),
            mk(80.0, 0.0, 0.1, 0.0),
        ];
        let z = Standardizer::fit(&corpus);

        let x = mk(100.0, 0.1, 0.0, 0.0);
        let y = mk(100.0, 0.0, 0.1, 0.0); // same loud dim, opposite quiet signal
        let raw = x.similarity(&y);
        let std = z.similarity(&x, &y);
        assert!(raw > 0.999, "raw cosine is dominated by dim 0: {raw}");
        assert!(
            std < raw,
            "standardized comparison sees the quiet difference: {std} < {raw}"
        );
    }
}
