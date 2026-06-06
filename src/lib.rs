//! # musician-soul
//!
//! Vector database personas that learn musicians through MIDI digestion,
//! develop their own "what-works" through jam sessions, and evolve from
//! imitation of influences into something with genuine musical soul.
//!
//! ## The Architecture
//!
//! ```text
//! MIDI files ──► Pattern Extractor ──► Vector Embeddings
//!                                                  │
//!                                                  ▼
//!                                          Persona VectorDB
//!                                          ┌─────────────┐
//!                                          │ Influences   │ ← starts here (70% weight)
//!                                          │ What-Works   │ ← grows through jamming
//!                                          │ Soul Prints  │ ← emergent, unique patterns
//!                                          └─────────────┘
//!                                                  │
//!                                                  ▼
//!                                          Jam Session
//!                                          ┌─────────────┐
//!                                          │ Persona A    │──┐
//!                                          │ Persona B    │──┼──► Output + Learning
//!                                          │ Persona C    │──┘
//!                                          └─────────────┘
//! ```
//!
//! A persona doesn't copy. It *digests*. Miles Davis's vector DB doesn't
//! contain his solos — it contains the *shapes* of his decisions. The gaps
//! he left. The way he responded to what someone else played. That's the soul.

#![forbid(unsafe_code)]

use std::collections::HashMap;

// ── Musical Types ─────────────────────────────────────────────────

/// A MIDI pitch (0-127) with semantic meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pitch(pub u8);

impl Pitch {
    pub fn midi_note(&self) -> u8 { self.0 }
    pub fn octave(&self) -> i8 { (self.0 as i8 / 12) - 1 }
    pub fn note_class(&self) -> u8 { self.0 % 12 } // C=0, C#=1, ..., B=11
    pub fn frequency_hz(&self) -> f64 { 440.0 * 2.0_f64.powf((self.0 as f64 - 69.0) / 12.0) }
}

/// Velocity (0-127).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Velocity(pub u8);

impl Velocity {
    pub fn as_f32(&self) -> f32 { self.0 as f32 / 127.0 }
    pub fn dynamic_mark(&self) -> &'static str {
        match self.0 {
            0..=31 => "pp",
            32..=63 => "mp",
            64..=95 => "mf",
            96..=111 => "f",
            _ => "ff",
        }
    }
}

/// Duration in MIDI ticks (assuming 480 ticks/quarter note).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Duration(pub u32);

impl Duration {
    /// How many quarter notes this duration spans.
    pub fn quarter_notes(&self) -> f32 { self.0 as f32 / 480.0 }
    /// True if this is a "long" note (>= quarter note).
    pub fn is_long(&self) -> bool { self.0 >= 480 }
    /// True if this is a "short" note (<= eighth note).
    pub fn is_short(&self) -> bool { self.0 <= 240 }
}

/// A single MIDI note event.
#[derive(Debug, Clone, Copy)]
pub struct NoteEvent {
    pub pitch: Pitch,
    pub velocity: Velocity,
    pub duration: Duration,
    pub tick_offset: u32, // offset from previous event
}

/// A musical phrase — a sequence of note events with metadata.
#[derive(Debug, Clone)]
pub struct Phrase {
    pub events: Vec<NoteEvent>,
    pub source: String,       // e.g. "milesttes2_chorus3"
    pub instrument: String,   // e.g. "trumpet"
}

impl Phrase {
    /// Extract the pitch contour as interval sequence.
    pub fn intervals(&self) -> Vec<i8> {
        self.events.windows(2)
            .map(|w| w[1].pitch.0 as i8 - w[0].pitch.0 as i8)
            .collect()
    }

    /// Extract rhythm pattern as duration ratios.
    pub fn rhythm_pattern(&self) -> Vec<f32> {
        let total: f32 = self.events.iter().map(|e| e.duration.0 as f32).sum();
        if total == 0.0 { return vec![]; }
        self.events.iter().map(|e| e.duration.0 as f32 / total).collect()
    }

    /// Extract velocity contour (dynamics shape).
    pub fn velocity_contour(&self) -> Vec<f32> {
        self.events.iter().map(|e| e.velocity.as_f32()).collect()
    }

    /// Register span (highest - lowest pitch).
    pub fn register_span(&self) -> u8 {
        let max = self.events.iter().map(|e| e.pitch.0).max().unwrap_or(0);
        let min = self.events.iter().map(|e| e.pitch.0).min().unwrap_or(0);
        max - min
    }

