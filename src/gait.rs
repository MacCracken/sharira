use hisab::Vec3;
use serde::{Deserialize, Serialize};

/// Gait phase within a cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GaitPhase {
    Stance,
    Swing,
    DoubleSupport,
    Flight,
}

/// Locomotion gait type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GaitType {
    Walk,
    Run,
    Trot,
    Canter,
    Gallop,
    Crawl,
    Slither,
    Hop,
    Fly,
    Swim,
}

/// A gait cycle definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaitCycle {
    pub gait_type: GaitType,
    pub cycle_duration_s: f32, // one full cycle
    pub duty_factor: f32,      // fraction of cycle foot is on ground (0-1)
    pub stride_length_m: f32,
    pub limb_phase_offsets: Vec<f32>, // phase offset per limb (0.0-1.0)
}

impl GaitCycle {
    /// Speed from stride length and cycle time (m/s).
    #[must_use]
    #[inline]
    pub fn speed(&self) -> f32 {
        if self.cycle_duration_s <= 0.0 {
            return 0.0;
        }
        self.stride_length_m / self.cycle_duration_s
    }
}

/// A complete gait definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gait {
    pub name: String,
    pub gait_type: GaitType,
    pub speed_range: (f32, f32), // min/max speed in m/s
    pub cycle: GaitCycle,
}

impl Gait {
    /// Human walk (~1.4 m/s, duty factor ~0.6).
    #[must_use]
    pub fn human_walk() -> Self {
        Self {
            name: "walk".into(),
            gait_type: GaitType::Walk,
            speed_range: (0.5, 2.0),
            cycle: GaitCycle {
                gait_type: GaitType::Walk,
                cycle_duration_s: 1.0,
                duty_factor: 0.6,
                stride_length_m: 1.4,
                limb_phase_offsets: vec![0.0, 0.5], // left, right
            },
        }
    }

    /// Human run (~3 m/s, duty factor ~0.35, includes flight phase).
    #[must_use]
    pub fn human_run() -> Self {
        Self {
            name: "run".into(),
            gait_type: GaitType::Run,
            speed_range: (2.0, 10.0),
            cycle: GaitCycle {
                gait_type: GaitType::Run,
                cycle_duration_s: 0.7,
                duty_factor: 0.35,
                stride_length_m: 2.5,
                limb_phase_offsets: vec![0.0, 0.5],
            },
        }
    }

    /// Quadruped walk (horse, duty factor ~0.75).
    #[must_use]
    pub fn quadruped_walk() -> Self {
        Self {
            name: "quadruped_walk".into(),
            gait_type: GaitType::Walk,
            speed_range: (0.5, 2.0),
            cycle: GaitCycle {
                gait_type: GaitType::Walk,
                cycle_duration_s: 1.2,
                duty_factor: 0.75,
                stride_length_m: 1.8,
                limb_phase_offsets: vec![0.0, 0.5, 0.25, 0.75], // LF, RF, LH, RH
            },
        }
    }

    /// Quadruped trot (diagonal pairs move together).
    #[must_use]
    pub fn quadruped_trot() -> Self {
        Self {
            name: "trot".into(),
            gait_type: GaitType::Trot,
            speed_range: (2.0, 5.0),
            cycle: GaitCycle {
                gait_type: GaitType::Trot,
                cycle_duration_s: 0.8,
                duty_factor: 0.5,
                stride_length_m: 2.5,
                limb_phase_offsets: vec![0.0, 0.5, 0.5, 0.0], // LF+RH, RF+LH
            },
        }
    }

    /// Current phase for a limb at given time.
    #[must_use]
    pub fn limb_phase(&self, limb_index: usize, time_s: f32) -> GaitPhase {
        if limb_index >= self.cycle.limb_phase_offsets.len() {
            return GaitPhase::Stance;
        }
        let cycle_pos = (time_s / self.cycle.cycle_duration_s).fract();
        let limb_pos = (cycle_pos + self.cycle.limb_phase_offsets[limb_index]).fract();
        if limb_pos < self.cycle.duty_factor {
            GaitPhase::Stance
        } else {
            GaitPhase::Swing
        }
    }

    /// Quadruped canter (3-beat asymmetric gait, 4-8 m/s).
    ///
    /// Right lead: RH → LH+RF → LF → flight
    #[must_use]
    pub fn quadruped_canter() -> Self {
        Self {
            name: "canter".into(),
            gait_type: GaitType::Canter,
            speed_range: (4.0, 8.0),
            cycle: GaitCycle {
                gait_type: GaitType::Canter,
                cycle_duration_s: 0.6,
                duty_factor: 0.4,
                stride_length_m: 3.5,
                limb_phase_offsets: vec![0.6, 0.3, 0.0, 0.3], // LF, RF, LH, RH (right lead)
            },
        }
    }

