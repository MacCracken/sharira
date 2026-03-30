//! Morphology variation — parametric anatomy for within-species diversity.
//!
//! A [`Morphology`] defines per-bone scale factors that modify a skeleton's
//! proportions without changing its hierarchy. This enables population-level
//! diversity (tall vs short, heavy vs lean) and individual variation.

use serde::{Deserialize, Serialize};

use crate::skeleton::{Bone, BoneId, Skeleton};

/// Per-bone scale factors for morphological variation.
///
/// Each factor multiplies the corresponding bone property:
/// - `length_scale`: bone length (default 1.0)
/// - `mass_scale`: bone mass (default 1.0)
/// - `width_scale`: visual width/girth (default 1.0, does not affect physics)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoneScale {
    pub bone_id: BoneId,
    pub length_scale: f32,
    pub mass_scale: f32,
    pub width_scale: f32,
}

impl BoneScale {
    /// Uniform scale (all factors equal).
    #[must_use]
    pub fn uniform(bone_id: BoneId, scale: f32) -> Self {
        Self {
            bone_id,
            length_scale: scale,
            mass_scale: scale * scale * scale, // volume scaling
            width_scale: scale,
        }
    }

    /// Identity (no change).
    #[must_use]
    pub fn identity(bone_id: BoneId) -> Self {
        Self {
            bone_id,
            length_scale: 1.0,
            mass_scale: 1.0,
            width_scale: 1.0,
        }
    }
}

/// A complete morphology specification for a skeleton.
///
/// Contains per-bone scale factors. Bones without explicit entries
/// use identity (no modification).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Morphology {
    pub name: String,
    pub bone_scales: Vec<BoneScale>,
    /// Global height multiplier (affects all bone lengths uniformly).
    pub height_factor: f32,
    /// Global mass multiplier (affects all bone masses uniformly).
    pub mass_factor: f32,
}

impl Morphology {
    /// Identity morphology (no changes).
    #[must_use]
    pub fn identity() -> Self {
        Self {
            name: "identity".into(),
            bone_scales: Vec::new(),
            height_factor: 1.0,
            mass_factor: 1.0,
        }
    }

    /// Average body proportions (identity).
    #[must_use]
    pub fn average() -> Self {
        Self {
            name: "average".into(),
            bone_scales: Vec::new(),
            height_factor: 1.0,
            mass_factor: 1.0,
        }
    }

    /// Heavy/stocky build: wider, heavier, slightly shorter.
    #[must_use]
    pub fn heavy() -> Self {
        Self {
            name: "heavy".into(),
            bone_scales: Vec::new(),
            height_factor: 0.95,
            mass_factor: 1.4,
        }
    }

    /// Lean/slender build: narrower, lighter.
    #[must_use]
    pub fn lean() -> Self {
        Self {
            name: "lean".into(),
            bone_scales: Vec::new(),
            height_factor: 1.0,
            mass_factor: 0.75,
        }
    }

    /// Tall build: longer bones, proportionally scaled mass.
    #[must_use]
    pub fn tall() -> Self {
        Self {
            name: "tall".into(),
            bone_scales: Vec::new(),
            height_factor: 1.15,
            mass_factor: 1.15 * 1.15 * 1.15, // ~1.52, volume scaling
        }
    }

    /// Compact build: shorter, denser.
    #[must_use]
    pub fn compact() -> Self {
        Self {
            name: "compact".into(),
            bone_scales: Vec::new(),
            height_factor: 0.85,
            mass_factor: 1.1,
        }
    }

    /// Add a per-bone scale override.
    pub fn with_bone_scale(mut self, scale: BoneScale) -> Self {
        self.bone_scales.push(scale);
        self
    }

    /// Generate a random morphology variation.
    ///
    /// `variance` controls the spread: 0.0 = no variation, 0.1 = ±10%.
    /// Uses a simple deterministic hash of `seed` for reproducibility.
    #[must_use]
    pub fn random(seed: u64, variance: f32) -> Self {
        let variance = variance.clamp(0.0, 0.5);
        // Simple deterministic pseudo-random from seed
        let h = |s: u64| -> f32 {
            let x = s
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let bits = ((x >> 33) ^ x) as u32;
            (bits as f32 / u32::MAX as f32) * 2.0 - 1.0 // -1.0 to 1.0
        };
        let height = 1.0 + h(seed) * variance;
        let mass = height.powi(3) * (1.0 + h(seed.wrapping_add(1)) * variance * 0.5);

        Self {
            name: format!("random_{seed}"),
            bone_scales: Vec::new(),
            height_factor: height,
            mass_factor: mass,
        }
    }

    /// Get the scale for a specific bone. Returns global factors if no override.
    #[must_use]
    fn bone_length_scale(&self, bone_id: BoneId) -> f32 {
        self.bone_scales
            .iter()
            .find(|s| s.bone_id == bone_id)
            .map_or(self.height_factor, |s| s.length_scale * self.height_factor)
    }

    /// Get the mass scale for a specific bone.
    #[must_use]
    fn bone_mass_scale(&self, bone_id: BoneId) -> f32 {
        self.bone_scales
            .iter()
            .find(|s| s.bone_id == bone_id)
            .map_or(self.mass_factor, |s| s.mass_scale * self.mass_factor)
    }
}

