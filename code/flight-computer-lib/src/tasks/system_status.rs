use defmt_or_log::{info, Debug2Format};
use embassy_futures::select::{select, select3, select4, Either, Either3, Either4};
use embassy_sync::{blocking_mutex::raw::RawMutex, channel, watch};
use embassy_time::{Instant, Timer};

use crate::model::system_status::{AltimeterSystemStatus, ArmButtonSystemStatus, FlightState, GpsSystemStatus, ImuSystemStatus, SdCardSystemStatus, SystemStatus};

#[inline]
pub async fn system_status_task<
    M, 
    const ALTITUDE_STATUS_CHANNEL_DEPTH: usize,
    const ARM_BUTTON_STATUS_CHANNEL_DEPTH: usize,
    const IMU_STATUS_CHANNEL_DEPTH: usize,
    const GPS_STATUS_CHANNEL_DEPTH: usize,
    const SD_CARD_STATUS_CHANNEL_DEPTH: usize,
    const FLIGHT_STATE_WATCH_CONSUMERS: usize,
> (
    altitude_status_receiver:   channel::Receiver<'static, M, Result<AltimeterSystemStatus, usize>, ALTITUDE_STATUS_CHANNEL_DEPTH>,
    arm_button_status_receiver: channel::Receiver<'static, M, Result<ArmButtonSystemStatus, usize>, ARM_BUTTON_STATUS_CHANNEL_DEPTH>,
    imu_status_receiver:        channel::Receiver<'static, M, Result<ImuSystemStatus, usize>, IMU_STATUS_CHANNEL_DEPTH>,
    gps_status_receiver:        channel::Receiver<'static, M, Result<GpsSystemStatus, usize>, GPS_STATUS_CHANNEL_DEPTH>,
    sd_card_status_receiver:    channel::Receiver<'static, M, Result<SdCardSystemStatus, usize>, SD_CARD_STATUS_CHANNEL_DEPTH>,
    mut flight_state_watch_receiver: watch::Receiver<'static, M, FlightState, FLIGHT_STATE_WATCH_CONSUMERS>,
) -> !
where
    M: RawMutex + 'static,
{
    let mut system_status = SystemStatus::default();

    let mut print_status_timeout = Instant::now();

    loop {
        match select(
            select4(
                altitude_status_receiver.receive(), 
                arm_button_status_receiver.receive(), 
                imu_status_receiver.receive(), 
                gps_status_receiver.receive(),
            ),
            select3(
                sd_card_status_receiver.receive(),
                flight_state_watch_receiver.changed(),
                Timer::at(print_status_timeout),
            ),
        ).await {
            Either::First(Either4::First(status)) => {
                info!("Altimeter status: {:?}", status);
                system_status.update_altimeter_status(status);
            },
            Either::First(Either4::Second(status)) => {
                info!("Arm button status: {:?}", status);
                system_status.update_arm_button_status(status);
            },
            Either::First(Either4::Third(status)) => {
                info!("IMU status: {:?}", status);
                system_status.update_imu_status(status);
            },
            Either::First(Either4::Fourth(status)) => {
                info!("GPS status: {:?}", status);
                system_status.update_gps_status(status);
            },
            Either::Second(Either3::First(status)) => {
                info!("SD Card status: {:?}", status);
                system_status.update_sd_card_status(status);
            },
            Either::Second(Either3::Second(status)) => {
                info!("FSM status: {:?}", status);
                system_status.update_finite_state_machine_status(status);
            },
            Either::Second(Either3::Third(())) => {
                info!("System status: {:?}", Debug2Format(&system_status));
                print_status_timeout = Instant::now() + embassy_time::Duration::from_secs(10);
            },
        }
    }
}
