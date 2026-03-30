//! Soorat integration — visualization data structures for physiology/biomechanics.
//!
//! Provides structured types that soorat can render: skeleton wireframes,
//! muscle overlays, gait cycle timelines, and body plan meshes.

use serde::{Deserialize, Serialize};

// ── Skeleton visualization ─────────────────────────────────────────────────

/// Skeleton data for wireframe/debug rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkeletonVisualization {
    /// Bone segments: each is `(start_position, end_position, bone_name)`.
    pub bones: Vec<BoneSegment>,
    /// Joint positions with constraint info.
    pub joints: Vec<JointViz>,
}

/// A bone segment for line rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoneSegment {
    /// World-space start position `[x, y, z]`.
    pub start: [f32; 3],
    /// World-space end position `[x, y, z]`.
    pub end: [f32; 3],
    /// Bone name.
    pub name: String,
    /// Bone mass (kg) for thickness scaling.
    pub mass: f32,
}

/// Joint visualization data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JointViz {
    /// World-space position `[x, y, z]`.
    pub position: [f32; 3],
    /// Joint type name (e.g. "Ball", "Hinge").
    pub joint_type: String,
    /// Degrees of freedom.
    pub dof: u8,
}

impl SkeletonVisualization {
    /// Create from a sharira `Skeleton` and computed `WorldTransforms`.
    #[must_use]
    pub fn from_skeleton(
        skeleton: &crate::skeleton::Skeleton,
        transforms: &crate::kinematics::WorldTransforms,
    ) -> Self {
        let mut bones = Vec::with_capacity(skeleton.bones.len());

        for bone in &skeleton.bones {
            let end_pos = transforms
                .position(bone.id)
                .map(|v| [v.x, v.y, v.z])
                .unwrap_or([0.0; 3]);

            let start_pos = bone
                .parent
                .and_then(|pid| transforms.position(pid))
                .map(|v| [v.x, v.y, v.z])
                .unwrap_or(end_pos);

            bones.push(BoneSegment {
                start: start_pos,
                end: end_pos,
                name: bone.name.clone(),
                mass: bone.mass,
            });
        }

        Self {
            bones,
            joints: Vec::new(), // populated separately with joint data
        }
    }

    /// Add joint visualization data from a list of joints.
    pub fn add_joints(
        &mut self,
        joints: &[crate::joint::Joint],
        transforms: &crate::kinematics::WorldTransforms,
    ) {
        for joint in joints {
            let position = transforms
                .position(joint.child_bone)
                .map(|v| [v.x, v.y, v.z])
                .unwrap_or([0.0; 3]);

            self.joints.push(JointViz {
                position,
                joint_type: format!("{:?}", joint.joint_type),
                dof: joint.joint_type.degrees_of_freedom(),
            });
        }
    }
}

// ── Muscle overlay ─────────────────────────────────────────────────────────

/// Muscle data for colored overlay rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MuscleOverlay {
    /// Muscles with attachment points and activation.
    pub muscles: Vec<MuscleViz>,
}

/// A single muscle for rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MuscleViz {
    /// Origin attachment point `[x, y, z]`.
    pub origin: [f32; 3],
    /// Insertion attachment point `[x, y, z]`.
    pub insertion: [f32; 3],
    /// Current activation level (0.0–1.0) for color intensity.
    pub activation: f32,
    /// Muscle group name.
    pub group: String,
    /// Maximum force (N) for thickness scaling.
    pub max_force: f32,
}

impl MuscleOverlay {
    /// Create from sharira muscles and world transforms.
    #[must_use]
    pub fn from_muscles(
        muscles: &[crate::muscle::Muscle],
        transforms: &crate::kinematics::WorldTransforms,
    ) -> Self {
        let vizs: Vec<MuscleViz> = muscles
            .iter()
            .map(|m| {
                let origin = transforms
                    .position(m.origin_bone)
                    .map(|v| [v.x, v.y, v.z])
                    .unwrap_or([0.0; 3]);
                let insertion = transforms
                    .position(m.insertion_bone)
                    .map(|v| [v.x, v.y, v.z])
                    .unwrap_or([0.0; 3]);

                MuscleViz {
                    origin,
                    insertion,
                    activation: m.activation,
                    group: format!("{:?}", m.group),
                    max_force: m.max_force_n,
                }
            })
            .collect();

        Self { muscles: vizs }
    }
}

// ── Gait cycle data ────────────────────────────────────────────────────────

/// Gait cycle timeline for animation rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GaitCycleVisualization {
    /// Gait type name.
    pub gait_type: String,
    /// Cycle duration (seconds).
    pub duration: f32,
    /// Stride length (meters).
    pub stride_length: f32,
    /// Speed (m/s).
    pub speed: f32,
    /// Per-limb phase tracks.
    pub limb_tracks: Vec<LimbTrack>,
}

