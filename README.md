# Sharira

**Sharira** (शरीर — Sanskrit for "body, physical form") — physiology engine for skeletal structures, musculature, locomotion, and biomechanics.

Part of the [AGNOS](https://github.com/MacCracken) ecosystem.

## What It Does

Sharira defines what a body **is** — bones, joints, muscles, gaits, and biomechanics. Other crates decide what it does, how it moves, and how it looks.

```text
soorat  → renders the body (mesh + skinned skeleton)
sharira → defines the body (bones, joints, muscles, gaits)  ← this crate
jantu   → decides what the body does (instinct, survival)
bhava   → shapes how it moves (personality, emotion)
impetus → physics (forces, collision, gravity)
raasta  → navigation (where to go)
```

## Features

- **Skeleton** — bone hierarchies with parent-child navigation, mass computation, chain traversal
- **Joints** — ball, hinge, pivot, saddle, fixed, planar — with axis limits, stiffness, damping
- **Muscles** — full Hill muscle model (active force-length, passive tension, force-velocity, pennation angle), activation levels, antagonist detection
- **Gaits** — walk, trot, canter, gallop, crawl, slither, hop, fly, swim — with duty factor, stride length, limb phasing
- **Biomechanics** — center of mass, ground reaction force, cost of transport, balance margin
- **Body Plans** — bipedal, quadruped, hexapod, octopod, serpentine, avian, aquatic, centipede

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
sharira = "0.1"
```

```rust
use sharira::{Skeleton, Bone, BoneId, Gait, Muscle, MuscleGroup};

// Build a skeleton
let mut skeleton = Skeleton::new("biped");
skeleton.add_bone(Bone::new(BoneId(0), "pelvis", 0.2, 5.0, None));
skeleton.add_bone(Bone::new(BoneId(1), "femur_l", 0.45, 4.0, Some(BoneId(0))));

// Query it
let mass = skeleton.total_mass();
let roots = skeleton.roots();
let chain = skeleton.chain_to_root(BoneId(1));

// Muscles with full Hill model
let mut quad = Muscle::new("quad", BoneId(0), BoneId(1), MuscleGroup::Extensor, 5000.0, 0.3);
quad.set_activation(0.8);
let force = quad.current_force(0.3);        // isometric force at rest length
let dynamic = quad.force_at(0.3, -3.0);     // force while shortening

// Gait preset
let walk = Gait::human_walk();
let speed = walk.speed();
```

### Optional Features

```toml
[dependencies]
sharira = { version = "0.1", features = ["logging"] }
```

| Feature   | Description                          |
|-----------|--------------------------------------|
| `logging` | Structured logging via `tracing`     |

## Dependencies

| Crate     | Purpose                              |
|-----------|--------------------------------------|
| `hisab`   | Math primitives (vectors, matrices)  |
| `serde`   | Serialization / deserialization      |
| `thiserror` | Error handling                     |
| `tracing` | Structured logging                   |

## Consumers

| Crate       | Usage                                          |
|-------------|------------------------------------------------|
| **kiran**   | Character/creature body definitions             |
| **soorat**  | Skeletal mesh rendering, muscle debug overlay   |
| **jantu**   | Body plan drives creature behavior capabilities |
| **bhava**   | Personality/emotion influences gait and posture |
| **impetus** | Body mass/inertia for physics simulation        |

## Development

```bash
make check      # fmt + clippy + test + audit
make test       # cargo test --all-features
make bench      # run benchmarks with history
make coverage   # HTML coverage report
make doc        # build docs (warnings = errors)
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full workflow.

## Roadmap

See [docs/development/roadmap.md](docs/development/roadmap.md) for completed work, planned cross-crate bridges, and future features.

## License

[GPL-3.0](LICENSE)