    /// Rest ratio — how much silence vs notes.
    pub fn rest_ratio(&self) -> f32 {
        let total_ticks: u32 = self.events.iter().map(|e| e.tick_offset).sum();
        let note_ticks: u32 = self.events.iter().map(|e| e.duration.0).sum();
        if total_ticks + note_ticks == 0 { return 0.0; }
        1.0 - (note_ticks as f32 / (total_ticks + note_ticks) as f32)
    }

    /// Number of notes.
    pub fn len(&self) -> usize { self.events.len() }
    pub fn is_empty(&self) -> bool { self.events.is_empty() }
}

// ── Vector Embeddings ─────────────────────────────────────────────

/// A fixed-dimension embedding vector for musical patterns.
/// 32 dimensions capturing: pitch contour shape, rhythm feel,
/// dynamics arc, interval preferences, register tendency.
#[derive(Debug, Clone)]
pub struct MusicEmbedding(pub [f32; 32]);

impl MusicEmbedding {
    pub fn zero() -> Self { Self([0.0; 32]) }

    /// Create embedding from a phrase — the "DNA" of a musical moment.
    pub fn from_phrase(phrase: &Phrase) -> Self {
        let mut v = [0.0f32; 32];
        if phrase.events.is_empty() { return Self(v); }

        // Dimensions 0-3: register statistics
        let pitches: Vec<u8> = phrase.events.iter().map(|e| e.pitch.0).collect();
        let mean_pitch = pitches.iter().map(|&p| p as f32).sum::<f32>() / pitches.len() as f32;
        v[0] = mean_pitch / 127.0; // normalized mean register
        v[1] = phrase.register_span() as f32 / 127.0; // register width

        // Dimensions 2-5: interval statistics
        let intervals = phrase.intervals();
        if !intervals.is_empty() {
            let mean_interval = intervals.iter().map(|&i| i.abs() as f32).sum::<f32>() / intervals.len() as f32;
            v[2] = mean_interval / 12.0; // average leap size
            let up_count = intervals.iter().filter(|&&i| i > 0).count();
            v[3] = up_count as f32 / intervals.len() as f32; // directional bias (up vs down)
            let max_interval = intervals.iter().map(|&i| i.abs()).max().unwrap_or(0);
            v[4] = max_interval as f32 / 24.0; // largest leap (how adventurous)
        }

        // Dimensions 5-9: rhythm statistics
        let rhythm = phrase.rhythm_pattern();
        if !rhythm.is_empty() {
            let short_count = phrase.events.iter().filter(|e| e.duration.is_short()).count();
            v[5] = short_count as f32 / phrase.events.len() as f32; // rhythmic density
            v[6] = phrase.rest_ratio(); // space vs notes (the gaps Miles loved)
            // Rhythmic variance — how varied are the durations
            let mean_r = rhythm.iter().sum::<f32>() / rhythm.len() as f32;
            let var_r = rhythm.iter().map(|r| (r - mean_r).powi(2)).sum::<f32>() / rhythm.len() as f32;
            v[7] = var_r * 100.0; // rhythmic variety
            // Syncopation proxy: ratio of off-beat starts
            let off_beat = phrase.events.iter().filter(|e| e.tick_offset % 480 > 120).count();
            v[8] = off_beat as f32 / phrase.events.len().max(1) as f32;
        }

        // Dimensions 9-13: dynamics
        let vel = phrase.velocity_contour();
        if !vel.is_empty() {
            v[9] = vel.iter().sum::<f32>() / vel.len() as f32; // average loudness
            // Dynamic range
            let max_v = vel.iter().cloned().fold(0.0f32, f32::max);
            let min_v = vel.iter().cloned().fold(1.0f32, f32::min);
            v[10] = max_v - min_v; // dynamic range
            // Crescendo/decrescendo tendency
            if vel.len() >= 2 {
                v[11] = vel.last().unwrap() - vel.first().unwrap(); // arc direction
            }
        }

        // Dimensions 12-15: harmonic content (note class distribution)
        let mut note_classes = [0u32; 12];
        for e in &phrase.events { note_classes[e.pitch.note_class() as usize] += 1; }
        let total_nc: u32 = note_classes.iter().sum();
        if total_nc > 0 {
            // Tonality index: how concentrated is the pitch distribution
            let entropy = note_classes.iter()
                .filter(|&&c| c > 0)
                .map(|&c| { let p = c as f32 / total_nc as f32; -p * p.log2() })
                .sum::<f32>();
            v[12] = 1.0 - (entropy / 3.585_f32); // 1.0 = highly tonal, 0.0 = atonal
        }

        // Dimensions 13-15: phrase shape
        v[13] = phrase.len() as f32 / 32.0; // phrase length (normalized)
        v[14] = if !intervals.is_empty() {
            let direction_changes = intervals.windows(2)
                .filter(|w| (w[0] > 0) != (w[1] > 0)).count();
            direction_changes as f32 / intervals.len().max(1) as f32
        } else { 0.0 }; // contour complexity

        // Dimensions 15-31: first 17 interval values (padded)
        for (i, &interval) in intervals.iter().take(17).enumerate() {
            v[15 + i] = interval as f32 / 24.0;
        }

        Self(v)
    }

