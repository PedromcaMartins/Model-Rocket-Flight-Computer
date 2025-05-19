pub struct DebugUartService<LOG, UART> 
where 
    LOG: embedded_io_async::BufRead, 
    UART: embedded_io_async::Write, 
{
    debug_uart: UART,
    log_message_bus: LOG,
}

impl<LOG, UART> DebugUartService<LOG, UART> 
where 
    LOG: embedded_io_async::BufRead, 
    UART: embedded_io_async::Write, 
{
    pub fn new(debug_uart: UART, log_message_bus: LOG) -> Self {
        Self {
            debug_uart,
            log_message_bus,
        }
    }

    #[inline]
    pub async fn run(mut self) -> ! {
        let DebugUartService { 
            ref mut debug_uart, 
            mut log_message_bus,
        } = self;

        loop {
            let bytes = log_message_bus.fill_buf().await.unwrap();
            let len = bytes.len();

            debug_uart.write_all(
                bytes
            ).await.unwrap();

            log_message_bus.consume(len);
        }
    }
}
