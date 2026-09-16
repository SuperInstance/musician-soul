//! A complete, runnable tour of musician-soul — no MIDI files required.
//!
//! Run it with:
//!
//! ```bash
//! cargo run --example jam
//! ```
//!
//! It walks the whole arc the crate is about:
//!   1. Build three personas, each with named influences.
//!   2. Feed them phrases (here: hand-written, standing in for digested MIDI).
//!   3. Put them in a room together and jam for many rounds.
//!   4. Watch harmony, surprise, and *soul* change over time.
//!
//! Everything printed is computed by the library — nothing is faked for the
//! demo. If you are new to the ideas here, read the "The 60-second model"
//! section of the README first; the terms below (phrase, embedding, soul print)
//! are all defined there.

use musician_soul::*;

/// A helper so the example reads like music, not like array literals.
/// `(pitch, velocity, duration_ticks, gap_before_ticks)`.
fn note(pitch: u8, vel: u8, dur: u32, gap: u32) -> NoteEvent {
    NoteEvent {
        pitch: Pitch(pitch),
        velocity: Velocity(vel),
        duration: Duration(dur),
        tick_offset: gap,
    }
}

/// Miles-flavored: sparse, mid-register, lots of air between notes.
fn miles_phrase(tag: &str) -> Phrase {
    Phrase {
        events: vec![
            note(62, 80, 480, 0),   // D  — a quarter note
            note(65, 70, 240, 240), // F  — an eighth, after a rest
            note(67, 90, 960, 120), // G  — held, half note
            note(65, 60, 240, 480), // F  — eighth, after a long rest
            note(62, 85, 480, 0),   // D  — home
        ],
        source: tag.to_string(),
        instrument: "trumpet".to_string(),
    }
}

/// Coltrane-flavored: dense, wide, a run up the horn ("sheets of sound").
fn coltrane_phrase(tag: &str) -> Phrase {
    Phrase {
        events: vec![
            note(60, 100, 120, 0),
            note(62, 95, 120, 0),
            note(64, 100, 120, 0),
            note(65, 105, 120, 0),
            note(67, 110, 120, 0),
            note(69, 100, 120, 0),
            note(71, 95, 120, 0),
            note(72, 100, 120, 0),
            note(74, 110, 240, 0),
        ],
        source: tag.to_string(),
        instrument: "tenor_sax".to_string(),
    }
}

/// Monk-flavored: angular, big surprising leaps, percussive.
fn monk_phrase(tag: &str) -> Phrase {
    Phrase {
        events: vec![
            note(60, 120, 240, 0),
            note(72, 100, 240, 960), // leap up an octave after silence
            note(63, 115, 120, 0),   // dissonant Eb
            note(55, 90, 480, 240),  // drop way down
        ],
        source: tag.to_string(),
        instrument: "piano".to_string(),
    }
}

fn bar() {
    println!("{}", "─".repeat(66));
}

