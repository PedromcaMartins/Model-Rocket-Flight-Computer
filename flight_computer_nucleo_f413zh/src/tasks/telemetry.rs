use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, pipe::{self, Pipe}};
use static_cell::ConstStaticCell;

use crate::{io_mapping::DebugUart, logger::{init_async_logger, init_logger_rtt}};
use flight_computer_lib::services::telemetry::{debug_uart::DebugUartService, TelemetryService};

const LOGGER_PIPE_CAPACITY: usize = 32;

type LoggerMutex = NoopRawMutex;
pub type LoggerPipe = pipe::Pipe<LoggerMutex, LOGGER_PIPE_CAPACITY>;
pub type LoggerPipeReader = pipe::Reader<'static, LoggerMutex, LOGGER_PIPE_CAPACITY>;
pub type LoggerPipeWriter = pipe::Writer<'static, LoggerMutex, LOGGER_PIPE_CAPACITY>;

static LOGGER_PIPE_CELL: ConstStaticCell<LoggerPipe> = ConstStaticCell::new(Pipe::new());
static DEBUG_UART_PIPE_CELL: ConstStaticCell<LoggerPipe> = ConstStaticCell::new(Pipe::new());

pub struct TelemetryTasks {
    telemetry_service: TelemetryService<LoggerPipeReader, LoggerPipeWriter>,
    debug_uart_service: Option<DebugUartService<LoggerPipeReader, DebugUart<'static>>>,
}

impl TelemetryTasks {
    pub fn new() -> Self {
        let logger_pipe = LOGGER_PIPE_CELL.take();
        let (logger_reader, logger_writer) = logger_pipe.split();
        init_async_logger(logger_writer);

        Self {
            telemetry_service: TelemetryService::new(logger_reader),
            debug_uart_service: None,
        }
    }

    pub fn use_rtt_service(self) -> Self {
        init_logger_rtt();

        self
    }

    pub fn use_debug_uart_service(mut self, debug_uart: DebugUart<'static>) -> Self {
        let debug_uart_pipe = DEBUG_UART_PIPE_CELL.take();
        let (debug_uart_reader, debug_uart_writer) = debug_uart_pipe.split();
        let debug_uart_service = DebugUartService::new(debug_uart, debug_uart_reader);

        self.telemetry_service.set_debug_uart(debug_uart_writer);

        Self {
            debug_uart_service: Some(debug_uart_service),
            ..self
        }
    }

    pub fn spawn(self, spawner: &Spawner) {
        defmt::unwrap!(spawner.spawn(telemetry_service_task(self.telemetry_service)));

        if let Some(debug_uart_service) = self.debug_uart_service {
            defmt::unwrap!(spawner.spawn(debug_uart_service_task(debug_uart_service)));
        }
    }
}

#[embassy_executor::task]
pub async fn telemetry_service_task(service: TelemetryService<LoggerPipeReader, LoggerPipeWriter>) -> ! {
    service.run().await;
}

#[embassy_executor::task]
pub async fn debug_uart_service_task(service: DebugUartService<LoggerPipeReader, DebugUart<'static>>) -> ! {
    service.run().await;
}