    /// Quadruped gallop (4-beat, duty ~0.3, 8-15 m/s).
    ///
    /// Transverse gallop: RH → LH → RF → LF → flight
    #[must_use]
    pub fn quadruped_gallop() -> Self {
        Self {
            name: "gallop".into(),
            gait_type: GaitType::Gallop,
            speed_range: (8.0, 15.0),
            cycle: GaitCycle {
                gait_type: GaitType::Gallop,
                cycle_duration_s: 0.45,
                duty_factor: 0.3,
                stride_length_m: 5.0,
                limb_phase_offsets: vec![0.7, 0.55, 0.15, 0.0], // LF, RF, LH, RH
            },
        }
    }

    /// Speed from stride length and cycle time.
    #[must_use]
    #[inline]
    pub fn speed(&self) -> f32 {
        if self.cycle.cycle_duration_s <= 0.0 {
            return 0.0;
        }
        self.cycle.stride_length_m / self.cycle.cycle_duration_s
    }

    /// Compute foot placements at a given time in the gait cycle.
    ///
    /// `stride_origin`: world-space position of the stride center
    /// `heading`: normalized direction of travel (XZ plane)
    ///
    /// Returns a `FootPlacement` per limb with ground contact info.
    #[must_use]
    pub fn foot_placements(
        &self,
        time_s: f32,
        stride_origin: Vec3,
        heading: Vec3,
    ) -> Vec<FootPlacement> {
        let heading_norm = if heading.length_squared() > 1e-8 {
            heading.normalize()
        } else {
            Vec3::X
        };
        // Perpendicular (lateral) direction
        let lateral = Vec3::new(-heading_norm.z, 0.0, heading_norm.x);
        let limb_count = self.cycle.limb_phase_offsets.len();
        let half_stride = self.cycle.stride_length_m * 0.5;

        (0..limb_count)
            .map(|i| {
                let phase = self.limb_phase(i, time_s);
                let cycle_pos = (time_s / self.cycle.cycle_duration_s).fract();
                let limb_pos = (cycle_pos + self.cycle.limb_phase_offsets[i]).fract();

                // Forward position along stride
                let forward_t = if limb_pos < self.cycle.duty_factor {
                    // Stance: foot moves backward relative to body
                    -(limb_pos / self.cycle.duty_factor - 0.5)
                } else {
                    // Swing: foot moves forward
                    let swing_t =
                        (limb_pos - self.cycle.duty_factor) / (1.0 - self.cycle.duty_factor);
                    swing_t - 0.5
                };
                let forward_offset = heading_norm * (forward_t * half_stride);

                // Lateral offset: alternate sides for bipedal, spread for quadrupeds
                let side = if i % 2 == 0 { -1.0 } else { 1.0 };
                let lateral_spread = 0.1; // default 10cm lateral spread
                let lateral_offset = lateral * (side * lateral_spread);

                // Height: on ground during stance, arced during swing
                let height = if phase == GaitPhase::Stance {
                    0.0
                } else {
                    let swing_t =
                        (limb_pos - self.cycle.duty_factor) / (1.0 - self.cycle.duty_factor);
                    // Parabolic arc: max height at midswing
                    0.05 * 4.0 * swing_t * (1.0 - swing_t)
                };

                let ground_position =
                    stride_origin + forward_offset + lateral_offset + Vec3::new(0.0, height, 0.0);

                FootPlacement {
                    limb_index: i,
                    ground_position,
                    contact_normal: Vec3::Y,
                    phase,
                }
            })
            .collect()
    }
}

