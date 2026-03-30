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

## v2.0 — Future

> Not scheduled — demand-gated

- [ ] Respiratory system (breath cycle, gas exchange, VO2)
- [ ] Circulatory system (heart rate, blood flow dynamics)
- [ ] Injury/damage model (fractures, sprains, healing, damage accumulation)
- [ ] Multi-body coordination (bimanual tasks, load carrying)
- [ ] Muscle wrapping (via-points for muscles that wrap around bones)
- [ ] Proprioception (joint angle feedback, muscle spindle signals)

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
| Cross-crate bridges | Yes | -- |
| Soorat visualization data | Yes | -- |
| Physics simulation (forces) | -- | impetus |
| Creature behavior/AI | -- | jantu |
| Material science (bone stress) | -- | dravya |
| Rendering (mesh/skeleton) | -- | soorat/kiran |
| Math (vectors, transforms) | -- | hisab |
| Thermal physiology | -- | ushma |
