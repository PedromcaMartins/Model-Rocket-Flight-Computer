use defmt_or_log::{info, Debug2Format};
use embassy_futures::select::{select, select4, Either, Either4};
use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal, watch};

use crate::model::system_status::{AltimeterSystemStatus, ArmButtonSystemStatus, FiniteStateMachineStatus, GpsSystemStatus, ImuSystemStatus, SdCardSystemStatus};

#[inline]
pub async fn system_status_task<
    M, 
    const FSM_WATCH_N_RECEIVERS: usize,
> (
    altimeter_status_signal: &'static Signal<M, AltimeterSystemStatus>,
    arm_button_status_signal: &'static Signal<M, ArmButtonSystemStatus>,
    imu_status_signal: &'static Signal<M, ImuSystemStatus>,
    gps_status_signal: &'static Signal<M, GpsSystemStatus>,
    sd_card_status_signal: &'static Signal<M, SdCardSystemStatus>,
    mut fsm_status_watch: watch::Receiver<'static, M, FiniteStateMachineStatus, FSM_WATCH_N_RECEIVERS>,
) -> !
where
    M: RawMutex + 'static,
{
    loop {
        match select(
            select4(
                altimeter_status_signal.wait(), 
                arm_button_status_signal.wait(), 
                imu_status_signal.wait(), 
                gps_status_signal.wait(),
            ),
            select(
                sd_card_status_signal.wait(),
                fsm_status_watch.changed(),
            ),
        ).await {
            Either::First(Either4::First(altimeter_status)) => {
                info!("Altimeter status: {:?}", Debug2Format(&altimeter_status));
            },
            Either::First(Either4::Second(arm_button_status)) => {
                info!("Arm button status: {:?}", Debug2Format(&arm_button_status));
            },
            Either::First(Either4::Third(imu_status)) => {
                info!("IMU status: {:?}", Debug2Format(&imu_status));
            },
            Either::First(Either4::Fourth(gps_status)) => {
                info!("GPS status: {:?}", Debug2Format(&gps_status));
            },
            Either::Second(Either::First(sd_card_status)) => {
                info!("SD Card status: {:?}", Debug2Format(&sd_card_status));
            },
            Either::Second(Either::Second(fsm_status)) => {
                info!("FSM status: {:?}", Debug2Format(&fsm_status));
            },
        }
    }
}
