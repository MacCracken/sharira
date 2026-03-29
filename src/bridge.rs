//! Cross-crate bridges — convert sharira physiology values to/from
//! other AGNOS science crate parameters.
//!
//! Always available — takes primitive values (f32/f64), no science crate deps.
//!
//! # Architecture
//!
//! ```text
//! sharira body data ──> bridge ──> impetus (physics constraints)
//!                              ──> ushma   (thermal parameters)
//!                              ──> dravya  (material properties)
//! ```

// ── Impetus bridges (physics) ──────────────────────────────────────────────

/// Convert joint angles (rad) and stiffness (0-1) to constraint restoring torque (Nm).
///
/// Linear spring model: τ = -k_max × stiffness × θ
/// where k_max = 100 Nm/rad is a reference stiffness.
#[must_use]
#[inline]
pub fn joint_to_constraint_torque(stiffness_normalized: f32, angle_rad: f32) -> f64 {
    let k_max = 100.0_f64;
    -k_max * stiffness_normalized.clamp(0.0, 1.0) as f64 * angle_rad as f64
}

/// Convert joint damping (0-1) and angular velocity (rad/s) to damping torque (Nm).
///
/// τ = -c_max × damping × ω, where c_max = 10 Nm·s/rad.
#[must_use]
#[inline]
pub fn joint_to_damping_torque(damping_normalized: f32, angular_vel_rads: f32) -> f64 {
    let c_max = 10.0_f64;
    -c_max * damping_normalized.clamp(0.0, 1.0) as f64 * angular_vel_rads as f64
}

/// Convert bone mass (kg) and dimensions to box inertia tensor diagonal (kg·m²).
///
/// Returns `[Ixx, Iyy, Izz]` for a solid rectangular approximation.
#[must_use]
pub fn bone_to_box_inertia(mass_kg: f32, length_m: f32, width_m: f32, depth_m: f32) -> [f64; 3] {
    let m12 = mass_kg as f64 / 12.0;
    let l2 = (length_m as f64).powi(2);
    let w2 = (width_m as f64).powi(2);
    let d2 = (depth_m as f64).powi(2);
    [m12 * (w2 + d2), m12 * (l2 + d2), m12 * (l2 + w2)]
}

/// Convert bone mass (kg), length (m), and radius (m) to cylinder inertia
/// tensor diagonal (kg·m²).
///
/// Models bone as a solid cylinder along Y-axis.
/// Returns `[Ixx, Iyy, Izz]`.
#[must_use]
pub fn bone_to_cylinder_inertia(mass_kg: f32, length_m: f32, radius_m: f32) -> [f64; 3] {
    let m = mass_kg as f64;
    let r2 = (radius_m as f64).powi(2);
    let h2 = (length_m as f64).powi(2);
    let ixx = m / 12.0 * (3.0 * r2 + h2);
    let iyy = m / 2.0 * r2;
    [ixx, iyy, ixx]
}

/// Convert muscle force (N) and moment arm (m) to joint torque (Nm).
///
/// τ = F × r
#[must_use]
#[inline]
pub fn muscle_to_joint_torque(force_n: f32, moment_arm_m: f32) -> f64 {
    force_n as f64 * moment_arm_m as f64
}

/// Convert body total mass (kg), limb count, and gravity (m/s²) to
/// per-limb ground contact force (N) for static standing.
///
/// F = m × g / n_limbs
#[must_use]
#[inline]
pub fn body_to_limb_force(mass_kg: f32, limb_count: u8, gravity: f32) -> f64 {
    if limb_count == 0 {
        return 0.0;
    }
    mass_kg as f64 * gravity as f64 / limb_count as f64
}

/// Convert gait GRF (N) to impetus force vector `[0, F, 0]` (upward reaction).
#[must_use]
#[inline]
pub fn grf_to_force_vector(grf_n: f32) -> [f64; 3] {
    [0.0, grf_n.abs() as f64, 0.0]
}

