# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/).

## [1.0.0]

### Added
- **ik** — inverse kinematics: `IKChain` for bone chain definition, `IKTarget` with position + optional orientation + pole vector; analytic 2-bone solver (closed-form via law of cosines); FABRIK n-bone solver (iterative forward/backward reaching); both respect joint limits during solve
- **fatigue** — three-compartment motor unit fatigue model (Xia & Frey-Law 2008): `FatigueState` with resting/active/fatigued motor unit pools; `update()` with implicit Euler integration; `capacity()` multiplier for muscle force; `time_to_exhaustion()` estimate; recovery dynamics (faster at rest, slower under load)
- **gait** — `Gait::blend()` for interpolating all cycle parameters between gaits; `GaitController` state machine with speed-dependent gait selection, smooth transitions, and configurable duration; `GaitController::bipedal_default()` (idle→walk→run) and `quadrupedal_default()` (walk→trot→canter→gallop) presets
- **allometry** — `AllometricParams` for power-law body scaling (McMahon 1975, Alexander 2003): bone length/diameter/mass, muscle force, stride length/frequency, heart rate, metabolic rate; `mammalian()` and `avian()` parameter presets; `scale_skeleton()` for geometric scaling; `allometric_skeleton()` to generate plausible skeleton from mass + body plan
- **morphology** — `Morphology` for parametric within-species variation: per-bone `BoneScale` overrides (length, mass, width); `MorphologyProfile` presets (average, heavy, lean, tall, compact); `Morphology::random()` for population diversity; `apply_morphology()` to generate variant skeletons
- **joint** — `JointLimits::clamp_rotation()` (Euler decomposition + per-axis clamping); `JointLimits::violation()` (angular violation in radians); `Joint::clamp_rotation()` and `Joint::violation()` convenience methods
- **body** — `Body::constrain_pose()` (enforce all joint limits, returns clamped count); `Body::total_violation()` (sum of angular violations across joints)

## [0.3.0]

### Added
- **integration/soorat** — feature-gated `soorat-compat` module with visualization data structures: `SkeletonVisualization` (bone segments + joints from FK transforms), `MuscleOverlay` (attachment points + activation from muscles), `GaitCycleVisualization` (timeline with limb phase tracks from `Gait`), `BodyPlanVisualization` (limb count, capabilities from `BodyPlan`)

### Updated
- hisab 1.1.0 -> 1.3.0, zerocopy 0.8.47 -> 0.8.48

## [0.2.0]

### Added
- **kinematics** — forward kinematics: world-space bone transforms via parent chain matrix multiplication; `WorldTransforms` with position/rotation/matrix accessors; `world_center_of_mass()` from FK transforms
- **pose** — `Pose` struct: sparse joint angle storage (`Vec<Option<Quat>>`), `set_joint()`, `get_joint()`, `clear_joint()`, `blend()` (slerp interpolation between poses)
- **body** — `Body` struct: aggregates skeleton + pose + joints + muscles + cached world transforms; `update()` recomputes FK and CoM; automatic invalidation on pose change
- **bridge** — cross-crate primitive-value bridges for impetus (constraint torque, damping torque, bone inertia, muscle-to-joint torque, limb force, GRF vector), ushma (metabolic heat, BMR, body surface area, skin radiation, muscle activation heat), dravya (bone density to Young's modulus/yield strength, tendon stress)
- **skeleton** — `Bone::new()` constructor, `.with_position()` / `.with_rotation()` builders, `Skeleton::new()`, `add_bone()`, `bones()` accessor; `#[inline]` on `find_bone`, `get_bone`
- **muscle** — `Muscle::new()` constructor, `.with_attachments()` builder; `Muscle::moment_arm()` static method; `update_activation()` (implicit Euler excitation→activation dynamics); `set_excitation()`; `tendon_force()` (exponential series elastic element); `force_at()` with explicit velocity; attachment offsets (`origin_offset`, `insertion_offset`); activation dynamics fields (`excitation`, `tau_activation`, `tau_deactivation`); tendon fields (`tendon_slack_length`, `tendon_stiffness`)
- **gait** — `Gait::quadruped_canter()` (3-beat, duty 0.4, 4-8 m/s), `Gait::quadruped_gallop()` (4-beat, duty 0.3, 8-15 m/s); `FootPlacement` struct; `Gait::foot_placements()` with stance/swing arc; `GaitCycle::speed()` method; `GaitType::Run` variant
- **biomechanics** — `support_polygon()` (convex hull of ground contacts via hisab), `zero_moment_point()` (ZMP from CoM + acceleration), `stability_margin()` (signed distance from point to polygon edge)

### Changed
- **skeleton** — `Bone.local_position` migrated from `[f32; 3]` to `hisab::Vec3`; `Bone.local_rotation` migrated from `[f32; 4]` to `hisab::Quat`
- **biomechanics** — `center_of_mass()` now takes `&[Vec3]` instead of `&[[f32; 3]]`, returns `Vec3`
- **muscle** — Hill model active force-length Gaussian width parameter: 0.18 → 0.45 (Thelen 2003); added passive tension (exponential), force-velocity (Hill 1938), and pennation angle to the force model
- **gait** — human run preset now uses `GaitType::Run` instead of `GaitType::Gallop`
- **Cargo.toml** — license identifier `GPL-3.0` → `GPL-3.0-only` (SPDX compliance)

### Performance
- muscle_current_force: 4 ns → 13 ns (full Hill model with passive tension, force-velocity, pennation; still sub-microsecond)

## [0.1.0]

### Added
- **skeleton** — `BoneId`, `Bone`, `Skeleton` with hierarchy navigation (`find_bone`, `get_bone`, `total_mass`, `roots`, `children`, `chain_to_root`)
- **joint** — `JointType` (Ball, Hinge, Pivot, Saddle, Fixed, Planar), `AxisLimit`, `JointLimits`, `Joint` with presets (`human_knee`, `human_shoulder`)
- **muscle** — `MuscleGroup`, `Muscle` with activation, force calculation, antagonist detection
- **gait** — `GaitPhase`, `GaitType`, `GaitCycle`, `Gait` with presets (`human_walk`, `human_run`, `quadruped_walk`, `quadruped_trot`), `limb_phase()`, `speed()`
- **biomechanics** — `center_of_mass()`, `ground_reaction_force()`, `cost_of_transport()`, `balance_margin()`
- **preset** — `BodyPlan` (Bipedal, Quadruped, Hexapod, Octopod, Serpentine, Avian, Aquatic, Centipede)
- **error** — `ShariraError` with variants for skeleton, joint, gait, bone, and computation errors
- **logging** — optional tracing-subscriber initialization (feature-gated)
