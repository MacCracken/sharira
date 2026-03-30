# Testing Guide

## Running Tests

```bash
# Default (no features)
cargo test

# All features including soorat-compat
cargo test --all-features

# Specific module
cargo test --all-features skeleton::
cargo test --all-features muscle::
cargo test --all-features ik::
cargo test --all-features fatigue::
```

## Test Categories

| Category | Count | Location |
|----------|-------|----------|
| Unit tests (gait) | 23 | `src/gait.rs` |
| Unit tests (muscle) | 18 | `src/muscle.rs` |
| Unit tests (bridge) | 18 | `src/bridge.rs` |
| Unit tests (biomechanics) | 17 | `src/biomechanics.rs` |
| Unit tests (fatigue) | 14 | `src/fatigue.rs` |
| Unit tests (joint) | 12 | `src/joint.rs` |
| Unit tests (ik) | 12 | `src/ik.rs` |
| Unit tests (allometry) | 12 | `src/allometry.rs` |
| Unit tests (morphology) | 11 | `src/morphology.rs` |
| Unit tests (body) | 9 | `src/body.rs` |
| Unit tests (pose) | 8 | `src/pose.rs` |
| Unit tests (soorat) | 8 | `src/integration/soorat.rs` |
| Unit tests (skeleton) | 7 | `src/skeleton.rs` |
| Unit tests (kinematics) | 7 | `src/kinematics.rs` |
| Unit tests (preset) | 6 | `src/preset.rs` |
| Unit tests (error) | 1 | `src/error.rs` |
| Integration tests | 6 | `tests/integration.rs` |
| Doc tests | 1 | `src/fatigue.rs` |
| **Total** | **190** | |

## Coverage

Target: 80%+ line coverage.

```bash
# Generate coverage report (requires cargo-llvm-cov)
make coverage

# Or directly
cargo llvm-cov --all-features --html --output-dir coverage/
```

Coverage configuration is in `codecov.yml` (80% project target).

## Benchmarks

```bash
# Run benchmarks with CSV history
make bench

# Or directly
./scripts/bench-history.sh

# Just criterion (no history tracking)
cargo bench --bench benchmarks
```

Results:
- `bench-history.csv` — timestamped benchmark results

### Benchmark groups (10)

| Group | What it measures |
|-------|-----------------|
| skeleton_total_mass_100 | Mass summation over 100 bones |
| skeleton_find_bone_100 | Name lookup in 100-bone skeleton |
| skeleton_chain_to_root_100 | Chain traversal from leaf to root |
| skeleton_children_100 | Child lookup for a bone |
| muscle_current_force | Full Hill model force calculation |
| center_of_mass_100 | Weighted CoM over 100 positions |
| ground_reaction_force | GRF computation |
| gait_limb_phase | Phase calculation for a limb |
| gait_speed | Speed from stride/duration |
| balance_margin | 1D stability margin |

## Testing Patterns

### Approximate equality

Use small epsilons for f32 physics comparisons:

```rust
assert!((value - expected).abs() < 0.01);
// or for Vec3:
assert!((pos - expected_pos).length() < 1e-4);
```

### Serde roundtrip

All serializable types should have roundtrip tests:

```rust
let json = serde_json::to_string(&value).expect("serialize");
let restored: Type = serde_json::from_str(&json).expect("deserialize");
```

### Mathematical property tests

Verify known physics properties:

```rust
// Conservation: MR + MA + MF = 1.0 (fatigue compartments)
// Hill model: force = 0 at zero activation
// FK: identity pose preserves local transforms
// IK: solve then FK should reach target
// Allometry: larger mass → larger bones
// Stability: CoM inside polygon → positive margin
```

### Edge case tests

Always test boundary conditions:

```rust
// Zero mass, zero length, zero activation
// Empty skeleton, empty polygon
// Division guards (zero duty factor, zero area)
// Free fall (ZMP undefined)
// Unreachable IK target
```

## Local CI

```bash
make check   # fmt + clippy + test + audit
```

This matches what CI runs, minus the platform matrix and coverage upload.
