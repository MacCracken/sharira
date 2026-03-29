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

pub mod error;
pub mod skeleton;
pub mod joint;
pub mod muscle;
pub mod gait;
pub mod biomechanics;
pub mod preset;

#[cfg(feature = "logging")]
pub mod logging;

pub use error::{ShariraError, Result};
pub use skeleton::{Skeleton, Bone, BoneId};
pub use joint::{Joint, JointType, JointLimits};
pub use muscle::{Muscle, MuscleGroup};
pub use gait::{Gait, GaitPhase, GaitCycle};
pub use preset::BodyPlan;
