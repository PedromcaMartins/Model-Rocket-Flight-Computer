use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal, watch::Watch};
use embassy_time::{Duration, MockDriver};
use flight_computer_lib::{model::system_status::FlightState, tasks::finite_state_machine_task};
use uom::si::{f64::Length, length::meter};
use log::info;
use rstest::{fixture, rstest};
use futures::join;

#[fixture]
fn arm_button_pushed_signal() -> &'static Signal<CriticalSectionRawMutex, ()> {
    static SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
    &SIGNAL
}

#[fixture]
fn latest_altitude_signal() -> &'static Signal<CriticalSectionRawMutex, Length> {
    static SIGNAL: Signal<CriticalSectionRawMutex, Length> = Signal::new();
    &SIGNAL
}

#[fixture]
fn flight_state_watch() -> &'static Watch<CriticalSectionRawMutex, FlightState, 1> {
    static WATCH: Watch<CriticalSectionRawMutex, FlightState, 1> = 
        embassy_sync::watch::Watch::new();
    &WATCH
}

#[fixture]
fn launchpad_altitude() -> Length {
    Length::new::<meter>(1.0)
}

#[fixture]
fn ascent_altitudes() -> std::vec::Vec<Length> {
    std::vec![
        Length::new::<meter>(1.0),
        Length::new::<meter>(2.5),
        Length::new::<meter>(4.0),
        Length::new::<meter>(5.0),
        Length::new::<meter>(4.0),
        Length::new::<meter>(3.5),
    ]
}

#[fixture]
fn descent_altitudes() -> std::vec::Vec<Length> {
    std::vec![
        Length::new::<meter>(3.0),
        Length::new::<meter>(2.5),
        Length::new::<meter>(2.0),
        Length::new::<meter>(1.5),
    ]
}

async fn wait(time_driver: &MockDriver) {
    async_std::task::sleep(Duration::from_millis(100).into()).await;
    time_driver.advance(Duration::from_millis(100));
}

#[rstest]
#[timeout(Duration::from_secs(5).into())]
#[test_log::test(async_std::test)]
async fn test_fsm_full_trajectory(
    arm_button_pushed_signal: &'static Signal<CriticalSectionRawMutex, ()>,
    latest_altitude_signal: &'static Signal<CriticalSectionRawMutex, Length>,
    flight_state_watch: &'static Watch<CriticalSectionRawMutex, FlightState, 1>,

    launchpad_altitude: Length,
    ascent_altitudes: std::vec::Vec<Length>,
    descent_altitudes: std::vec::Vec<Length>,
) {
    let flight_state_sender = flight_state_watch.sender();
    let mut flight_state_receiver = flight_state_watch.receiver().unwrap();

    // --- Task: Simulate Flight States ---
    let flight_state_task = async move {
        let time_driver = MockDriver::get();

        // Pre-Armed state
        latest_altitude_signal.signal(launchpad_altitude);
        wait(time_driver).await;
        assert_eq!(flight_state_receiver.get().await, FlightState::PreArmed);
        info!("Pre-Armed State!");

        // Armed state
        arm_button_pushed_signal.signal(());
        wait(time_driver).await;
        assert_eq!(flight_state_receiver.get().await, FlightState::Armed);
        info!("Armed State!");

        // Recovery Activated state
        for altitude in ascent_altitudes {
            latest_altitude_signal.signal(altitude);
            info!("Altitude: {} m", altitude.get::<meter>());
            wait(time_driver).await;
        }
        assert_eq!(flight_state_receiver.get().await, FlightState::RecoveryActivated);
        info!("Recovery Activated State!");

        // Touchdown state
        for altitude in descent_altitudes {
            latest_altitude_signal.signal(altitude);
            info!("Altitude: {} m", altitude.get::<meter>());
            wait(time_driver).await;
        }
        assert_eq!(flight_state_receiver.get().await, FlightState::Touchdown);
        info!("Touchdown State!");
    };

    // --- FSM Task ---
    let fsm_task = finite_state_machine_task(
        arm_button_pushed_signal, 
        latest_altitude_signal,
        flight_state_sender,
    );

    // --- Run all tasks concurrently ---
    join!(flight_state_task, fsm_task);
}
