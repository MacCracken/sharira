# Architecture Overview

## Module Map

```
sharira
├── skeleton            — Bone hierarchy, BoneId, traversal, mass computation
├── joint               — JointType (6 types), axis limits, constraint clamping, violation detection
├── muscle              — Hill muscle model (active FL, passive, force-velocity, pennation),
│                         tendon model, activation dynamics, moment arm, attachment points
├── pose                — Sparse joint rotation storage (Vec<Option<Quat>>), slerp blending
���── kinematics          — Forward kinematics (parent-chain Mat4 multiplication),
│                         WorldTransforms (position/rotation/matrix per bone), world CoM
├─��� body                — Aggregation of skeleton + pose + joints + muscles,
│                         FK caching with invalidation, joint constraint enforcement
├── ik                  — Inverse kinematics: analytic 2-bone solver (law of cosines),
│                         FABRIK n-bone solver (iterative), IKChain, IKTarget, pole vectors
├── biomechanics        — Center of mass, ground reaction force, cost of transport,
│                         support polygon (convex hull), Zero Moment Point, stability margin
├── gait                — GaitType (10 types), GaitCycle, duty factor, limb phasing,
│                         6 presets, Gait::blend(), GaitController state machine,
│                         FootPlacement with stance/swing arc
├── fatigue             — Three-compartment motor unit model (Xia & Frey-Law 2008),
│                         resting/active/fatigued pools, implicit Euler integration
├── allometry           — Power-law scaling (McMahon 1975, Alexander 2003),
│                         AllometricParams (mammalian, avian), skeleton generation from mass
├── morphology          — Parametric anatomy variation: BoneScale per-bone overrides,
│                         5 presets (average/heavy/lean/tall/compact), random variation
├── preset              — BodyPlan enum (8 body plans), limb count, capabilities
├── bridge              — Cross-crate converters (16 functions) for impetus, ushma, dravya
├── integration/soorat  — Visualization data exporters (skeleton, muscles, gaits, body plans)
├── error               — ShariraError enum (#[non_exhaustive])
└── logging             — Optional tracing-subscriber init (feature: logging)
```

## Feature Flags

| Flag | Dependencies | Description |
|------|-------------|-------------|
| `soorat-compat` | — | Visualization data structures for soorat renderer |
| `logging` | `tracing-subscriber` | Structured logging via `SHARIRA_LOG` env |

Default features: none. All core modules are always available.

## Design Principles

- **Defines what a body IS** — structure, not behavior. Behavior belongs in jantu/bhava
- **Pure physiology** — no I/O, no rendering, no physics simulation
- **Built on hisab** — leverages glam's SIMD-optimized Vec3/Quat/Mat4 via hisab
- **Zero unsafe** — no `unsafe` blocks anywhere
- **Thread-safe** — all types are Send + Sync
- **`#[non_exhaustive]`** — on all public enums
- **`#[must_use]`** — on all pure functions
- **`#[inline]`** — on hot-path functions
- **Result over panic** — no `unwrap()` or `panic!()` in library code
- **Units in comments** — all physical quantities annotated (meters, kg, Newtons, radians)
- **Literature-backed** — Hill 1938, Thelen 2003, Xia & Frey-Law 2008, McMahon 1975

## Data Flow — Body Pipeline

```
BodyPlan + mass
       │
       ▼
┌──────────────┐    ┌──────────────┐
│  allometry   │ or │  morphology  │
│ (from mass)  │    │ (variation)  │
└──────┬───────┘    └──────┬───────┘
       │                   │
       └─────────┬─────────┘
                 ▼
           Skeleton (bone hierarchy)
                 │
          ┌──────┼──────┐
          ▼      ▼      ▼
       Joints  Muscles  Pose
          │      │      │
          └──────┼──────┘
                 ▼
              Body (aggregation)
                 │
                 ▼
          ┌──────────────┐
          │  kinematics  │ ← forward_kinematics(skeleton, pose)
          │  (FK chain)  │
          └──────┬───────┘
                 │
                 ▼
          WorldTransforms
          (Mat4 per bone)
                 │
     ┌───────────┼───────────┐
     ▼           ▼           ▼
 biomechanics  bridge    integration
 (CoM, ZMP,   (impetus,  (soorat
  stability)   ushma,     viz data)
               dravya)
```

## Data Flow — Muscle Force

```
Excitation (neural drive, 0-1)
       │
       ▼
┌──────────────────┐
│  Activation      │ ← implicit Euler (tau_act=15ms, tau_deact=50ms)
│  Dynamics        │
└──────┬───────────┘
       │
       ▼
   Activation (0-1)
       │
       ├── Active FL (Gaussian, γ=0.45)     ──┐
       ├── Passive FL (exponential)           ├──> F = F_max × (a×FL×FV + passive) × cos(α)
       ├── Force-Velocity (Hill hyperbola)   ──┘
       └── Pennation angle (cos)
       │
       ▼
   Muscle Force (N)
       │
       ├── × Moment arm (m) ──> Joint Torque (Nm)
       ├── × Fatigue capacity ──> Reduced Force
       └── / Tendon area ──> Tendon Stress (Pa)
```

## Consumers

| Project | What it uses |
|---------|-------------|
| **kiran** | Body, Skeleton, Pose, FK, body plans, allometry for creature generation |
| **soorat** | SkeletonVisualization, MuscleOverlay, GaitCycleVisualization via soorat-compat |
| **jantu** | BodyPlan capabilities (can_fly, can_swim, limb_count), gait selection |
| **bhava** | GaitController for speed-dependent locomotion, muscle tension for emotion |
| **impetus** | Bridge: bone inertia, joint torques, GRF, constraint forces |
| **ushma** | Bridge: BMR, BSA, muscle heat, skin radiation |
| **dravya** | Bridge: bone density → modulus/yield, tendon stress |
