//! The meta layer, made visible: a phrase as a *gesture*, not a point.
//!
//! ```bash
//! cargo run --example meta
//! ```
//!
//! Where `cargo run --example jam` shows personas learning, this shows the
//! deeper representation underneath: each phrase traced as a spline through
//! abstraction space, and read by its geometry — how far it travels, how much
//! it bends, which way it is going. Two phrases can share the same *average*
//! features yet be utterly different *motions*; the meta layer sees that.

use musician_soul::meta::{self, AbstractionSpline};
use musician_soul::{
    Duration, MusicEmbedding, MusicianPersona, NoteEvent, Phrase, Pitch, Velocity,
};

fn phrase(name: &str, pitches: &[u8]) -> Phrase {
    Phrase {
        events: pitches
            .iter()
            .map(|&p| NoteEvent {
                pitch: Pitch(p),
                velocity: Velocity(84),
                duration: Duration(240),
                tick_offset: 0,
            })
            .collect(),
        source: name.into(),
        instrument: "line".into(),
    }
}

fn bar() {
    println!("{}", "─".repeat(68));
}

fn geometry(label: &str, p: &Phrase) {
    let s: AbstractionSpline = p.reality_spline(3);
    println!(
        "  {:<10} points {:>2} | arc {:>5.2} | bending {:>5.2} | twist {:>5.2} | control-pts {}",
        label,
        p.len(),
        s.arc_length(),
        s.bending_energy(),
        s.twist_energy(),
        s.len(),
    );
}

fn main() {
    println!("🌀 musician-soul — the meta layer (abstraction as gesture)\n");
    bar();

    // Three lines. The rising and the descending share almost every aggregate
    // feature (same notes, same rhythm, reversed); the arch turns hard.
    let rising = phrase("rising", &[60, 62, 64, 65, 67, 69, 71, 72]);
    let falling = phrase("falling", &[72, 71, 69, 67, 65, 64, 62, 60]);
    let arch = phrase("arch", &[60, 64, 67, 72, 67, 64, 60, 55]);
    let jagged = phrase("jagged", &[60, 72, 61, 71, 62, 70, 63, 69]);

    println!("1. A phrase is a path, not a point. Its spline's geometry:\n");
    geometry("rising", &rising);
    geometry("falling", &falling);
    geometry("arch", &arch);
    geometry("jagged", &jagged);
    println!("\n   The smooth scales barely move through abstraction space (each");
    println!("   3-note window looks like the next); the arch travels far and turns");
    println!("   steadily; the jagged line travels farthest and bends the hardest.");
    println!("   A single 32-vector averages all of that away.");

    // Aggregate cosine vs. gesture similarity.
    bar();
    println!("2. Same features, different motion — cosine vs. the meta layer:\n");
    let cos_rf =
        MusicEmbedding::from_phrase(&rising).similarity(&MusicEmbedding::from_phrase(&falling));
    let meta_rf = meta::meta_similarity(&rising, &falling, 3);
    let cos_rj =
        MusicEmbedding::from_phrase(&rising).similarity(&MusicEmbedding::from_phrase(&jagged));
    let meta_rj = meta::meta_similarity(&rising, &jagged, 3);
    println!(
        "   rising vs falling :  cosine {:.2}   meta {:.2}",
        cos_rf, meta_rf
    );
    println!(
        "   rising vs jagged  :  cosine {:.2}   meta {:.2}",
        cos_rj, meta_rj
    );
    println!("\n   The meta score reads the shape of the movement, not the summary.");

    // The tangent: a velocity through abstraction space.
    bar();
    println!("3. The tangent is a direction of travel — a d_mu:\n");
    let s = rising.reality_spline(3);
    let t_mid = s.tangent(0.5);
    let mag: f32 = t_mid.iter().map(|x| x * x).sum::<f32>().sqrt();
    println!("   rising's tangent at the midpoint is a unit direction (|t| = {mag:.2}).");

    // A persona's identity as a trajectory, and its Vibe velocity.
    bar();
    println!("4. A persona's soul is a trajectory, not a centroid:\n");
    let mut p = MusicianPersona::new("Wanderer", "sax");
    for (i, line) in [&rising, &arch, &falling, &jagged].iter().enumerate() {
        let mut ph = (*line).clone();
        ph.source = format!("digest{i}");
        p.digest_phrase(&ph, "influence");
    }
    let soul = p.soul_spline();
    println!(
        "   soul spline: {} control points, arc {:.2}, bending {:.2}, twist {:.2}",
        soul.len(),
        soul.arc_length(),
        soul.bending_energy(),
        soul.twist_energy()
    );
    println!(
        "   Vibe velocity (d_mu, how fast identity is moving): {:.3}",
        p.vibe_velocity()
    );
    println!(
        "   soul twist (does its becoming open new dimensions?): {:.3}",
        p.soul_twist()
    );

    // The third order: curvature vs. torsion. A planar arch bends but never
    // leaves its plane; a helix bends the same and keeps opening a new one.
    bar();
    println!("5. Third order — bending stays in a plane, twist leaves it:\n");
    let planar: Vec<meta::Point> = (0..8)
        .map(|i| {
            let a = i as f32 * 0.6;
            let mut pt = [0.0f32; meta::DIM];
            pt[0] = a.cos();
            pt[1] = a.sin();
            pt
        })
        .collect();
    let helix: Vec<meta::Point> = (0..8)
        .map(|i| {
            let a = i as f32 * 0.6;
            let mut pt = [0.0f32; meta::DIM];
            pt[0] = a.cos();
            pt[1] = a.sin();
            pt[2] = a * 0.5;
            pt
        })
        .collect();
    let sp = AbstractionSpline::new(planar);
    let sh = AbstractionSpline::new(helix);
    println!(
        "   planar circle: bending {:.2}, twist {:.2}, planarity {:.3}",
        sp.bending_energy(),
        sp.twist_energy(),
        sp.planarity()
    );
    println!(
        "   helix        : bending {:.2}, twist {:.2}, planarity {:.3}",
        sh.bending_energy(),
        sh.twist_energy(),
        sh.planarity()
    );
    println!("   → same bend, but only the helix reaches a new dimension.");
    println!("     The property is in the twist.");

    // Gesture distance: motion compared free of scale and offset.
    bar();
    println!("6. Gesture distance — the going, free of where and how big:\n");
    println!(
        "   rising↔rising {:.3}   rising↔arch {:.3}   rising↔falling {:.3}",
        meta::gesture_distance(&rising, &rising, 3),
        meta::gesture_distance(&rising, &arch, 3),
        meta::gesture_distance(&rising, &falling, 3)
    );
    println!("   (0 = same motion, 2 = opposed; invariant to scale and offset,");
    println!("    so it is the comparison that travels across fleet nodes.)");

    bar();
    println!("A tensor approximates a function. This approximates the abstraction —");
    println!("the smooth gesture reality traces, seen through our own lens. Not talk:");
    println!("it is 32-dimensional differential geometry, and it runs.");
}
