//! Inverse kinematics — compute joint rotations to reach a target position.
//!
//! Provides both analytic (two-bone) and iterative (FABRIK) solvers.
//! The analytic solver uses the law of cosines for exact closed-form solutions
//! on two-bone chains (arms, legs). FABRIK handles arbitrary-length chains.

use hisab::{Quat, Vec3};
use tracing::{instrument, trace};

use crate::joint::Joint;
use crate::kinematics::{WorldTransforms, forward_kinematics};
use crate::pose::Pose;
use crate::skeleton::{BoneId, Skeleton};

/// An IK chain: ordered sequence of bones from root to effector.
#[derive(Debug, Clone)]
pub struct IKChain {
    /// Bone IDs ordered from root to effector.
    pub bone_ids: Vec<BoneId>,
    /// Joint constraints per bone (parallel to `bone_ids`).
    pub joints: Vec<Joint>,
}

/// Target specification for an IK solve.
#[derive(Debug, Clone)]
pub struct IKTarget {
    /// World-space target position for the effector.
    pub position: Vec3,
    /// Optional orientation constraint for the effector.
    pub orientation: Option<Quat>,
    /// Hint vector for bending plane (e.g. elbow/knee direction).
    pub pole_vector: Option<Vec3>,
}

impl IKChain {
    /// Create a new IK chain from explicit bone IDs and joints.
    #[must_use]
    pub fn new(bone_ids: Vec<BoneId>, joints: Vec<Joint>) -> Self {
        Self { bone_ids, joints }
    }

    /// Build an IK chain by walking the skeleton hierarchy from `effector` up to `root`.
    ///
    /// Returns `None` if `root` is not an ancestor of `effector`, or if either bone is missing.
    /// Joints are created with free limits for each bone in the chain.
    #[must_use]
    pub fn from_skeleton(skeleton: &Skeleton, root: BoneId, effector: BoneId) -> Option<Self> {
        // Walk from effector to root, collecting bone IDs.
        let mut path = Vec::new();
        let mut current = effector;
        loop {
            skeleton.get_bone(current)?;
            path.push(current);
            if current == root {
                break;
            }
            let bone = skeleton.get_bone(current)?;
            current = bone.parent?;
        }
        // path is effector→root, we need root→effector
        path.reverse();

        // Create default free joints for each bone in the chain.
        let joints: Vec<Joint> = path
            .iter()
            .map(|&id| {
                let bone = skeleton.get_bone(id).expect("bone verified above");
                Joint {
                    name: bone.name.clone(),
                    joint_type: crate::joint::JointType::Ball,
                    parent_bone: bone.parent.unwrap_or(id),
                    child_bone: id,
                    limits: crate::joint::JointLimits::free(),
                    stiffness: 0.0,
                    damping: 0.0,
                }
            })
            .collect();

        Some(Self {
            bone_ids: path,
            joints,
        })
    }

    /// Total chain length (sum of bone lengths).
    #[must_use]
    pub fn total_length(&self, skeleton: &Skeleton) -> f32 {
        self.bone_ids
            .iter()
            .filter_map(|&id| skeleton.get_bone(id))
            .map(|b| b.length)
            .sum()
    }
}

/// Compute world-space joint positions along the chain given current pose.
fn chain_world_positions(
    chain: &IKChain,
    skeleton: &Skeleton,
    pose: &Pose,
    root_position: Vec3,
    root_rotation: Quat,
) -> Vec<Vec3> {
    let transforms = forward_kinematics(skeleton, pose, root_position, root_rotation);
    let mut positions: Vec<Vec3> = chain
        .bone_ids
        .iter()
        .filter_map(|&id| transforms.position(id))
        .collect();
    // Append the effector tip position (last bone position + bone direction * length).
    if let Some(&effector_id) = chain.bone_ids.last()
        && let Some(effector_tip) = effector_tip_position(&transforms, skeleton, effector_id)
    {
        positions.push(effector_tip);
    }
    positions
}

/// Compute the tip position of a bone (joint position + bone direction * length).
fn effector_tip_position(
    transforms: &WorldTransforms,
    skeleton: &Skeleton,
    bone_id: BoneId,
) -> Option<Vec3> {
    let pos = transforms.position(bone_id)?;
    let rot = transforms.rotation(bone_id)?;
    let bone = skeleton.get_bone(bone_id)?;
    // Bone extends along local Y axis (consistent with the skeleton convention).
    let tip = pos + rot * Vec3::new(0.0, bone.length, 0.0);
    Some(tip)
}

