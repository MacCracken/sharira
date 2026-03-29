//! Pose representation — joint angles separate from skeleton structure.
//!
//! A [`Skeleton`](crate::Skeleton) defines what a body IS (fixed structure).
//! A [`Pose`] defines what configuration it's IN (mutable joint angles).

use hisab::Quat;
use serde::{Deserialize, Serialize};

use crate::skeleton::BoneId;

/// Joint angle storage indexed by bone ID.
///
/// Each entry represents the local rotation override for a bone.
/// `None` means the bone uses its rest-pose rotation from the skeleton.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pose {
    /// Joint rotations indexed by `bone_id.0`. `None` = rest pose (identity override).
    joint_rotations: Vec<Option<Quat>>,
}

impl Pose {
    /// Create a rest pose (all joints at identity) for a skeleton with `bone_count` bones.
    #[must_use]
    pub fn rest(bone_count: usize) -> Self {
        Self {
            joint_rotations: vec![None; bone_count],
        }
    }

    /// Set a joint rotation for a bone.
    pub fn set_joint(&mut self, bone_id: BoneId, rotation: Quat) {
        let idx = bone_id.0 as usize;
        if idx >= self.joint_rotations.len() {
            self.joint_rotations.resize(idx + 1, None);
        }
        self.joint_rotations[idx] = Some(rotation);
    }

    /// Get the joint rotation for a bone. Returns `Quat::IDENTITY` if unset (rest pose).
    #[must_use]
    #[inline]
    pub fn get_joint(&self, bone_id: BoneId) -> Quat {
        let idx = bone_id.0 as usize;
        self.joint_rotations
            .get(idx)
            .copied()
            .flatten()
            .unwrap_or(Quat::IDENTITY)
    }

    /// Clear a joint back to rest pose.
    pub fn clear_joint(&mut self, bone_id: BoneId) {
        let idx = bone_id.0 as usize;
        if let Some(slot) = self.joint_rotations.get_mut(idx) {
            *slot = None;
        }
    }

    /// Number of joint slots.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.joint_rotations.len()
    }

    /// Whether the pose has no joint slots.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.joint_rotations.is_empty()
    }

    /// Blend two poses using spherical interpolation.
    /// `t = 0.0` returns `a`, `t = 1.0` returns `b`.
    #[must_use]
    pub fn blend(a: &Pose, b: &Pose, t: f32) -> Pose {
        let len = a.joint_rotations.len().max(b.joint_rotations.len());
        let mut result = Vec::with_capacity(len);
        for i in 0..len {
            let qa = a
                .joint_rotations
                .get(i)
                .copied()
                .flatten()
                .unwrap_or(Quat::IDENTITY);
            let qb = b
                .joint_rotations
                .get(i)
                .copied()
                .flatten()
                .unwrap_or(Quat::IDENTITY);
            let blended = qa.slerp(qb, t);
            // Only store if non-identity
            if (blended.x.abs() + blended.y.abs() + blended.z.abs()) > 1e-6
                || (blended.w - 1.0).abs() > 1e-6
            {
                result.push(Some(blended));
            } else {
                result.push(None);
            }
        }
        Pose {
            joint_rotations: result,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_4;

    #[test]
    fn rest_pose_is_identity() {
        let pose = Pose::rest(5);
        assert_eq!(pose.len(), 5);
        for i in 0..5 {
            assert_eq!(pose.get_joint(BoneId(i)), Quat::IDENTITY);
        }
    }

    #[test]
    fn set_and_get_joint() {
        let mut pose = Pose::rest(3);
        let rot = Quat::from_rotation_z(FRAC_PI_4);
        pose.set_joint(BoneId(1), rot);
        let got = pose.get_joint(BoneId(1));
        assert!((got.x - rot.x).abs() < 1e-6);
        assert!((got.w - rot.w).abs() < 1e-6);
    }

    #[test]
    fn get_unset_returns_identity() {
        let pose = Pose::rest(3);
        assert_eq!(pose.get_joint(BoneId(0)), Quat::IDENTITY);
    }

    #[test]
    fn get_out_of_bounds_returns_identity() {
        let pose = Pose::rest(2);
        assert_eq!(pose.get_joint(BoneId(99)), Quat::IDENTITY);
    }

    #[test]
    fn clear_joint() {
        let mut pose = Pose::rest(3);
        pose.set_joint(BoneId(1), Quat::from_rotation_z(1.0));
        pose.clear_joint(BoneId(1));
        assert_eq!(pose.get_joint(BoneId(1)), Quat::IDENTITY);
    }

    #[test]
    fn blend_endpoints() {
        let mut a = Pose::rest(2);
        let mut b = Pose::rest(2);
        let rot = Quat::from_rotation_z(FRAC_PI_4);
        a.set_joint(BoneId(0), Quat::IDENTITY);
        b.set_joint(BoneId(0), rot);

        // t=0 should match a
        let at_zero = Pose::blend(&a, &b, 0.0);
        let q0 = at_zero.get_joint(BoneId(0));
        assert!((q0.w - 1.0).abs() < 1e-5, "blend at t=0 should be identity");

        // t=1 should match b
        let at_one = Pose::blend(&a, &b, 1.0);
        let q1 = at_one.get_joint(BoneId(0));
        assert!(
            (q1.z - rot.z).abs() < 1e-5,
            "blend at t=1 should match target"
        );
    }

    #[test]
    fn blend_midpoint() {
        let mut a = Pose::rest(1);
        let mut b = Pose::rest(1);
        let rot = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        a.set_joint(BoneId(0), Quat::IDENTITY);
        b.set_joint(BoneId(0), rot);

        let mid = Pose::blend(&a, &b, 0.5);
        let q = mid.get_joint(BoneId(0));
        // Midpoint of identity and 90° should be ~45°
        let expected = Quat::from_rotation_z(FRAC_PI_4);
        assert!(
            (q.z - expected.z).abs() < 1e-4,
            "blend midpoint should be ~45°"
        );
    }

    #[test]
    fn auto_resize_on_set() {
        let mut pose = Pose::rest(2);
        pose.set_joint(BoneId(10), Quat::from_rotation_x(1.0));
        assert_eq!(pose.len(), 11);
        assert_eq!(pose.get_joint(BoneId(5)), Quat::IDENTITY); // gap is identity
    }
}