/// Foot placement data for a single limb at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootPlacement {
    pub limb_index: usize,
    pub ground_position: Vec3,
    pub contact_normal: Vec3,
    pub phase: GaitPhase,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_walk_speed() {
        let w = Gait::human_walk();
        assert!(
            (w.speed() - 1.4).abs() < 0.01,
            "walk speed should be ~1.4 m/s, got {}",
            w.speed()
        );
    }

    #[test]
    fn run_faster_than_walk() {
        assert!(Gait::human_run().speed() > Gait::human_walk().speed());
    }

    #[test]
    fn stance_and_swing_alternate() {
        let w = Gait::human_walk();
        let stance = w.limb_phase(0, 0.0);
        let swing = w.limb_phase(0, 0.8); // past duty factor
        assert_eq!(stance, GaitPhase::Stance);
        assert_eq!(swing, GaitPhase::Swing);
    }

    #[test]
    fn left_right_offset() {
        let w = Gait::human_walk();
        // At t=0, left is stance. At t=0, right should be swing (offset 0.5, duty 0.6)
        let left = w.limb_phase(0, 0.0);
        let right = w.limb_phase(1, 0.0);
        // Left at 0.0 → stance (0.0 < 0.6)
        // Right at 0.5 → stance (0.5 < 0.6)
        assert_eq!(left, GaitPhase::Stance);
        assert_eq!(right, GaitPhase::Stance); // both in stance during double support
    }

    #[test]
    fn quadruped_four_limbs() {
        let t = Gait::quadruped_trot();
        assert_eq!(t.cycle.limb_phase_offsets.len(), 4);
    }

    #[test]
    fn trot_diagonal_pairs() {
        let t = Gait::quadruped_trot();
        // LF and RH should have same phase (both 0.0)
        assert_eq!(t.cycle.limb_phase_offsets[0], t.cycle.limb_phase_offsets[3]);
        // RF and LH should have same phase (both 0.5)
        assert_eq!(t.cycle.limb_phase_offsets[1], t.cycle.limb_phase_offsets[2]);
    }

    #[test]
    fn canter_speed() {
        let c = Gait::quadruped_canter();
        let speed = c.speed();
        assert!(
            speed > 4.0 && speed < 8.0,
            "canter speed ~5.8 m/s, got {speed}"
        );
        assert_eq!(c.cycle.limb_phase_offsets.len(), 4);
    }

    #[test]
    fn gallop_faster_than_canter() {
        assert!(Gait::quadruped_gallop().speed() > Gait::quadruped_canter().speed());
    }

    #[test]
    fn gallop_four_beat() {
        let g = Gait::quadruped_gallop();
        let offsets = &g.cycle.limb_phase_offsets;
        // All four offsets should be different (4-beat gait)
        for i in 0..4 {
            for j in (i + 1)..4 {
                assert!(
                    (offsets[i] - offsets[j]).abs() > 0.01,
                    "gallop should be 4-beat: limb {} and {} have same phase",
                    i,
                    j
                );
            }
        }
    }

    #[test]
    fn foot_placements_count() {
        let walk = Gait::human_walk();
        let placements = walk.foot_placements(0.0, Vec3::ZERO, Vec3::X);
        assert_eq!(placements.len(), 2, "biped should have 2 foot placements");

        let trot = Gait::quadruped_trot();
        let placements = trot.foot_placements(0.0, Vec3::ZERO, Vec3::X);
        assert_eq!(
            placements.len(),
            4,
            "quadruped should have 4 foot placements"
        );
    }

    #[test]
    fn foot_placements_stance_on_ground() {
        let walk = Gait::human_walk();
        let placements = walk.foot_placements(0.0, Vec3::ZERO, Vec3::X);
        for fp in &placements {
            if fp.phase == GaitPhase::Stance {
                assert!(
                    fp.ground_position.y.abs() < 0.001,
                    "stance foot should be on ground, y={}",
                    fp.ground_position.y
                );
            }
        }
    }

    #[test]
    fn foot_placements_swing_elevated() {
        let walk = Gait::human_walk();
        // Find a time where left foot is in swing
        let placements = walk.foot_placements(0.75, Vec3::ZERO, Vec3::X);
        for fp in &placements {
            if fp.phase == GaitPhase::Swing {
                assert!(
                    fp.ground_position.y > 0.0,
                    "swing foot should be elevated, y={}",
                    fp.ground_position.y
                );
            }
        }
    }

    #[test]
    fn all_presets_valid() {
        let gaits = [
            Gait::human_walk(),
            Gait::human_run(),
            Gait::quadruped_walk(),
            Gait::quadruped_trot(),
            Gait::quadruped_canter(),
            Gait::quadruped_gallop(),
        ];
        for gait in &gaits {
            assert!(
                gait.cycle.cycle_duration_s > 0.0,
                "{}: invalid duration",
                gait.name
            );
            assert!(gait.cycle.duty_factor > 0.0, "{}: invalid duty", gait.name);
            assert!(
                gait.cycle.stride_length_m > 0.0,
                "{}: invalid stride",
                gait.name
            );
            assert!(gait.speed() > 0.0, "{}: invalid speed", gait.name);
            assert!(
                gait.speed_range.0 < gait.speed_range.1,
                "{}: invalid range",
                gait.name
            );
        }
    }
}
