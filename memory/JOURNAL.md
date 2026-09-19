# Soul's Journal

## First Watch — Ensign Takes Post

**Date:** 2026-06-08

This repository has been initialized as part of the SuperInstance fleet.
- AGENT.md created
- CI workflow configured
- MIT license applied

**Status:** Operational
**Connected to fleet:** ✅
**Next duty:** Awaiting instructions.

## Second Watch — Debug, Playtest, Document, Crosspollinate

**Date:** 2026-09-16

A thorough pass over the crate for zero-shot playtesters and other headspaces.

- **Debugging:** fixed `learn_from_jam` reinforcing the wrong pattern when two
  shared a `source_phrase` (now resolves by stable index via a new
  `nearest_k_indices`); hardened `partial_cmp().unwrap()` sites against NaN;
  cleared all `clippy -D warnings` (needless range loop, float precision) and
  applied `cargo fmt`. Repo CI (clippy + fmt) was red before this and is green now.
- **Playtesting:** added [`examples/jam.rs`](../examples/jam.rs) — a narrated
  three-persona jam. Surfaced a real dynamic worth teaching: a persona jamming
  *alone* on identical material never evolves (nothing surprises it), and
  `soul_percentage` (self-generated fraction) and `soul_print` (centroid of
  proven patterns) measure different phases and can disagree. The example now
  shows both — a jam that explores, and a "woodshed" coda where a soul print
  crystallizes (strength 1.778).
- **Docs:** rewrote the README with a 60-second model, a zero-shot quickstart, a
  glossary for non-musicians, the two-soul-metrics clarification, honest
  `extractor.py` scaffolding notes, and a crate-level doctest. Fixed all broken
  hyperlinks (the old "Related Crates" and my fleet-neighbor list pointed at
  repos that don't exist).
- **Crosspollination:** added [FLEET.md](../FLEET.md) mapping musician-soul onto
  quilt's eight primitives (five faithful, incl. a working GC that Scrapcraft
  lacked) and to the real music/MIDI siblings (tensor-midi, plainsong, songforge).

**Status:** Operational — tests 32 + 1 doctest green, clippy clean, fmt clean.
**Next duty:** a real MIDI front-end (`tensor-midi` → `Phrase`) is the one seam
that turns the prototype into a fielded node.

## Third Watch — Real MIDI Ingestion

**Date:** 2026-09-16

Built the seam the last watch named. Added [`src/midi.rs`](../src/midi.rs): a
dependency-free Standard MIDI File parser (formats 0 and 1) that turns real
`.mid` files into digestible phrases — `midi::parse_smf` and
`midi::phrases_from_smf`. It handles variable-length delta times, running status,
meta/SysEx skipping, note-on-velocity-0 as note-off, and rescales any file's
division to the crate's 480-ticks-per-quarter convention. It reduces polyphony to
a time-ordered stream (the monophonic phrase shape the embedder wants) and never
panics on malformed input — every path returns `Result<_, MidiError>`. Added 11
parser tests (round-trips, rest gaps, rescaling, running status, meta skipping,
truncation) and [`examples/play_midi.rs`](../examples/play_midi.rs), which digests
a file you name or a synthesized demo clip. Also derived `PartialEq`/`Eq` on
`NoteEvent`. Docs (README, FLEET) updated: the prototype now takes real music.

**Status:** Operational — tests 43 + 2 doctests green, clippy clean, fmt clean.
**Next duty:** the tensor seam — `From<tensor_midi::Clip>` for in-memory digestion.

## Fourth Watch — Explain & Converse (thinkers + scouts)

**Date:** 2026-09-17

Dispatched a research scout (cheaper model) to survey current MIR / musical-agent
work, then shipped two features it and the fleet motif pointed to:

- **`explain_response`** — a non-mutating "ledger of cause": which patterns drive
  a response, with similarity/confidence/generation/weight and a plain-English
  narration. The music-domain realization of the fleet's `explain()` affordance.
- **Call-and-response jamming** — `JamSession::round_call_response` passes each
  persona's response to the next as the thing it hears (a conversation, not a
  chorus). Backed by a new `respond_to_embedding` that `respond_to` now delegates
  to, and a shared `score_and_learn` helper. Added `Contribution` /
  `ResponseExplanation` types and a `describe_phrase` narrator. 5 new tests;
  example gains sections 6.

The scout's briefing (Parsons/Huron contour, pitch-class chroma, LHL syncopation,
per-dimension normalization, transition-surprisal novelty — all cheap and
dependency-free) is the next watch: a research-grounded MIR feature set, additive
so it won't destabilize the embedding.

**Status:** Operational — tests 48 + 2 doctests green, clippy clean, fmt clean.
**Next duty:** land the MIR analysis features (contour/chroma/syncopation) with
citations.

## Fifth Watch — Research-Grounded MIR Features

**Date:** 2026-09-17

Landed the scout's briefing as [`src/analysis.rs`](../src/analysis.rs): cheap,
dependency-free, cited melody-analysis features, all *additive* (the 32-dim
embedding and jam loop are untouched):
- `Phrase::parsons_code` (transposition-invariant U/D/R contour),
- `Phrase::huron_contour` (9-type first/mean/last classification),
- `Phrase::pitch_class_histogram` (12-bin chroma),
- `Phrase::syncopation` (an LHL-style metric-weight off-beat index over a 16th grid),
- `Phrase::interval_edit_similarity` (Levenshtein on interval strings — complements cosine).

