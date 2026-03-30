//! Allometric scaling — generate body proportions from mass using power laws.
//!
//! Allometry relates body dimensions to mass across species via:
//! `Y = a × M^b` where M is body mass (kg) and a, b are scaling constants.
//!
//! Based on McMahon (1975), Alexander (2003), and comparative physiology data.

use hisab::Vec3;
use serde::{Deserialize, Serialize};

use crate::preset::BodyPlan;
use crate::skeleton::{Bone, BoneId, Skeleton};

/// Allometric scaling exponents and coefficients.
///
/// Each parameter follows `Y = coefficient × mass^exponent`.
/// Default values are from mammalian comparative physiology (McMahon 1975).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllometricParams {
    /// Bone length: L = coeff × M^exp (meters). Default: 0.30 × M^0.33
    pub bone_length_coeff: f64,
    pub bone_length_exp: f64,
    /// Bone diameter: d = coeff × M^exp (meters). Default: 0.012 × M^0.36
    pub bone_diameter_coeff: f64,
    pub bone_diameter_exp: f64,
    /// Bone mass fraction: m_bone = coeff × M^exp (kg). Default: 0.061 × M^1.09
    pub bone_mass_coeff: f64,
    pub bone_mass_exp: f64,
    /// Muscle force: F = coeff × M^exp (Newtons). Default: 300 × M^0.67
    pub muscle_force_coeff: f64,
    pub muscle_force_exp: f64,
    /// Stride length at walk: L_s = coeff × M^exp (meters). Default: 1.1 × M^0.38
    pub stride_length_coeff: f64,
    pub stride_length_exp: f64,
    /// Stride frequency at walk: f_s = coeff × M^exp (Hz). Default: 1.0 × M^-0.17
    pub stride_frequency_coeff: f64,
    pub stride_frequency_exp: f64,
    /// Heart rate: HR = coeff × M^exp (beats/min). Default: 241 × M^-0.25
    pub heart_rate_coeff: f64,
    pub heart_rate_exp: f64,
    /// Metabolic rate: BMR = coeff × M^exp (Watts). Default: 3.5 × M^0.75
    pub metabolic_rate_coeff: f64,
    pub metabolic_rate_exp: f64,
}

impl Default for AllometricParams {
    fn default() -> Self {
        Self::mammalian()
    }
}

impl AllometricParams {
    /// Standard mammalian scaling (McMahon 1975, Alexander 2003).
    #[must_use]
    pub fn mammalian() -> Self {
        Self {
            bone_length_coeff: 0.30,
            bone_length_exp: 0.33,
            bone_diameter_coeff: 0.012,
            bone_diameter_exp: 0.36,
            bone_mass_coeff: 0.061,
            bone_mass_exp: 1.09,
            muscle_force_coeff: 300.0,
            muscle_force_exp: 0.67,
            stride_length_coeff: 1.1,
            stride_length_exp: 0.38,
            stride_frequency_coeff: 1.0,
            stride_frequency_exp: -0.17,
            heart_rate_coeff: 241.0,
            heart_rate_exp: -0.25,
            metabolic_rate_coeff: 3.5,
            metabolic_rate_exp: 0.75,
        }
    }

    /// Avian scaling (different exponents for flight biomechanics).
    #[must_use]
    pub fn avian() -> Self {
        Self {
            bone_length_coeff: 0.25,
            bone_length_exp: 0.33,
            bone_diameter_coeff: 0.010,
            bone_diameter_exp: 0.34,
            bone_mass_coeff: 0.050,
            bone_mass_exp: 1.07,
            muscle_force_coeff: 250.0,
            muscle_force_exp: 0.67,
            stride_length_coeff: 1.5,
            stride_length_exp: 0.35,
            stride_frequency_coeff: 5.0,
            stride_frequency_exp: -0.20,
            heart_rate_coeff: 480.0,
            heart_rate_exp: -0.23,
            metabolic_rate_coeff: 4.1,
            metabolic_rate_exp: 0.72,
        }
    }

    /// Compute a scaled value: `coefficient × mass^exponent`.
    #[must_use]
    #[inline]
    fn scale(coeff: f64, exp: f64, mass_kg: f64) -> f64 {
        if mass_kg <= 0.0 {
            return 0.0;
        }
        coeff * mass_kg.powf(exp)
    }

