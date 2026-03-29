use serde::{Deserialize, Serialize};

/// Muscle group classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MuscleGroup { Flexor, Extensor, Abductor, Adductor, Rotator, Sphincter }

/// A muscle connecting two bones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Muscle {
    pub name: String,
    pub group: MuscleGroup,
    pub origin_bone: super::skeleton::BoneId,
    pub insertion_bone: super::skeleton::BoneId,
    pub max_force_n: f32,       // maximum isometric force (Newtons)
    pub rest_length: f32,       // meters
    pub activation: f32,        // 0.0 = relaxed, 1.0 = fully contracted
}

impl Muscle {
    /// Force output based on current activation and length.
    ///
    /// Hill muscle model (simplified): F = F_max × activation × force-length factor
    #[must_use]
    pub fn current_force(&self, current_length: f32) -> f32 {
        if self.rest_length <= 0.0 { return 0.0; }
        let length_ratio = current_length / self.rest_length;
        // Force-length relationship (Gaussian around optimal length)
        let fl_factor = (-(length_ratio - 1.0).powi(2) / 0.18).exp();
        self.max_force_n * self.activation * fl_factor
    }

    /// Set activation level (clamped 0-1).
    pub fn set_activation(&mut self, level: f32) {
        self.activation = level.clamp(0.0, 1.0);
    }

    /// Is this muscle an antagonist to the other? (opposite group)
    #[must_use]
    pub fn is_antagonist(&self, other: &Self) -> bool {
        matches!(
            (self.group, other.group),
            (MuscleGroup::Flexor, MuscleGroup::Extensor) | (MuscleGroup::Extensor, MuscleGroup::Flexor) |
            (MuscleGroup::Abductor, MuscleGroup::Adductor) | (MuscleGroup::Adductor, MuscleGroup::Abductor)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skeleton::BoneId;

    fn test_muscle() -> Muscle {
        Muscle {
            name: "biceps".into(), group: MuscleGroup::Flexor,
            origin_bone: BoneId(0), insertion_bone: BoneId(1),
            max_force_n: 300.0, rest_length: 0.3, activation: 0.0,
        }
    }

    #[test]
    fn zero_activation_no_force() {
        let m = test_muscle();
        assert_eq!(m.current_force(0.3), 0.0);
    }

    #[test]
    fn max_force_at_rest_length() {
        let mut m = test_muscle();
        m.activation = 1.0;
        let f = m.current_force(0.3); // at rest length
        assert!((f - 300.0).abs() < 1.0, "max force at rest length, got {f}");
    }

    #[test]
    fn force_decreases_when_stretched() {
        let mut m = test_muscle();
        m.activation = 1.0;
        let at_rest = m.current_force(0.3);
        let stretched = m.current_force(0.45); // 50% stretched
        assert!(stretched < at_rest, "force should decrease when stretched");
    }

    #[test]
    fn flexor_extensor_antagonist() {
        let flexor = test_muscle();
        let extensor = Muscle { group: MuscleGroup::Extensor, ..test_muscle() };
        assert!(flexor.is_antagonist(&extensor));
    }

    #[test]
    fn same_group_not_antagonist() {
        let m1 = test_muscle();
        let m2 = test_muscle();
        assert!(!m1.is_antagonist(&m2));
    }

    #[test]
    fn activation_clamps() {
        let mut m = test_muscle();
        m.set_activation(1.5);
        assert_eq!(m.activation, 1.0);
        m.set_activation(-0.5);
        assert_eq!(m.activation, 0.0);
    }
}
