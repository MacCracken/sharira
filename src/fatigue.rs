use serde::{Deserialize, Serialize};
use tracing::trace;

/// Three-compartment fatigue model (Xia & Frey-Law 2008).
///
/// Motor units exist in three pools:
/// - **Resting (MR)**: available but not recruited
/// - **Active (MA)**: currently producing force
/// - **Fatigued (MF)**: exhausted, recovering
///
/// MR + MA + MF = 1.0 (conservation)
///
/// # Usage
///
/// ```rust
/// use sharira::FatigueState;
///
/// let mut fatigue = FatigueState::fresh();
/// // Simulate 10 seconds at 80% activation, 10ms steps
/// for _ in 0..1000 {
///     fatigue.update(0.8, 0.01);
/// }
/// let capacity = fatigue.capacity();
/// // capacity < 1.0 due to fatigue
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FatigueState {
    /// MR: resting motor units (0-1).
    pub resting: f32,
    /// MA: active motor units (0-1).
    pub active: f32,
    /// MF: fatigued motor units (0-1).
    pub fatigued: f32,
    /// F: fatigue rate constant (1/s, typ. 0.009).
    pub fatigue_rate: f32,
    /// R: recovery rate constant (1/s, typ. 0.002).
    pub recovery_rate: f32,
}

/// Small epsilon to avoid division by zero.
const EPS: f32 = 1e-9;

impl FatigueState {
    /// Fresh (unfatigued) state. All motor units resting.
    #[must_use]
    pub fn fresh() -> Self {
        Self {
            resting: 1.0,
            active: 0.0,
            fatigued: 0.0,
            fatigue_rate: 0.009,
            recovery_rate: 0.002,
        }
    }

    /// Create with custom fatigue/recovery rates.
    #[must_use]
    pub fn with_rates(fatigue_rate: f32, recovery_rate: f32) -> Self {
        Self {
            resting: 1.0,
            active: 0.0,
            fatigued: 0.0,
            fatigue_rate,
            recovery_rate,
        }
    }

    /// Update fatigue state based on current muscle activation demand.
    ///
    /// Three-compartment ODE (Xia & Frey-Law 2008):
    /// ```text
    /// dMA/dt = C(t) - (F + R)·MA    (recruitment minus fatigue/recovery drain)
    /// dMR/dt = -C(t) + R·MF          (recovery replenishes resting pool)
    /// dMF/dt = F·MA - R·MF           (active units fatigue, fatigued recover)
    /// ```
    ///
    /// where `C(t) = activation_demand × (MR / (MR + MF + ε))`
    /// represents the recruitment drive proportional to available resting units.
    ///
    /// Uses implicit Euler for unconditional stability.
    pub fn update(&mut self, activation_demand: f32, dt: f32) {
        if dt <= 0.0 {
            return;
        }

        let demand = activation_demand.clamp(0.0, 1.0);
        let f = self.fatigue_rate.max(0.0);
        let r = self.recovery_rate.max(0.0);

        // Current state
        let mr = self.resting;
        let ma = self.active;
        let mf = self.fatigued;

        // Recruitment drive: C(t) = demand * MR / (MR + MF + ε)
        let pool = mr + mf + EPS;
        let c = demand * (mr / pool);

        // Implicit Euler: solve (x_new - x_old) / dt = f(x_new)
        //
        // For MA: (ma_new - ma) / dt = c - (f + r) * ma_new
        //   => ma_new * (1 + dt*(f+r)) = ma + dt*c
        //   => ma_new = (ma + dt*c) / (1 + dt*(f+r))
        let ma_new = (ma + dt * c) / (1.0 + dt * (f + r));

        // For MF: (mf_new - mf) / dt = f * ma_new - r * mf_new
        //   => mf_new * (1 + dt*r) = mf + dt*f*ma_new
        //   => mf_new = (mf + dt*f*ma_new) / (1 + dt*r)
        let mf_new = (mf + dt * f * ma_new) / (1.0 + dt * r);

        // MR from conservation: MR = 1 - MA - MF
        let mr_new = 1.0 - ma_new - mf_new;

        // Clamp to valid range and renormalize
        let mr_c = mr_new.max(0.0);
        let ma_c = ma_new.max(0.0);
        let mf_c = mf_new.max(0.0);
        let sum = mr_c + ma_c + mf_c;
        if sum > EPS {
            self.resting = mr_c / sum;
            self.active = ma_c / sum;
            self.fatigued = mf_c / sum;
        } else {
            // Degenerate — reset to fresh
            self.resting = 1.0;
            self.active = 0.0;
            self.fatigued = 0.0;
        }

        trace!(
            mr = self.resting,
            ma = self.active,
            mf = self.fatigued,
            demand,
            "fatigue update"
        );
    }

