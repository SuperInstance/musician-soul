//! Digest a real Standard MIDI File (`.mid`) into a persona.
//!
//! ```bash
//! # Point it at your own file:
//! cargo run --example play_midi -- path/to/solo.mid
//!
//! # …or run with no argument to use a small MIDI clip synthesized in memory:
//! cargo run --example play_midi
//! ```
//!
//! This is the bridge FLEET.md calls "the one seam that turns the prototype into
//! a fielded node": actual MIDI in, a persona's learned patterns out — no
//! external crates.

use musician_soul::{midi, MusicEmbedding, MusicianPersona};
use std::process::ExitCode;

fn main() -> ExitCode {
    let arg = std::env::args().nth(1);

    // Read the file the user named, or synthesize a demo clip so the example
    // always runs with no setup.
    let (bytes, label) = match &arg {
        Some(path) => match std::fs::read(path) {
            Ok(b) => (b, path.clone()),
            Err(e) => {
                eprintln!("Could not read {path}: {e}");
                return ExitCode::FAILURE;
            }
        },
        None => {
            println!("No file given — using a synthesized demo clip.");
            println!("(Try: cargo run --example play_midi -- your_solo.mid)\n");
            (demo_smf(), "demo.mid (synthesized)".to_string())
        }
    };

    // Parse the MIDI into phrases (split at rest boundaries).
    let phrases = match midi::phrases_from_smf(&bytes, "unknown", &label) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Could not parse {label}: {e}");
            return ExitCode::FAILURE;
        }
    };

    println!("🎼 Digesting {label}");
    println!("   {} phrase(s) found.\n", phrases.len());
    if phrases.is_empty() {
        println!("   No note phrases in this file — nothing to digest.");
        return ExitCode::SUCCESS;
    }

    // Digest them into a fresh persona.
    let mut persona = MusicianPersona::new("Listener", "unknown");
    for (i, ph) in phrases.iter().enumerate() {
        let e = MusicEmbedding::from_phrase(ph);
        println!(
            "   phrase {:>2}: {:>2} notes | span {:>2} | rest {:.0}% | identity {:.2}",
            i + 1,
            ph.len(),
            ph.register_span(),
            ph.rest_ratio() * 100.0,
            e.identity_strength(),
        );
        persona.digest_phrase(ph, "source");
    }

    println!(
        "\n   Digested into {} patterns.",
        persona.vector_db.patterns.len()
    );
    println!("   Now jam these phrases together, and a soul begins to form —");
    println!("   see `cargo run --example jam` for the full arc.");
    ExitCode::SUCCESS
}

/// A tiny, valid format-0 SMF (480 TPQN): a five-note phrase, a rest, three more.
fn demo_smf() -> Vec<u8> {
    fn vlq(mut n: u32) -> Vec<u8> {
        let mut buf = vec![(n & 0x7f) as u8];
        n >>= 7;
        while n > 0 {
            buf.push(((n & 0x7f) as u8) | 0x80);
            n >>= 7;
        }
        buf.reverse();
        buf
    }
    fn on(delta: u32, pitch: u8, vel: u8) -> Vec<u8> {
        let mut v = vlq(delta);
        v.extend_from_slice(&[0x90, pitch, vel]);
        v
    }
    fn off(delta: u32, pitch: u8) -> Vec<u8> {
        let mut v = vlq(delta);
        v.extend_from_slice(&[0x80, pitch, 0]);
        v
    }

    let mut track = Vec::new();
    // Phrase 1: a little rising line (notes back-to-back, no gaps).
    for &p in &[60u8, 62, 64, 65, 67] {
        track.extend(on(0, p, 88));
        track.extend(off(240, p));
    }
    // Long rest, then phrase 2.
    for (i, &p) in [67u8, 64, 60].iter().enumerate() {
        track.extend(on(if i == 0 { 960 } else { 0 }, p, 80));
        track.extend(off(240, p));
    }
    track.extend_from_slice(&[0x00, 0xff, 0x2f, 0x00]); // end of track

    let mut smf = Vec::new();
    smf.extend_from_slice(b"MThd");
    smf.extend_from_slice(&6u32.to_be_bytes());
    smf.extend_from_slice(&0u16.to_be_bytes());
    smf.extend_from_slice(&1u16.to_be_bytes());
    smf.extend_from_slice(&480u16.to_be_bytes());
    smf.extend_from_slice(b"MTrk");
    smf.extend_from_slice(&(track.len() as u32).to_be_bytes());
    smf.extend_from_slice(&track);
    smf
}