    /// Predicted bone length (m) for given body mass (kg).
    #[must_use]
    pub fn bone_length(&self, mass_kg: f64) -> f64 {
        Self::scale(self.bone_length_coeff, self.bone_length_exp, mass_kg)
    }

    /// Predicted bone diameter (m) for given body mass.
    #[must_use]
    pub fn bone_diameter(&self, mass_kg: f64) -> f64 {
        Self::scale(self.bone_diameter_coeff, self.bone_diameter_exp, mass_kg)
    }

    /// Predicted total bone mass (kg) for given body mass.
    #[must_use]
    pub fn bone_mass(&self, mass_kg: f64) -> f64 {
        Self::scale(self.bone_mass_coeff, self.bone_mass_exp, mass_kg)
    }

    /// Predicted max muscle force (N) for given body mass.
    #[must_use]
    pub fn muscle_force(&self, mass_kg: f64) -> f64 {
        Self::scale(self.muscle_force_coeff, self.muscle_force_exp, mass_kg)
    }

    /// Predicted stride length (m) for given body mass.
    #[must_use]
    pub fn stride_length(&self, mass_kg: f64) -> f64 {
        Self::scale(self.stride_length_coeff, self.stride_length_exp, mass_kg)
    }

    /// Predicted stride frequency (Hz) for given body mass.
    #[must_use]
    pub fn stride_frequency(&self, mass_kg: f64) -> f64 {
        Self::scale(
            self.stride_frequency_coeff,
            self.stride_frequency_exp,
            mass_kg,
        )
    }

    /// Predicted resting heart rate (beats/min) for given body mass.
    #[must_use]
    pub fn heart_rate(&self, mass_kg: f64) -> f64 {
        Self::scale(self.heart_rate_coeff, self.heart_rate_exp, mass_kg)
    }

    /// Predicted basal metabolic rate (Watts) for given body mass.
    #[must_use]
    pub fn metabolic_rate(&self, mass_kg: f64) -> f64 {
        Self::scale(self.metabolic_rate_coeff, self.metabolic_rate_exp, mass_kg)
    }

    /// Predicted walking speed (m/s) from stride length × frequency.
    #[must_use]
    pub fn walking_speed(&self, mass_kg: f64) -> f64 {
        self.stride_length(mass_kg) * self.stride_frequency(mass_kg)
    }
}

/// Geometrically scale a skeleton by a uniform factor.
///
/// Scales all bone lengths and positions. Mass scales with volume (factor^3).
#[must_use]
pub fn scale_skeleton(skeleton: &Skeleton, factor: f32) -> Skeleton {
    let factor3 = factor * factor * factor;
    let mut scaled = Skeleton::new(format!("{}_scaled_{:.2}x", skeleton.name, factor));
    for bone in skeleton.bones() {
        let mut new_bone = Bone::new(
            bone.id,
            bone.name.clone(),
            bone.length * factor,
            bone.mass * factor3,
            bone.parent,
        );
        new_bone.local_position = bone.local_position * factor;
        new_bone.local_rotation = bone.local_rotation;
        scaled.add_bone(new_bone);
    }
    scaled
}

