//! Body state — aggregation of skeleton, pose, joints, muscles, and computed transforms.
//!
//! A [`Body`] is the complete runtime representation of a physical body. It owns the
//! skeleton structure, current pose, attached joints and muscles, and caches computed
//! world-space transforms and center of mass.

use hisab::{Quat, Vec3};
use tracing::instrument;

use crate::joint::Joint;
use crate::kinematics::{self, WorldTransforms};
use crate::muscle::Muscle;
use crate::pose::Pose;
use crate::skeleton::{BoneId, Skeleton};

/// Complete runtime body state.
#[derive(Debug, Clone)]
pub struct Body {
    pub skeleton: Skeleton,
    pub pose: Pose,
    pub joints: Vec<Joint>,
    pub muscles: Vec<Muscle>,
    transforms: Option<WorldTransforms>,
    center_of_mass: Option<Vec3>,
}

impl Body {
    /// Create a new body from a skeleton. Pose starts at rest.
    #[must_use]
    pub fn new(skeleton: Skeleton) -> Self {
        let bone_count = skeleton.bone_count();
        Self {
            skeleton,
            pose: Pose::rest(bone_count),
            joints: Vec::new(),
            muscles: Vec::new(),
            transforms: None,
            center_of_mass: None,
        }
    }

    /// Set the body's pose, invalidating cached transforms.
    pub fn set_pose(&mut self, pose: Pose) {
        self.pose = pose;
        self.invalidate();
    }

    /// Add a joint to the body.
    pub fn add_joint(&mut self, joint: Joint) {
        self.joints.push(joint);
    }

    /// Add a muscle to the body.
    pub fn add_muscle(&mut self, muscle: Muscle) {
        self.muscles.push(muscle);
    }

    /// Recompute world-space transforms and cached values.
    ///
    /// Call after changing the pose or before querying world-space positions.
    #[instrument(skip(self), fields(skeleton = %self.skeleton.name))]
    pub fn update(&mut self, root_position: Vec3, root_rotation: Quat) {
        let transforms = kinematics::forward_kinematics(
            &self.skeleton,
            &self.pose,
            root_position,
            root_rotation,
        );
        self.center_of_mass = Some(kinematics::world_center_of_mass(
            &self.skeleton,
            &transforms,
        ));
        self.transforms = Some(transforms);
    }

    /// World-space position of a bone. Returns `None` if FK hasn't been computed.
    #[must_use]
    #[inline]
    pub fn bone_world_position(&self, id: BoneId) -> Option<Vec3> {
        self.transforms.as_ref()?.position(id)
    }

    /// World-space rotation of a bone. Returns `None` if FK hasn't been computed.
    #[must_use]
    #[inline]
    pub fn bone_world_rotation(&self, id: BoneId) -> Option<Quat> {
        self.transforms.as_ref()?.rotation(id)
    }

    /// Cached world-space center of mass. Returns `None` if FK hasn't been computed.
    #[must_use]
    #[inline]
    pub fn center_of_mass(&self) -> Option<Vec3> {
        self.center_of_mass
    }

    /// Access the computed world transforms. Returns `None` if FK hasn't been computed.
    #[must_use]
    #[inline]
    pub fn world_transforms(&self) -> Option<&WorldTransforms> {
        self.transforms.as_ref()
    }

    /// Whether FK has been computed and transforms are available.
    #[must_use]
    #[inline]
    pub fn is_updated(&self) -> bool {
        self.transforms.is_some()
    }

    /// Invalidate cached transforms. Called automatically when pose changes.
    fn invalidate(&mut self) {
        self.transforms = None;
        self.center_of_mass = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skeleton::Bone;

    fn biped_skeleton() -> Skeleton {
        let mut skeleton = Skeleton::new("biped");
        skeleton.add_bone(Bone::new(BoneId(0), "pelvis", 0.2, 10.0, None));
        skeleton.add_bone(
            Bone::new(BoneId(1), "femur_l", 0.45, 4.0, Some(BoneId(0)))
                .with_position(Vec3::new(-0.1, -0.1, 0.0)),
        );
        skeleton.add_bone(
            Bone::new(BoneId(2), "tibia_l", 0.4, 3.0, Some(BoneId(1)))
                .with_position(Vec3::new(0.0, -0.45, 0.0)),
        );
        skeleton
    }

    #[test]
    fn new_body_not_updated() {
        let body = Body::new(biped_skeleton());
        assert!(!body.is_updated());
        assert!(body.bone_world_position(BoneId(0)).is_none());
        assert!(body.center_of_mass().is_none());
    }

    #[test]
    fn update_computes_transforms() {
        let mut body = Body::new(biped_skeleton());
        body.update(Vec3::ZERO, Quat::IDENTITY);

        assert!(body.is_updated());
        assert!(body.bone_world_position(BoneId(0)).is_some());
        assert!(body.center_of_mass().is_some());
    }

    #[test]
    fn set_pose_invalidates() {
        let mut body = Body::new(biped_skeleton());
        body.update(Vec3::ZERO, Quat::IDENTITY);
        assert!(body.is_updated());

        body.set_pose(Pose::rest(3));
        assert!(!body.is_updated());
    }

    #[test]
    fn bone_positions_match_fk() {
        let mut body = Body::new(biped_skeleton());
        body.update(Vec3::ZERO, Quat::IDENTITY);

        let pelvis = body.bone_world_position(BoneId(0)).unwrap();
        assert!((pelvis - Vec3::ZERO).length() < 1e-5, "pelvis at origin");

        let femur = body.bone_world_position(BoneId(1)).unwrap();
        assert!(
            (femur - Vec3::new(-0.1, -0.1, 0.0)).length() < 1e-5,
            "femur at local offset"
        );
    }

    #[test]
    fn com_is_weighted_average() {
        let mut body = Body::new(biped_skeleton());
        body.update(Vec3::ZERO, Quat::IDENTITY);

        let com = body.center_of_mass().unwrap();
        // Pelvis(10kg at 0,0,0) + femur(4kg at -0.1,-0.1,0) + tibia(3kg at -0.1,-0.55,0)
        // Total mass = 17
        assert!(com.is_finite(), "CoM should be finite");
        assert!(com.y < 0.0, "CoM should be below origin due to leg bones");
    }

    #[test]
    fn pose_changes_bone_positions() {
        let mut body = Body::new(biped_skeleton());

        // Rest pose
        body.update(Vec3::ZERO, Quat::IDENTITY);
        let tibia_rest = body.bone_world_position(BoneId(2)).unwrap();

        // Rotate femur
        let mut posed = Pose::rest(3);
        posed.set_joint(
            BoneId(1),
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_4),
        );
        body.set_pose(posed);
        body.update(Vec3::ZERO, Quat::IDENTITY);
        let tibia_posed = body.bone_world_position(BoneId(2)).unwrap();

        assert!(
            (tibia_rest - tibia_posed).length() > 0.01,
            "tibia should move when femur is rotated"
        );
    }

    #[test]
    fn add_joint_and_muscle() {
        let mut body = Body::new(biped_skeleton());
        body.add_joint(Joint::human_knee(BoneId(1), BoneId(2)));
        body.add_muscle(Muscle::new(
            "quad",
            BoneId(1),
            BoneId(2),
            crate::muscle::MuscleGroup::Extensor,
            5000.0,
            0.3,
        ));
        assert_eq!(body.joints.len(), 1);
        assert_eq!(body.muscles.len(), 1);
    }
}
