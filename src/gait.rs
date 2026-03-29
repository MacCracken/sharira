use serde::{Deserialize, Serialize};

/// Gait phase within a cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GaitPhase { Stance, Swing, DoubleSupport, Flight }

/// Locomotion gait type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum GaitType { Walk, Trot, Canter, Gallop, Crawl, Slither, Hop, Fly, Swim }

/// A gait cycle definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaitCycle {
    pub gait_type: GaitType,
    pub cycle_duration_s: f32,       // one full cycle
    pub duty_factor: f32,            // fraction of cycle foot is on ground (0-1)
    pub stride_length_m: f32,
    pub limb_phase_offsets: Vec<f32>, // phase offset per limb (0.0-1.0)
}

/// A complete gait definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gait {
    pub name: String,
    pub gait_type: GaitType,
    pub speed_range: (f32, f32),   // min/max speed in m/s
    pub cycle: GaitCycle,
}

impl Gait {
    /// Human walk (~1.4 m/s, duty factor ~0.6).
    #[must_use]
    pub fn human_walk() -> Self {
        Self {
            name: "walk".into(), gait_type: GaitType::Walk,
            speed_range: (0.5, 2.0),
            cycle: GaitCycle {
                gait_type: GaitType::Walk, cycle_duration_s: 1.0, duty_factor: 0.6,
                stride_length_m: 1.4, limb_phase_offsets: vec![0.0, 0.5], // left, right
            },
        }
    }

    /// Human run (~3 m/s, duty factor ~0.35, includes flight phase).
    #[must_use]
    pub fn human_run() -> Self {
        Self {
            name: "run".into(), gait_type: GaitType::Gallop,
            speed_range: (2.0, 10.0),
            cycle: GaitCycle {
                gait_type: GaitType::Gallop, cycle_duration_s: 0.7, duty_factor: 0.35,
                stride_length_m: 2.5, limb_phase_offsets: vec![0.0, 0.5],
            },
        }
    }

    /// Quadruped walk (horse, duty factor ~0.75).
    #[must_use]
    pub fn quadruped_walk() -> Self {
        Self {
            name: "quadruped_walk".into(), gait_type: GaitType::Walk,
            speed_range: (0.5, 2.0),
            cycle: GaitCycle {
                gait_type: GaitType::Walk, cycle_duration_s: 1.2, duty_factor: 0.75,
                stride_length_m: 1.8,
                limb_phase_offsets: vec![0.0, 0.5, 0.25, 0.75], // LF, RF, LH, RH
            },
        }
    }

    /// Quadruped trot (diagonal pairs move together).
    #[must_use]
    pub fn quadruped_trot() -> Self {
        Self {
            name: "trot".into(), gait_type: GaitType::Trot,
            speed_range: (2.0, 5.0),
            cycle: GaitCycle {
                gait_type: GaitType::Trot, cycle_duration_s: 0.8, duty_factor: 0.5,
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

    /// Speed from stride length and cycle time.
    #[must_use]
    #[inline]
    pub fn speed(&self) -> f32 {
        if self.cycle.cycle_duration_s <= 0.0 { return 0.0; }
        self.cycle.stride_length_m / self.cycle.cycle_duration_s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_walk_speed() {
        let w = Gait::human_walk();
        assert!((w.speed() - 1.4).abs() < 0.01, "walk speed should be ~1.4 m/s, got {}", w.speed());
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
}