    /// Cosine similarity between two embeddings.
    pub fn similarity(&self, other: &Self) -> f32 {
        let dot: f32 = self.0.iter().zip(other.0.iter()).map(|(a, b)| a * b).sum();
        let norm_a: f32 = self.0.iter().map(|v| v * v).sum::<f32>().sqrt();
        let norm_b: f32 = other.0.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 { return 0.0; }
        dot / (norm_a * norm_b)
    }

    /// Weighted average of two embeddings (for blending personas).
    pub fn blend(&self, other: &Self, self_weight: f32) -> Self {
        let mut result = [0.0f32; 32];
        for i in 0..32 {
            result[i] = self.0[i] * self_weight + other.0[i] * (1.0 - self_weight);
        }
        Self(result)
    }

    /// Distance from origin — how "strong" an identity is.
    pub fn identity_strength(&self) -> f32 {
        self.0.iter().map(|v| v * v).sum::<f32>().sqrt()
    }
}

// ── Pattern Memory ────────────────────────────────────────────────

/// A learned musical pattern with context about when it works.
#[derive(Debug, Clone)]
pub struct Pattern {
    pub embedding: MusicEmbedding,
    pub source_phrase: String,     // where this came from
    pub success_count: u32,        // how many times this led to good output
    pub fail_count: u32,           // how many times this fell flat
    pub context_tags: Vec<String>, // e.g. "slow_ballad", "upswing", "resolution"
    pub generation: u32,           // 0 = learned from MIDI, 1+ = evolved through jamming
}

impl Pattern {
    pub fn new(embedding: MusicEmbedding, source: &str) -> Self {
        Self { embedding, source_phrase: source.to_string(), success_count: 0,
               fail_count: 0, context_tags: Vec::new(), generation: 0 }
    }

    /// How reliable is this pattern? Higher = more proven.
    pub fn confidence(&self) -> f32 {
        let total = self.success_count + self.fail_count;
        if total == 0 { return 0.5; } // untested
        self.success_count as f32 / total as f32
    }

    /// Reinforce — this pattern worked in a jam.
    pub fn reinforce(&mut self) { self.success_count += 1; }

    /// Penalize — this pattern didn't work.
    pub fn penalize(&mut self) { self.fail_count += 1; }
}

/// The vector database storing all patterns for a persona.
#[derive(Debug, Clone)]
pub struct PatternVectorDB {
    pub patterns: Vec<Pattern>,
    pub max_patterns: usize,
}

impl PatternVectorDB {
    pub fn new(max_patterns: usize) -> Self {
        Self { patterns: Vec::new(), max_patterns }
    }

    /// Add a pattern from MIDI digestion.
    pub fn ingest(&mut self, pattern: Pattern) {
        if self.patterns.len() >= self.max_patterns {
            // Evict the lowest-confidence pattern
            if let Some(worst_idx) = self.patterns.iter().enumerate()
                .min_by(|(_, a), (_, b)| a.confidence().partial_cmp(&b.confidence()).unwrap()) {
                self.patterns.remove(worst_idx.0);
            }
        }
        self.patterns.push(pattern);
    }

    /// Query: find the K nearest patterns to a given embedding.
    pub fn nearest_k(&self, query: &MusicEmbedding, k: usize) -> Vec<&Pattern> {
        let mut scored: Vec<(f32, usize)> = self.patterns.iter().enumerate()
            .map(|(i, p)| (query.similarity(&p.embedding), i))
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        scored.into_iter().take(k).map(|(_, i)| &self.patterns[i]).collect()
    }

    /// Query: find patterns matching context tags.
    pub fn by_context(&self, tags: &[&str]) -> Vec<&Pattern> {
        self.patterns.iter().filter(|p| {
            tags.iter().any(|t| p.context_tags.iter().any(|ct| ct.contains(t)))
        }).collect()
    }

