use std::time::Duration;

use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal, watch::Watch};
use flight_computer_lib::{model::system_status::FlightState, tasks::finite_state_machine_task};
use uom::si::{f64::Length, length::meter};
use log::info;
use rstest::{fixture, rstest};
use futures::join;

const CONSUMERS: usize = 100; // Number of consumers for the flight state watch

#[fixture]
fn arm_button_pushed_signal() -> &'static Signal<CriticalSectionRawMutex, ()> {
    static SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
    SIGNAL.reset();
    &SIGNAL
}

#[fixture]
fn latest_altitude_signal() -> &'static Signal<CriticalSectionRawMutex, Length> {
    static SIGNAL: Signal<CriticalSectionRawMutex, Length> = Signal::new();
    SIGNAL.reset();
    &SIGNAL
}

#[fixture]
fn flight_state_watch() -> &'static Watch<CriticalSectionRawMutex, FlightState, CONSUMERS> {
    static WATCH: Watch<CriticalSectionRawMutex, FlightState, CONSUMERS> = 
        embassy_sync::watch::Watch::new();
    WATCH.sender().send(FlightState::PreArmed);
    &WATCH
}

#[fixture]
fn flight_altitudes() -> std::vec::Vec<Length> {
    let csv_content = std::fs::read_to_string(
        format!("{}/tests/simulation.csv", env!("CARGO_MANIFEST_DIR"))
    ).expect("Failed to read simulation.csv");

    csv_content
        .lines()
        .filter_map(|line| {
            line.trim()
                .parse::<f64>()
                .or_else(|_| 
                    line.trim()
                    .parse::<u64>()
                    .and_then(|value| Ok(value as f64))
                ).ok()
        })
        .map(|altitude| Length::new::<meter>(altitude))
        .collect()
}

/// 1000x speed up time driver advancement
async fn wait() {
    async_std::task::sleep(Duration::from_micros(1)).await;
}

/// 1000x speed up time driver advancement
async fn send_altitude(
    latest_altitude_signal: &'static Signal<CriticalSectionRawMutex, Length>,
    altitude: Length, 
) {
    latest_altitude_signal.signal(altitude);
    wait().await;
}

async fn send_arm_button_pushed(
    arm_button_pushed_signal: &'static Signal<CriticalSectionRawMutex, ()>,
) {
    arm_button_pushed_signal.signal(());
    wait().await;
}

async fn pre_arm(
    latest_altitude_signal: &'static Signal<CriticalSectionRawMutex, Length>,
    flight_state_receiver: &mut embassy_sync::watch::Receiver<'static, CriticalSectionRawMutex, FlightState, CONSUMERS>,
) {
    send_altitude(latest_altitude_signal, Length::new::<meter>(1.0)).await;
    assert_eq!(flight_state_receiver.get().await, FlightState::PreArmed);
    info!("Pre-Armed State!");
}

async fn arm(
    arm_button_pushed_signal: &'static Signal<CriticalSectionRawMutex, ()>,
    flight_state_receiver: &mut embassy_sync::watch::Receiver<'static, CriticalSectionRawMutex, FlightState, CONSUMERS>,
) {
    send_arm_button_pushed(arm_button_pushed_signal).await;
    assert_eq!(flight_state_receiver.get().await, FlightState::Armed);
    info!("Armed State!");
}

#[rstest]
#[timeout(Duration::from_secs(5).into())]
#[test_log::test(async_std::test)]
#[serial_test::serial]
async fn test_fsm_full_trajectory(
    arm_button_pushed_signal: &'static Signal<CriticalSectionRawMutex, ()>,
    latest_altitude_signal: &'static Signal<CriticalSectionRawMutex, Length>,
    flight_state_watch: &'static Watch<CriticalSectionRawMutex, FlightState, CONSUMERS>,

    flight_altitudes: std::vec::Vec<Length>,
) {
    let flight_state_sender = flight_state_watch.sender();
    let mut flight_state_receiver = flight_state_watch.receiver().unwrap();

    let test_task = async move {
        // Pre-Armed state
        pre_arm(latest_altitude_signal, &mut flight_state_receiver).await;

        // Armed state
        arm(arm_button_pushed_signal, &mut flight_state_receiver).await;

        // Recovery Activated state
        for altitude in flight_altitudes {
            send_altitude(latest_altitude_signal, altitude).await;
            info!("Altitude: {} m", altitude.get::<meter>());
        }
        assert_eq!(flight_state_receiver.get().await, FlightState::Touchdown);
        info!("Touchdown State!");
    };

    let fsm_task = finite_state_machine_task(
        arm_button_pushed_signal, 
        latest_altitude_signal,
        flight_state_sender,
    );

    // --- Run all tasks concurrently ---
    join!(test_task, fsm_task);
}

