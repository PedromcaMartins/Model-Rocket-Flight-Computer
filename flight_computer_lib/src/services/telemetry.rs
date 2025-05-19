pub mod debug_uart;

pub struct TelemetryService<LOG, WRITERS>
where
    LOG: embedded_io_async::BufRead, 
    WRITERS: embedded_io_async::Write,
{
    log_message_bus: LOG,
    debug_uart_bus: Option<WRITERS>
}

impl<LOG, WRITERS> TelemetryService<LOG, WRITERS>
where
    LOG: embedded_io_async::BufRead, 
    WRITERS: embedded_io_async::Write,
{
    pub fn new(
        log_message_bus: LOG,
    ) -> Self {
        Self {
            log_message_bus,
            debug_uart_bus: None,
        }
    }

    pub fn set_debug_uart(&mut self, debug_uart_bus: WRITERS) {
        self.debug_uart_bus = Some(debug_uart_bus);
    }

    #[inline]
    pub async fn run(self) -> ! {
        let TelemetryService {
            mut log_message_bus,
            mut debug_uart_bus,
        } = self;

        loop {
            let bytes = log_message_bus.fill_buf().await.unwrap();
            let len = bytes.len();

            if let Some(ref mut debug_uart_bus) = debug_uart_bus {
                debug_uart_bus.write_all(
                    bytes
                ).await.unwrap();
            }

            log_message_bus.consume(len);
        }
    }
}