/// Apply a morphology to a skeleton, producing a new skeleton with modified proportions.
///
/// Bone hierarchy and names are preserved. Lengths, masses, and positions are scaled.
#[must_use]
pub fn apply_morphology(skeleton: &Skeleton, morphology: &Morphology) -> Skeleton {
    let mut result = Skeleton::new(format!("{}_{}", skeleton.name, morphology.name));

    for bone in skeleton.bones() {
        let len_scale = morphology.bone_length_scale(bone.id);
        let mass_scale = morphology.bone_mass_scale(bone.id);

        let mut new_bone = Bone::new(
            bone.id,
            bone.name.clone(),
            bone.length * len_scale,
            bone.mass * mass_scale,
            bone.parent,
        );
        // Scale position by parent's length scale (approximate)
        let pos_scale = bone
            .parent
            .map_or(1.0, |pid| morphology.bone_length_scale(pid));
        new_bone.local_position = bone.local_position * pos_scale;
        new_bone.local_rotation = bone.local_rotation;
        result.add_bone(new_bone);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use hisab::Vec3;

    fn test_skeleton() -> Skeleton {
        let mut skeleton = Skeleton::new("test");
        skeleton.add_bone(Bone::new(BoneId(0), "root", 0.5, 10.0, None));
        skeleton.add_bone(
            Bone::new(BoneId(1), "spine", 0.4, 8.0, Some(BoneId(0)))
                .with_position(Vec3::new(0.0, 0.5, 0.0)),
        );
        skeleton.add_bone(
            Bone::new(BoneId(2), "arm", 0.6, 4.0, Some(BoneId(1)))
                .with_position(Vec3::new(0.2, 0.3, 0.0)),
        );
        skeleton
    }

    #[test]
    fn identity_preserves_skeleton() {
        let orig = test_skeleton();
        let result = apply_morphology(&orig, &Morphology::identity());
        assert_eq!(result.bone_count(), orig.bone_count());
        for (a, b) in orig.bones().iter().zip(result.bones().iter()) {
            assert!((a.length - b.length).abs() < 1e-5);
            assert!((a.mass - b.mass).abs() < 1e-5);
        }
    }

    #[test]
    fn tall_increases_lengths() {
        let orig = test_skeleton();
        let tall = apply_morphology(&orig, &Morphology::tall());
        for (a, b) in orig.bones().iter().zip(tall.bones().iter()) {
            assert!(b.length > a.length, "tall should increase bone lengths");
        }
    }

    #[test]
    fn heavy_increases_mass() {
        let orig = test_skeleton();
        let heavy = apply_morphology(&orig, &Morphology::heavy());
        assert!(
            heavy.total_mass() > orig.total_mass(),
            "heavy should increase total mass"
        );
    }

    #[test]
    fn lean_decreases_mass() {
        let orig = test_skeleton();
        let lean = apply_morphology(&orig, &Morphology::lean());
        assert!(
            lean.total_mass() < orig.total_mass(),
            "lean should decrease total mass"
        );
    }

    #[test]
    fn compact_shorter_bones() {
        let orig = test_skeleton();
        let compact = apply_morphology(&orig, &Morphology::compact());
        for (a, b) in orig.bones().iter().zip(compact.bones().iter()) {
            assert!(b.length < a.length, "compact should shorten bones");
        }
    }

    #[test]
    fn preserves_hierarchy() {
        let orig = test_skeleton();
        let result = apply_morphology(&orig, &Morphology::heavy());
        for (a, b) in orig.bones().iter().zip(result.bones().iter()) {
            assert_eq!(a.parent, b.parent);
            assert_eq!(a.id, b.id);
        }
    }

    #[test]
    fn per_bone_override() {
        let orig = test_skeleton();
        let morph = Morphology::identity().with_bone_scale(BoneScale {
            bone_id: BoneId(2),
            length_scale: 2.0,
            mass_scale: 8.0, // volume scaled
            width_scale: 2.0,
        });
        let result = apply_morphology(&orig, &morph);
        // Bone 2 should be doubled, others unchanged
        assert!((result.bones()[2].length - 1.2).abs() < 0.01, "arm doubled");
        assert!(
            (result.bones()[0].length - 0.5).abs() < 0.01,
            "root unchanged"
        );
    }

    #[test]
    fn random_produces_variation() {
        let m1 = Morphology::random(42, 0.1);
        let m2 = Morphology::random(99, 0.1);
        assert!(
            (m1.height_factor - m2.height_factor).abs() > 0.001,
            "different seeds should produce different morphologies"
        );
    }

    #[test]
    fn random_bounded() {
        for seed in 0..100 {
            let m = Morphology::random(seed, 0.1);
            assert!(
                m.height_factor > 0.8 && m.height_factor < 1.2,
                "height should be within ±20% at 10% variance: {}",
                m.height_factor
            );
        }
    }

    #[test]
    fn zero_variance_is_identity() {
        let m = Morphology::random(42, 0.0);
        assert!((m.height_factor - 1.0).abs() < 1e-5);
    }

    #[test]
    fn presets_have_names() {
        assert_eq!(Morphology::average().name, "average");
        assert_eq!(Morphology::heavy().name, "heavy");
        assert_eq!(Morphology::lean().name, "lean");
        assert_eq!(Morphology::tall().name, "tall");
        assert_eq!(Morphology::compact().name, "compact");
    }
}