#[rstest]
#[timeout(Duration::from_secs(5).into())]
#[test_log::test(async_std::test)]
#[serial_test::serial]
async fn test_fsm_negative_altitude_handling(
    arm_button_pushed_signal: &'static Signal<CriticalSectionRawMutex, ()>,
    latest_altitude_signal: &'static Signal<CriticalSectionRawMutex, Length>,
    flight_state_watch: &'static Watch<CriticalSectionRawMutex, FlightState, CONSUMERS>,

    flight_altitudes: std::vec::Vec<Length>,
) {
    let flight_state_sender = flight_state_watch.sender();
    let mut flight_state_receiver = flight_state_watch.receiver().unwrap();
    let offset = Length::new::<meter>(-100.0);

    let test_task = async move {
        // Start below sea level
        send_altitude(latest_altitude_signal, offset).await;
        assert_eq!(flight_state_receiver.get().await, FlightState::PreArmed);

        arm(arm_button_pushed_signal, &mut flight_state_receiver).await;

        // Recovery Activated state
        for altitude in flight_altitudes {
            send_altitude(latest_altitude_signal, altitude + offset).await;
            info!("Altitude: {} m", altitude.get::<meter>());
        }
        assert_eq!(flight_state_receiver.get().await, FlightState::Touchdown);
        info!("Touchdown State!");
    };

    let fsm_task = finite_state_machine_task(
        arm_button_pushed_signal,
        latest_altitude_signal,
        flight_state_sender,
    );

    // --- Run all tasks concurrently ---
    join!(test_task, fsm_task);
}

#[rstest]
#[timeout(Duration::from_secs(10).into())]
#[test_log::test(async_std::test)]
#[serial_test::serial]
async fn test_fsm_prolonged_flight_duration(
    arm_button_pushed_signal: &'static Signal<CriticalSectionRawMutex, ()>,
    latest_altitude_signal: &'static Signal<CriticalSectionRawMutex, Length>,
    flight_state_watch: &'static Watch<CriticalSectionRawMutex, FlightState, CONSUMERS>,
) {
    let flight_state_sender = flight_state_watch.sender();
    let mut flight_state_receiver = flight_state_watch.receiver().unwrap();

    let test_task = async move {
        pre_arm(latest_altitude_signal, &mut flight_state_receiver).await;

        arm(arm_button_pushed_signal, &mut flight_state_receiver).await;

        // Long ascent phase
        let mut altitude = 1.0;
        for _ in 0..20 {
            altitude += 0.5;
            send_altitude(latest_altitude_signal, Length::new::<meter>(altitude)).await;
        }
        assert_eq!(flight_state_receiver.get().await, FlightState::RecoveryActivated);

        // Long descent phase
        for _ in 0..20 {
            altitude -= 0.4;
            send_altitude(latest_altitude_signal, Length::new::<meter>(altitude)).await;
        }
        assert_eq!(flight_state_receiver.get().await, FlightState::Touchdown);
    };

    let fsm_task = finite_state_machine_task(
        arm_button_pushed_signal,
        latest_altitude_signal,
        flight_state_sender,
    );

    // --- Run all tasks concurrently ---
    join!(test_task, fsm_task);
}

#[rstest]
#[timeout(Duration::from_secs(10).into())]
#[test_log::test(async_std::test)]
#[serial_test::serial]
async fn test_fsm_stuck_in_pre_armed_without_button_press(
    arm_button_pushed_signal: &'static Signal<CriticalSectionRawMutex, ()>,
    latest_altitude_signal: &'static Signal<CriticalSectionRawMutex, Length>,
    flight_state_watch: &'static Watch<CriticalSectionRawMutex, FlightState, CONSUMERS>,
) {
    let flight_state_sender = flight_state_watch.sender();
    let mut flight_state_receiver = flight_state_watch.receiver().unwrap();

    let test_task = async move {
        pre_arm(latest_altitude_signal, &mut flight_state_receiver).await;

        // Wait longer and verify still in PreArmed state
        for _ in 0..u16::MAX as u32 * 20 {
            wait().await;
        }
        assert_eq!(flight_state_receiver.get().await, FlightState::PreArmed);
        info!("Still in Pre-Armed State after extended wait");
    };

    let fsm_task = finite_state_machine_task(
        arm_button_pushed_signal,
        latest_altitude_signal,
        flight_state_sender,
    );

    // --- FSM is not expected to stop ---
    match select (
        test_task, 
        fsm_task, 
    ).await {
        Either::First(()) => (),
        Either::Second(()) => panic!("FSM completed unexpectedly"),
    }
}