/// Analytic two-bone IK solver using the law of cosines.
///
/// For chains of exactly two bones (e.g. upper arm + forearm, thigh + shin).
/// Uses closed-form trigonometry for an exact solution.
///
/// Returns `None` if the chain doesn't have exactly 2 bones, or if the target
/// is unreachable (beyond the sum of bone lengths).
#[must_use]
#[instrument(skip_all)]
pub fn solve_two_bone(
    chain: &IKChain,
    target: &IKTarget,
    skeleton: &Skeleton,
    root_position: Vec3,
    root_rotation: Quat,
) -> Option<Pose> {
    if chain.bone_ids.len() != 2 {
        return None;
    }

    let bone_a_id = chain.bone_ids[0];
    let bone_b_id = chain.bone_ids[1];
    let bone_a = skeleton.get_bone(bone_a_id)?;
    let bone_b = skeleton.get_bone(bone_b_id)?;

    let len_a = bone_a.length;
    let len_b = bone_b.length;
    let total_len = len_a + len_b;

    // Compute current world-space position of the chain root joint.
    let pose_rest = Pose::rest(skeleton.bone_count());
    let transforms = forward_kinematics(skeleton, &pose_rest, root_position, root_rotation);
    let joint_a_pos = transforms.position(bone_a_id)?;
    let joint_a_rot = transforms.rotation(bone_a_id)?;

    // Vector from root joint to target.
    let to_target = target.position - joint_a_pos;
    let dist = to_target.length();

    // Unreachable check.
    if dist > total_len - 1e-6 {
        trace!(dist, total_len, "target unreachable for two-bone solver");
        return None;
    }

    // Nearly zero distance — bones fold to zero.
    if dist < 1e-6 {
        let mut pose = Pose::rest(skeleton.bone_count());
        let rot_a = Quat::from_rotation_x(std::f32::consts::PI);
        let rot_a = chain.joints[0].clamp_rotation(rot_a);
        pose.set_joint(bone_a_id, rot_a);
        return Some(pose);
    }

    // Law of cosines: angle at joint A (the root joint).
    // cos(angle_a) = (a² + d² - b²) / (2·a·d)
    let cos_angle_a =
        ((len_a * len_a + dist * dist - len_b * len_b) / (2.0 * len_a * dist)).clamp(-1.0, 1.0);
    let angle_a = cos_angle_a.acos();

    // Law of cosines: angle at joint B (the elbow/knee).
    // cos(angle_b) = (a² + b² - d²) / (2·a·b)
    let cos_angle_b =
        ((len_a * len_a + len_b * len_b - dist * dist) / (2.0 * len_a * len_b)).clamp(-1.0, 1.0);
    let angle_b = cos_angle_b.acos();

    // Determine the bending plane.
    let target_dir = to_target.normalize();

    // Default up direction for computing the bending plane.
    let bone_dir = (joint_a_rot * Vec3::Y).normalize();
    let plane_normal = if let Some(pole) = target.pole_vector {
        // Use pole vector to define the plane containing the chain.
        let pole_dir = (pole - joint_a_pos).normalize();
        let normal = target_dir.cross(pole_dir);
        if normal.length_squared() > 1e-8 {
            normal.normalize()
        } else {
            fallback_plane_normal(target_dir, bone_dir)
        }
    } else {
        fallback_plane_normal(target_dir, bone_dir)
    };

    // Rotation that aligns bone A's default direction (local Y) to the target direction.
    let parent_rot = joint_a_rot;
    let local_bone_dir = Vec3::Y;

    // Bring target direction into the local frame of bone A's parent.
    let inv_parent = parent_rot.inverse();
    let local_target_dir = (inv_parent * target_dir).normalize();

    // Rotation from default bone direction to target direction (in local space).
    let aim_rot = rotation_between(local_bone_dir, local_target_dir);

    // Apply the bend angle around the plane normal (in local space).
    let local_plane_normal = (inv_parent * plane_normal).normalize();
    let bend_rot_a = Quat::from_axis_angle(local_plane_normal, -angle_a);
    let rot_a = bend_rot_a * aim_rot;

    // Joint B: bend by the supplementary angle.
    let bend_angle_b = std::f32::consts::PI - angle_b;
    let rot_b = Quat::from_axis_angle(local_plane_normal, bend_angle_b);

    // Clamp to joint limits.
    let rot_a = chain.joints[0].clamp_rotation(rot_a);
    let rot_b = chain.joints[1].clamp_rotation(rot_b);

    let mut pose = Pose::rest(skeleton.bone_count());
    pose.set_joint(bone_a_id, rot_a);
    pose.set_joint(bone_b_id, rot_b);

    trace!("two-bone IK solved");
    Some(pose)
}