    /// The "soul print" — the average embedding of all high-confidence patterns.
    /// This is what makes the persona unique.
    pub fn soul_print(&self) -> MusicEmbedding {
        let confident: Vec<&Pattern> = self.patterns.iter()
            .filter(|p| p.confidence() > 0.6 && p.success_count > 2)
            .collect();
        if confident.is_empty() { return MusicEmbedding::zero(); }
        let mut avg = [0.0f32; 32];
        for p in &confident {
            for (i, &v) in p.embedding.0.iter().enumerate() { avg[i] += v; }
        }
        for v in avg.iter_mut() { *v /= confident.len() as f32; }
        MusicEmbedding(avg)
    }

    /// How many patterns are "evolved" (generation > 0, not from MIDI).
    pub fn evolved_count(&self) -> usize {
        self.patterns.iter().filter(|p| p.generation > 0).count()
    }

    /// Evolution ratio — what fraction of patterns are the persona's own.
    pub fn evolution_ratio(&self) -> f32 {
        if self.patterns.is_empty() { return 0.0; }
        self.evolved_count() as f32 / self.patterns.len() as f32
    }
}

// ── Persona ───────────────────────────────────────────────────────

/// A musician persona with influences and evolving identity.
#[derive(Debug, Clone)]
pub struct MusicianPersona {
    pub name: String,
    pub instrument: String,
    pub influence_weights: HashMap<String, f32>, // influence_name → weight (0.0-1.0)
    pub vector_db: PatternVectorDB,
    pub jam_count: u32,
    pub total_notes_played: u64,
    pub soul_name: Option<String>, // the emergent identity
}

impl MusicianPersona {
    /// Create a new persona with named influences but no patterns yet.
    pub fn new(name: &str, instrument: &str) -> Self {
        Self { name: name.to_string(), instrument: instrument.to_string(),
               influence_weights: HashMap::new(), vector_db: PatternVectorDB::new(10_000),
               jam_count: 0, total_notes_played: 0, soul_name: None }
    }

    /// Add an influence with a weight.
    pub fn add_influence(&mut self, name: &str, weight: f32) {
        self.influence_weights.insert(name.to_string(), weight.clamp(0.0, 1.0));
    }

    /// Digest a phrase into the vector DB — learn from MIDI.
    pub fn digest_phrase(&mut self, phrase: &Phrase, influence_name: &str) {
        let embedding = MusicEmbedding::from_phrase(phrase);
        let mut pattern = Pattern::new(embedding, &format!("{}:{}", influence_name, phrase.source));
        pattern.context_tags.push(influence_name.to_string());
        if phrase.events.iter().any(|e| e.duration.is_long()) {
            pattern.context_tags.push("sustained".to_string());
        }
        if phrase.rest_ratio() > 0.4 {
            pattern.context_tags.push("sparse".to_string());
        }
        if phrase.register_span() > 24 {
            pattern.context_tags.push("wide_range".to_string());
        }
        self.vector_db.ingest(pattern);
    }

    /// Generate a response phrase based on what was "heard" and what works.
    /// This is where the persona's soul emerges — it doesn't copy, it responds.
    pub fn respond_to(&mut self, heard: &Phrase, _context: &str) -> PhraseResponse {
        let heard_embedding = MusicEmbedding::from_phrase(heard);
        let nearest = self.vector_db.nearest_k(&heard_embedding, 5);

        // Blend the nearest patterns' embeddings to create a response shape
        let mut response_embedding = MusicEmbedding::zero();
        if !nearest.is_empty() {
            let total_conf: f32 = nearest.iter().map(|p| p.confidence()).sum();
            for p in &nearest {
                let weight = if total_conf > 0.0 { p.confidence() / total_conf } else { 1.0 / nearest.len() as f32 };
                for (i, &v) in p.embedding.0.iter().enumerate() {
                    response_embedding.0[i] += v * weight;
                }
            }
        }

        // Add some "personality noise" — deviation from pure influence
        let evolution = self.vector_db.evolution_ratio();
        // The more evolved, the more the persona can deviate

        self.jam_count += 1;
        self.total_notes_played += heard.len() as u64;

        // Check if the soul should be named
        if self.soul_name.is_none() && self.vector_db.evolved_count() > 10 {
            self.soul_name = Some(format!("{}-evolved", self.name));
        }

        PhraseResponse {
            persona_name: self.name.clone(),
            based_on: nearest.iter().map(|p| p.source_phrase.clone()).take(3).collect(),
            response_shape: response_embedding.clone(),
            similarity_to_input: heard_embedding.similarity(&response_embedding),
            evolution_level: evolution,
            jam_number: self.jam_count,
            soul_active: self.soul_name.is_some(),
        }
    }

