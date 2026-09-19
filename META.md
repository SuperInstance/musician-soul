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

## The three orders of a gesture

A curve carries structure at each derivative, and the meta layer now reads all
three:

| Order | Method | Reads | In one line |
|---|---|---|---|
| 1st | `tangent` / `arc_length` | direction & distance of travel | *where it's going, how far it's gone* |
| 2nd | `bending_energy` | curvature — turning **within** a plane | *how hard it changes its mind* |
| 3rd | `twist_energy` | torsion — turning **out of** that plane | *whether it reaches a new dimension* |

Curvature and torsion are different animals, and the distinction is the whole
point. A phrase can bend as hard as you like and still live in a single plane of
abstraction — an arch, a zig-zag confined to two axes. Its `bending_energy` is
high; its `twist_energy` is **zero**. A gesture only has twist when its turning
*leaves the plane it was turning in* — when the next move opens a direction the
last two moves did not span. `cargo run --example meta` shows it directly: a flat
circle and a helix bend by almost the same amount (~0.1), but only the helix
twists (0.00 vs 0.28).

This is the crate's reading of the fleet's oldest law — **the property is in the
twist** ([twist-engine](https://github.com/SuperInstance/twist-engine): *layers +
deliberate offset → interference → emergence; no new atoms, a new angle*). New
structure does not come from more of the same turning; it comes from the offset
that reaches out of the current plane. Curvature rearranges what is already
there; torsion touches what was not. `planarity` is the scale-free inverse — how
flat a gesture stays — and `MusicianPersona::soul_twist()` asks it of a persona's
*becoming*: does its identity keep refining along one axis, or keep opening
genuinely new dimensions of itself?

## Comparing motion across nodes: `gesture_distance`

`meta_similarity` is offset- and scale-sensitive (it compares positions along the
curves). `gesture_distance` strips those away: it reduces each spline to its
**unit directions of travel** and scores the mean angular difference, so two
identical motions at different sizes or positions read as distance ~0. That
invariance is exactly what a comparison *between fleet nodes* needs — a
musician-soul phrase, an elephant room, and a tensor-midi clip sit in different
coordinate systems, but the *shape of their going* is now directly comparable.
Range 0 (same motion) to 2 (opposed at every step); symmetric; zero to itself.

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
- `twist_energy` is a **discrete, dimension-agnostic** generalization of torsion,
  not the classical signed scalar (which is defined in 3-space). It measures the
  angle by which each step leaves the osculating plane of the previous two — an
  unsigned magnitude in `[0, 1]` per vertex — which is the meaningful reading in a
  32-dim space where a single signed binormal does not exist. It needs ≥4 points
  to be non-trivial, and like `bending_energy` it is measured at a fixed resample
  resolution, so compare twists taken at the same scale.

*A tensor approximates a function. We approximate the abstraction. And it runs.*