fn main() {
    println!("🎷 musician-soul — a jam you can watch\n");
    bar();

    // ── 1. Build personas with influences ────────────────────────────
    println!("1. Three personas take the stage, each shaped by influences:\n");

    let mut miles = MusicianPersona::new("Miles", "trumpet");
    miles.add_influence("Miles Davis", 1.0);
    miles.add_influence("Clark Terry", 0.3);

    let mut trane = MusicianPersona::new("Trane", "tenor_sax");
    trane.add_influence("John Coltrane", 1.0);
    trane.add_influence("Johnny Hodges", 0.4);

    let mut monk = MusicianPersona::new("Monk", "piano");
    monk.add_influence("Thelonious Monk", 1.0);

    for p in [&miles, &trane, &monk] {
        let infl: Vec<String> = p
            .influence_weights
            .iter()
            .map(|(name, w)| format!("{name} ({w:.1})"))
            .collect();
        println!(
            "   • {:<6} on {:<10} ← {}",
            p.name,
            p.instrument,
            infl.join(", ")
        );
    }

    // ── 2. Digest phrases (stand-ins for MIDI) ───────────────────────
    println!("\n2. Each persona digests phrases in the style of its influences.");
    for i in 0..8 {
        miles.digest_phrase(&miles_phrase(&format!("miles_{i}")), "Miles Davis");
        trane.digest_phrase(&coltrane_phrase(&format!("trane_{i}")), "John Coltrane");
        monk.digest_phrase(&monk_phrase(&format!("monk_{i}")), "Thelonious Monk");
    }
    println!(
        "   Patterns stored — Miles: {}, Trane: {}, Monk: {}",
        miles.vector_db.patterns.len(),
        trane.vector_db.patterns.len(),
        monk.vector_db.patterns.len()
    );

    // A soul print needs *proven* patterns, so nothing is unique yet:
    println!(
        "   Soul so far — Miles: {:.0}%, Trane: {:.0}%, Monk: {:.0}%   (all borrowed)",
        miles.soul_percentage(),
        trane.soul_percentage(),
        monk.soul_percentage()
    );

    // ── 3. Jam ───────────────────────────────────────────────────────
    bar();
    println!("3. They jam. Each round: everyone responds to a seed phrase,\n   the room scores harmony + surprise, and productive rounds teach.\n");

    let mut jam = JamSession::new(vec![miles, trane, monk], "late_night_session");

    let seeds = [
        miles_phrase("seed_miles"),
        coltrane_phrase("seed_trane"),
        monk_phrase("seed_monk"),
    ];

    println!("   round │ harmony │ surprise │ productive?");
    println!("   ──────┼─────────┼──────────┼────────────");
    for r in 0..18 {
        let seed = &seeds[r % seeds.len()];
        let round = jam.round(seed);
        if r % 3 == 0 || round.productive {
            println!(
                "   {:>5} │  {:.2}   │   {:.2}   │  {}",
                r + 1,
                round.harmony_score,
                round.surprise_score,
                if round.productive { "yes ✓" } else { "no" }
            );
        }
    }

    // ── 4. Read the result ───────────────────────────────────────────
    bar();
    println!("4. After the session:\n");
    println!(
        "   Session harmony: {:.2}   Productive rounds: {}/{}",
        jam.session_harmony(),
        jam.productive_rounds(),
        jam.rounds.len()
    );
    println!("\n   Soul report (how much of each voice is now its own):");
    for (name, soul_pct) in jam.soul_report() {
        let evolved = jam
            .personas
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.vector_db.evolved_count())
            .unwrap_or(0);
        let named = jam
            .personas
            .iter()
            .find(|p| p.name == name)
            .and_then(|p| p.soul_name.clone())
            .unwrap_or_else(|| "(still emerging)".to_string());
        println!(
            "   • {:<6} {:>5.1}% own  │ {} evolved pattern(s) │ soul: {}",
            name, soul_pct, evolved, named
        );
    }

    // The soul print is a concrete vector. Two different questions are being
    // asked here, and they can disagree — that disagreement is the point:
    //   • soul_percentage() = how much of the library is *self-generated*
    //     (evolved, generation > 0). Rises fast when jams are surprising.
    //   • soul_print()      = the centroid of patterns that have *reliably
    //     worked* (high confidence). Stays empty until something proves out.
    if let Some(p) = jam.personas.iter().find(|p| p.name == "Miles") {
        let print = p.vector_db.soul_print();
        println!(
            "\n   Miles' soul print strength (centroid of *proven* patterns): {:.3}",
            print.identity_strength()
        );
        if print.identity_strength() == 0.0 {
            println!("   → still 0: Miles is exploring (evolving) faster than any single");
            println!("     pattern has proven reliable. Exploration ≠ a settled signature.");
        }
    }

    // ── 5. Woodshedding: where a soul print actually crystallizes ─────
    bar();
    println!("5. Woodshedding — the other half of identity.\n");
    println!("   Jamming explores. But a signature forms when a player keeps");
    println!("   what *reliably works*. Here one persona practices a phrase that");
    println!("   consistently lands, and its soul print crystallizes:\n");

    let mut soloist = MusicianPersona::new("Soloist", "trumpet");
    soloist.add_influence("Miles Davis", 1.0);
    for i in 0..6 {
        soloist.digest_phrase(&miles_phrase(&format!("woodshed_{i}")), "Miles Davis");
    }
    // Reinforce every pattern a few times — "this works, keep it."
    for pat in &mut soloist.vector_db.patterns {
        for _ in 0..5 {
            pat.reinforce();
        }
    }
    let print = soloist.vector_db.soul_print();
    println!(
        "   Soloist soul print strength after consolidation: {:.3}  (now non-zero!)",
        print.identity_strength()
    );
    println!(
        "   Proven patterns in the print: {}",
        soloist
            .vector_db
            .patterns
            .iter()
            .filter(|p| p.confidence() > 0.6 && p.success_count > 2)
            .count()
    );

    bar();
    println!("Done. Re-run it — the arc is deterministic, so the numbers repeat.");
    println!("Change the influences, the phrases, or the round count and watch");
    println!("how the soul report moves. That knob-turning IS the experiment.");
}
