use serde::{Deserialize, Serialize};

/// Body plan classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum BodyPlan {
    Bipedal,    // human, bird
    Quadruped,  // wolf, horse, cat
    Hexapod,    // insect (6 legs)
    Octopod,    // spider (8 legs)
    Serpentine, // snake (no limbs)
    Avian,      // bird (wings + legs)
    Aquatic,    // fish (fins)
    Centipede,  // many legs
}

impl BodyPlan {
    /// Number of locomotion limbs.
    #[must_use]
    pub fn limb_count(&self) -> u8 {
        match self {
            Self::Bipedal => 2,
            Self::Quadruped => 4,
            Self::Hexapod => 6,
            Self::Octopod => 8,
            Self::Serpentine => 0,
            Self::Avian => 4,
            Self::Aquatic => 0,
            Self::Centipede => 30,
        }
    }

    /// Can this body plan fly?
    #[must_use]
    pub fn can_fly(&self) -> bool {
        matches!(self, Self::Avian)
    }

    /// Can this body plan swim?
    #[must_use]
    pub fn can_swim(&self) -> bool {
        matches!(self, Self::Aquatic | Self::Serpentine)
    }

    /// Typical joint count (approximate).
    #[must_use]
    pub fn typical_joint_count(&self) -> u16 {
        match self {
            Self::Bipedal => 20,
            Self::Quadruped => 30,
            Self::Hexapod => 18,
            Self::Octopod => 24,
            Self::Serpentine => 200,
            Self::Avian => 25,
            Self::Aquatic => 50,
            Self::Centipede => 90,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bipedal_two_legs() {
        assert_eq!(BodyPlan::Bipedal.limb_count(), 2);
    }

    #[test]
    fn snake_no_limbs() {
        assert_eq!(BodyPlan::Serpentine.limb_count(), 0);
    }

    #[test]
    fn bird_can_fly() {
        assert!(BodyPlan::Avian.can_fly());
        assert!(!BodyPlan::Quadruped.can_fly());
    }

    #[test]
    fn fish_can_swim() {
        assert!(BodyPlan::Aquatic.can_swim());
        assert!(!BodyPlan::Bipedal.can_swim());
    }

    #[test]
    fn snake_many_joints() {
        assert!(
            BodyPlan::Serpentine.typical_joint_count() > BodyPlan::Bipedal.typical_joint_count()
        );
    }

    #[test]
    fn hexapod_six_legs() {
        assert_eq!(BodyPlan::Hexapod.limb_count(), 6);
    }
}