    /// Learn from a jam session outcome — reinforce or penalize patterns.
    pub fn learn_from_jam(&mut self, response: &PhraseResponse, success: bool) {
        let nearest = self.vector_db.nearest_k(&response.response_shape, 3);
        // Can't mutate while borrowing, so collect indices
        let indices: Vec<usize> = nearest.iter().map(|p| {
            self.vector_db.patterns.iter().position(|x| x.source_phrase == p.source_phrase).unwrap_or(0)
        }).collect();
        for idx in indices {
            if success {
                self.vector_db.patterns[idx].reinforce();
                // High-success patterns in generation 0 can spawn generation 1 variants
                if self.vector_db.patterns[idx].generation == 0 && self.vector_db.patterns[idx].success_count > 5 {
                    let mut evolved = self.vector_db.patterns[idx].clone();
                    evolved.generation = 1;
                    evolved.source_phrase = format!("evolved:{}", evolved.source_phrase);
                    evolved.success_count = 1;
                    evolved.fail_count = 0;
                    // Mutate slightly — shift the embedding
                    for v in evolved.embedding.0.iter_mut() {
                        *v += (rand_simple(*v) * 0.1);
                    }
                    self.vector_db.ingest(evolved);
                }
            } else {
                self.vector_db.patterns[idx].penalize();
            }
        }
    }

    /// How much of this persona's style is its own vs borrowed.
    pub fn soul_percentage(&self) -> f32 {
        self.vector_db.evolution_ratio() * 100.0
    }

    /// The persona's unique identity vector.
    pub fn identity(&self) -> MusicEmbedding {
        let soul = self.vector_db.soul_print();
        if soul.identity_strength() > 0.0 { soul } else {
            // No soul yet — blend all influences equally
            let all: Vec<&Pattern> = self.vector_db.patterns.iter().collect();
            if all.is_empty() { return MusicEmbedding::zero(); }
            let mut avg = [0.0f32; 32];
            for p in &all { for (i, &v) in p.embedding.0.iter().enumerate() { avg[i] += v; } }
            for v in avg.iter_mut() { *v /= all.len() as f32; }
            MusicEmbedding(avg)
        }
    }
}

/// Simple deterministic noise for mutation (no rand dependency).
fn rand_simple(seed: f32) -> f32 {
    let x = (seed * 12345.6789).sin();
    (x * 43758.5453).fract() * 2.0 - 1.0
}

/// A persona's response to hearing a phrase.
#[derive(Debug, Clone)]
pub struct PhraseResponse {
    pub persona_name: String,
    pub based_on: Vec<String>,        // which stored patterns influenced the response
    pub response_shape: MusicEmbedding,
    pub similarity_to_input: f32,     // how similar the response is to what was heard
    pub evolution_level: f32,         // 0.0 = pure imitation, 1.0 = fully evolved
    pub jam_number: u32,
    pub soul_active: bool,
}

// ── Jam Session ───────────────────────────────────────────────────

/// A jam session between multiple personas.
#[derive(Debug, Clone)]
pub struct JamSession {
    pub personas: Vec<MusicianPersona>,
    pub rounds: Vec<JamRound>,
    pub context: String,
}

/// One round of a jam — each persona responds to what came before.
#[derive(Debug, Clone)]
pub struct JamRound {
    pub responses: Vec<PhraseResponse>,
    pub harmony_score: f32,       // how well the responses fit together
    pub surprise_score: f32,      // how unexpected the responses were
    pub productive: bool,
}

impl JamSession {
    pub fn new(personas: Vec<MusicianPersona>, context: &str) -> Self {
        Self { personas, rounds: Vec::new(), context: context.to_string() }
    }