/// FABRIK (Forward And Backward Reaching Inverse Kinematics) solver.
///
/// Works for chains of arbitrary length. Iteratively adjusts joint positions
/// by alternating forward (effector→root) and backward (root→effector) passes
/// until the effector is within `tolerance` of the target, or `max_iterations`
/// is reached.
///
/// Returns `None` if the target is unreachable (beyond total chain length).
#[must_use]
#[instrument(skip_all, fields(chain_len = chain.bone_ids.len(), max_iterations))]
pub fn solve_fabrik(
    chain: &IKChain,
    target: &IKTarget,
    skeleton: &Skeleton,
    root_position: Vec3,
    root_rotation: Quat,
    max_iterations: u32,
    tolerance: f32,
) -> Option<Pose> {
    let n = chain.bone_ids.len();
    if n == 0 {
        return None;
    }

    // Collect bone lengths (distances between consecutive joints).
    let mut bone_lengths: Vec<f32> = Vec::with_capacity(n);
    for &id in &chain.bone_ids {
        let bone = skeleton.get_bone(id)?;
        bone_lengths.push(bone.length);
    }

    let total_len: f32 = bone_lengths.iter().sum();

    // Get initial joint positions from FK.
    let initial_pose = Pose::rest(skeleton.bone_count());
    let mut positions =
        chain_world_positions(chain, skeleton, &initial_pose, root_position, root_rotation);

    // positions has n+1 entries: n joint positions + 1 effector tip.
    if positions.len() != n + 1 {
        return None;
    }

    let root_pos = positions[0];
    let dist_to_target = (target.position - root_pos).length();

    // Unreachable check.
    if dist_to_target > total_len - 1e-6 {
        trace!(dist_to_target, total_len, "target unreachable for FABRIK");
        return None;
    }

    // FABRIK iteration.
    for iteration in 0..max_iterations {
        let effector_pos = positions[n];
        let err = (effector_pos - target.position).length();
        if err < tolerance {
            trace!(iteration, err, "FABRIK converged");
            break;
        }

        // Forward pass: move effector to target, propagate to root.
        positions[n] = target.position;
        for i in (0..n).rev() {
            let dir = (positions[i] - positions[i + 1]).normalize_or_zero();
            positions[i] = positions[i + 1] + dir * bone_lengths[i];
        }

        // Backward pass: move root back to its original position, propagate to effector.
        positions[0] = root_pos;
        for i in 0..n {
            let dir = (positions[i + 1] - positions[i]).normalize_or_zero();
            positions[i + 1] = positions[i] + dir * bone_lengths[i];
        }
    }

    // Convert positions back to joint rotations.
    positions_to_pose(chain, skeleton, &positions, root_position, root_rotation)
}

/// Convert FABRIK world-space positions back into a Pose (joint rotations).
fn positions_to_pose(
    chain: &IKChain,
    skeleton: &Skeleton,
    positions: &[Vec3],
    _root_position: Vec3,
    root_rotation: Quat,
) -> Option<Pose> {
    let n = chain.bone_ids.len();
    let mut pose = Pose::rest(skeleton.bone_count());

    // We need to compute rotations for each bone in the chain.
    // For each bone i, the desired direction is from positions[i] to positions[i+1].
    // We compute the local rotation needed to achieve this direction given the parent's
    // accumulated world rotation.
    let mut parent_world_rot = root_rotation;

    for i in 0..n {
        let bone_id = chain.bone_ids[i];
        let bone = skeleton.get_bone(bone_id)?;

        // Desired world direction for this bone.
        let desired_dir = (positions[i + 1] - positions[i]).normalize_or_zero();

        // The bone's rest direction in world space (through parent rotation and bone's local rotation).
        let rest_world_dir = (parent_world_rot * bone.local_rotation * Vec3::Y).normalize();

        // Rotation from rest direction to desired direction in world space.
        let world_correction = rotation_between(rest_world_dir, desired_dir);

        // Convert to local space: the pose rotation is applied after bone.local_rotation,
        // so we need: parent_world_rot * bone.local_rotation * pose_rot * Y = desired_dir
        // → pose_rot = inverse(parent_world_rot * bone.local_rotation) * world_correction * (parent_world_rot * bone.local_rotation) ... simplified:
        let parent_bone_rot = parent_world_rot * bone.local_rotation;
        let inv_pb = parent_bone_rot.inverse();
        let local_correction = inv_pb * world_correction * parent_bone_rot;

        // Clamp to joint limits.
        let clamped = chain
            .joints
            .get(i)
            .map_or(local_correction, |j| j.clamp_rotation(local_correction));

        pose.set_joint(bone_id, clamped);

        // Update parent world rotation for next bone.
        parent_world_rot = parent_bone_rot * clamped;
    }

    Some(pose)
}