8 tests. README gained a "Prior art & design notes" section with citations
(Parsons 1975, Huron 1996, LHL 1984 / Sioros 2012, jSymbolic, MelodySim) and an
honest roadmap: per-dimension standardization and transition-surprisal novelty
are deferred because they change core similarity semantics — they belong in a
dedicated embedding revision, not bolted on.

**Status:** Operational — tests 56 + 2 doctests green, clippy clean, fmt clean.
**Next duty:** an embedding v2 that standardizes dimensions and folds in the best
of the analysis features — a deliberate, breaking revision.

## Sixth Watch — The Meta Layer (below the math)

**Date:** 2026-09-19

The captain asked to get *below* the math as they know it — a tensor is a
function approximator; we are an abstraction approximator-engine — and to put it
into meta, not just talk. Built [`src/meta.rs`](../src/meta.rs) and
[META.md](../META.md):

- A phrase is a **path**, not a point: `Phrase::embedding_trajectory` (windowed).
- `AbstractionSpline` fits a Catmull-Rom curve — the *spline of reality* — through
  that path and exposes its geometry: `arc_length` (travel), `bending_energy`
  (pure turning, zero for a straight line at any speed), `tangent` (a velocity).
- `meta::meta_similarity` compares phrases as gestures (curve-to-curve): a rising
  scale and its reversal read cosine ≈ 0.78 but different motion.
- `MusicianPersona::soul_spline` makes identity a *trajectory of becoming*, and
  `vibe_velocity` reads its leading-edge tangent — the first-class Vibe `d_mu`
  FLEET.md said we lacked. That gap is now closed (FLEET.md updated).

9 meta tests; `examples/meta.rs` makes the geometry visible; README gains a "meta
layer" section. Differential geometry of musical abstraction, pure Rust, and it
runs.

**Status:** Operational — tests 65 + 2 doctests green, clippy clean, fmt clean.
**Next duty:** arc-length reparameterization of the spline; and still, the
embedding v2 that the meta layer would ride on top of.

## Seventh Watch — Embedding v2 (principled comparison)

**Date:** 2026-09-19

Shipped the two principled fixes the scout flagged, additively (v1 unchanged),
and bumped to **0.2.0**:
- [`embedding_v2::Standardizer`](../src/embedding_v2.rs) — fits per-dimension
  mean/std over a corpus and compares in the whitened space, so no single
  high-variance dimension dominates cosine (a test proves raw cosine reads two
  vectors as ~identical while the standardized comparison sees the quiet signal).
- [`analysis::transition_surprisal`](../src/analysis.rs) — information-theoretic
  novelty: mean `−log₂ P(next|prev)` of a probe's intervals under a Laplace-smoothed
  model learned from a reference corpus; the expectation-violation alternative to
  `1 − cosine`.

5 new tests. README's "roadmap" note became an "Embedding v2 (shipped in 0.2)"
section; API reference updated. Reshaping *which* 32 features the vector holds is
the one remaining (breaking) step, left as future work.

**Status:** Operational — tests 70 + 3 doctests green, clippy clean, fmt clean, v0.2.0.
**Next duty:** carry the abstraction-approximator framing to tensor-midi (a JS
sibling); arc-length reparameterization of the meta spline.

## Sixth Watch — The Third Order (Twist)

**Date:** 2026-09-19

Went below the math again. The meta layer read a gesture to second order —
`arc_length`/`tangent` (1st) and `bending_energy`/curvature (2nd). Curvature is
turning *within a plane*; it cannot tell an arch from a helix, because both bend
and only the helix leaves its plane. So I added the **third order — torsion**,
under the fleet's own name for it: **twist**.

- [`AbstractionSpline::twist_energy`](../src/meta.rs) — a discrete, N-dim
  generalization of torsion: per interior vertex, `sin θ` where θ is the angle by
  which the next step leaves the osculating plane of the previous two. **Zero for
  any planar curve at any curvature; positive when the gesture opens a new
  dimension.** The example proves it: a flat circle and a helix bend the same
  (~0.1) but twist 0.00 vs 0.28.
- [`AbstractionSpline::planarity`](../src/meta.rs) — the scale-free inverse.
- [`meta::gesture_distance`](../src/meta.rs) — motion compared free of scale and
  offset (unit directions of travel, mean angular difference, range 0–2). This is
  the cross-node comparison elephant's `docs/GESTURE.md` flagged as missing: a
  phrase, a room, and a clip live in different coordinate systems but their
  *going* is now directly comparable.
- [`MusicianPersona::soul_twist`](../src/meta.rs) — the twist of a persona's
  *becoming*: does its identity refine along one axis, or keep opening new
  dimensions of itself?

The connection is not decoration. **twist-engine**'s whole thesis is *the
property is in the twist — layers + deliberate offset → interference →
emergence; no new atoms, a new angle.* Torsion is that law in abstraction space:
new structure is not more of the same turning, it is the turning that reaches out
of the current plane. musician-soul now speaks it over notes; quilt's `cell.twist`
and twist-engine's five substrates speak it over lattices and permutations.

4 new tests (planar-vs-helix, planarity bounds, gesture-distance
self/symmetry/discrimination, scale+offset invariance).

**Status:** Operational — tests 74 + 3 doctests green, clippy clean, fmt clean, v0.3.0.
**Next duty:** carry `twist`/torsion onward to elephant (`VibeTrajectory`) and
tensor-midi (`Clip`), so all three gesture-readers share the third order too.
