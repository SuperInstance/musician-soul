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
