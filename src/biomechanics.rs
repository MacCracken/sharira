/// Center of mass from weighted positions.
///
/// CoM = Σ(m_i × p_i) / Σ(m_i)
#[must_use]
pub fn center_of_mass(masses: &[f32], positions: &[[f32; 3]]) -> [f32; 3] {
    if masses.len() != positions.len() || masses.is_empty() {
        return [0.0; 3];
    }
    let total_mass: f32 = masses.iter().sum();
    if total_mass <= 0.0 { return [0.0; 3]; }

    let mut com = [0.0_f32; 3];
    for (m, p) in masses.iter().zip(positions.iter()) {
        com[0] += m * p[0];
        com[1] += m * p[1];
        com[2] += m * p[2];
    }
    com[0] /= total_mass;
    com[1] /= total_mass;
    com[2] /= total_mass;
    com
}

/// Ground reaction force for bipedal stance (simplified).
///
/// GRF = mass × g during stance. During swing, GRF = 0.
#[must_use]
#[inline]
pub fn ground_reaction_force(mass_kg: f32, gravity: f32, duty_factor: f32) -> f32 {
    // Average GRF over cycle: body weight / duty factor
    if duty_factor <= 0.0 { return 0.0; }
    mass_kg * gravity / duty_factor
}

/// Metabolic cost of locomotion (cost of transport).
///
/// CoT = energy_per_step / (mass × distance)
/// Approximate: CoT ≈ 3.4 J/(kg·m) for walking, ~3.8 for running.
#[must_use]
#[inline]
pub fn cost_of_transport(energy_j: f32, mass_kg: f32, distance_m: f32) -> f32 {
    if mass_kg <= 0.0 || distance_m <= 0.0 { return 0.0; }
    energy_j / (mass_kg * distance_m)
}

/// Balance margin — distance from CoM projection to support polygon edge.
/// Positive = stable, negative = falling.
#[must_use]
pub fn balance_margin(com_x: f32, support_min_x: f32, support_max_x: f32) -> f32 {
    let to_min = com_x - support_min_x;
    let to_max = support_max_x - com_x;
    to_min.min(to_max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn com_two_equal_masses() {
        let com = center_of_mass(&[1.0, 1.0], &[[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]]);
        assert!((com[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn com_weighted() {
        let com = center_of_mass(&[3.0, 1.0], &[[0.0, 0.0, 0.0], [4.0, 0.0, 0.0]]);
        assert!((com[0] - 1.0).abs() < 0.01, "CoM should be at 1.0, got {}", com[0]);
    }

    #[test]
    fn grf_walking() {
        // 70kg, 9.81, duty 0.6 → GRF ≈ 1144 N (more than body weight due to duty < 1.0)
        let grf = ground_reaction_force(70.0, 9.81, 0.6);
        assert!(grf > 70.0 * 9.81, "GRF should exceed body weight during walking");
    }

    #[test]
    fn cot_walking() {
        let cot = cost_of_transport(240.0, 70.0, 1.0);
        assert!((cot - 3.43).abs() < 0.1, "walking CoT should be ~3.4, got {cot}");
    }

    #[test]
    fn balance_stable() {
        let margin = balance_margin(0.5, 0.0, 1.0);
        assert!(margin > 0.0, "CoM within support → positive margin");
    }

    #[test]
    fn balance_unstable() {
        let margin = balance_margin(1.5, 0.0, 1.0);
        assert!(margin < 0.0, "CoM outside support → negative margin");
    }

    #[test]
    fn com_empty_returns_zero() {
        let com = center_of_mass(&[], &[]);
        assert_eq!(com, [0.0; 3]);
    }
}