/// Compute a quaternion that rotates vector `from` to vector `to`.
/// Both should be normalized.
fn rotation_between(from: Vec3, to: Vec3) -> Quat {
    let dot = from.dot(to).clamp(-1.0, 1.0);
    if dot > 0.999_999 {
        return Quat::IDENTITY;
    }
    if dot < -0.999_999 {
        // Vectors are anti-parallel; pick an arbitrary perpendicular axis.
        let perp = if from.x.abs() < 0.9 { Vec3::X } else { Vec3::Y };
        let axis = from.cross(perp).normalize();
        return Quat::from_axis_angle(axis, std::f32::consts::PI);
    }
    let axis = from.cross(to).normalize();
    let angle = dot.acos();
    Quat::from_axis_angle(axis, angle)
}

/// Compute a fallback plane normal when pole vector is unavailable.
fn fallback_plane_normal(target_dir: Vec3, bone_dir: Vec3) -> Vec3 {
    let normal = target_dir.cross(bone_dir);
    if normal.length_squared() > 1e-8 {
        normal.normalize()
    } else {
        // Target and bone are collinear; use an arbitrary perpendicular.
        let perp = if target_dir.x.abs() < 0.9 {
            Vec3::X
        } else {
            Vec3::Z
        };
        target_dir.cross(perp).normalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::joint::{AxisLimit, JointLimits, JointType};
    use crate::skeleton::Bone;

    /// Build a simple two-bone chain skeleton: bone 0 (upper, length 1.0) → bone 1 (lower, length 1.0).
    /// Bones extend along the Y axis from the origin.
    fn two_bone_skeleton() -> Skeleton {
        Skeleton {
            name: "two_bone".into(),
            bones: vec![
                Bone {
                    id: BoneId(0),
                    name: "upper".into(),
                    parent: None,
                    length: 1.0,
                    mass: 1.0,
                    local_position: Vec3::ZERO,
                    local_rotation: Quat::IDENTITY,
                },
                Bone {
                    id: BoneId(1),
                    name: "lower".into(),
                    parent: Some(BoneId(0)),
                    length: 1.0,
                    mass: 1.0,
                    local_position: Vec3::new(0.0, 1.0, 0.0),
                    local_rotation: Quat::IDENTITY,
                },
            ],
        }
    }

    fn two_bone_chain() -> IKChain {
        IKChain::new(
            vec![BoneId(0), BoneId(1)],
            vec![
                Joint {
                    name: "shoulder".into(),
                    joint_type: JointType::Ball,
                    parent_bone: BoneId(0),
                    child_bone: BoneId(0),
                    limits: JointLimits::free(),
                    stiffness: 0.0,
                    damping: 0.0,
                },
                Joint {
                    name: "elbow".into(),
                    joint_type: JointType::Ball,
                    parent_bone: BoneId(0),
                    child_bone: BoneId(1),
                    limits: JointLimits::free(),
                    stiffness: 0.0,
                    damping: 0.0,
                },
            ],
        )
    }

    /// Build a three-bone chain skeleton for FABRIK tests.
    fn three_bone_skeleton() -> Skeleton {
        Skeleton {
            name: "three_bone".into(),
            bones: vec![
                Bone {
                    id: BoneId(0),
                    name: "a".into(),
                    parent: None,
                    length: 1.0,
                    mass: 1.0,
                    local_position: Vec3::ZERO,
                    local_rotation: Quat::IDENTITY,
                },
                Bone {
                    id: BoneId(1),
                    name: "b".into(),
                    parent: Some(BoneId(0)),
                    length: 1.0,
                    mass: 1.0,
                    local_position: Vec3::new(0.0, 1.0, 0.0),
                    local_rotation: Quat::IDENTITY,
                },
                Bone {
                    id: BoneId(2),
                    name: "c".into(),
                    parent: Some(BoneId(1)),
                    length: 1.0,
                    mass: 1.0,
                    local_position: Vec3::new(0.0, 1.0, 0.0),
                    local_rotation: Quat::IDENTITY,
                },
            ],
        }
    }

    fn three_bone_chain() -> IKChain {
        IKChain::new(
            vec![BoneId(0), BoneId(1), BoneId(2)],
            vec![
                Joint {
                    name: "j0".into(),
                    joint_type: JointType::Ball,
                    parent_bone: BoneId(0),
                    child_bone: BoneId(0),
                    limits: JointLimits::free(),
                    stiffness: 0.0,
                    damping: 0.0,
                },
                Joint {
                    name: "j1".into(),
                    joint_type: JointType::Ball,
                    parent_bone: BoneId(0),
                    child_bone: BoneId(1),
                    limits: JointLimits::free(),
                    stiffness: 0.0,
                    damping: 0.0,
                },
                Joint {
                    name: "j2".into(),
                    joint_type: JointType::Ball,
                    parent_bone: BoneId(1),
                    child_bone: BoneId(2),
                    limits: JointLimits::free(),
                    stiffness: 0.0,
                    damping: 0.0,
                },
            ],
        )
    }

    // ---- Test 1: two_bone_reaches_target ----
    #[test]
    fn two_bone_reaches_target() {
        let skeleton = two_bone_skeleton();
        let chain = two_bone_chain();
        // Target at (1.0, 1.0, 0.0) — reachable (dist = sqrt(2) ≈ 1.414, total = 2.0).
        let target = IKTarget {
            position: Vec3::new(1.0, 1.0, 0.0),
            orientation: None,
            pole_vector: None,
        };
        let result = solve_two_bone(&chain, &target, &skeleton, Vec3::ZERO, Quat::IDENTITY);
        assert!(result.is_some(), "two-bone solver should find a solution");
    }

    // ---- Test 2: two_bone_unreachable ----
    #[test]
    fn two_bone_unreachable() {
        let skeleton = two_bone_skeleton();
        let chain = two_bone_chain();
        // Target at (0, 3, 0) — unreachable (dist 3.0 > total 2.0).
        let target = IKTarget {
            position: Vec3::new(0.0, 3.0, 0.0),
            orientation: None,
            pole_vector: None,
        };
        let result = solve_two_bone(&chain, &target, &skeleton, Vec3::ZERO, Quat::IDENTITY);
        assert!(
            result.is_none(),
            "should return None for unreachable target"
        );
    }

    // ---- Test 3: two_bone_with_pole_vector ----
    #[test]
    fn two_bone_with_pole_vector() {
        let skeleton = two_bone_skeleton();
        let chain = two_bone_chain();
        let target = IKTarget {
            position: Vec3::new(0.0, 1.5, 0.0),
            orientation: None,
            pole_vector: Some(Vec3::new(0.0, 0.5, 1.0)), // bend toward +Z
        };
        let result = solve_two_bone(&chain, &target, &skeleton, Vec3::ZERO, Quat::IDENTITY);
        assert!(
            result.is_some(),
            "two-bone solver with pole vector should find a solution"
        );
        // Verify that the solution produced actual rotations (not rest pose).
        let pose = result.unwrap();
        let rot_a = pose.get_joint(BoneId(0));
        // With a pole vector, the root bone should have a non-trivial rotation.
        assert!(
            rot_a.dot(Quat::IDENTITY).abs() < 0.9999,
            "pole vector should influence the bending plane"
        );
    }

    // ---- Test 4: two_bone_with_joint_limits ----
    #[test]
    fn two_bone_with_joint_limits() {
        let skeleton = two_bone_skeleton();
        let limited_chain = IKChain::new(
            vec![BoneId(0), BoneId(1)],
            vec![
                Joint {
                    name: "shoulder".into(),
                    joint_type: JointType::Ball,
                    parent_bone: BoneId(0),
                    child_bone: BoneId(0),
                    limits: JointLimits {
                        x: Some(AxisLimit::new(-30.0, 30.0)),
                        y: Some(AxisLimit::new(-30.0, 30.0)),
                        z: Some(AxisLimit::new(-30.0, 30.0)),
                    },
                    stiffness: 0.0,
                    damping: 0.0,
                },
                Joint {
                    name: "elbow".into(),
                    joint_type: JointType::Hinge,
                    parent_bone: BoneId(0),
                    child_bone: BoneId(1),
                    limits: JointLimits::hinge(0.0, 90.0),
                    stiffness: 0.0,
                    damping: 0.0,
                },
            ],
        );
        let target = IKTarget {
            position: Vec3::new(1.0, 1.0, 0.0),
            orientation: None,
            pole_vector: None,
        };
        let result = solve_two_bone(
            &limited_chain,
            &target,
            &skeleton,
            Vec3::ZERO,
            Quat::IDENTITY,
        );
        assert!(
            result.is_some(),
            "two-bone solver should return a (clamped) solution"
        );
        // Verify joint limits are respected on the elbow.
        let pose = result.unwrap();
        let rot_b = pose.get_joint(BoneId(1));
        let (x, _, _) = rot_b.to_euler(hisab::transforms::glam::EulerRot::XYZ);
        let max_rad = 90.0_f32.to_radians();
        assert!(
            x <= max_rad + 0.01,
            "elbow rotation should be clamped to 90° max, got {:.1}°",
            x.to_degrees()
        );
    }

    // ---- Test 5: fabrik_reaches_target ----
    #[test]
    fn fabrik_reaches_target() {
        let skeleton = three_bone_skeleton();
        let chain = three_bone_chain();
        // Target at (1, 2, 0) — reachable (dist ≈ 2.24, total = 3.0).
        let target = IKTarget {
            position: Vec3::new(1.0, 2.0, 0.0),
            orientation: None,
            pole_vector: None,
        };
        let result = solve_fabrik(
            &chain,
            &target,
            &skeleton,
            Vec3::ZERO,
            Quat::IDENTITY,
            50,
            0.01,
        );
        assert!(result.is_some(), "FABRIK should find a solution");
    }

    // ---- Test 6: fabrik_unreachable ----
    #[test]
    fn fabrik_unreachable() {
        let skeleton = three_bone_skeleton();
        let chain = three_bone_chain();
        // Target at (0, 5, 0) — unreachable (dist 5.0 > total 3.0).
        let target = IKTarget {
            position: Vec3::new(0.0, 5.0, 0.0),
            orientation: None,
            pole_vector: None,
        };
        let result = solve_fabrik(
            &chain,
            &target,
            &skeleton,
            Vec3::ZERO,
            Quat::IDENTITY,
            50,
            0.01,
        );
        assert!(
            result.is_none(),
            "FABRIK should return None for unreachable target"
        );
    }

    // ---- Test 7: fabrik_respects_joint_limits ----
    #[test]
    fn fabrik_respects_joint_limits() {
        let skeleton = three_bone_skeleton();
        let limited_chain = IKChain::new(
            vec![BoneId(0), BoneId(1), BoneId(2)],
            vec![
                Joint {
                    name: "j0".into(),
                    joint_type: JointType::Ball,
                    parent_bone: BoneId(0),
                    child_bone: BoneId(0),
                    limits: JointLimits {
                        x: Some(AxisLimit::new(-45.0, 45.0)),
                        y: Some(AxisLimit::new(-45.0, 45.0)),
                        z: Some(AxisLimit::new(-45.0, 45.0)),
                    },
                    stiffness: 0.0,
                    damping: 0.0,
                },
                Joint {
                    name: "j1".into(),
                    joint_type: JointType::Ball,
                    parent_bone: BoneId(0),
                    child_bone: BoneId(1),
                    limits: JointLimits {
                        x: Some(AxisLimit::new(-45.0, 45.0)),
                        y: Some(AxisLimit::new(-45.0, 45.0)),
                        z: Some(AxisLimit::new(-45.0, 45.0)),
                    },
                    stiffness: 0.0,
                    damping: 0.0,
                },
                Joint {
                    name: "j2".into(),
                    joint_type: JointType::Ball,
                    parent_bone: BoneId(1),
                    child_bone: BoneId(2),
                    limits: JointLimits {
                        x: Some(AxisLimit::new(-45.0, 45.0)),
                        y: Some(AxisLimit::new(-45.0, 45.0)),
                        z: Some(AxisLimit::new(-45.0, 45.0)),
                    },
                    stiffness: 0.0,
                    damping: 0.0,
                },
            ],
        );
        let target = IKTarget {
            position: Vec3::new(1.0, 2.0, 0.0),
            orientation: None,
            pole_vector: None,
        };
        let result = solve_fabrik(
            &limited_chain,
            &target,
            &skeleton,
            Vec3::ZERO,
            Quat::IDENTITY,
            50,
            0.01,
        );
        assert!(
            result.is_some(),
            "FABRIK with limits should return a solution"
        );
        // Verify all rotations are within limits.
        let pose = result.unwrap();
        let max_rad = 45.0_f32.to_radians();
        for i in 0..3 {
            let rot = pose.get_joint(BoneId(i as u16));
            let (x, y, z) = rot.to_euler(hisab::transforms::glam::EulerRot::XYZ);
            assert!(
                x <= max_rad + 0.01 && x >= -max_rad - 0.01,
                "bone {i} x-rotation {:.1}° exceeds ±45°",
                x.to_degrees()
            );
            assert!(
                y <= max_rad + 0.01 && y >= -max_rad - 0.01,
                "bone {i} y-rotation {:.1}° exceeds ±45°",
                y.to_degrees()
            );
            assert!(
                z <= max_rad + 0.01 && z >= -max_rad - 0.01,
                "bone {i} z-rotation {:.1}° exceeds ±45°",
                z.to_degrees()
            );
        }
    }

    // ---- Test 8: fabrik_converges_within_iterations ----
    #[test]
    fn fabrik_converges_within_iterations() {
        let skeleton = three_bone_skeleton();
        let chain = three_bone_chain();
        // A target that's easily reachable should converge quickly.
        let target = IKTarget {
            position: Vec3::new(0.5, 2.5, 0.0),
            orientation: None,
            pole_vector: None,
        };
        // Give it only 10 iterations with tight tolerance.
        let result = solve_fabrik(
            &chain,
            &target,
            &skeleton,
            Vec3::ZERO,
            Quat::IDENTITY,
            10,
            0.05,
        );
        assert!(
            result.is_some(),
            "FABRIK should converge for a reachable target within 10 iterations"
        );
    }

    // ---- Test 9: chain_from_skeleton ----
    #[test]
    fn chain_from_skeleton() {
        let skeleton = three_bone_skeleton();
        let chain = IKChain::from_skeleton(&skeleton, BoneId(0), BoneId(2));
        assert!(
            chain.is_some(),
            "should build chain from skeleton hierarchy"
        );
        let chain = chain.unwrap();
        assert_eq!(chain.bone_ids, vec![BoneId(0), BoneId(1), BoneId(2)]);
        assert_eq!(chain.joints.len(), 3);
    }

    // ---- Test 10: chain_total_length ----
    #[test]
    fn chain_total_length() {
        let skeleton = three_bone_skeleton();
        let chain = three_bone_chain();
        let total = chain.total_length(&skeleton);
        assert!(
            (total - 3.0).abs() < 1e-6,
            "total length should be 3.0, got {total}"
        );
    }

    // ---- Test 11: chain_from_skeleton_invalid ----
    #[test]
    fn chain_from_skeleton_invalid() {
        let skeleton = two_bone_skeleton();
        // BoneId(5) doesn't exist.
        let chain = IKChain::from_skeleton(&skeleton, BoneId(0), BoneId(5));
        assert!(chain.is_none(), "should return None for missing bone");
    }

    // ---- Test 12: two_bone_wrong_chain_length ----
    #[test]
    fn two_bone_wrong_chain_length() {
        let skeleton = three_bone_skeleton();
        let chain = three_bone_chain(); // 3 bones, not 2
        let target = IKTarget {
            position: Vec3::new(0.0, 1.0, 0.0),
            orientation: None,
            pole_vector: None,
        };
        let result = solve_two_bone(&chain, &target, &skeleton, Vec3::ZERO, Quat::IDENTITY);
        assert!(
            result.is_none(),
            "two-bone solver should reject non-2-bone chains"
        );
    }
}
