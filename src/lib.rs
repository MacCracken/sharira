//! # Sharira
//!
//! **Sharira** (शरीर — Sanskrit for "body, physical form") — physiology engine for
//! the AGNOS ecosystem.
//!
//! Provides skeletal structures, joint articulation, musculature, locomotion gaits,
//! biomechanics, and anatomy presets. The physical body that jantu (instinct) drives
//! and soorat (renderer) displays.
//!
//! ## Stack Position
//!
//! ```text
//! soorat  → renders the body (mesh + skinned skeleton)
//! sharira → defines the body (bones, joints, muscles, gaits)
//! jantu   → decides what the body does (instinct, survival)
//! bhava   → shapes how it moves (personality, emotion)
//! impetus → physics (forces, collision, gravity)
//! raasta  → navigation (where to go)
//! ```

pub mod allometry;
pub mod biomechanics;
pub mod body;
pub mod bridge;
pub mod error;
pub mod fatigue;
pub mod gait;
pub mod ik;
/// Integration APIs for downstream consumers (soorat rendering).
pub mod integration;
pub mod joint;
pub mod kinematics;
pub mod morphology;
pub mod muscle;
pub mod pose;
pub mod preset;
pub mod skeleton;

#[cfg(feature = "logging")]
pub mod logging;

pub use allometry::{AllometricParams, allometric_skeleton, scale_skeleton};
pub use body::Body;
pub use error::{Result, ShariraError};
pub use fatigue::FatigueState;
pub use gait::{FootPlacement, Gait, GaitController, GaitCycle, GaitPhase, GaitType};
pub use ik::{IKChain, IKTarget, solve_fabrik, solve_two_bone};
pub use joint::{Joint, JointLimits, JointType};
pub use kinematics::{WorldTransforms, forward_kinematics};
pub use morphology::{Morphology, apply_morphology};
pub use muscle::{Muscle, MuscleGroup};
pub use pose::Pose;
pub use preset::BodyPlan;
pub use skeleton::{Bone, BoneId, Skeleton};
