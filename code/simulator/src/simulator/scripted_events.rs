use tokio::time::Duration;

#[derive(Copy, Clone)]
pub struct ScriptedEvents {
    pub arm_button_press_time: Option<Duration>,
    pub auto_motor_ignition: Option<Duration>,
}

impl Default for ScriptedEvents {
    fn default() -> Self {
        Self {
            arm_button_press_time: Some(Duration::from_secs(5)),
            auto_motor_ignition: Some(Duration::from_secs(10)),
        }
    }
}