// ── Ushma bridges (thermodynamics) ─────────────────────────────────────────

/// Estimate metabolic heat rate (W) from muscle mechanical power (W).
///
/// Muscles are ~25% efficient; ~75% of metabolic energy is heat.
/// P_heat = P_mech × (1/η - 1) ≈ P_mech × 3.0
#[must_use]
#[inline]
pub fn muscle_power_to_heat(mechanical_power_w: f32) -> f64 {
    mechanical_power_w.abs() as f64 * 3.0
}

/// Estimate basal metabolic rate (W) from body mass (kg).
///
/// Kleiber's law: BMR = 3.5 × m^0.75
#[must_use]
#[inline]
pub fn body_mass_to_bmr(mass_kg: f32) -> f64 {
    if mass_kg <= 0.0 {
        return 0.0;
    }
    3.5 * (mass_kg as f64).powf(0.75)
}

/// Estimate body surface area (m²) from mass (kg) and height (m).
///
/// Du Bois formula: BSA = 0.007184 × m^0.425 × h_cm^0.725
#[must_use]
pub fn body_surface_area(mass_kg: f32, height_m: f32) -> f64 {
    if mass_kg <= 0.0 || height_m <= 0.0 {
        return 0.0;
    }
    let h_cm = height_m as f64 * 100.0;
    0.007184 * (mass_kg as f64).powf(0.425) * h_cm.powf(0.725)
}

/// Convert skin surface area (m²) and temperatures (K) to radiative heat loss (W).
///
/// Stefan-Boltzmann: Q = ε × σ × A × (T_skin⁴ - T_env⁴)
/// Skin emissivity ≈ 0.98.
#[must_use]
pub fn skin_radiation_loss(
    surface_area_m2: f64,
    skin_temperature_k: f64,
    environment_temperature_k: f64,
) -> f64 {
    const STEFAN_BOLTZMANN: f64 = 5.670_374_419e-8;
    const SKIN_EMISSIVITY: f64 = 0.98;
    if surface_area_m2 <= 0.0 || skin_temperature_k <= 0.0 || environment_temperature_k <= 0.0 {
        return 0.0;
    }
    SKIN_EMISSIVITY
        * STEFAN_BOLTZMANN
        * surface_area_m2
        * (skin_temperature_k.powi(4) - environment_temperature_k.powi(4))
}

/// Estimate heat generation (W) from muscle activation level and max force.
///
/// Empirical: ~0.15 W per Newton of max force at full activation.
#[must_use]
#[inline]
pub fn muscle_activation_heat(activation: f32, max_force_n: f32) -> f64 {
    let a = activation.clamp(0.0, 1.0) as f64;
    0.15 * a * max_force_n.abs() as f64
}

// ── Dravya bridges (material science) ──────────────────────────────────────

/// Convert bone density (kg/m³) to estimated Young's modulus (Pa).
///
/// Carter & Hayes (1977): E = 3790 × ρ³ (ρ in g/cm³).
#[must_use]
#[inline]
pub fn bone_density_to_youngs_modulus(density_kg_m3: f32) -> f64 {
    if density_kg_m3 <= 0.0 {
        return 0.0;
    }
    let rho = density_kg_m3 as f64 / 1000.0;
    3790.0e6 * rho.powi(3)
}

/// Convert bone density (kg/m³) to estimated yield strength (Pa).
///
/// Keller (1994): σ_y = 137 × ρ^1.88 (ρ in g/cm³).
#[must_use]
#[inline]
pub fn bone_density_to_yield_strength(density_kg_m3: f32) -> f64 {
    if density_kg_m3 <= 0.0 {
        return 0.0;
    }
    let rho = density_kg_m3 as f64 / 1000.0;
    137.0e6 * rho.powf(1.88)
}

