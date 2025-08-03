use core::fmt::Debug;

use defmt_or_log::{info, error};
use bno055::{BNO055OperationMode, BNO055PowerMode, Bno055};
use defmt_or_log::Debug2Format;
use embassy_time::{Delay, Instant, Timer};
use embedded_hal::i2c::{I2c, SevenBitAddress};
use postcard_rpc::{header::VarSeq, server::{Sender as PostcardSender, WireTx}};
use telemetry_messages::{nalgebra::{Quaternion, Vector3, Vector4}, EulerAngles, ImuMessage, ImuTopic};
use uom::si::{acceleration::meter_per_second_squared, angle::degree, angular_velocity::degree_per_second, magnetic_flux_density::microtesla, quantities::{Acceleration, Angle, AngularVelocity, MagneticFluxDensity, ThermodynamicTemperature, Time}, thermodynamic_temperature::degree_celsius, time::microsecond};

#[inline]
pub async fn bno055_task<I, E, Tx>(
    bno055: Bno055<I>,
    sender: PostcardSender<Tx>,
) -> !
where
    I: I2c<SevenBitAddress, Error = E>,
    E: Debug,
    Tx: WireTx,
{
    let mut parser = Bno055Parser::init(bno055).await.unwrap();
    let mut seq = 0_u32;

    loop {
        match parser.parse_new_message() {
            Ok(msg) => {
                info!("IMU Message {:#?}", Debug2Format(&msg));

                if sender.publish::<ImuTopic>(VarSeq::Seq4(seq), &msg).await.is_ok() {
                    seq = seq.wrapping_add(1);
                } else {
                    error!("Failed to publish IMU message");
                }
            },
            Err(e) => error!("Failed to read BNO055: {:?}", Debug2Format(&e)),
        }

        Timer::after_millis(100).await;
    }
}

struct Bno055Parser<I, E>
where
    I: I2c<SevenBitAddress, Error = E>,
    E: Debug,
{
    bno055: Bno055<I>,
    _error: core::marker::PhantomData<E>,
}

impl<I, E> Bno055Parser<I, E>
where
    I: I2c<SevenBitAddress, Error = E>,
    E: Debug,
{
    pub async fn init(mut bno055: Bno055<I>) -> Result<Self, bno055::Error<E>> {
        // The sensor has an initial startup time of 400ms - 650ms during which interaction with it will fail
        Timer::at(Instant::from_millis(650)).await;
        let mut delay = Delay;

        bno055.init(&mut delay)?;

        // Enable 9-degrees-of-freedom sensor fusion mode with fast magnetometer calibration
        bno055.set_mode(BNO055OperationMode::NDOF, &mut delay)?;

        // Set power mode to normal
        bno055.set_power_mode(BNO055PowerMode::NORMAL)?;

        // Enable usage of external crystal
        bno055.set_external_crystal(true, &mut delay)?;

        Ok(Self {
            bno055,
            _error: core::marker::PhantomData,
        })
    }

    pub fn parse_new_message(&mut self) -> Result<ImuMessage, bno055::Error<E>> {
        let euler_angles = self.bno055.euler_angles()?;
        let quaternion = self.bno055.quaternion()?;
        let linear_acceleration = self.bno055.linear_acceleration()?;
        let gravity = self.bno055.gravity()?;
        let acceleration = self.bno055.accel_data()?;
        let gyro = self.bno055.gyro_data()?;
        let mag = self.bno055.mag_data()?;
        let temperature = self.bno055.temperature()
            .map(|t| ThermodynamicTemperature::new::<degree_celsius>(t.into()))?;

        let euler_angles = EulerAngles {
            roll: Angle::new::<degree>(euler_angles.c),
            pitch: Angle::new::<degree>(euler_angles.a),
            yaw: Angle::new::<degree>(euler_angles.b),
        };

        let quaternion = Quaternion::from_vector(
            Vector4::new(
                quaternion.v.x,
                quaternion.v.y,
                quaternion.v.z,
                quaternion.s, 
            )
        );

        let linear_acceleration = Vector3::new(
            Acceleration::new::<meter_per_second_squared>(linear_acceleration.x), 
            Acceleration::new::<meter_per_second_squared>(linear_acceleration.y),
            Acceleration::new::<meter_per_second_squared>(linear_acceleration.z) 
        );
        let gravity = Vector3::new(
            Acceleration::new::<meter_per_second_squared>(gravity.x), 
            Acceleration::new::<meter_per_second_squared>(gravity.y),
            Acceleration::new::<meter_per_second_squared>(gravity.z) 
        );
        let acceleration = Vector3::new(
            Acceleration::new::<meter_per_second_squared>(acceleration.x), 
            Acceleration::new::<meter_per_second_squared>(acceleration.y),
            Acceleration::new::<meter_per_second_squared>(acceleration.z) 
        );
        let gyro = Vector3::new(
            AngularVelocity::new::<degree_per_second>(gyro.x), 
            AngularVelocity::new::<degree_per_second>(gyro.y),
            AngularVelocity::new::<degree_per_second>(gyro.z) 
        );
        let mag = Vector3::new(
            MagneticFluxDensity::new::<microtesla>(mag.x), 
            MagneticFluxDensity::new::<microtesla>(mag.y),
            MagneticFluxDensity::new::<microtesla>(mag.z) 
        );

        Ok(ImuMessage {
            euler_angles,
            quaternion,
            linear_acceleration,
            gravity,
            acceleration,
            gyro,
            mag,
            temperature,
            timestamp: Time::new::<microsecond>(Instant::now().as_micros()),
        })
    }
}
