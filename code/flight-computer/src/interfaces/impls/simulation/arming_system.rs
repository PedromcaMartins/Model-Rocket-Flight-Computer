use core::convert::Infallible;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

use crate::interfaces::ArmingSystem;

static LATEST_DATA: Signal<CriticalSectionRawMutex, ()> = Signal::new();

pub struct SimArming;
impl SimArming {
    pub fn activate() {
        LATEST_DATA.signal(());
    }
}

impl ArmingSystem for SimArming {
    type Error = Infallible;

    /// Wait for button press signal from simulator
    async fn wait_arm(&mut self) -> Result<(), Self::Error> {
        LATEST_DATA.wait().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::fixture;

    use crate::{interfaces::ArmingSystem, test_utils::ms};

    use super::*;

    #[fixture]
    fn sim_arming() -> SimArming {
        LATEST_DATA.reset();
        SimArming
    }

    #[test_log::test(rstest::rstest)]
    #[async_std::test]
    #[serial_test::serial]
    #[case(1)]
    #[case(10)]
    #[case(1_000)]
    #[timeout(ms(100))]
    async fn sim_arming_system(
        #[case] activations: usize,
        mut sim_arming: SimArming,
    ) {
        for _ in 0..activations {
            SimArming::activate();

            let result = sim_arming.wait_arm().await;
            assert!(result.is_ok(), "wait_arm returned an error: {:?}", result.err());
        }
    }

    #[test_log::test(rstest::rstest)]
    #[async_std::test]
    #[serial_test::serial]
    #[timeout(ms(100))]
    #[should_panic(expected = "Timeout 100ms expired")]
    async fn sim_arming_blocks_when_no_data(mut sim_arming: SimArming) {
        let _ = sim_arming.wait_arm().await;
    }
}
