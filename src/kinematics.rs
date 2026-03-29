//! Forward kinematics — compute world-space bone transforms from skeleton + pose.
//!
//! Given a skeleton hierarchy (local transforms per bone) and a pose (joint angle overrides),
//! FK computes the world-space position and rotation of every bone.

use std::collections::HashMap;

use hisab::{Mat4, Quat, Vec3};
use tracing::instrument;

use crate::pose::Pose;
use crate::skeleton::{BoneId, Skeleton};

/// World-space transforms for all bones in a skeleton.
///
/// Stores `Mat4` internally because FK is a chain of matrix multiplications.
/// Provides decomposed accessors for position and rotation.
#[derive(Debug, Clone)]
pub struct WorldTransforms {
    /// World-space 4x4 matrix per bone, indexed by bone position in skeleton.bones.
    matrices: Vec<Mat4>,
    /// BoneId -> index mapping for O(1) lookup.
    index: HashMap<BoneId, usize>,
}

impl WorldTransforms {
    /// World-space position of a bone.
    #[must_use]
    #[inline]
    pub fn position(&self, bone_id: BoneId) -> Option<Vec3> {
        self.index
            .get(&bone_id)
            .map(|&i| self.matrices[i].col(3).truncate())
    }

    /// World-space rotation of a bone (extracted from the 3x3 sub-matrix).
    #[must_use]
    pub fn rotation(&self, bone_id: BoneId) -> Option<Quat> {
        self.index.get(&bone_id).map(|&i| {
            let m = self.matrices[i];
            // Extract 3x3, remove scale, convert to quaternion
            let col0 = m.col(0).truncate().normalize();
            let col1 = m.col(1).truncate().normalize();
            let col2 = m.col(2).truncate().normalize();
            let rot_mat = hisab::Mat3::from_cols(col0, col1, col2);
            Quat::from_mat3(&rot_mat)
        })
    }

    /// World-space 4x4 matrix of a bone.
    #[must_use]
    #[inline]
    pub fn matrix(&self, bone_id: BoneId) -> Option<Mat4> {
        self.index.get(&bone_id).map(|&i| self.matrices[i])
    }

    /// Number of bones with computed transforms.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.matrices.len()
    }

    /// Whether the transform set is empty.
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.matrices.is_empty()
    }
}

/// Compute forward kinematics for a skeleton with a given pose.
///
/// `root_position` and `root_rotation` define the skeleton's placement in world space.
/// Each bone's world transform = parent_world × local_translation × (local_rotation * pose_rotation).
///
/// Bones must be ordered so that parents appear before children.
/// If a parent is missing (not found), the bone is treated as a root.
#[must_use]
#[instrument(skip_all, fields(skeleton = %skeleton.name, bone_count = skeleton.bone_count()))]
pub fn forward_kinematics(
    skeleton: &Skeleton,
    pose: &Pose,
    root_position: Vec3,
    root_rotation: Quat,
) -> WorldTransforms {
    let bones = skeleton.bones();
    let mut matrices = Vec::with_capacity(bones.len());
    let mut index = HashMap::with_capacity(bones.len());

    let root_mat = Mat4::from_rotation_translation(root_rotation, root_position);

    for (i, bone) in bones.iter().enumerate() {
        index.insert(bone.id, i);

        // Get parent's world matrix
        let parent_mat = bone
            .parent
            .and_then(|pid| index.get(&pid))
            .map(|&pi| matrices[pi])
            .unwrap_or(root_mat);

        // Pose rotation override for this bone's joint
        let pose_rot = pose.get_joint(bone.id);

        // Local transform: translate to bone position, then apply combined rotation
        let combined_rot = bone.local_rotation * pose_rot;
        let local_mat = Mat4::from_rotation_translation(combined_rot, bone.local_position);

        // World = parent × local
        let world_mat = parent_mat * local_mat;
        matrices.push(world_mat);
    }

    WorldTransforms { matrices, index }
}

