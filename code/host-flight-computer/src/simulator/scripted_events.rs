use tokio::time::Duration;

#[derive(Copy, Clone)]
pub struct ScriptedEvents {
    pub auto_motor_ignition: Option<Duration>,
}

impl Default for ScriptedEvents {
    fn default() -> Self {
        Self {
            auto_motor_ignition: Some(Duration::from_secs(5)),
        }
    }
}
