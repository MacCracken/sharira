use hisab::Quat;
use serde::{Deserialize, Serialize};

/// Joint type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum JointType {
    Ball,   // 3 DOF (shoulder, hip)
    Hinge,  // 1 DOF (elbow, knee)
    Pivot,  // 1 DOF rotation (neck atlas)
    Saddle, // 2 DOF (thumb)
    Fixed,  // 0 DOF (skull sutures)
    Planar, // 2 DOF sliding (wrist)
}

impl JointType {
    /// Degrees of freedom for this joint type.
    #[must_use]
    pub fn degrees_of_freedom(&self) -> u8 {
        match self {
            Self::Ball => 3,
            Self::Hinge => 1,
            Self::Pivot => 1,
            Self::Saddle => 2,
            Self::Fixed => 0,
            Self::Planar => 2,
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
        Self {
            min_rad: min_deg.to_radians(),
            max_rad: max_deg.to_radians(),
        }
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
    pub x: Option<AxisLimit>, // pitch / flexion-extension
    pub y: Option<AxisLimit>, // yaw / abduction-adduction
    pub z: Option<AxisLimit>, // roll / rotation
}

impl JointLimits {
    /// Fully free (no limits).
    #[must_use]
    pub fn free() -> Self {
        Self {
            x: None,
            y: None,
            z: None,
        }
    }

    /// Hinge joint (one axis limited).
    #[must_use]
    pub fn hinge(min_deg: f32, max_deg: f32) -> Self {
        Self {
            x: Some(AxisLimit::new(min_deg, max_deg)),
            y: None,
            z: None,
        }
    }

    /// Clamp a rotation quaternion to these joint limits.
    ///
    /// Decomposes the quaternion into XYZ Euler angles, clamps each axis
    /// to its limit (if set), and recomposes. Unconstrained axes pass through.
    #[must_use]
    pub fn clamp_rotation(&self, rotation: Quat) -> Quat {
        let (x, y, z) = rotation.to_euler(hisab::transforms::glam::EulerRot::XYZ);
        let cx = self.x.map_or(x, |lim| lim.clamp(x));
        let cy = self.y.map_or(y, |lim| lim.clamp(y));
        let cz = self.z.map_or(z, |lim| lim.clamp(z));
        Quat::from_euler(hisab::transforms::glam::EulerRot::XYZ, cx, cy, cz)
    }

    /// Compute the total angular violation (radians) of a rotation against these limits.
    ///
    /// Returns 0.0 if the rotation is within limits. Otherwise returns the sum
    /// of per-axis violations (how far each axis exceeds its limit).
    #[must_use]
    pub fn violation(&self, rotation: Quat) -> f32 {
        let (x, y, z) = rotation.to_euler(hisab::transforms::glam::EulerRot::XYZ);
        let mut total = 0.0_f32;
        if let Some(lim) = &self.x {
            let clamped = lim.clamp(x);
            total += (x - clamped).abs();
        }
        if let Some(lim) = &self.y {
            let clamped = lim.clamp(y);
            total += (y - clamped).abs();
        }
        if let Some(lim) = &self.z {
            let clamped = lim.clamp(z);
            total += (z - clamped).abs();
        }
        total
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
    pub stiffness: f32, // resistance to movement (0=free, 1=rigid)
    pub damping: f32,   // velocity decay (0=none, 1=full)
}

impl Joint {
    /// Human knee (hinge, 0° to ~135° flexion).
    #[must_use]
    pub fn human_knee(parent: super::skeleton::BoneId, child: super::skeleton::BoneId) -> Self {
        Self {
            name: "knee".into(),
            joint_type: JointType::Hinge,
            parent_bone: parent,
            child_bone: child,
            limits: JointLimits::hinge(0.0, 135.0),
            stiffness: 0.1,
            damping: 0.3,
        }
    }

    /// Clamp a rotation quaternion to this joint's limits.
    #[must_use]
    #[inline]
    pub fn clamp_rotation(&self, rotation: Quat) -> Quat {
        self.limits.clamp_rotation(rotation)
    }

    /// Compute angular violation (radians) of a rotation against this joint's limits.
    /// Returns 0.0 if within limits.
    #[must_use]
    #[inline]
    pub fn violation(&self, rotation: Quat) -> f32 {
        self.limits.violation(rotation)
    }

    /// Human shoulder (ball, wide range).
    #[must_use]
    pub fn human_shoulder(parent: super::skeleton::BoneId, child: super::skeleton::BoneId) -> Self {
        Self {
            name: "shoulder".into(),
            joint_type: JointType::Ball,
            parent_bone: parent,
            child_bone: child,
            limits: JointLimits {
                x: Some(AxisLimit::new(-60.0, 180.0)),
                y: Some(AxisLimit::new(-45.0, 180.0)),
                z: Some(AxisLimit::new(-90.0, 90.0)),
            },
            stiffness: 0.05,
            damping: 0.2,
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
        let knee = Joint::human_knee(
            super::super::skeleton::BoneId(0),
            super::super::skeleton::BoneId(1),
        );
        assert_eq!(knee.joint_type, JointType::Hinge);
        assert!(knee.limits.x.is_some());
        assert!(knee.limits.y.is_none());
    }

    #[test]
    fn shoulder_is_ball() {
        let shoulder = Joint::human_shoulder(
            super::super::skeleton::BoneId(0),
            super::super::skeleton::BoneId(1),
        );
        assert_eq!(shoulder.joint_type, JointType::Ball);
        assert_eq!(shoulder.joint_type.degrees_of_freedom(), 3);
    }

    #[test]
    fn hinge_limits() {
        let limits = JointLimits::hinge(0.0, 135.0);
        assert!(limits.x.is_some());
        assert!((limits.x.unwrap().range_degrees() - 135.0).abs() < 0.1);
    }

    #[test]
    fn clamp_rotation_within_limits() {
        let limits = JointLimits::hinge(0.0, 90.0);
        let rot = Quat::from_rotation_x(0.5); // ~28.6°, within 0-90°
        let clamped = limits.clamp_rotation(rot);
        assert!(
            rot.dot(clamped).abs() > 0.999,
            "rotation within limits should not change"
        );
    }

    #[test]
    fn clamp_rotation_exceeds_limits() {
        let limits = JointLimits::hinge(0.0, 90.0);
        let rot = Quat::from_rotation_x(2.5); // ~143°, exceeds 90°
        let clamped = limits.clamp_rotation(rot);
        let (x, _, _) = clamped.to_euler(hisab::transforms::glam::EulerRot::XYZ);
        let max_rad = 90.0_f32.to_radians();
        assert!(
            (x - max_rad).abs() < 0.01,
            "should clamp to 90°, got {:.1}°",
            x.to_degrees()
        );
    }

    #[test]
    fn violation_zero_within_limits() {
        let limits = JointLimits::hinge(0.0, 90.0);
        let rot = Quat::from_rotation_x(0.5);
        assert!(
            limits.violation(rot) < 0.01,
            "should have zero violation within limits"
        );
    }

    #[test]
    fn violation_positive_outside_limits() {
        let limits = JointLimits::hinge(0.0, 90.0);
        let rot = Quat::from_rotation_x(2.0); // ~115°, exceeds by ~25°
        let v = limits.violation(rot);
        assert!(v > 0.1, "should have positive violation, got {v}");
    }

    #[test]
    fn free_limits_no_clamping() {
        let limits = JointLimits::free();
        let rot = Quat::from_rotation_x(3.0);
        let clamped = limits.clamp_rotation(rot);
        assert!(
            rot.dot(clamped).abs() > 0.999,
            "free limits should not clamp"
        );
    }

    #[test]
    fn joint_clamp_delegates_to_limits() {
        let knee = Joint::human_knee(
            super::super::skeleton::BoneId(0),
            super::super::skeleton::BoneId(1),
        );
        // Knee is 0-135° hinge. -0.5 rad ≈ -28.6° should clamp to 0°
        let rot = Quat::from_rotation_x(-0.5);
        let clamped = knee.clamp_rotation(rot);
        let (x, _, _) = clamped.to_euler(hisab::transforms::glam::EulerRot::XYZ);
        assert!(x.abs() < 0.01, "knee should clamp negative angle to 0°");
    }
}
