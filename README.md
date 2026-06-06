# musician-soul

Vector database personas that learn musicians through MIDI digestion, develop their own "what-works" through jam sessions, and evolve from imitation into something with genuine musical identity.

## Why This Exists

A music AI that copies Miles Davis isn't Miles Davis — it's a photocopy. Real musical identity isn't about reproducing solos; it's about *digesting* influences and developing independent taste. This crate implements that process: MIDI files get parsed into phrases, phrases get embedded into 32-dimensional vectors capturing pitch contour, rhythm feel, dynamics arc, interval preferences, and register tendency. Those vectors go into a per-persona pattern database. When personas jam together, they reinforce or penalize patterns based on harmonic fit and surprise. Over time, the most successful patterns *mutate* into new generation-1 patterns that no MIDI file ever contained — that's the soul.

The architecture mirrors real musical development: start by copying your influences (generation 0), then through enough productive jam sessions, develop patterns that are yours alone (generation 1+). The `soul_print()` is the centroid of high-confidence patterns — the mathematical signature of what makes this persona unique.

## Architecture

```text
MIDI files ──► Phrase Extraction ──► MusicEmbedding (32-dim)
                                           │
                                           ▼
                                    PatternVectorDB
                                    ┌─────────────────┐
                                    │ Gen-0 patterns   │ ← from MIDI digestion
                                    │ Gen-1+ patterns  │ ← evolved through jamming
                                    │ Soul Print       │ ← centroid of confident patterns
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

### Key Types

- **`Pitch`** / **`Velocity`** / **`Duration`** — MIDI primitives with semantic methods
- **`NoteEvent`** — Single MIDI note with pitch, velocity, duration, tick offset
- **`Phrase`** — Sequence of note events with source/instrument metadata
- **`MusicEmbedding`** — 32-dimensional vector: register stats (0-1), interval stats (2-4), rhythm (5-8), dynamics (9-11), tonality (12), phrase shape (13-14), raw intervals (15-31)
- **`Pattern`** — Embedding + source + confidence tracking + generation counter
- **`PatternVectorDB`** — Fixed-capacity vector store with eviction, K-nearest query, soul print computation
- **`MusicianPersona`** — A musician with influences, pattern DB, jam tracking, and emergent soul name
- **`JamSession`** — Multi-persona jam with harmony scoring, surprise measurement, and learning feedback

### Embedding Dimensions

| Dim | Feature | Meaning |
|-----|---------|---------|
| 0 | Mean register | Average pitch height |
| 1 | Register span | Range covered |
| 2 | Mean interval | Average leap size |
| 3 | Directional bias | Up vs down tendency |
| 4 | Largest leap | Adventurousness |
| 5 | Rhythmic density | Short note ratio |
| 6 | Rest ratio | Space between notes |
| 7 | Rhythmic variance | Duration variety |
| 8 | Syncopation proxy | Off-beat ratio |
| 9 | Average loudness | Mean velocity |
| 10 | Dynamic range | Velocity spread |
| 11 | Arc direction | Crescendo vs decrescendo |
| 12 | Tonality index | Pitch distribution concentration |
| 13 | Phrase length | Normalized note count |
| 14 | Contour complexity | Direction change frequency |
| 15-31 | Raw intervals | First 17 interval values |

## Usage

```rust
use musician_soul::*;

// Create a persona
let mut miles = MusicianPersona::new("Miles", "trumpet");
miles.add_influence("Miles Davis", 1.0);
miles.add_influence("Clark Terry", 0.3);

// Digest MIDI phrases (simulated — real MIDI uses the midly crate)
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

// Query nearest patterns
let query = MusicEmbedding::from_phrase(&phrase);
let nearest = miles.vector_db.nearest_k(&query, 3);

// Jam session with multiple personas
let mut jam = JamSession::new(vec![miles], "late_night_session");
let round = jam.round(&phrase);
assert!(round.responses.len() >= 1);
println!("Harmony: {:.2}, Surprise: {:.2}", round.harmony_score, round.surprise_score);