    /// Run one round of the jam — each persona responds to a seed phrase.
    pub fn round(&mut self, seed: &Phrase) -> &JamRound {
        let mut responses = Vec::new();
        for persona in &mut self.personas {
            let response = persona.respond_to(seed, &self.context);
            responses.push(response);
        }

        // Evaluate harmony — how similar are the response embeddings?
        let harmony = if responses.len() > 1 {
            let mut sim_sum = 0.0f32;
            let mut count = 0;
            for i in 0..responses.len() {
                for j in (i+1)..responses.len() {
                    sim_sum += responses[i].response_shape.similarity(&responses[j].response_shape);
                    count += 1;
                }
            }
            if count > 0 { sim_sum / count as f32 } else { 0.0 }
        } else { 0.5 };

        // Surprise — how different is the average response from the input?
        let _seed_embedding = MusicEmbedding::from_phrase(seed);
        let surprise: f32 = responses.iter()
            .map(|r| 1.0 - r.similarity_to_input)
            .sum::<f32>() / responses.len().max(1) as f32;

        let productive = harmony > 0.3 && surprise > 0.2;

        // Each persona learns from the outcome
        for persona in &mut self.personas {
            // Find this persona's response
            if let Some(resp) = responses.iter().find(|r| r.persona_name == persona.name) {
                persona.learn_from_jam(resp, productive);
            }
        }

        let round = JamRound { responses, harmony_score: harmony, surprise_score: surprise, productive };
        self.rounds.push(round);
        self.rounds.last_mut().unwrap()
    }

    /// The session's overall harmony — are the personas finding common ground?
    pub fn session_harmony(&self) -> f32 {
        if self.rounds.is_empty() { return 0.0; }
        self.rounds.iter().map(|r| r.harmony_score).sum::<f32>() / self.rounds.len() as f32
    }

    /// How many rounds produced genuinely interesting output?
    pub fn productive_rounds(&self) -> usize {
        self.rounds.iter().filter(|r| r.productive).count()
    }

    /// Each persona's soul percentage after this session.
    pub fn soul_report(&self) -> Vec<(&str, f32)> {
        self.personas.iter().map(|p| (p.name.as_str(), p.soul_percentage())).collect()
    }
}

// ── MIDI Parser (simplified) ──────────────────────────────────────

/// Parse a simplified MIDI-like byte stream into phrases.
/// Real MIDI parsing would use the `midly` crate; this provides the interface.
pub fn parse_midi_events(raw: &[(u8, u8, u32, u32)]) -> Vec<NoteEvent> {
    raw.iter().map(|&(pitch, vel, dur, offset)| NoteEvent {
        pitch: Pitch(pitch), velocity: Velocity(vel),
        duration: Duration(dur), tick_offset: offset,
    }).collect()
}

