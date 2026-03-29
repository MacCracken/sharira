use serde::{Deserialize, Serialize};

/// Joint type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum JointType {
    Ball,       // 3 DOF (shoulder, hip)
    Hinge,      // 1 DOF (elbow, knee)
    Pivot,      // 1 DOF rotation (neck atlas)
    Saddle,     // 2 DOF (thumb)
    Fixed,      // 0 DOF (skull sutures)
    Planar,     // 2 DOF sliding (wrist)
}

impl JointType {
    /// Degrees of freedom for this joint type.
    #[must_use]
    pub fn degrees_of_freedom(&self) -> u8 {
        match self {
            Self::Ball => 3, Self::Hinge => 1, Self::Pivot => 1,
            Self::Saddle => 2, Self::Fixed => 0, Self::Planar => 2,
        }
    }
}

/// Angular limits for a joint axis (radians).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AxisLimit {
    pub min_rad: f32,
    pub max_rad: f32,
}

impl AxisLimit {
    #[must_use]
    pub fn new(min_deg: f32, max_deg: f32) -> Self {
        Self { min_rad: min_deg.to_radians(), max_rad: max_deg.to_radians() }
    }

    /// Clamp an angle to this limit.
    #[must_use]
    #[inline]
    pub fn clamp(&self, angle_rad: f32) -> f32 {
        angle_rad.clamp(self.min_rad, self.max_rad)
    }

    /// Range of motion in degrees.
    #[must_use]
    #[inline]
    pub fn range_degrees(&self) -> f32 {
        (self.max_rad - self.min_rad).to_degrees()
    }
}

/// Joint limits for all axes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct JointLimits {
    pub x: Option<AxisLimit>,  // pitch / flexion-extension
    pub y: Option<AxisLimit>,  // yaw / abduction-adduction
    pub z: Option<AxisLimit>,  // roll / rotation
}

impl JointLimits {
    /// Fully free (no limits).
    #[must_use]
    pub fn free() -> Self {
        Self { x: None, y: None, z: None }
    }

    /// Hinge joint (one axis limited).
    #[must_use]
    pub fn hinge(min_deg: f32, max_deg: f32) -> Self {
        Self { x: Some(AxisLimit::new(min_deg, max_deg)), y: None, z: None }
    }
}

/// A joint connecting two bones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Joint {
    pub name: String,
    pub joint_type: JointType,
    pub parent_bone: super::skeleton::BoneId,
    pub child_bone: super::skeleton::BoneId,
    pub limits: JointLimits,
    pub stiffness: f32,    // resistance to movement (0=free, 1=rigid)
    pub damping: f32,      // velocity decay (0=none, 1=full)
}

impl Joint {
    /// Human knee (hinge, 0° to ~135° flexion).
    #[must_use]
    pub fn human_knee(parent: super::skeleton::BoneId, child: super::skeleton::BoneId) -> Self {
        Self {
            name: "knee".into(), joint_type: JointType::Hinge,
            parent_bone: parent, child_bone: child,
            limits: JointLimits::hinge(0.0, 135.0),
            stiffness: 0.1, damping: 0.3,
        }
    }

    /// Human shoulder (ball, wide range).
    #[must_use]
    pub fn human_shoulder(parent: super::skeleton::BoneId, child: super::skeleton::BoneId) -> Self {
        Self {
            name: "shoulder".into(), joint_type: JointType::Ball,
            parent_bone: parent, child_bone: child,
            limits: JointLimits {
                x: Some(AxisLimit::new(-60.0, 180.0)),
                y: Some(AxisLimit::new(-45.0, 180.0)),
                z: Some(AxisLimit::new(-90.0, 90.0)),
            },
            stiffness: 0.05, damping: 0.2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joint_dof() {
        assert_eq!(JointType::Ball.degrees_of_freedom(), 3);
        assert_eq!(JointType::Hinge.degrees_of_freedom(), 1);
        assert_eq!(JointType::Fixed.degrees_of_freedom(), 0);
    }

    #[test]
    fn axis_limit_clamp() {
        let limit = AxisLimit::new(0.0, 90.0);
        let clamped = limit.clamp(2.0); // ~114.6°, should clamp to ~90° = 1.5708
        assert!(clamped <= limit.max_rad + 0.001);
    }

    #[test]
    fn axis_range() {
        let limit = AxisLimit::new(-45.0, 135.0);
        assert!((limit.range_degrees() - 180.0).abs() < 0.1);
    }

    #[test]
    fn knee_is_hinge() {
        let knee = Joint::human_knee(super::super::skeleton::BoneId(0), super::super::skeleton::BoneId(1));
        assert_eq!(knee.joint_type, JointType::Hinge);
        assert!(knee.limits.x.is_some());
        assert!(knee.limits.y.is_none());
    }

    #[test]
    fn shoulder_is_ball() {
        let shoulder = Joint::human_shoulder(super::super::skeleton::BoneId(0), super::super::skeleton::BoneId(1));
        assert_eq!(shoulder.joint_type, JointType::Ball);
        assert_eq!(shoulder.joint_type.degrees_of_freedom(), 3);
    }

    #[test]
    fn hinge_limits() {
        let limits = JointLimits::hinge(0.0, 135.0);
        assert!(limits.x.is_some());
        assert!((limits.x.unwrap().range_degrees() - 135.0).abs() < 0.1);
    }
}