// Check soul development
let report = jam.soul_report();
for (name, soul_pct) in report {
    println!("{}: {:.1}% soul", name, soul_pct);
}
```

## API Reference

### MIDI Primitives
- `Pitch(u8)` — `.midi_note()`, `.octave()`, `.note_class()`, `.frequency_hz()`
- `Velocity(u8)` — `.as_f32()`, `.dynamic_mark()` (pp/mp/mf/f/ff)
- `Duration(u32)` — `.quarter_notes()`, `.is_long()`, `.is_short()`

### Phrase
- `Phrase { events, source, instrument }`
- `.intervals()` — Pitch interval sequence
- `.rhythm_pattern()` — Duration ratios (normalized)
- `.velocity_contour()` — Dynamics shape
- `.register_span()` — Highest minus lowest pitch
- `.rest_ratio()` — Silence vs note ratio

### MusicEmbedding
- `MusicEmbedding::from_phrase(phrase)` — Extract 32-dim embedding
- `MusicEmbedding::zero()` — Zero vector
- `.similarity(other)` — Cosine similarity
- `.blend(other, weight)` — Weighted average
- `.identity_strength()` — L2 norm

### Pattern
- `Pattern::new(embedding, source)` — Fresh pattern
- `.confidence()` — Success rate (0.5 for untested)
- `.reinforce()` / `.penalize()` — Feedback from jams

### PatternVectorDB
- `new(max_patterns)` — Fixed-capacity store
- `.ingest(pattern)` — Add with eviction of lowest-confidence
- `.nearest_k(query, k)` — K-nearest by cosine similarity
- `.by_context(tags)` — Filter by context tags
- `.soul_print()` — Centroid of high-confidence patterns
- `.evolution_ratio()` — Fraction of evolved (gen>0) patterns

### MusicianPersona
- `new(name, instrument)` — Fresh persona
- `.add_influence(name, weight)` — Weighted influence
- `.digest_phrase(phrase, influence)` — Learn from MIDI
- `.respond_to(phrase, context)` → `PhraseResponse` — Generate response
- `.learn_from_jam(response, success)` — Reinforce/penalize
- `.soul_percentage()` — How much is the persona's own vs borrowed
- `.identity()` — The persona's unique embedding

### JamSession
- `new(personas, context)` — Multi-persona jam
- `.round(seed)` → `&JamRound` — One round of jamming
- `.session_harmony()` — Average harmony across rounds
- `.productive_rounds()` — Count of productive rounds
- `.soul_report()` — Each persona's soul percentage

### Parsing
- `parse_midi_events(raw)` → `Vec<NoteEvent>` — From `(pitch, vel, dur, offset)` tuples
- `split_phrases(events, instrument, source)` → `Vec<Phrase>` — Split at rest boundaries

## The Deeper Idea

This is a prototype for *emergent musical identity*. The hypothesis: if you give a system enough influences, a mechanism for testing what works, and a feedback loop that rewards successful deviations, it will develop something that looks like artistic taste. The `soul_print()` isn't a metaphor — it's a concrete vector that represents what this persona has independently discovered works.

The generation counter is the key mechanism. Gen-0 patterns come from MIDI (imitation). Gen-1 patterns come from mutating successful Gen-0 patterns during jams (exploration). When a persona has enough Gen-1+ patterns, it "names its soul" — transitioning from "Miles-influenced" to something genuinely new.

## Related Crates

- [`musician-soul-v2`](../musician-soul-v2) — Adds cross-persona influence graphs, genre emergence, temporal evolution, and call-response chains
- [`ternary-cuda-kernels`](../ternary-cuda-kernels) — GPU-accelerated harmony computation for large jam sessions
- [`position-aware-embed`](../position-aware-embed) — Position-weighted text embedding used in pattern matching