/// Phase timeline for a single limb.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LimbTrack {
    /// Limb index.
    pub limb_index: usize,
    /// Phase offset (0.0–1.0) within the cycle.
    pub phase_offset: f32,
    /// Duty factor (fraction of cycle in stance).
    pub duty_factor: f32,
}

impl GaitCycleVisualization {
    /// Create from a sharira `Gait`.
    #[must_use]
    pub fn from_gait(gait: &crate::gait::Gait) -> Self {
        let limb_tracks: Vec<LimbTrack> = gait
            .cycle
            .limb_phase_offsets
            .iter()
            .enumerate()
            .map(|(i, &offset)| LimbTrack {
                limb_index: i,
                phase_offset: offset,
                duty_factor: gait.cycle.duty_factor,
            })
            .collect();

        Self {
            gait_type: format!("{:?}", gait.gait_type),
            duration: gait.cycle.cycle_duration_s,
            stride_length: gait.cycle.stride_length_m,
            speed: gait.speed(),
            limb_tracks,
        }
    }
}

// ── Body plan mesh data ────────────────────────────────────────────────────

/// Body plan dimensions for procedural mesh generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BodyPlanVisualization {
    /// Body plan type name.
    pub plan_type: String,
    /// Number of limbs.
    pub limb_count: u8,
    /// Whether this body can fly.
    pub can_fly: bool,
    /// Whether this body can swim.
    pub can_swim: bool,
    /// Typical joint count.
    pub joint_count: u16,
}

impl BodyPlanVisualization {
    /// Create from a sharira `BodyPlan`.
    #[must_use]
    pub fn from_body_plan(plan: crate::preset::BodyPlan) -> Self {
        Self {
            plan_type: format!("{plan:?}"),
            limb_count: plan.limb_count(),
            can_fly: plan.can_fly(),
            can_swim: plan.can_swim(),
            joint_count: plan.typical_joint_count(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_plan_biped() {
        let viz = BodyPlanVisualization::from_body_plan(crate::preset::BodyPlan::Bipedal);
        assert_eq!(viz.limb_count, 2);
        assert!(!viz.can_fly);
        assert!(viz.plan_type.contains("Bipedal"));
    }

    #[test]
    fn body_plan_avian() {
        let viz = BodyPlanVisualization::from_body_plan(crate::preset::BodyPlan::Avian);
        assert!(viz.can_fly);
    }

    #[test]
    fn body_plan_aquatic() {
        let viz = BodyPlanVisualization::from_body_plan(crate::preset::BodyPlan::Aquatic);
        assert!(viz.can_swim);
    }

    #[test]
    fn gait_cycle_human_walk() {
        let gait = crate::gait::Gait::human_walk();
        let viz = GaitCycleVisualization::from_gait(&gait);
        assert_eq!(viz.gait_type, "Walk");
        assert!(viz.duration > 0.0);
        assert!(viz.speed > 0.0);
        assert!(!viz.limb_tracks.is_empty());
    }

    #[test]
    fn gait_cycle_quadruped_trot() {
        let gait = crate::gait::Gait::quadruped_trot();
        let viz = GaitCycleVisualization::from_gait(&gait);
        assert_eq!(viz.gait_type, "Trot");
        assert_eq!(viz.limb_tracks.len(), 4);
    }

    #[test]
    fn muscle_overlay_manual() {
        let overlay = MuscleOverlay {
            muscles: vec![MuscleViz {
                origin: [0.0, 1.0, 0.0],
                insertion: [0.0, 0.5, 0.0],
                activation: 0.7,
                group: "Flexor".into(),
                max_force: 500.0,
            }],
        };
        assert_eq!(overlay.muscles.len(), 1);
        assert!((overlay.muscles[0].activation - 0.7).abs() < 0.01);
    }

    #[test]
    fn skeleton_viz_manual() {
        let viz = SkeletonVisualization {
            bones: vec![BoneSegment {
                start: [0.0, 0.0, 0.0],
                end: [0.0, 0.5, 0.0],
                name: "femur".into(),
                mass: 2.0,
            }],
            joints: vec![JointViz {
                position: [0.0, 0.5, 0.0],
                joint_type: "Hinge".into(),
                dof: 1,
            }],
        };
        assert_eq!(viz.bones.len(), 1);
        assert_eq!(viz.joints.len(), 1);
    }

    #[test]
    fn body_plan_serializes() {
        let viz = BodyPlanVisualization::from_body_plan(crate::preset::BodyPlan::Hexapod);
        let json = serde_json::to_string(&viz);
        assert!(json.is_ok());
    }
}
