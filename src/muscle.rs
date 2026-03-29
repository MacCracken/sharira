use serde::{Deserialize, Serialize};
use tracing::trace;

/// Muscle group classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MuscleGroup {
    Flexor,
    Extensor,
    Abductor,
    Adductor,
    Rotator,
    Sphincter,
}

/// A muscle connecting two bones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Muscle {
    pub name: String,
    pub group: MuscleGroup,
    pub origin_bone: super::skeleton::BoneId,
    pub insertion_bone: super::skeleton::BoneId,
    pub max_force_n: f32,     // maximum isometric force (Newtons)
    pub rest_length: f32,     // optimal fiber length (meters)
    pub activation: f32,      // 0.0 = relaxed, 1.0 = fully contracted
    pub max_velocity: f32,    // maximum shortening velocity (lengths/s, typ. 10.0)
    pub passive_strain: f32,  // strain at which passive force equals max_force (typ. 0.6)
    pub pennation_angle: f32, // fiber pennation angle at rest (radians, typ. 0.0)
}

impl Muscle {
    /// Create a new muscle with default dynamics parameters.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        origin_bone: super::skeleton::BoneId,
        insertion_bone: super::skeleton::BoneId,
        group: MuscleGroup,
        max_force_n: f32,
        rest_length: f32,
    ) -> Self {
        Self {
            name: name.into(),
            group,
            origin_bone,
            insertion_bone,
            max_force_n,
            rest_length,
            activation: 0.0,
            max_velocity: 10.0,
            passive_strain: 0.6,
            pennation_angle: 0.0,
        }
    }

    /// Active force-length factor (Gaussian, Thelen 2003).
    ///
    /// Width parameter γ=0.45 matches published muscle physiology data.
    #[must_use]
    #[inline]
    fn active_force_length(length_ratio: f32) -> f32 {
        (-(length_ratio - 1.0).powi(2) / 0.45).exp()
    }

    /// Passive force-length factor (exponential, engages beyond rest length).
    ///
    /// Returns force as fraction of max isometric force.
    #[must_use]
    #[inline]
    fn passive_force_length(length_ratio: f32, passive_strain: f32) -> f32 {
        if length_ratio <= 1.0 || passive_strain <= 0.0 {
            return 0.0;
        }
        let k_pe = 4.0;
        let normalized = (length_ratio - 1.0) / passive_strain;
        ((k_pe * normalized).exp() - 1.0) / (k_pe.exp() - 1.0)
    }

    /// Force-velocity factor (Hill 1938).
    ///
    /// `velocity` is in optimal-fiber-lengths per second (negative = shortening).
    /// Returns force multiplier: <1.0 for shortening, up to ~1.4 for lengthening.
    #[must_use]
    #[inline]
    fn force_velocity(velocity_normalized: f32, max_velocity: f32) -> f32 {
        if max_velocity <= 0.0 {
            return 1.0;
        }
        let v_norm = velocity_normalized / max_velocity;
        if v_norm <= 0.0 {
            // Concentric (shortening): force decreases with speed
            let k = 0.25; // curvature constant
            (1.0 + v_norm) / (1.0 - v_norm / k)
        } else {
            // Eccentric (lengthening): force increases, capped at 1.4x
            let eccentric_max = 1.4;
            eccentric_max - (eccentric_max - 1.0) * (1.0 - v_norm).max(0.0)
        }
    }

    /// Force output based on current activation, length, and velocity.
    ///
    /// Full Hill muscle model: F = F_max × (activation × fl_active × fv + fl_passive)
    /// Pennation angle reduces effective force by cos(pennation).
    #[must_use]
    pub fn current_force(&self, current_length: f32) -> f32 {
        self.force_at(current_length, 0.0)
    }

    /// Force output with explicit velocity (lengths/s, negative = shortening).
    ///
    /// Full Hill model: F = F_max × (activation × fl_active × fv + fl_passive) × cos(pennation)
    #[must_use]
    pub fn force_at(&self, current_length: f32, velocity: f32) -> f32 {
        if self.rest_length <= 0.0 {
            return 0.0;
        }
        let length_ratio = current_length / self.rest_length;

        let fl_active = Self::active_force_length(length_ratio);
        let fl_passive = Self::passive_force_length(length_ratio, self.passive_strain);
        let fv = Self::force_velocity(velocity, self.max_velocity);
        let pennation_cos = self.pennation_angle.cos();

        self.max_force_n * (self.activation * fl_active * fv + fl_passive) * pennation_cos
    }

    /// Set activation level (clamped 0-1).
    pub fn set_activation(&mut self, level: f32) {
        self.activation = level.clamp(0.0, 1.0);
        trace!(muscle = %self.name, activation = self.activation, "activation set");
    }

    /// Is this muscle an antagonist to the other? (opposite group)
    #[must_use]
    pub fn is_antagonist(&self, other: &Self) -> bool {
        matches!(
            (self.group, other.group),
            (MuscleGroup::Flexor, MuscleGroup::Extensor)
                | (MuscleGroup::Extensor, MuscleGroup::Flexor)
                | (MuscleGroup::Abductor, MuscleGroup::Adductor)
                | (MuscleGroup::Adductor, MuscleGroup::Abductor)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skeleton::BoneId;

    fn test_muscle() -> Muscle {
        Muscle::new(
            "biceps",
            BoneId(0),
            BoneId(1),
            MuscleGroup::Flexor,
            300.0,
            0.3,
        )
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
    fn active_force_decreases_when_stretched() {
        // Active force-length factor should decrease away from optimal length
        let fl_at_rest = Muscle::active_force_length(1.0);
        let fl_stretched = Muscle::active_force_length(1.5);
        assert!(
            fl_stretched < fl_at_rest,
            "active fl should decrease when stretched: rest={fl_at_rest}, stretched={fl_stretched}"
        );
    }

    #[test]
    fn passive_tension_when_stretched() {
        let m = test_muscle(); // activation = 0
        // At rest length, no passive force
        assert_eq!(m.current_force(0.3), 0.0);
        // Stretched beyond rest, passive force engages
        let stretched = m.current_force(0.48); // 1.6x rest length
        assert!(
            stretched > 0.0,
            "passive tension should engage when stretched beyond rest"
        );
    }

    #[test]
    fn force_velocity_shortening_reduces_force() {
        let mut m = test_muscle();
        m.activation = 1.0;
        let isometric = m.force_at(0.3, 0.0);
        let shortening = m.force_at(0.3, -5.0); // shortening at 5 lengths/s
        assert!(
            shortening < isometric,
            "shortening should reduce force: isometric={isometric}, shortening={shortening}"
        );
    }

    #[test]
    fn force_velocity_lengthening_increases_force() {
        let mut m = test_muscle();
        m.activation = 1.0;
        let isometric = m.force_at(0.3, 0.0);
        let lengthening = m.force_at(0.3, 2.0); // lengthening at 2 lengths/s
        assert!(
            lengthening > isometric,
            "lengthening should increase force: isometric={isometric}, lengthening={lengthening}"
        );
    }

    #[test]
    fn pennation_reduces_force() {
        let mut m = test_muscle();
        m.activation = 1.0;
        let no_pennation = m.current_force(0.3);
        m.pennation_angle = 0.5; // ~28.6 degrees
        let with_pennation = m.current_force(0.3);
        assert!(
            with_pennation < no_pennation,
            "pennation should reduce force"
        );
    }

    #[test]
    fn flexor_extensor_antagonist() {
        let flexor = test_muscle();
        let extensor = Muscle {
            group: MuscleGroup::Extensor,
            ..test_muscle()
        };
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
