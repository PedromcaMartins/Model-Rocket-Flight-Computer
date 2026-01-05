use tokio::sync::watch;

pub struct SimulatorUi {
    button_tx: watch::Sender<bool>,
    sd_card_detect_tx: watch::Sender<bool>,
    sd_card_status_led_rx: watch::Receiver<bool>,
}

impl SimulatorUi {
    pub fn new(
        button_tx: watch::Sender<bool>,
        sd_card_detect_tx: watch::Sender<bool>,
        sd_card_status_led_rx: watch::Receiver<bool>,
    ) -> Self {
        Self {
            button_tx,
            sd_card_detect_tx,
            sd_card_status_led_rx,
        }
    }
}