#[rstest]
#[timeout(Duration::from_secs(5).into())]
#[test_log::test(async_std::test)]
#[serial_test::serial]
async fn test_fsm_recovery_activation_failsafe(
    arm_button_pushed_signal: &'static Signal<CriticalSectionRawMutex, ()>,
    latest_altitude_signal: &'static Signal<CriticalSectionRawMutex, Length>,
    flight_state_watch: &'static Watch<CriticalSectionRawMutex, FlightState, CONSUMERS>,
) {
    let flight_state_sender = flight_state_watch.sender();
    let mut flight_state_receiver = flight_state_watch.receiver().unwrap();

    let test_task = async move {
        pre_arm(latest_altitude_signal, &mut flight_state_receiver).await;

        arm(arm_button_pushed_signal, &mut flight_state_receiver).await;

        // Send altitude changes that are too small to trigger recovery
        let low_altitudes = vec![
            Length::new::<meter>(1.1),
            Length::new::<meter>(1.2),
            Length::new::<meter>(1.0),
            Length::new::<meter>(0.9),
        ];

        for _ in 0..u16::MAX {
            for altitude in low_altitudes.clone() {
                send_altitude(latest_altitude_signal, altitude).await;
                assert_eq!(flight_state_receiver.get().await, FlightState::Armed);
            }
        }
    };

    let fsm_task = finite_state_machine_task(
        arm_button_pushed_signal,
        latest_altitude_signal,
        flight_state_sender,
    );

    // --- Run all tasks concurrently ---
    match select (
        test_task, 
        fsm_task, 
    ).await {
        Either::First(()) => (),
        Either::Second(()) => panic!("FSM completed unexpectedly"),
    }
}

#[rstest]
#[timeout(Duration::from_secs(5).into())]
#[test_log::test(async_std::test)]
#[serial_test::serial]
async fn test_fsm_rapid_altitude_changes_during_prearm(
    arm_button_pushed_signal: &'static Signal<CriticalSectionRawMutex, ()>,
    latest_altitude_signal: &'static Signal<CriticalSectionRawMutex, Length>,
    flight_state_watch: &'static Watch<CriticalSectionRawMutex, FlightState, CONSUMERS>,
) {
    let flight_state_sender = flight_state_watch.sender();
    let mut flight_state_receiver = flight_state_watch.receiver().unwrap();

    let test_task = async move {
        pre_arm(latest_altitude_signal, &mut flight_state_receiver).await;

        // Rapid altitude changes
        for _ in 0..10 {
            send_altitude(latest_altitude_signal, Length::new::<meter>(4.0)).await;
            send_altitude(latest_altitude_signal, Length::new::<meter>(2.0)).await;
            send_altitude(latest_altitude_signal, Length::new::<meter>(3.0)).await;
        }
        assert_eq!(flight_state_receiver.get().await, FlightState::PreArmed);
    };

    let fsm_task = finite_state_machine_task(
        arm_button_pushed_signal,
        latest_altitude_signal,
        flight_state_sender,
    );

    // --- Run all tasks concurrently ---
    match select (
        test_task, 
        fsm_task, 
    ).await {
        Either::First(()) => (),
        Either::Second(()) => panic!("FSM completed unexpectedly"),
    }
}

#[rstest]
#[timeout(Duration::from_secs(5).into())]
#[test_log::test(async_std::test)]
#[serial_test::serial]
async fn test_fsm_multiple_arm_button_presses(
    arm_button_pushed_signal: &'static Signal<CriticalSectionRawMutex, ()>,
    latest_altitude_signal: &'static Signal<CriticalSectionRawMutex, Length>,
    flight_state_watch: &'static Watch<CriticalSectionRawMutex, FlightState, CONSUMERS>,
) {
    let flight_state_sender = flight_state_watch.sender();
    let mut flight_state_receiver = flight_state_watch.receiver().unwrap();

    let test_task = async move {
        pre_arm(latest_altitude_signal, &mut flight_state_receiver).await;

        arm(arm_button_pushed_signal, &mut flight_state_receiver).await;

        // Press arm button multiple times
        send_arm_button_pushed(arm_button_pushed_signal).await;
        send_arm_button_pushed(arm_button_pushed_signal).await;
        send_arm_button_pushed(arm_button_pushed_signal).await;
        send_arm_button_pushed(arm_button_pushed_signal).await;
        send_arm_button_pushed(arm_button_pushed_signal).await;
        send_arm_button_pushed(arm_button_pushed_signal).await;
    };

    let fsm_task = finite_state_machine_task(
        arm_button_pushed_signal,
        latest_altitude_signal,
        flight_state_sender,
    );

    // --- Run all tasks concurrently ---
    match select (
        test_task, 
        fsm_task, 
    ).await {
        Either::First(()) => (),
        Either::Second(()) => panic!("FSM completed unexpectedly"),
    }
}