    /// Current force capacity (0-1). Scales muscle max force.
    ///
    /// Capacity = MA / activation_demand when demand > 0.
    /// When no demand is active, capacity reflects the available resting pool.
    /// At full fatigue (MA → 0), capacity → 0.
    #[must_use]
    pub fn capacity(&self) -> f32 {
        // Capacity is the fraction of motor units that are not fatigued.
        // resting + active = the units available or currently working.
        (self.resting + self.active).clamp(0.0, 1.0)
    }

    /// Whether significantly fatigued (capacity < 0.9).
    #[must_use]
    pub fn is_fatigued(&self) -> bool {
        self.capacity() < 0.9
    }

    /// Time to full exhaustion at current activation (approximate, seconds).
    ///
    /// Estimates how long until capacity drops below 10% at the given
    /// sustained activation demand. Returns `f32::INFINITY` if demand is zero.
    #[must_use]
    pub fn time_to_exhaustion(&self, activation_demand: f32) -> f32 {
        if activation_demand <= 0.0 || self.fatigue_rate <= 0.0 {
            return f32::INFINITY;
        }
        let demand = activation_demand.clamp(0.0, 1.0);

        // Approximate: at steady state, fatigue drains active units at rate F.
        // The available pool (resting) is consumed by recruitment.
        // Rough estimate: time ≈ resting / (demand * fatigue_rate)
        // This gives a first-order approximation.
        let available = self.resting + self.active;
        if available <= 0.1 {
            return 0.0;
        }
        available / (demand * self.fatigue_rate)
    }

