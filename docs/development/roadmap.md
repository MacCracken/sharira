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

---

## v1.1.0 — Muscle Wrapping & Proprioception

### Muscle wrapping (`src/muscle.rs` extension)
- [ ] `WrapPoint` struct: via-point position + bone attachment for muscles that wrap around bones
- [ ] `Muscle::with_wrap_points(Vec<WrapPoint>)` builder
- [ ] Multi-segment muscle path: origin → wrap₁ → wrap₂ → insertion
- [ ] Moment arm recalculation through wrap geometry
- [ ] Effective muscle length from piecewise path segments
- [ ] Force direction per segment (not just origin-to-insertion)

### Proprioception (`src/proprioception.rs`)
- [ ] `MuscleSpindle` struct: stretch sensor with sensitivity, firing rate
- [ ] `GolgiTendonOrgan` struct: tendon tension sensor with threshold
- [ ] `JointReceptor` struct: joint angle + velocity feedback
- [ ] `ProprioceptiveState` aggregation: all sensory feedback for a body
- [ ] `ProprioceptiveState::from_body(body, previous_state, dt)` — compute all feedback
- [ ] Firing rate models: linear + sigmoid activation functions

---

## v1.2.0 — Respiratory System

### Respiratory (`src/respiratory.rs`)
- [ ] `LungState` struct: tidal volume, residual volume, vital capacity
- [ ] `BreathCycle`: inhale/exhale phases, respiratory rate, cycle duration
- [ ] `gas_exchange()`: O₂ uptake, CO₂ output based on alveolar ventilation
- [ ] `vo2_from_activity()`: VO₂ from metabolic demand (ml/kg/min)
- [ ] `vo2_max(mass_kg, fitness)`: maximum aerobic capacity
- [ ] `respiratory_quotient()`: CO₂ produced / O₂ consumed (RQ, diet-dependent)
- [ ] `ventilation_rate(vo2, vco2)`: minute ventilation from gas exchange
- [ ] Integration with fatigue: VO₂ drives aerobic energy supply
- [ ] Breath-by-breath dynamics: `LungState::update(dt, demand)`

---

## v1.3.0 — Circulatory System

### Circulatory (`src/circulatory.rs`)
- [ ] `HeartState` struct: heart rate, stroke volume, cardiac output
- [ ] `BloodPressure` struct: systolic, diastolic, mean arterial pressure
- [ ] `heart_rate_from_demand()`: HR responds to metabolic demand + fitness
- [ ] `cardiac_output()`: CO = HR × stroke volume (L/min)
- [ ] `blood_flow_distribution()`: fraction of CO to muscles vs organs
- [ ] `oxygen_delivery()`: O₂ to muscles from blood flow × hemoglobin
- [ ] `lactate_threshold()`: transition from aerobic to anaerobic metabolism
- [ ] Baroreceptor reflex: BP regulation via HR/vessel tone feedback
- [ ] Integration with respiratory: O₂ uptake → blood O₂ → muscle delivery
- [ ] Allometric scaling: HR and CO scale with body mass (already in allometry)

---

## v1.4.0 — Injury & Damage Model

### Injury (`src/injury.rs`)
- [ ] `DamageState` per bone/muscle/tendon: accumulated damage (0-1)
- [ ] `InjuryType` enum: Fracture, Sprain, Strain, Contusion, Dislocation
- [ ] `Injury` struct: type, location (bone/muscle ID), severity (0-1), healing progress
- [ ] `damage_accumulation()`: stress/strain exceeding yield → damage increment
- [ ] `healing_rate()`: recovery over time based on blood flow, rest, severity
- [ ] Fracture model: bone stress > yield strength → fracture event
- [ ] Strain model: muscle/tendon elongation beyond safe range → micro-tears
- [ ] Fatigue-induced injury: accumulated fatigue lowers injury threshold
- [ ] `Body::injuries()` accessor and `Body::apply_damage()`
- [ ] Force capacity reduction from active injuries
- [ ] Integration with dravya bridge: bone safety factor drives fracture risk

---

## v1.5.0 — Multi-Body Coordination

### Multi-body (`src/coordination.rs`)
- [ ] `ContactConstraint` struct: two bodies linked at contact points
- [ ] `GraspState`: hand/foot grip on external object (position, force, slip margin)
- [ ] `LoadCarry`: object mass distributed across attachment points on body
- [ ] `BimanualTask`: coordinated two-arm operation (carry, push, pull, throw)
- [ ] Center of mass shift from carried loads
- [ ] Gait modification under load: stride shortening, frequency increase
- [ ] Balance recalculation with external mass (shifted support polygon)
- [ ] `Body::attach_load(mass, attachment_bone, offset)` API
- [ ] `Body::detach_load()` API
- [ ] Integration with fatigue: load carrying accelerates fatigue

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
| Forward kinematics | Yes | -- |
| Inverse kinematics | Yes | -- |
| Pose representation | Yes | -- |
| Body state aggregation | Yes | -- |
| Joint constraint enforcement | Yes | -- |
| Biomechanics (CoM, ZMP, stability) | Yes | -- |
| Gait cycles, blending, & foot placement | Yes | -- |
| Muscle fatigue (3-compartment model) | Yes | -- |
| Allometric scaling (power laws) | Yes | -- |
| Morphology variation (parametric) | Yes | -- |
| Muscle wrapping & proprioception | v1.1 | -- |
| Respiratory system (VO₂, gas exchange) | v1.2 | -- |
| Circulatory system (HR, blood flow) | v1.3 | -- |
| Injury & damage model | v1.4 | -- |
| Multi-body coordination | v1.5 | -- |
| Cross-crate bridges | Yes | -- |
| Soorat visualization data | Yes | -- |
| Physics simulation (forces) | -- | impetus |
| Creature behavior/AI | -- | jantu |
| Material science (bone stress) | -- | dravya |
| Rendering (mesh/skeleton) | -- | soorat/kiran |
| Math (vectors, transforms) | -- | hisab |
| Thermal physiology | -- | ushma |
