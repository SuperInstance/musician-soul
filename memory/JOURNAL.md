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
