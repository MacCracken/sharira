//! Bioenergetics bridge — rasayan integration for energy-driven fatigue dynamics.
//!
//! Uses rasayan's energy module to ground sharira's fatigue model in ATP/phosphocreatine
//! dynamics rather than empirical rate constants.
//!
//! Requires the `bioenergetics` feature.
//!
//! # Coupling Points
//!
//! - **Mechanical power → ATP demand**: [`atp_demand`] converts muscle watts to metabolic cost
//! - **Energy state → Fatigue rate**: [`modulate_fatigue`] scales fatigue/recovery rates
//!   based on phosphocreatine and glycogen reserves
//! - **Activity → MET**: [`met_from_activity`] converts muscle power + body mass to MET

use rasayan::energy;
use serde::{Deserialize, Serialize};

use crate::fatigue::FatigueState;

/// Metabolic state tracked alongside fatigue.
///
/// Wraps rasayan's [`energy::BioenergyState`] with sharira-specific coupling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetabolicCoupling {
    /// Rasayan bioenergy state (phosphocreatine, glycogen, MET).
    pub energy: energy::BioenergyState,
    /// Body mass in kg (for MET calculations).
    pub body_mass_kg: f64,
    /// How strongly energy depletion amplifies fatigue (1.0 = no effect, 5.0 = strong).
    pub fatigue_amplification: f64,
    /// How much anaerobic metabolism slows recovery (0.0–1.0 fraction of aerobic rate).
    pub anaerobic_recovery_fraction: f64,
}

impl Default for MetabolicCoupling {
    fn default() -> Self {
        Self {
            energy: energy::BioenergyState::default(),
            body_mass_kg: 70.0,
            fatigue_amplification: 3.0,
            anaerobic_recovery_fraction: 0.3,
        }
    }
}

impl MetabolicCoupling {
    /// Create with specific body mass.
    #[must_use]
    pub fn with_mass(body_mass_kg: f64) -> Self {
        Self {
            body_mass_kg,
            ..Self::default()
        }
    }

    /// Compute ATP demand (mM/s) from total mechanical power output.
    #[must_use]
    #[inline]
    pub fn atp_demand(&self, mechanical_power_w: f64) -> f64 {
        energy::atp_demand_from_power(mechanical_power_w)
    }

    /// Compute MET level from mechanical power output.
    #[must_use]
    #[inline]
    pub fn met_from_activity(&self, mechanical_power_w: f64) -> f64 {
        energy::met_from_power(mechanical_power_w, self.body_mass_kg)
    }

    /// Update energy state and modulate fatigue rates for the current tick.
    ///
    /// Call this each tick before `FatigueState::update`. It:
    /// 1. Sets MET level from mechanical power
    /// 2. Ticks the bioenergy state (phosphocreatine/glycogen depletion/recovery)
    /// 3. Adjusts fatigue and recovery rates based on energy availability
    pub fn tick(&mut self, fatigue: &mut FatigueState, mechanical_power_w: f64, dt_seconds: f32) {
        // Compute MET from power
        let met = self.met_from_activity(mechanical_power_w);
        self.energy.set_exertion(met);

        // Tick energy state (expects dt in minutes)
        let dt_minutes = f64::from(dt_seconds) / 60.0;
        self.energy.tick(dt_minutes);

        // Modulate fatigue rate based on energy depletion
        let energy_avail = self.energy.energy_available();
        let fatigue_multiplier =
            energy::fatigue_rate_from_energy(energy_avail, self.fatigue_amplification);
        let recovery_multiplier = energy::recovery_rate_modifier(
            self.energy.is_anaerobic(),
            self.anaerobic_recovery_fraction,
        );

        // Apply to fatigue state — scale from baseline rates
        // Base rates: F=0.009, R=0.002 (Xia & Frey-Law 2008)
        fatigue.fatigue_rate = (0.009 * fatigue_multiplier as f32).min(0.1);
        fatigue.recovery_rate = (0.002 * recovery_multiplier as f32).max(0.0005);

        tracing::trace!(
            met,
            energy_avail,
            fatigue_rate = fatigue.fatigue_rate,
            recovery_rate = fatigue.recovery_rate,
            anaerobic = self.energy.is_anaerobic(),
            "bioenergetics tick"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_metabolic_coupling() {
        let mc = MetabolicCoupling::default();
        assert!((mc.body_mass_kg - 70.0).abs() < f64::EPSILON);
        assert!(mc.energy.energy_available() > 0.7);
    }

    #[test]
    fn test_atp_demand_scales_with_power() {
        let mc = MetabolicCoupling::default();
        let d100 = mc.atp_demand(100.0);
        let d200 = mc.atp_demand(200.0);
        assert!(d200 > d100, "Higher power should demand more ATP");
    }

    #[test]
    fn test_met_from_activity() {
        let mc = MetabolicCoupling::default();
        let met_rest = mc.met_from_activity(0.0);
        let met_jog = mc.met_from_activity(100.0);
        assert!((met_rest - 1.0).abs() < 0.1);
        assert!(met_jog > 5.0);
    }

    #[test]
    fn test_tick_modulates_fatigue_rates() {
        let mut mc = MetabolicCoupling::default();
        let mut fatigue = FatigueState::fresh();

        // At rest: rates should be near baseline
        mc.tick(&mut fatigue, 0.0, 0.1);
        assert!(
            (fatigue.fatigue_rate - 0.009).abs() < 0.005,
            "Resting fatigue rate should be near baseline"
        );

        // After heavy exertion depleting energy: fatigue rate should increase
        for _ in 0..600 {
            mc.tick(&mut fatigue, 500.0, 0.1);
            fatigue.update(0.9, 0.1);
        }
        assert!(
            fatigue.fatigue_rate > 0.009,
            "Depleted energy should amplify fatigue rate: {}",
            fatigue.fatigue_rate
        );
    }

    #[test]
    fn test_energy_depletes_under_exertion() {
        let mut mc = MetabolicCoupling::default();
        let mut fatigue = FatigueState::fresh();
        let initial_energy = mc.energy.energy_available();

        for _ in 0..600 {
            mc.tick(&mut fatigue, 300.0, 0.1);
        }

        assert!(
            mc.energy.energy_available() < initial_energy,
            "Energy should deplete under exertion"
        );
    }

    #[test]
    fn test_recovery_slowed_when_anaerobic() {
        let mut mc = MetabolicCoupling::default();
        let mut fatigue = FatigueState::fresh();

        // Push into anaerobic territory
        mc.energy.set_exertion(10.0);
        for _ in 0..100 {
            mc.energy.tick(1.0);
        }

        mc.tick(&mut fatigue, 0.0, 0.1);
        if mc.energy.is_anaerobic() {
            assert!(
                fatigue.recovery_rate < 0.002,
                "Anaerobic should slow recovery: {}",
                fatigue.recovery_rate
            );
        }
    }
}