/// Compute world-space center of mass from skeleton masses and FK transforms.
#[must_use]
pub fn world_center_of_mass(skeleton: &Skeleton, transforms: &WorldTransforms) -> Vec3 {
    let mut total_mass = 0.0_f32;
    let mut weighted_pos = Vec3::ZERO;

    for bone in skeleton.bones() {
        if let Some(pos) = transforms.position(bone.id) {
            weighted_pos += bone.mass * pos;
            total_mass += bone.mass;
        }
    }

    if total_mass > 0.0 {
        weighted_pos / total_mass
    } else {
        Vec3::ZERO
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skeleton::Bone;

    fn make_chain_skeleton() -> Skeleton {
        // Simple 3-bone vertical chain: root at origin, each bone 1m up
        Skeleton {
            name: "chain".into(),
            bones: vec![
                Bone {
                    id: BoneId(0),
                    name: "root".into(),
                    parent: None,
                    length: 1.0,
                    mass: 1.0,
                    local_position: Vec3::ZERO,
                    local_rotation: Quat::IDENTITY,
                },
                Bone {
                    id: BoneId(1),
                    name: "mid".into(),
                    parent: Some(BoneId(0)),
                    length: 1.0,
                    mass: 1.0,
                    local_position: Vec3::new(0.0, 1.0, 0.0),
                    local_rotation: Quat::IDENTITY,
                },
                Bone {
                    id: BoneId(2),
                    name: "tip".into(),
                    parent: Some(BoneId(1)),
                    length: 1.0,
                    mass: 1.0,
                    local_position: Vec3::new(0.0, 1.0, 0.0),
                    local_rotation: Quat::IDENTITY,
                },
            ],
        }
    }

    #[test]
    fn rest_pose_preserves_local_transforms() {
        let skeleton = make_chain_skeleton();
        let pose = Pose::rest(3);
        let transforms = forward_kinematics(&skeleton, &pose, Vec3::ZERO, Quat::IDENTITY);

        assert_eq!(transforms.len(), 3);

        let p0 = transforms.position(BoneId(0)).unwrap();
        let p1 = transforms.position(BoneId(1)).unwrap();
        let p2 = transforms.position(BoneId(2)).unwrap();

        assert!((p0 - Vec3::ZERO).length() < 1e-5, "root at origin");
        assert!(
            (p1 - Vec3::new(0.0, 1.0, 0.0)).length() < 1e-5,
            "mid at y=1"
        );
        assert!(
            (p2 - Vec3::new(0.0, 2.0, 0.0)).length() < 1e-5,
            "tip at y=2"
        );
    }

    #[test]
    fn root_offset_shifts_all_bones() {
        let skeleton = make_chain_skeleton();
        let pose = Pose::rest(3);
        let offset = Vec3::new(10.0, 0.0, 0.0);
        let transforms = forward_kinematics(&skeleton, &pose, offset, Quat::IDENTITY);

        let p0 = transforms.position(BoneId(0)).unwrap();
        let p2 = transforms.position(BoneId(2)).unwrap();

        assert!(
            (p0 - Vec3::new(10.0, 0.0, 0.0)).length() < 1e-5,
            "root shifted by offset"
        );
        assert!(
            (p2 - Vec3::new(10.0, 2.0, 0.0)).length() < 1e-5,
            "tip shifted by offset"
        );
    }

    #[test]
    fn pose_rotation_affects_children() {
        let skeleton = make_chain_skeleton();
        let mut pose = Pose::rest(3);
        // Rotate root 90° around Z — children should swing from Y-axis to X-axis
        pose.set_joint(
            BoneId(0),
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        );
        let transforms = forward_kinematics(&skeleton, &pose, Vec3::ZERO, Quat::IDENTITY);

        let p1 = transforms.position(BoneId(1)).unwrap();
        // Mid bone was at local (0,1,0), rotated 90° around Z → should be at (-1,0,0)
        assert!(
            (p1 - Vec3::new(-1.0, 0.0, 0.0)).length() < 1e-4,
            "mid bone should be at (-1,0,0) after 90° Z rotation, got ({:.3},{:.3},{:.3})",
            p1.x,
            p1.y,
            p1.z
        );

        let p2 = transforms.position(BoneId(2)).unwrap();
        // Tip bone was at local (0,1,0) relative to mid → should be at (-2,0,0)
        assert!(
            (p2 - Vec3::new(-2.0, 0.0, 0.0)).length() < 1e-4,
            "tip should be at (-2,0,0), got ({:.3},{:.3},{:.3})",
            p2.x,
            p2.y,
            p2.z
        );
    }

    #[test]
    fn root_rotation_rotates_entire_skeleton() {
        let skeleton = make_chain_skeleton();
        let pose = Pose::rest(3);
        // Rotate entire skeleton 90° around Z
        let root_rot = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        let transforms = forward_kinematics(&skeleton, &pose, Vec3::ZERO, root_rot);

        let p2 = transforms.position(BoneId(2)).unwrap();
        assert!(
            (p2 - Vec3::new(-2.0, 0.0, 0.0)).length() < 1e-4,
            "tip should be at (-2,0,0) with root rotated 90°"
        );
    }

    #[test]
    fn world_com_at_rest() {
        let skeleton = make_chain_skeleton();
        let pose = Pose::rest(3);
        let transforms = forward_kinematics(&skeleton, &pose, Vec3::ZERO, Quat::IDENTITY);
        let com = world_center_of_mass(&skeleton, &transforms);

        // 3 equal masses at y=0, y=1, y=2 → CoM at y=1
        assert!(
            (com - Vec3::new(0.0, 1.0, 0.0)).length() < 1e-5,
            "CoM should be at (0,1,0), got ({:.3},{:.3},{:.3})",
            com.x,
            com.y,
            com.z
        );
    }

    #[test]
    fn rotation_extraction() {
        let skeleton = make_chain_skeleton();
        let mut pose = Pose::rest(3);
        let rot45 = Quat::from_rotation_z(std::f32::consts::FRAC_PI_4);
        pose.set_joint(BoneId(0), rot45);
        let transforms = forward_kinematics(&skeleton, &pose, Vec3::ZERO, Quat::IDENTITY);

        let got = transforms.rotation(BoneId(0)).unwrap();
        // Should be close to the 45° rotation we set
        assert!(
            got.dot(rot45).abs() > 0.999,
            "extracted rotation should match pose rotation"
        );
    }

    #[test]
    fn empty_skeleton_produces_empty_transforms() {
        let skeleton = Skeleton::new("empty");
        let pose = Pose::rest(0);
        let transforms = forward_kinematics(&skeleton, &pose, Vec3::ZERO, Quat::IDENTITY);
        assert!(transforms.is_empty());
    }
}