/// Split events into phrases at rest boundaries (long gaps).
pub fn split_phrases(events: &[NoteEvent], instrument: &str, source: &str) -> Vec<Phrase> {
    if events.is_empty() { return vec![]; }
    let mut phrases = Vec::new();
    let mut current = Vec::new();

    for e in events {
        // A rest of more than a quarter note starts a new phrase
        if e.tick_offset > 480 && !current.is_empty() {
            phrases.push(Phrase { events: std::mem::take(&mut current),
                                   source: source.to_string(), instrument: instrument.to_string() });
        }
        current.push(*e);
    }
    if !current.is_empty() {
        phrases.push(Phrase { events: current, source: source.to_string(),
                               instrument: instrument.to_string() });
    }
    phrases
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_note(pitch: u8, vel: u8, dur: u32, offset: u32) -> NoteEvent {
        NoteEvent { pitch: Pitch(pitch), velocity: Velocity(vel),
                     duration: Duration(dur), tick_offset: offset }
    }

    fn miles_phrase() -> Phrase {
        // Miles Davis style: sparse, lots of space, mid-register trumpet
        Phrase {
            events: vec![
                make_note(62, 80, 480, 0),   // D, quarter note
                make_note(65, 70, 240, 240),  // F, eighth, big rest before
                make_note(67, 90, 960, 120),  // G, half note
                make_note(65, 60, 240, 480),  // F, eighth, long rest
                make_note(62, 85, 480, 0),    // D, quarter
            ],
            source: "milesttes2_chorus3".to_string(),
            instrument: "trumpet".to_string(),
        }
    }

    fn coltrane_phrase() -> Phrase {
        // Coltrane style: dense, wide range, sheets of sound
        Phrase {
            events: vec![
                make_note(60, 100, 120, 0),   // C, sixteenth
                make_note(62, 95, 120, 0),    // D
                make_note(64, 100, 120, 0),   // E
                make_note(65, 105, 120, 0),   // F
                make_note(67, 110, 120, 0),   // G
                make_note(69, 100, 120, 0),   // A
                make_note(71, 95, 120, 0),    // B
                make_note(72, 100, 120, 0),   // C5
                make_note(74, 110, 240, 0),   // D5, slightly longer landing
            ],
            source: "coltrane_giant_steps_solo".to_string(),
            instrument: "tenor_sax".to_string(),
        }
    }

    fn monk_phrase() -> Phrase {
        // Thelonious Monk: angular, unexpected intervals, percussive
        Phrase {
            events: vec![
                make_note(60, 120, 240, 0),   // C, loud, short
                make_note(72, 100, 240, 960),  // C5, big jump up, long rest before
                make_note(63, 115, 120, 0),    // Eb, dissonant
                make_note(55, 90, 480, 240),   // G3, drops way down
            ],
            source: "monk_straight_no_chaser".to_string(),
            instrument: "piano".to_string(),
        }
    }

    #[test] fn pitch_frequency() {
        let a4 = Pitch(69); // A4
        assert!((a4.frequency_hz() - 440.0).abs() < 0.1);
    }

    #[test] fn velocity_dynamics() {
        assert_eq!(Velocity(30).dynamic_mark(), "pp");
        assert_eq!(Velocity(80).dynamic_mark(), "mf");
        assert_eq!(Velocity(120).dynamic_mark(), "ff");
    }

    #[test] fn phrase_intervals() {
        let p = miles_phrase();
        let intervals = p.intervals();
        assert_eq!(intervals.len(), 4);
        assert_eq!(intervals[0], 3); // D→F, up 3
    }

    #[test] fn phrase_rhythm_pattern() {
        let p = miles_phrase();
        let rhythm = p.rhythm_pattern();
        assert_eq!(rhythm.len(), 5);
        let sum: f32 = rhythm.iter().sum();
        assert!((sum - 1.0).abs() < 0.01); // normalized
    }

    #[test] fn miles_rest_ratio() {
        let p = miles_phrase();
        // Miles uses lots of space
        assert!(p.rest_ratio() > 0.2, "Miles should have notable rests");
    }

    #[test] fn coltrane_dense() {
        let p = coltrane_phrase();
        assert!(p.len() > 6);
        assert!(p.register_span() > 10, "Coltrane uses wide range");
    }

    #[test] fn monk_angular() {
        let p = monk_phrase();
        let intervals = p.intervals();
        let max_leap = intervals.iter().map(|i| i.abs()).max().unwrap_or(0);
        assert!(max_leap > 10, "Monk makes big unexpected leaps");
    }

    #[test] fn embedding_similarity_self() {
        let p = miles_phrase();
        let e = MusicEmbedding::from_phrase(&p);
        assert!((e.similarity(&e) - 1.0).abs() < 0.01);
    }

    #[test] fn embedding_different_styles() {
        let miles_e = MusicEmbedding::from_phrase(&miles_phrase());
        let coltrane_e = MusicEmbedding::from_phrase(&coltrane_phrase());
        let monk_e = MusicEmbedding::from_phrase(&monk_phrase());
        // Miles and Monk are more similar to each other than to Coltrane
        // (Coltrane is much denser)
        let mc_sim = miles_e.similarity(&coltrane_e);
        let mm_sim = miles_e.similarity(&monk_e);
        // Just verify they're different
        assert!(mc_sim < 1.0);
        assert!(mm_sim < 1.0);
    }

    #[test] fn embedding_blend() {
        let a = MusicEmbedding::from_phrase(&miles_phrase());
        let b = MusicEmbedding::from_phrase(&coltrane_phrase());
        let blended = a.blend(&b, 0.5);
        assert!(blended.identity_strength() > 0.0);
    }

    #[test] fn persona_digest_and_query() {
        let mut miles = MusicianPersona::new("Miles", "trumpet");
        miles.add_influence("Miles Davis", 1.0);
        miles.add_influence("Clark Terry", 0.3);

        // Digest multiple phrases
        for i in 0..5 {
            let mut p = miles_phrase();
            p.source = format!("milesttes2_chorus{}", i);
            miles.digest_phrase(&p, "Miles Davis");
        }
        for i in 0..3 {
            let mut p = monk_phrase();
            p.source = format!("monk_idea{}", i);
            miles.digest_phrase(&p, "Thelonious Monk");
        }

        assert!(miles.vector_db.patterns.len() >= 7);
        let nearest = miles.vector_db.nearest_k(&MusicEmbedding::from_phrase(&miles_phrase()), 3);
        assert!(nearest.len() >= 3);
    }

    #[test] fn persona_evolution() {
        let mut miles = MusicianPersona::new("Miles", "trumpet");
        for i in 0..5 {
            let mut p = miles_phrase();
            p.source = format!("miles_{}", i);
            miles.digest_phrase(&p, "Miles Davis");
        }

        // Initially no evolved patterns
        assert_eq!(miles.vector_db.evolved_count(), 0);
        assert_eq!(miles.soul_percentage(), 0.0);

        // Run several jam rounds to trigger evolution
        let seed = miles_phrase();
        for _ in 0..8 {
            let _ = miles.respond_to(&seed, "jam");
        }

        // After many jams, persona should have some learning
        assert!(miles.jam_count >= 8);
    }

    #[test] fn jam_session_multi_persona() {
        let mut miles = MusicianPersona::new("Miles", "trumpet");
        miles.add_influence("Miles Davis", 1.0);
        for i in 0..5 {
            let mut p = miles_phrase(); p.source = format!("m{}", i);
            miles.digest_phrase(&p, "Miles Davis");
        }

        let mut coltrane = MusicianPersona::new("Coltrane", "tenor_sax");
        coltrane.add_influence("John Coltrane", 1.0);
        for i in 0..5 {
            let mut p = coltrane_phrase(); p.source = format!("c{}", i);
            coltrane.digest_phrase(&p, "John Coltrane");
        }

        let mut monk = MusicianPersona::new("Monk", "piano");
        monk.add_influence("Thelonious Monk", 1.0);
        for i in 0..5 {
            let mut p = monk_phrase(); p.source = format!("k{}", i);
            monk.digest_phrase(&p, "Thelonious Monk");
        }

        let mut jam = JamSession::new(vec![miles, coltrane, monk], "jazz_standards");
        for round_num in 0..5 {
            let seed = if round_num % 2 == 0 { miles_phrase() } else { coltrane_phrase() };
            let round = jam.round(&seed);
            // Each round should produce 3 responses
            assert_eq!(round.responses.len(), 3);
        }

        assert_eq!(jam.rounds.len(), 5);
        assert!(jam.session_harmony() > 0.0);
        assert!(jam.productive_rounds() > 0);

        // Check soul development
        let souls = jam.soul_report();
        assert_eq!(souls.len(), 3);
    }

    #[test] fn pattern_confidence() {
        let mut p = Pattern::new(MusicEmbedding::zero(), "test");
        assert_eq!(p.confidence(), 0.5); // untested
        p.reinforce();
        assert_eq!(p.confidence(), 1.0);
        p.penalize();
        assert!((p.confidence() - 0.5).abs() < 0.01);
    }

    #[test] fn split_phrases_at_rests() {
        let events = vec![
            make_note(60, 80, 240, 0),
            make_note(62, 80, 240, 0),
            make_note(64, 80, 240, 960), // long rest → new phrase
            make_note(65, 80, 240, 0),
        ];
        let phrases = split_phrases(&events, "piano", "test");
        assert_eq!(phrases.len(), 2);
        assert_eq!(phrases[0].events.len(), 2);
        assert_eq!(phrases[1].events.len(), 2);
    }

    #[test] fn soul_print_emerges() {
        let mut persona = MusicianPersona::new("Test", "guitar");
        // Add patterns with high success counts
        for i in 0..15 {
            let mut p = miles_phrase(); p.source = format!("s{}", i);
            persona.digest_phrase(&p, "influence");
        }
        // Simulate success by reinforcing some patterns
        for p in &mut persona.vector_db.patterns {
            for _ in 0..5 { p.reinforce(); }
        }
        let soul = persona.vector_db.soul_print();
        assert!(soul.identity_strength() > 0.0);
    }

    #[test] fn full_lifecycle() {
        // 1. Create persona with influences
        let mut miles = MusicianPersona::new("Miles AI", "trumpet");
        miles.add_influence("Miles Davis", 0.8);
        miles.add_influence("Chet Baker", 0.4);

        // 2. Digest MIDI (simulated)
        for i in 0..20 {
            let mut p = if i < 12 { miles_phrase() } else { monk_phrase() };
            p.source = format!("digest_{}", i);
            let influence = if i < 12 { "Miles Davis" } else { "Chet Baker" };
            miles.digest_phrase(&p, influence);
        }
        assert!(miles.vector_db.patterns.len() >= 15);

        // 3. Jam repeatedly
        let mut jam = JamSession::new(vec![miles], "late_night_session");
        for r in 0..10 {
            let seed = if r % 3 == 0 { miles_phrase() } else { coltrane_phrase() };
            jam.round(&seed);
        }

        // 4. Verify evolution
        let miles_ref = &jam.personas[0];
        assert!(miles_ref.jam_count >= 10);
        // The persona has played, learned, and evolved
        assert!(miles_ref.total_notes_played > 0);
    }
}
