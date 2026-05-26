use std::sync::Arc;

use arc_swap::ArcSwap;
use derive_more::Display;
use proto::actuator_data::{ActuatorStatus, LedStatus};
use proto::uom::si::f32::{Force, Time, Velocity};

use crate::config::SimulatorConfig;
use crate::physics::state::PhysicsState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum ForceEvent {
    #[display("Motor Thrust")]
    MotorThrust,
    #[display("Recovery")]
    Recovery,
    #[display("Gravity")]
    Gravity,
    #[display("Ground")]
    Ground,
}

impl ForceEvent {
    // Computes the force applied by this event on the rocket.
    // Represented under 1D.
    // Positive values indicating upward force.
    pub fn compute_force(&self, state: &PhysicsState) -> Force {
        let gravity_force = SimulatorConfig::rocket_mass() * SimulatorConfig::gravity();

        match self {
            ForceEvent::Gravity => -(gravity_force),
            ForceEvent::Ground => gravity_force,
            ForceEvent::MotorThrust => SimulatorConfig::motor_avg_thrust(),
            ForceEvent::Recovery => recovery_force(state, gravity_force),
        }
    }
}

/// Parachute drag force during recovery.
///
/// Model:
///   1. **Canopy inflation** — over `[0, recovery_activation_delay]` the
///      effective drag area ramps from 0 (closed) to 1 (fully open) on a
///      smoothstep curve `u²(3 − 2u)`. The curve is C¹ so velocity has no
///      jerk at deployment or full inflation.
///   2. **Quadratic drag** — at full inflation, drag opposes motion with
///      magnitude `(m·g) · (v / v_terminal)²`. Calibrated so that
///      `|drag| = m·g` exactly when `|v| = v_terminal` (the definition of
///      terminal velocity).
fn recovery_force(state: &PhysicsState, weight: Force) -> Force {
    let Some(t_deployed) = state.recovery_deployed else {
        return Force::default();
    };

    let act = SimulatorConfig::recovery_activation_delay();
    let u = if act > Time::default() {
        let dt = state.time - t_deployed;
        (dt / act).value.clamp(0.0, 1.0)
    } else {
        1.0
    };
    let area_factor = u.powi(2) * (3.0 - 2.0 * u);

    let v_term = SimulatorConfig::terminal_velocity();
    if v_term <= Velocity::default() {
        return Force::default();
    }
    let ratio = state.velocity / v_term;
    let sign = if state.velocity < Velocity::default() { 1.0 } else { -1.0 };

    weight * (ratio.value.powi(2) * area_factor * sign)
}

#[derive(Debug, Default)]
pub struct ActiveForceEvent(ArcSwap<Vec<ForceEvent>>);

impl ActiveForceEvent {
    pub fn recompute(&self, state: &PhysicsState) {
        self.0.store(Arc::new(state.active_force_events()));
    }

    pub fn load(&self) -> Arc<Vec<ForceEvent>> {
        self.0.load_full()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SimActuatorSnapshot {
    pub postcard_led: LedStatus,
    pub altimeter_led: LedStatus,
    pub gps_led: LedStatus,
    pub imu_led: LedStatus,
    pub arm_led: LedStatus,
    pub file_system_led: LedStatus,
    pub deployment_led: LedStatus,
    pub ground_station_led: LedStatus,
    pub deployment: ActuatorStatus,
}


