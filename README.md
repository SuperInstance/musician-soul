# musician-soul

Vector-database *personas* that learn musicians by digesting MIDI, develop their
own "what-works" by jamming, and evolve from imitating influences into something
with a genuine musical identity — a **soul**.

> Part of the [SuperInstance](https://github.com/SuperInstance) fleet.
> Pure Rust, zero dependencies, `#![forbid(unsafe_code)]`.

---

## The 60-second model (read this first)

You don't need to read music — or Rust — to get this. Here is the whole idea in
five sentences:

1. A **phrase** is a short run of notes (pitch, loudness, how long, the gaps).
2. Each phrase is boiled down to a **32-number fingerprint** (an *embedding*) —
   things like "how high," "how leapy," "how dense," "how loud," "how much
   silence." Similar-feeling phrases get similar fingerprints.
3. A **persona** (say, a Miles-flavored trumpet) keeps a bag of these
   fingerprints — its **pattern library** — learned from its influences.
4. When personas **jam**, each one answers a phrase by blending the fingerprints
   nearest to what it just heard. The room scores how well the answers fit
   (**harmony**) and how unexpected they are (**surprise**). Answers that work
   get **reinforced**; ones that don't get **penalized**.
5. Patterns that keep working spawn slightly-**mutated** copies the source MIDI
   never contained. When enough of a persona's library is self-made, it stops
   being "Miles-influenced" and becomes *itself*. That drift is the **soul**.

Nothing here plays audio. This crate is the *decision-making layer* — the shapes
behind the notes — not a synthesizer. Want to hear it? See
[Fleet & related work](#fleet--related-work) for the audio neighbors.

## Run it (zero-shot quickstart)

```bash
# 1. See it work end-to-end — a narrated three-persona jam that prints
#    harmony, surprise, and soul emerging. No MIDI files needed.
cargo run --example jam

# 2. Digest a real .mid file (or a synthesized demo clip with no argument).
cargo run --example play_midi -- path/to/solo.mid
cargo run --example play_midi          # no file → built-in demo clip

# 3. Run the test suite (unit, integration, MIDI parser, edge cases).
cargo test

# 4. Read the API docs in your browser.
cargo doc --open
```

`cargo run --example jam` is the fastest way to understand the whole system —
it's a guided tour ([examples/jam.rs](examples/jam.rs)) you can edit and re-run.
The arc is deterministic, so your numbers will match the comments.

## Why this exists

A music AI that copies Miles Davis isn't Miles Davis — it's a photocopy. Real
musical identity isn't about reproducing solos; it's about *digesting*
influences and developing independent taste. This crate implements that process.
MIDI phrases become 32-dimensional vectors capturing pitch contour, rhythm feel,
dynamics arc, interval preferences, and register tendency. Those vectors go into
a per-persona pattern database. When personas jam, they reinforce or penalize
patterns based on harmonic fit and surprise. Over time, the most successful
patterns *mutate* into new patterns that no MIDI file ever contained — that's the
soul.

The architecture mirrors real musical development: start by copying your
influences (generation 0), then, through enough productive jam sessions, develop
patterns that are yours alone (generation 1+).

## Architecture

```text
MIDI files ──► Phrase Extraction ──► MusicEmbedding (32-dim)
                                           │
                                           ▼
                                    PatternVectorDB
                                    ┌─────────────────┐
                                    │ Gen-0 patterns   │ ← from MIDI digestion
                                    │ Gen-1+ patterns  │ ← evolved through jamming
                                    │ Soul Print       │ ← centroid of proven patterns
                                    └─────────────────┘
                                           │
                                           ▼
                                    JamSession
                                    ┌─────────────────┐
                                    │ Persona A        │──┐
                                    │ Persona B        │──┼──► Output + Learning
                                    │ Persona C        │──┘
                                    └─────────────────┘
                                           │
                                    ┌──────┴──────┐
                                    │ harmony > 0.3│
                                    │ surprise > 0.2│
                                    └──────┬──────┘
                                           ▼
                                    Productive? → reinforce patterns
                                    Unproductive? → penalize patterns
                                    Gen-0 with >5 successes → spawn mutated Gen-1
```

### Key types

- **`Pitch`** / **`Velocity`** / **`Duration`** — MIDI primitives with semantic methods
- **`NoteEvent`** — single MIDI note: pitch, velocity, duration, tick offset
- **`Phrase`** — sequence of note events with source/instrument metadata
- **`MusicEmbedding`** — the 32-dim fingerprint (see the table below)
- **`Pattern`** — an embedding + source + confidence tracking + generation counter
- **`PatternVectorDB`** — fixed-capacity vector store with eviction, K-nearest query, soul-print computation
- **`MusicianPersona`** — a musician with influences, a pattern DB, jam tracking, and an emergent soul name
- **`JamSession`** — a multi-persona jam with harmony scoring, surprise measurement, and learning feedback

### Embedding dimensions

| Dim | Feature | Meaning |
|-----|---------|---------|
| 0 | Mean register | Average pitch height |
| 1 | Register span | Range covered |
| 2 | Mean interval | Average leap size |
| 3 | Directional bias | Up vs down tendency |
| 4 | Largest leap | Adventurousness |
| 5 | Rhythmic density | Short-note ratio |
| 6 | Rest ratio | Space between notes |
| 7 | Rhythmic variance | Duration variety |
| 8 | Syncopation proxy | Off-beat ratio |
| 9 | Average loudness | Mean velocity |
| 10 | Dynamic range | Velocity spread |
| 11 | Arc direction | Crescendo vs decrescendo |
| 12 | Tonality index | Pitch-distribution concentration |
| 13 | Phrase length | Normalized note count |
| 14 | Contour complexity | Direction-change frequency |
| 15–31 | Raw intervals | First 17 interval values |

## Usage

```rust
use musician_soul::*;

// Create a persona and give it influences.
let mut miles = MusicianPersona::new("Miles", "trumpet");
miles.add_influence("Miles Davis", 1.0);
miles.add_influence("Clark Terry", 0.3);

// Digest a phrase. (Build events by hand as below, or load a real .mid file
// with the dependency-free `midi` module — see "Ingesting real MIDI".)
let phrase = Phrase {
    events: vec![
        NoteEvent { pitch: Pitch(62), velocity: Velocity(80), duration: Duration(480), tick_offset: 0 },
        NoteEvent { pitch: Pitch(65), velocity: Velocity(70), duration: Duration(240), tick_offset: 240 },
        NoteEvent { pitch: Pitch(67), velocity: Velocity(90), duration: Duration(960), tick_offset: 120 },
    ],
    source: "miles_chorus3".into(),
    instrument: "trumpet".into(),
};
miles.digest_phrase(&phrase, "Miles Davis");

// Query the nearest stored patterns to a phrase.
let query = MusicEmbedding::from_phrase(&phrase);
let nearest = miles.vector_db.nearest_k(&query, 3);
assert!(nearest.len() <= 3);

// Jam with one or more personas.
let mut jam = JamSession::new(vec![miles], "late_night_session");
let round = jam.round(&phrase);
println!("Harmony: {:.2}, Surprise: {:.2}", round.harmony_score, round.surprise_score);

// Inspect soul development.
for (name, soul_pct) in jam.soul_report() {
    println!("{name}: {soul_pct:.1}% own");
}
```

For the full, annotated version — three personas, many rounds, soul emerging —
run [`cargo run --example jam`](examples/jam.rs).

## Two kinds of "soul" (don't let these confuse you)

The system reports identity two different ways, and **they can disagree** — that
disagreement is meaningful, not a bug:

| Method | Question it answers | Rises when… |
|--------|--------------------|-------------|
| `MusicianPersona::soul_percentage()` | How much of the library is **self-generated** (evolved, generation > 0)? | Jams are *surprising* — lots of exploration. |
| `PatternVectorDB::soul_print()` | What is the centroid of patterns that have **reliably worked** (high confidence)? | Patterns *prove out* — consolidation. |

A persona can be 90% "its own" by evolution while its soul print is still empty:
it is exploring faster than any single idea has proven reliable. **Exploration
and consolidation are different phases.** The `jam` example demonstrates both — a
jam that drives `soul_percentage` up, and a "woodshedding" coda where consistent
success finally crystallizes a `soul_print`.

One more property worth knowing: **a persona jamming *alone* on identical
material never evolves** — nothing surprises it, so nothing is reinforced. Soul
emerges from *diversity*. Put different voices in the room.

## API reference

### MIDI primitives
- `Pitch(u8)` — `.midi_note()`, `.octave()`, `.note_class()`, `.frequency_hz()`
- `Velocity(u8)` — `.as_f32()`, `.dynamic_mark()` (pp/mp/mf/f/ff)
- `Duration(u32)` — `.quarter_notes()`, `.is_long()`, `.is_short()`

### Phrase
- `Phrase { events, source, instrument }`
- `.intervals()` — pitch-interval sequence
- `.rhythm_pattern()` — normalized duration ratios
- `.velocity_contour()` — dynamics shape
- `.register_span()` — highest minus lowest pitch
- `.rest_ratio()` — silence vs note ratio
- `.len()` / `.is_empty()`

### MusicEmbedding
- `MusicEmbedding::from_phrase(phrase)` — extract the 32-dim embedding
- `MusicEmbedding::zero()` — zero vector
- `.similarity(other)` — cosine similarity
- `.blend(other, self_weight)` — weighted average
- `.identity_strength()` — L2 norm

### Pattern
- `Pattern::new(embedding, source)` — fresh pattern
- `.confidence()` — success rate (0.5 for untested)
- `.reinforce()` / `.penalize()` — feedback from jams

### PatternVectorDB
- `new(max_patterns)` — fixed-capacity store
- `.ingest(pattern)` — add, evicting the lowest-confidence pattern at capacity
- `.nearest_k(query, k)` — K nearest patterns by cosine similarity
- `.nearest_k_indices(query, k)` — the same, as stable indices into `patterns`
- `.by_context(tags)` — filter by context tags
- `.soul_print()` — centroid of high-confidence patterns
- `.evolved_count()` — number of generation > 0 patterns
- `.evolution_ratio()` — fraction of evolved patterns

### MusicianPersona
- `new(name, instrument)` — fresh persona
- `.add_influence(name, weight)` — weighted influence (clamped 0.0–1.0)
- `.digest_phrase(phrase, influence)` — learn from MIDI
- `.respond_to(phrase, context)` → `PhraseResponse` — generate a response
- `.learn_from_jam(response, success)` — reinforce/penalize the patterns behind a response
- `.soul_percentage()` — self-generated fraction × 100
- `.identity()` — the persona's unique embedding

### JamSession
- `new(personas, context)` — multi-persona jam
- `.round(seed)` → `&JamRound` — one round of jamming
- `.session_harmony()` — average harmony across rounds
- `.productive_rounds()` — count of productive rounds
- `.soul_report()` — each persona's soul percentage

### Free functions
- `parse_midi_events(raw)` → `Vec<NoteEvent>` — from `(pitch, vel, dur, offset)` tuples
- `split_phrases(events, instrument, source)` → `Vec<Phrase>` — split at rest boundaries

### Ingesting real MIDI (`midi` module)
A dependency-free Standard MIDI File parser (formats 0 and 1) so a persona can
digest actual `.mid` files, not just hand-built events. It never panics on bad
input — every path returns `Result<_, midi::MidiError>`.

- `midi::parse_smf(bytes)` → `Result<Vec<NoteEvent>, MidiError>` — one
  time-ordered, monophonic-reduced note stream; timing rescaled to 480 TPQN.
- `midi::phrases_from_smf(bytes, instrument, source)` → `Result<Vec<Phrase>, MidiError>`
  — `parse_smf` + `split_phrases`.

```rust
use musician_soul::midi;

let bytes = std::fs::read("solo.mid")?;
let phrases = midi::phrases_from_smf(&bytes, "trumpet", "solo.mid")?;
println!("{} phrases", phrases.len());
```

Try it: `cargo run --example play_midi -- your_solo.mid`. For the fleet's
tensor-domain MIDI representation, see
[tensor-midi](https://github.com/SuperInstance/tensor-midi) /
[flux-tensor-midi](https://github.com/SuperInstance/flux-tensor-midi).

## Glossary (for readers from other headspaces)

**Musical terms**

- **MIDI** — a note-event format (not audio): "play pitch P, this loud, this long."
- **Pitch / note** — how high or low. MIDI numbers pitches 0–127; 60 = middle C.
- **Velocity** — how hard a note is struck → how loud (0–127).
- **Tick** — MIDI's time unit. Here, 480 ticks = one quarter note.
- **Interval** — the distance between two pitches; the shape of a melody is its interval sequence.
- **Register** — the height band a phrase lives in (low/mid/high).
- **Phrase** — a musical sentence: a short run of notes with breaths (rests) around it.
- **Woodshedding** — jazz slang for private practice; here, consolidating what works.

**System terms**

- **Embedding** — a fixed-length vector of numbers summarizing something so that
  "similar" items sit near each other. Here: 32 numbers per phrase.
- **Cosine similarity** — how aligned two vectors are, from 0 (unrelated) to 1 (identical direction).
- **Pattern** — a stored embedding plus its track record (successes/failures) and generation.
- **Generation** — 0 = learned from MIDI (imitation); 1+ = mutated during jamming (invention).
- **Confidence** — a pattern's success rate; 0.5 when untested.
- **Soul print** — the average (centroid) of a persona's *proven* patterns; a concrete identity vector.

## The Python extractor (optional scaffold)

[`extractor.py`](extractor.py) is a **standalone scaffold**, not part of the Rust
build. It walks a directory for audio files (`.wav`/`.mp3`/`.flac`/`.ogg`) and
emits one human-readable prompt per file — a placeholder for a future
audio→prompt→embedding bridge. The Rust crate today works on MIDI-style note
events, not audio; the extractor is a stub showing where an audio front-end would
plug in. Run it with:

```bash
python3 extractor.py
```

It has no third-party dependencies and prints nothing if there are no audio files
present (the repo ships an empty `src/dummy.wav` only as a walk target).

## The deeper idea

This is a prototype for *emergent musical identity*. The hypothesis: if you give
a system enough influences, a mechanism for testing what works, and a feedback
loop that rewards successful deviation, it will develop something that looks like
artistic taste. The `soul_print()` isn't a metaphor — it's a concrete vector
representing what this persona has independently discovered works. The generation
counter is the key mechanism: Gen-0 is imitation, Gen-1+ is invention, and when a
persona has enough Gen-1+ patterns it "names its soul."

## Fleet & related work

musician-soul is the *decision layer*. These fleet siblings are its real
neighbors (all links are live):

**Music & MIDI**
- [tensor-midi](https://github.com/SuperInstance/tensor-midi) — tensor-based MIDI timing for musical-agent dialogue cadence.
- [flux-tensor-midi](https://github.com/SuperInstance/flux-tensor-midi) — a 4-D tensor representation of MIDI events across six languages ("room musicians").
- [plainsong](https://github.com/SuperInstance/plainsong) — plain-text music notation that compiles to MIDI and embeds in markdown like mermaid.
- [songforge](https://github.com/SuperInstance/songforge) — AI-powered song-cover generation from imperfect source recordings.
- [ACE-Step-1.5](https://github.com/SuperInstance/ACE-Step-1.5) — a music-generation model in the fleet.

**Substrate & ideas it shares**
- [quilt](https://github.com/SuperInstance/quilt) — the fleet's reactive cell runtime; musician-soul's reinforce/penalize loop and soul-print centroid mirror quilt's cell-feedback and aggregation model. See [FLEET.md](FLEET.md) for the mapping.
- [elephant](https://github.com/SuperInstance/elephant) — the fleet's substrate of "dials" and the Vibe/energy primitive that a persona's evolving taste echoes.
- [Scrapcraft](https://github.com/SuperInstance/Scrapcraft) — a sibling that turns the same "digest → decide → evolve, and explain why" pattern into a game children play.

See **[FLEET.md](FLEET.md)** for how musician-soul maps onto the fleet's cell
model, and [AGENT.md](AGENT.md) for this repo's steward.

## Contributing & license

See [CONTRIBUTING.md](CONTRIBUTING.md). CI enforces `cargo fmt`, `cargo clippy
-- -D warnings`, and `cargo test`. Dual-licensed under
[MIT OR Apache-2.0](LICENSE).
