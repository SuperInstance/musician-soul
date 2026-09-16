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
