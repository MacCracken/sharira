# Sharira Roadmap

> **Sharira** (Sanskrit: body) — physiology engine for skeletal structures, musculature, locomotion, and biomechanics.

## Scope

Sharira owns the **science of bodies**: bone hierarchies, joint articulation, muscle force models, gait cycles, and biomechanics. It defines what a body IS; other crates decide what it DOES, how it MOVES, and how it LOOKS.

```
soorat  -> renders the body (mesh + skinned skeleton)
sharira -> defines the body (bones, joints, muscles, gaits)  <- THIS CRATE
jantu   -> decides what the body does (instinct, survival)
bhava   -> shapes how it moves (personality, emotion)
impetus -> physics (forces, collision, gravity)
raasta  -> navigation (where to go)
```

Sharira does NOT own:
- **Rendering** -> soorat/kiran (they consume sharira for body visualization)
- **Physics integration** -> impetus (forces, collision)
- **Behavior/AI** -> jantu (creature decisions)
- **Math primitives** -> hisab (vectors, matrices, transforms)
- **Material stress** -> dravya (bone/tissue material science)

## V0.1.0 — Foundation (done)

### skeleton
- [x] BoneId (u16 identifier)
- [x] Bone struct (position, rotation, length, mass, parent hierarchy)
- [x] Skeleton struct (bone collection with hierarchy navigation)
- [x] find_bone, get_bone, total_mass, roots, children, chain_to_root

### joint
- [x] JointType enum (Ball, Hinge, Pivot, Saddle, Fixed, Planar)
- [x] Degrees of freedom per joint type
- [x] AxisLimit (min/max radians with clamping)
- [x] JointLimits (x/y/z limits with factory methods)
- [x] Joint struct (bone connection with stiffness, damping)
- [x] Presets: human_knee, human_shoulder

### muscle
- [x] MuscleGroup enum (Flexor, Extensor, Abductor, Adductor, Rotator, Sphincter)
- [x] Muscle struct (origin/insertion bones, Hill muscle model)
- [x] Force calculation with activation level
- [x] Antagonist detection

### gait
- [x] GaitPhase enum (Stance, Swing, DoubleSupport, Flight)
- [x] GaitType enum (Walk, Trot, Canter, Gallop, Crawl, Slither, Hop, Fly, Swim)
- [x] GaitCycle (duration, duty factor, stride length, limb phase offsets)
- [x] Gait presets: human_walk, human_run, quadruped_walk, quadruped_trot
- [x] Limb phase calculation, speed computation

### biomechanics
- [x] Center of mass (weighted position average)
- [x] Ground reaction force (stance mechanics)
- [x] Cost of transport (energy efficiency)
- [x] Balance margin (stability computation)

### preset
- [x] BodyPlan enum (Bipedal, Quadruped, Hexapod, Octopod, Serpentine, Avian, Aquatic, Centipede)
- [x] Limb count, flight/swim capability, typical joint count

---

## Cross-Crate Bridges

- [ ] **`bridge.rs` module** — primitive-value conversions for cross-crate physiology
- [ ] **impetus bridge**: joint angles (rad), angular velocities (rad/s) -> constraint forces; body segment mass (kg) -> inertia properties
- [ ] **dravya bridge**: bone density (kg/m3) -> material stiffness; muscle force (N) -> tendon stress (Pa)
- [ ] **ushma bridge**: metabolic rate (W) -> body heat generation; skin surface area (m2) -> radiation heat loss

---

## Soorat Integration (`integration/soorat.rs`)

> Feature-gated `soorat-compat` — structured visualization types for soorat rendering

- [ ] **`integration/soorat.rs` module** — visualization data structures
- [ ] **Skeleton visualization**: bone hierarchy, joint positions, joint limits for wireframe/debug rendering
- [ ] **Muscle overlay**: muscle attachment points, current activation level for colored overlay rendering
- [ ] **Gait cycle data**: foot contact phases, center-of-mass trajectory for animation timeline rendering
- [ ] **Body plan mesh**: body segment dimensions and proportions for procedural mesh generation

---

## Future

> Not scheduled — demand-gated

- [ ] Respiratory system (breath cycle, gas exchange)
- [ ] Circulatory system (heart rate, blood flow)
- [ ] Fatigue model (muscle exhaustion, recovery)
- [ ] Injury/damage model (fractures, sprains, healing)
- [ ] Morphology variation (size, proportion scaling)
- [ ] Multi-body coordination (bimanual tasks, load carrying)

---

## Consumers

| Consumer | What it uses |
|----------|-------------|
| **kiran** | Character/creature body definitions for game entities |
| **soorat** | Skeletal mesh rendering, muscle debug overlay |
| **jantu** | Body plan drives creature behavior capabilities |
| **bhava** | Personality/emotion influences gait and posture |
| **impetus** | Body mass/inertia for physics simulation |

## Boundary with Other Crates

| Feature | sharira | other |
|---------|---------|-------|
| Bone/joint/muscle definition | Yes | -- |
| Physics simulation (forces) | -- | impetus |
| Creature behavior/AI | -- | jantu |
| Material science (bone stress) | -- | dravya |
| Rendering (mesh/skeleton) | -- | soorat/kiran |
| Math (vectors, transforms) | -- | hisab |
| Thermal physiology | -- | ushma |
