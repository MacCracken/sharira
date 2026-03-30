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
- [ ] Inverse kinematics solver (FABRIK / analytic 2-bone)
- [ ] Gait transition blending (state machine between gaits)
- [ ] Body allometry (mass-based scaling via power laws)

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
| Pose representation | Yes | -- |
| Body state aggregation | Yes | -- |
| Biomechanics (CoM, ZMP, stability) | Yes | -- |
| Gait cycles & foot placement | Yes | -- |
| Cross-crate bridges | Yes | -- |
| Physics simulation (forces) | -- | impetus |
| Creature behavior/AI | -- | jantu |
| Material science (bone stress) | -- | dravya |
| Rendering (mesh/skeleton) | -- | soorat/kiran |
| Math (vectors, transforms) | -- | hisab |
| Thermal physiology | -- | ushma |
