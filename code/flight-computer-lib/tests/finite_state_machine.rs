use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, MockDriver};
use flight_computer_lib::tasks::finite_state_machine_task;
use uom::si::{f64::Length, length::meter};
use log::info;
use rstest::{fixture, rstest};
use futures::join;

#[fixture]
fn arm_button_signal() -> &'static Signal<CriticalSectionRawMutex, ()> {
    static SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
    &SIGNAL
}

#[fixture]
fn altitude_signal() -> &'static Signal<CriticalSectionRawMutex, Length> {
    static SIGNAL: Signal<CriticalSectionRawMutex, Length> = Signal::new();
    &SIGNAL
}

#[fixture]
fn altitudes() -> (std::vec::Vec<Length>, Duration) {
    let altitudes = std::vec![
        Length::new::<meter>(1.0),
        Length::new::<meter>(1.0),
        Length::new::<meter>(1.0),
        Length::new::<meter>(2.0),
        Length::new::<meter>(3.0),
        Length::new::<meter>(4.0),
        Length::new::<meter>(3.0),
        Length::new::<meter>(2.0),
        Length::new::<meter>(1.0),
        Length::new::<meter>(1.0),
        Length::new::<meter>(1.0),
    ];

    (altitudes, Duration::from_millis(500))
}

#[rstest]
#[timeout(Duration::from_secs(10).into())]
#[test_log::test(async_std::test)]
async fn test_fsm(
    arm_button_signal: &'static Signal<CriticalSectionRawMutex, ()>,
    altitude_signal: &'static Signal<CriticalSectionRawMutex, Length>,
    altitudes: (std::vec::Vec<Length>, Duration),
) {
    // --- Task: Simulate Flight States ---
    let arm_task = async move {
        let time_driver = MockDriver::get();

        time_driver.advance(Duration::from_secs(1));
        arm_button_signal.signal(());
        info!("Arming System!");

        async_std::task::sleep(Duration::from_millis(100).into()).await;

        let (altitudes, interval) = altitudes;
        for altitude in altitudes {
            altitude_signal.signal(altitude);
            time_driver.advance(interval);
            info!("Altitude: {} m", altitude.get::<meter>());
            async_std::task::sleep(Duration::from_millis(100).into()).await;
        }
    };

    // --- FSM Task ---
    let fsm_task = finite_state_machine_task(
        arm_button_signal, 
        altitude_signal
    );

    // --- Run all tasks concurrently ---
    join!(arm_task, fsm_task);
}
