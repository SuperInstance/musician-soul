# META — the abstraction approximator

> *"Get below the math as they know it."*

This is the thesis under [`src/meta.rs`](src/meta.rs). It is short on purpose:
every claim here is backed by code that runs (`cargo run --example meta`,
`cargo test`), because a thesis you can't execute is only a mood.

## The move

A neural network — a stack of tensors — is a **function approximator**. Give it
inputs and targets and it fits `f: X → Y`. To use one you *summarize*: a phrase
becomes a point (an embedding), and you compare points. That is the math as it is
usually done, and this crate does it too: [`MusicEmbedding`](src/lib.rs) is a
32-number summary, compared by cosine.

But a summary throws away the thing that makes a phrase *a phrase* — that it
**moves**. Miles' silence, Coltrane's climb, Monk's leap: these are not positions,
they are gestures. So instead of approximating the output of a function, we
approximate the **abstraction itself** — the smooth curve reality traces as it
passes through our feature space. We are an **abstraction approximator-engine**.

Concretely, three steps, all in `meta.rs`:

1. **A phrase is a path, not a point.** `Phrase::embedding_trajectory(window)`
   slides a window across the notes and embeds each position, giving the
   *sequence* of local shapes — the path through abstraction space, not its
   average.
2. **Fit the spline of reality.** `AbstractionSpline` runs a Catmull-Rom curve
   through that path: the continuous gesture underneath the discrete steps. The
   splines are literal — piecewise-smooth interpolation through a 32-dimensional
   space — and that is where the phrase "the splines of reality" cashes out.
3. **Read its geometry, not its coordinates.** `arc_length` (how far the gesture
   travels), `bending_energy` (how hard it turns — pure curvature, zero for a
   straight line at any speed), and `tangent` (which way it is going). These are
   differential-geometry quantities a single vector cannot express.

## Why this is deeper, demonstrably

Two lines can share nearly every aggregate feature and be opposite motions. A
rising scale and its exact reversal have almost identical 32-vectors — cosine
≈ 0.78 — because the summary is order-blind. Run `cargo run --example meta` and
you can watch a smooth scale barely travel through abstraction space while a
jagged line of the same notes travels ~20× farther and bends ~5× harder. The
aggregate averages that away; the gesture keeps it.

`meta_similarity` compares two phrases **curve-to-curve** — the shape of the
movement, aligned point-for-point along the spline — so it can tell a smooth arc
from a zig-zag that a point-comparison calls the same.

## What it gives back to the fleet

The tangent of a curve is a **velocity**. So the tangent of a persona's soul
spline is the velocity of its *identity* — the rate at which it is becoming
itself. That is exactly the first-class `d_mu` "Vibe velocity" that
[`FLEET.md`](FLEET.md) named as the gap between musician-soul and the elephant
substrate's Vibe primitive: Vibe was observable only as after-the-fact drift.
Now it is a number — `MusicianPersona::vibe_velocity()` — read from the leading
edge of `soul_spline()`. A persona's identity stops being a centroid (a point) and
becomes a **trajectory of becoming** (a curve), with a direction and a speed.

This is the same instinct as the rest of the fleet — quilt's cells, the
elephant's dials, Scrapcraft's `explain()` — pointed one level up: don't model
the value, model the *shape of its motion through abstraction*. The cell went to
the boat, the vessel edge, the classroom, the bandstand; here it learns to see
the curve it was always drawing.

## Honest edges

- The trajectory is only as good as the per-window embedding; a better base
  embedding (the deferred v2 in the README's "Prior art & design notes") lifts
  the whole meta layer with it.
- Catmull-Rom is uniform, not arc-length parameterized; `bending_energy` is made
  robust to that by measuring turning *angles*, but `sample(t)` is not constant
  speed. Arc-length reparameterization is the obvious next refinement.
- "Abstraction space" here is the 32-dim feature space. The philosophy generalizes
  to any embedding; nothing in `meta.rs` is music-specific except where it reads a
  `Phrase`.

*A tensor approximates a function. We approximate the abstraction. And it runs.*