/// Generate a skeleton from body mass and body plan using allometric scaling.
///
/// Creates a plausible skeleton with bone counts, lengths, and masses
/// derived from power-law scaling relationships.
#[must_use]
pub fn allometric_skeleton(
    mass_kg: f64,
    body_plan: BodyPlan,
    params: &AllometricParams,
) -> Skeleton {
    let mass_f = mass_kg as f32;
    let bone_len = params.bone_length(mass_kg) as f32;
    let total_bone_mass = params.bone_mass(mass_kg) as f32;
    let limb_count = body_plan.limb_count();
    let joint_count = body_plan.typical_joint_count();

    // Distribute mass across bones proportionally
    let bone_count = joint_count.max(1);
    let mass_per_bone = total_bone_mass / bone_count as f32;

    let mut skeleton = Skeleton::new(format!("{body_plan:?}_{mass_kg:.1}kg"));

    // Build spine (approximately 40% of joints)
    let spine_count = (bone_count as f32 * 0.4).max(1.0) as u16;
    let spine_len = bone_len * 1.2; // spine bones are slightly longer
    let spine_spacing = spine_len / spine_count as f32;

    skeleton.add_bone(Bone::new(
        BoneId(0),
        "pelvis",
        spine_len * 0.3,
        mass_per_bone * 2.0, // pelvis is heavier
        None,
    ));

    for i in 1..spine_count {
        skeleton.add_bone(
            Bone::new(
                BoneId(i),
                format!("spine_{i}"),
                spine_len / spine_count as f32,
                mass_per_bone,
                Some(BoneId(i - 1)),
            )
            .with_position(Vec3::new(0.0, spine_spacing, 0.0)),
        );
    }

    // Head
    let head_id = spine_count;
    skeleton.add_bone(
        Bone::new(
            BoneId(head_id),
            "head",
            bone_len * 0.4,
            mass_per_bone * 1.5,
            Some(BoneId(spine_count - 1)),
        )
        .with_position(Vec3::new(0.0, spine_spacing, 0.0)),
    );

    // Limbs (3 bones per limb: upper, lower, extremity)
    let limb_bone_len = bone_len;
    let lateral_spread = bone_len * 0.3;
    let mut next_id = head_id + 1;

    for limb_idx in 0..limb_count {
        let side = if limb_idx % 2 == 0 { -1.0 } else { 1.0 };
        let attach_bone = if limb_idx < 2 {
            // Front limbs attach to upper spine
            BoneId((spine_count - 1).min(spine_count / 2))
        } else {
            // Rear limbs attach to pelvis
            BoneId(0)
        };

        let lateral_offset = Vec3::new(side * lateral_spread, 0.0, 0.0);

        // Upper limb
        let upper_id = BoneId(next_id);
        skeleton.add_bone(
            Bone::new(
                upper_id,
                format!("limb_{limb_idx}_upper"),
                limb_bone_len * 0.45,
                mass_per_bone * 1.2,
                Some(attach_bone),
            )
            .with_position(lateral_offset + Vec3::new(0.0, -0.05 * mass_f.powf(0.33), 0.0)),
        );
        next_id += 1;

        // Lower limb
        let lower_id = BoneId(next_id);
        skeleton.add_bone(
            Bone::new(
                lower_id,
                format!("limb_{limb_idx}_lower"),
                limb_bone_len * 0.4,
                mass_per_bone,
                Some(upper_id),
            )
            .with_position(Vec3::new(0.0, -limb_bone_len * 0.45, 0.0)),
        );
        next_id += 1;

        // Extremity (foot/hand)
        let ext_id = BoneId(next_id);
        skeleton.add_bone(
            Bone::new(
                ext_id,
                format!("limb_{limb_idx}_extremity"),
                limb_bone_len * 0.15,
                mass_per_bone * 0.5,
                Some(lower_id),
            )
            .with_position(Vec3::new(0.0, -limb_bone_len * 0.4, 0.0)),
        );
        next_id += 1;
    }

    skeleton
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mammalian_bone_length_scales() {
        let params = AllometricParams::mammalian();
        let mouse = params.bone_length(0.025); // 25g mouse
        let human = params.bone_length(70.0); // 70kg human
        let elephant = params.bone_length(5000.0); // 5000kg elephant
        assert!(mouse < human, "mouse bones shorter than human");
        assert!(human < elephant, "human bones shorter than elephant");
    }

    #[test]
    fn muscle_force_scales_sublinearly() {
        let params = AllometricParams::mammalian();
        let f10 = params.muscle_force(10.0);
        let f100 = params.muscle_force(100.0);
        // Force ∝ M^0.67, so 10x mass → 10^0.67 ≈ 4.7x force (not 10x)
        let ratio = f100 / f10;
        assert!(
            (ratio - 4.68).abs() < 0.5,
            "force scaling ratio should be ~4.7, got {ratio:.2}"
        );
    }

    #[test]
    fn stride_frequency_decreases_with_mass() {
        let params = AllometricParams::mammalian();
        let small = params.stride_frequency(1.0);
        let large = params.stride_frequency(1000.0);
        assert!(small > large, "small animals have higher stride frequency");
    }

    #[test]
    fn heart_rate_decreases_with_mass() {
        let params = AllometricParams::mammalian();
        let mouse_hr = params.heart_rate(0.025);
        let human_hr = params.heart_rate(70.0);
        assert!(
            mouse_hr > human_hr,
            "mouse HR ({mouse_hr:.0}) > human HR ({human_hr:.0})"
        );
        // Human resting HR should be ~60-80 bpm
        assert!(
            human_hr > 50.0 && human_hr < 100.0,
            "human HR should be ~70 bpm, got {human_hr:.0}"
        );
    }

    #[test]
    fn walking_speed_reasonable() {
        let params = AllometricParams::mammalian();
        let human_speed = params.walking_speed(70.0);
        // Allometric walking speed for 70kg mammal (~3-5 m/s from power laws)
        // This is preferred speed, not slow walking — larger mammals walk faster
        assert!(
            human_speed > 1.0 && human_speed < 8.0,
            "human walk speed should be reasonable, got {human_speed:.2}"
        );
    }

    #[test]
    fn zero_mass_returns_zero() {
        let params = AllometricParams::mammalian();
        assert_eq!(params.bone_length(0.0), 0.0);
        assert_eq!(params.muscle_force(-1.0), 0.0);
    }

    #[test]
    fn scale_skeleton_doubles_size() {
        let mut orig = Skeleton::new("test");
        orig.add_bone(Bone::new(BoneId(0), "root", 1.0, 10.0, None));
        orig.add_bone(
            Bone::new(BoneId(1), "child", 0.5, 5.0, Some(BoneId(0)))
                .with_position(Vec3::new(0.0, 1.0, 0.0)),
        );

        let scaled = scale_skeleton(&orig, 2.0);
        assert_eq!(scaled.bone_count(), 2);
        assert!(
            (scaled.bones()[0].length - 2.0).abs() < 0.01,
            "length doubled"
        );
        assert!(
            (scaled.bones()[0].mass - 80.0).abs() < 0.01,
            "mass scales with volume (8x)"
        );
        assert!(
            (scaled.bones()[1].local_position.y - 2.0).abs() < 0.01,
            "position doubled"
        );
    }

    #[test]
    fn scale_skeleton_preserves_hierarchy() {
        let mut orig = Skeleton::new("test");
        orig.add_bone(Bone::new(BoneId(0), "root", 1.0, 10.0, None));
        orig.add_bone(Bone::new(BoneId(1), "child", 0.5, 5.0, Some(BoneId(0))));

        let scaled = scale_skeleton(&orig, 0.5);
        assert_eq!(scaled.bones()[1].parent, Some(BoneId(0)));
    }

    #[test]
    fn allometric_skeleton_biped() {
        let params = AllometricParams::mammalian();
        let skeleton = allometric_skeleton(70.0, BodyPlan::Bipedal, &params);

        assert!(
            skeleton.bone_count() > 5,
            "biped should have multiple bones"
        );
        assert_eq!(skeleton.roots().len(), 1, "should have one root");
        assert!(
            skeleton.total_mass() > 1.0,
            "total bone mass should be positive"
        );
    }

    #[test]
    fn allometric_skeleton_quadruped() {
        let params = AllometricParams::mammalian();
        let skeleton = allometric_skeleton(30.0, BodyPlan::Quadruped, &params);

        assert!(
            skeleton.bone_count() > 10,
            "quadruped should have many bones"
        );
        assert!(
            skeleton.total_mass() > 0.5,
            "should have positive total mass"
        );
    }

    #[test]
    fn avian_params_differ_from_mammalian() {
        let mam = AllometricParams::mammalian();
        let avi = AllometricParams::avian();
        // Avian heart rate should be higher at same mass
        assert!(
            avi.heart_rate(1.0) > mam.heart_rate(1.0),
            "birds have faster hearts than mammals at same mass"
        );
    }

    #[test]
    fn metabolic_rate_matches_kleiber() {
        let params = AllometricParams::mammalian();
        let bmr = params.metabolic_rate(70.0);
        // Kleiber: 3.5 × 70^0.75 ≈ 84.7W
        assert!(
            (bmr - 84.7).abs() < 1.0,
            "BMR should match Kleiber's law, got {bmr:.1}"
        );
    }
}
