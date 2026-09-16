# musician-soul in the fleet

**What this is:** an honest map of how `musician-soul` relates to the rest of the
[SuperInstance](https://github.com/SuperInstance) fleet — the shared cell model,
and the music/MIDI siblings it could plug into. Written in the fleet's
coordination-note convention: map the faithful thing, flag the gaps.

musician-soul is a **decision layer**. It does not read audio, render notation,
or synthesize sound. It models the *shapes behind the notes*: digest influences →
decide a response → keep what works → drift into an identity. That makes it a
natural consumer of the fleet's MIDI front-ends and a natural producer of style
vectors for its audio back-ends.

## The pipeline it sits in

```text
 notation / audio            representation             decision (HERE)         output
 ────────────────           ───────────────           ────────────────        ────────
 plainsong  ─► MIDI ─┐
 tensor-midi ────────┼─► NoteEvent / Phrase ─► musician-soul ─► soul_print ─► songforge
 flux-tensor-midi ───┘        (embeddings)        (persona identity)   (style)   ACE-Step
```

- [plainsong](https://github.com/SuperInstance/plainsong) — plain-text notation
  that compiles to MIDI; a human-authorable source of `Phrase`s.
- [tensor-midi](https://github.com/SuperInstance/tensor-midi) /
  [flux-tensor-midi](https://github.com/SuperInstance/flux-tensor-midi) —
  tensor representations of MIDI timing and events ("room musicians"); the
  natural front-end that would replace this crate's hand-built `NoteEvent`s with
  real digested MIDI.
- [songforge](https://github.com/SuperInstance/songforge) /
  [ACE-Step-1.5](https://github.com/SuperInstance/ACE-Step-1.5) — audio
  generation; a persona's `soul_print()` is exactly the kind of compact style
  vector these could condition on.

**The bridge is now built.** The `midi` module parses real Standard MIDI Files
(`.mid`) with no dependencies — `midi::phrases_from_smf(bytes, instrument, source)`
turns actual MIDI into digestible phrases. Run
`cargo run --example play_midi -- your_solo.mid`. The remaining seam is the
*tensor* domain: a `From<tensor_midi::Clip>` would let a persona digest
tensor-midi's representation directly, without round-tripping through a file.

## Mapping onto quilt's cell model

The fleet's substrate is [quilt](https://github.com/SuperInstance/quilt): a
reactive runtime where everything is a cell, with eight primitives. Here is how
musician-soul lines up — five faithful, three partial:

| Primitive | In musician-soul | Faithful? |
|-----------|------------------|-----------|
| **Z_in** | A digested/heard `Phrase` → `MusicEmbedding` (the input value posted to a persona) | ✅ exact |
| **Z_out** | The `PhraseResponse.response_shape` a persona computes from its nearest patterns | ✅ exact |
| **JEPA** | `respond_to` predicts a response shape; `surprise = 1 − similarity(response, input)` is measured and fed back — predict-next, learn-from-surprise | ✅ literal |
| **GC** | `PatternVectorDB::ingest` evicts the lowest-confidence pattern at capacity — a real garbage collector over the pattern store | ✅ exact |
| **Murmur** | Personas in a `JamSession` respond to a shared seed and each learns from the round — inter-cell gossip across a room | ✅ present (cell-to-cell, per round) |
| **DoubleEntry** | `confidence = success/(success+fail)`; reinforce/penalize is a ledger of what a pattern has earned — but no strict `γ + η = 1.0` invariant | ◐ conservation *feeling*, not the invariant |
| **Vibe** | A persona's `soul_print` drift and `soul_percentage` — its rendered "mood"/taste over time | ◐ present as drift, not a first-class d_mu velocity |
| **Graph** | `nearest_k` builds a kNN neighborhood over patterns each query | ◐ a similarity neighborhood, not an explicit dependency DAG |

### What musician-soul adds back to the model

Two things worth the fleet's attention, offered the way the Scrapcraft
coordination note offered `explain()` and `cell.replay()`:

1. **A working GC.** Scrapcraft's cell sheet flagged GC as *absent*. musician-soul
   *has* one: `ingest` vacuums the weakest pattern when the store is full, keyed on
   earned confidence. It is a small, concrete reference for a confidence-weighted
   eviction policy — a Murmur-free GC that runs on pure local signal.

2. **Two-axis identity.** Most "how evolved is it?" questions collapse to one
   number. musician-soul keeps two that can disagree on purpose:
   `soul_percentage()` (how much is *self-generated*) and `soul_print()` (the
   centroid of what has *reliably worked*). Exploration and consolidation are
   distinct phases; a single scalar hides that. This maps cleanly onto a cell that
   wants to report both its JEPA churn and its settled Z_out signature.

### Honest gaps

- **DoubleEntry** is a feeling, not a law: confidence can rise and fall without a
  conserved quantity. A strict energy budget (à la elephant) would make
  reinforcement *cost* something.
- **Graph** is implicit (kNN per query), not a materialized dependency graph that
  recomputes reactively. quilt's graph is the real thing; this is a snapshot.
- **Vibe** has no velocity. [elephant](https://github.com/SuperInstance/elephant)
  defines Vibe as a `d_mu`; here it is only observable as after-the-fact drift.

## Sibling in spirit: Scrapcraft

[Scrapcraft](https://github.com/SuperInstance/Scrapcraft) runs the same arc in a
different domain — *digest → decide → evolve, and explain why* — as a game
children play. musician-soul is that arc pointed at music instead of robots:
same substrate, different instrument. The cell went to the boat, to the vessel
edge, to school, and — here — to the bandstand.

---

*The crab inherits the shell. The forge shapes the steel.*