    /// Reset to fresh state.
    pub fn reset(&mut self) {
        self.resting = 1.0;
        self.active = 0.0;
        self.fatigued = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 0.01; // 10ms timestep

    #[test]
    fn fresh_state_full_capacity() {
        let state = FatigueState::fresh();
        assert!(
            (state.capacity() - 1.0).abs() < 1e-6,
            "fresh capacity should be 1.0, got {}",
            state.capacity()
        );
    }

    #[test]
    fn sustained_effort_reduces_capacity() {
        let mut state = FatigueState::fresh();
        // 60 seconds at full activation
        for _ in 0..6000 {
            state.update(1.0, DT);
        }
        assert!(
            state.capacity() < 0.95,
            "capacity should drop after sustained effort, got {}",
            state.capacity()
        );
    }

    #[test]
    fn recovery_at_rest() {
        let mut state = FatigueState::fresh();
        // Fatigue first: 60s at full activation
        for _ in 0..6000 {
            state.update(1.0, DT);
        }
        let fatigued_capacity = state.capacity();

        // Recover: 600s at rest (recovery rate R=0.002 → τ≈500s, need multiple τ)
        for _ in 0..60000 {
            state.update(0.0, DT);
        }
        let recovered_capacity = state.capacity();
        assert!(
            recovered_capacity > fatigued_capacity,
            "capacity should recover at rest: fatigued={fatigued_capacity}, recovered={recovered_capacity}"
        );
    }

    #[test]
    fn conservation() {
        let mut state = FatigueState::fresh();
        for _ in 0..1000 {
            state.update(0.7, DT);
            let sum = state.resting + state.active + state.fatigued;
            assert!(
                (sum - 1.0).abs() < 1e-5,
                "MR + MA + MF should be 1.0, got {sum}"
            );
        }
    }

    #[test]
    fn higher_activation_fatigues_faster() {
        let mut state_high = FatigueState::fresh();
        let mut state_low = FatigueState::fresh();

        for _ in 0..6000 {
            state_high.update(1.0, DT);
            state_low.update(0.5, DT);
        }
        assert!(
            state_high.capacity() < state_low.capacity(),
            "100% demand should fatigue faster than 50%: high={}, low={}",
            state_high.capacity(),
            state_low.capacity()
        );
    }

    #[test]
    fn custom_rates() {
        // Fast fatigue rate should exhaust quicker
        let mut fast = FatigueState::with_rates(0.05, 0.002);
        let mut slow = FatigueState::with_rates(0.005, 0.002);

        for _ in 0..3000 {
            fast.update(1.0, DT);
            slow.update(1.0, DT);
        }
        assert!(
            fast.capacity() < slow.capacity(),
            "faster fatigue rate should exhaust quicker: fast={}, slow={}",
            fast.capacity(),
            slow.capacity()
        );
    }

    #[test]
    fn zero_dt_no_change() {
        let mut state = FatigueState::fresh();
        state.update(1.0, 0.0);
        assert!(
            (state.resting - 1.0).abs() < 1e-6,
            "zero dt should not change state"
        );
        assert!(state.active.abs() < 1e-6);
        assert!(state.fatigued.abs() < 1e-6);
    }

    #[test]
    fn large_dt_stable() {
        let mut state = FatigueState::fresh();
        // Absurdly large timestep — implicit Euler should not explode
        state.update(1.0, 1000.0);
        assert!(
            state.resting >= 0.0 && state.resting <= 1.0,
            "resting out of range: {}",
            state.resting
        );
        assert!(
            state.active >= 0.0 && state.active <= 1.0,
            "active out of range: {}",
            state.active
        );
        assert!(
            state.fatigued >= 0.0 && state.fatigued <= 1.0,
            "fatigued out of range: {}",
            state.fatigued
        );
        let sum = state.resting + state.active + state.fatigued;
        assert!(
            (sum - 1.0).abs() < 1e-5,
            "conservation violated with large dt: {sum}"
        );
    }

    #[test]
    fn time_to_exhaustion_finite() {
        let state = FatigueState::fresh();
        let tte = state.time_to_exhaustion(1.0);
        assert!(
            tte > 0.0 && tte < f32::INFINITY,
            "time to exhaustion should be finite and positive, got {tte}"
        );
        // With default rates (F=0.009), rough estimate ~ 1.0 / 0.009 ≈ 111s
        assert!(
            tte > 10.0 && tte < 500.0,
            "time to exhaustion should be reasonable, got {tte}"
        );
    }

    #[test]
    fn time_to_exhaustion_zero_demand() {
        let state = FatigueState::fresh();
        let tte = state.time_to_exhaustion(0.0);
        assert!(
            tte == f32::INFINITY,
            "zero demand should give infinite time, got {tte}"
        );
    }

    #[test]
    fn reset_restores_fresh() {
        let mut state = FatigueState::fresh();
        for _ in 0..6000 {
            state.update(1.0, DT);
        }
        assert!(state.capacity() < 1.0);
        state.reset();
        assert!(
            (state.capacity() - 1.0).abs() < 1e-6,
            "reset should restore full capacity"
        );
        assert!((state.resting - 1.0).abs() < 1e-6);
        assert!(state.active.abs() < 1e-6);
        assert!(state.fatigued.abs() < 1e-6);
    }

    #[test]
    fn is_fatigued_threshold() {
        let state = FatigueState::fresh();
        assert!(!state.is_fatigued(), "fresh state should not be fatigued");

        let mut state = FatigueState::fresh();
        // Long sustained effort
        for _ in 0..10000 {
            state.update(1.0, DT);
        }
        assert!(
            state.is_fatigued(),
            "should be fatigued after sustained effort, capacity={}",
            state.capacity()
        );
    }

    #[test]
    fn negative_dt_no_change() {
        let mut state = FatigueState::fresh();
        state.update(1.0, -0.01);
        assert!(
            (state.resting - 1.0).abs() < 1e-6,
            "negative dt should not change state"
        );
    }

    #[test]
    fn serde_roundtrip() {
        let mut state = FatigueState::fresh();
        for _ in 0..100 {
            state.update(0.5, DT);
        }
        let json = serde_json::to_string(&state).expect("serialize");
        let deserialized: FatigueState = serde_json::from_str(&json).expect("deserialize");
        assert!((deserialized.resting - state.resting).abs() < 1e-6);
        assert!((deserialized.active - state.active).abs() < 1e-6);
        assert!((deserialized.fatigued - state.fatigued).abs() < 1e-6);
    }
}