/// Convert muscle force (N) and tendon cross-section (m²) to tendon stress (Pa).
///
/// σ = F / A
#[must_use]
#[inline]
pub fn muscle_force_to_tendon_stress(force_n: f32, cross_section_m2: f64) -> f64 {
    if cross_section_m2 <= 0.0 {
        return 0.0;
    }
    force_n.abs() as f64 / cross_section_m2
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Impetus ────────────────────────────────────────────────────────

    #[test]
    fn constraint_torque_spring() {
        let t = joint_to_constraint_torque(0.5, 0.1);
        // -100 × 0.5 × 0.1 = -5.0
        assert!((t - -5.0).abs() < 0.01);
    }

    #[test]
    fn damping_torque() {
        let t = joint_to_damping_torque(1.0, 2.0);
        // -10 × 1.0 × 2.0 = -20.0
        assert!((t - -20.0).abs() < 0.01);
    }

    #[test]
    fn box_inertia_cube() {
        let i = bone_to_box_inertia(1.0, 1.0, 1.0, 1.0);
        let expected = 1.0 / 6.0;
        assert!((i[0] - expected).abs() < 0.001);
    }

    #[test]
    fn cylinder_inertia() {
        let i = bone_to_cylinder_inertia(1.0, 1.0, 0.1);
        assert!(i[0] > 0.0 && i[1] > 0.0);
        assert!(i[0] > i[1], "Ixx should be > Iyy for a long cylinder");
    }

    #[test]
    fn muscle_torque() {
        let t = muscle_to_joint_torque(1000.0, 0.04);
        assert!((t - 40.0).abs() < 0.01);
    }

    #[test]
    fn limb_force_biped() {
        let f = body_to_limb_force(70.0, 2, 9.81);
        assert!((f - 343.35).abs() < 0.1);
    }

    #[test]
    fn limb_force_zero_limbs() {
        assert_eq!(body_to_limb_force(70.0, 0, 9.81), 0.0);
    }

    #[test]
    fn grf_vector() {
        let v = grf_to_force_vector(686.7);
        assert_eq!(v[0], 0.0);
        assert!((v[1] - 686.7).abs() < 0.1);
        assert_eq!(v[2], 0.0);
    }

    // ── Ushma ──────────────────────────────────────────────────────────

    #[test]
    fn muscle_heat() {
        let h = muscle_power_to_heat(100.0);
        assert!((h - 300.0).abs() < 0.1);
    }

    #[test]
    fn bmr_human() {
        let bmr = body_mass_to_bmr(70.0);
        assert!((bmr - 84.7).abs() < 1.0);
    }

    #[test]
    fn bmr_zero() {
        assert_eq!(body_mass_to_bmr(0.0), 0.0);
    }

    #[test]
    fn bsa_human() {
        let bsa = body_surface_area(70.0, 1.75);
        assert!((bsa - 1.85).abs() < 0.1);
    }

    #[test]
    fn radiation_positive() {
        let q = skin_radiation_loss(1.85, 310.0, 293.0);
        assert!(q > 50.0 && q < 300.0, "radiation = {q}W");
    }

    #[test]
    fn activation_heat() {
        let h = muscle_activation_heat(0.5, 1000.0);
        assert!((h - 75.0).abs() < 0.1);
    }

    // ── Dravya ─────────────────────────────────────────────────────────

    #[test]
    fn bone_youngs_cortical() {
        let e = bone_density_to_youngs_modulus(1800.0);
        assert!(e > 10e9 && e < 30e9, "cortical bone E = {e}");
    }

    #[test]
    fn bone_yield_cortical() {
        let sy = bone_density_to_yield_strength(1800.0);
        assert!(sy > 50e6 && sy < 500e6, "cortical bone σ_y = {sy}");
    }

    #[test]
    fn tendon_stress() {
        let s = muscle_force_to_tendon_stress(1000.0, 50e-6);
        assert!((s - 20e6).abs() < 1e6);
    }

    #[test]
    fn tendon_stress_zero_area() {
        assert_eq!(muscle_force_to_tendon_stress(1000.0, 0.0), 0.0);
    }
}
