#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct ImuMessage {
    /// Euler angles representation of heading in degrees.
    /// Euler angles is represented as (`roll`, `pitch`, `yaw/heading`).
    pub euler_angles: [f32; 3],
    /// Standard quaternion represented by the scalar and vector parts. Corresponds to a right-handed rotation matrix.
    /// Quaternion is represented as (x, y, z, s).
    ///
    /// where:
    /// x, y, z: Vector part of a quaternion;
    /// s: Scalar part of a quaternion.
    pub quaternion: [f32; 4],
    /// Linear acceleration vector in m/s^2 units.
    pub linear_acceleration: [f32; 3],
    /// Gravity vector in m/s^2 units.
    pub gravity: [f32; 3],
    /// Acceleration vector in m/s^2 units.
    pub acceleration: [f32; 3],
    /// Gyroscope vector in deg/s units.
    pub gyro: [f32; 3],
    /// Magnetometer vector in uT units.
    pub mag: [f32; 3],
    /// Temperature of the chip in Celsius degrees.
    pub temperature: f32,
    /// Timestamp in microseconds.
    pub timestamp: u64,
}

use nom::{
    bytes::complete::tag, character::complete::{char, space0, u64}, number::complete::double, Finish, IResult
};

// Helper parser for a list of floats inside square brackets (e.g., [0.0, 0.0, 0.0]).
fn parse_f32_3_list(input: &str) -> IResult<&str, [f32; 3]> {
    let (input, _) = char('[')(input)?;
    let (input, _) = space0(input)?;
    let (input, value_1) = double(input)?;
    let (input, _) = char(',')(input)?;

    let (input, _) = space0(input)?;
    let (input, value_2) = double(input)?;
    let (input, _) = char(',')(input)?;

    let (input, _) = space0(input)?;
    let (input, value_3) = double(input)?;
    let (input, _) = char(']')(input)?;

    Ok((input, [value_1 as f32, value_2 as f32, value_3 as f32]))
}

// Helper parser for a list of floats inside square brackets (e.g., [0.0, 0.0, 0.0]).
fn parse_f32_4_list(input: &str) -> IResult<&str, [f32; 4]> {
    let (input, _) = char('[')(input)?;
    let (input, _) = space0(input)?;
    let (input, value_1) = double(input)?;
    let (input, _) = char(',')(input)?;

    let (input, _) = space0(input)?;
    let (input, value_2) = double(input)?;
    let (input, _) = char(',')(input)?;

    let (input, _) = space0(input)?;
    let (input, value_3) = double(input)?;
    let (input, _) = char(',')(input)?;

    let (input, _) = space0(input)?;
    let (input, value_4) = double(input)?;
    let (input, _) = char(']')(input)?;

    Ok((input, [value_1 as f32, value_2 as f32, value_3 as f32, value_4 as f32]))
}

fn parse_imu_message(input: &str) -> IResult<&str, ImuMessage> {
    let (input, _) = tag("ImuMessage")(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('{')(input)?;
    let (input, _) = space0(input)?;

    // Parse euler_angles
    let (input, _) = tag("euler_angles:")(input)?;
    let (input, _) = space0(input)?;
    let (input, euler_angles) = parse_f32_3_list(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = space0(input)?;

    // Parse quaternion
    let (input, _) = tag("quaternion:")(input)?;
    let (input, _) = space0(input)?;
    let (input, quaternion) = parse_f32_4_list(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = space0(input)?;

    // Parse linear_acceleration
    let (input, _) = tag("linear_acceleration:")(input)?;
    let (input, _) = space0(input)?;
    let (input, linear_acceleration) = parse_f32_3_list(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = space0(input)?;

    // Parse gravity
    let (input, _) = tag("gravity:")(input)?;
    let (input, _) = space0(input)?;
    let (input, gravity) = parse_f32_3_list(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = space0(input)?;

    // Parse acceleration
    let (input, _) = tag("acceleration:")(input)?;
    let (input, _) = space0(input)?;
    let (input, acceleration) = parse_f32_3_list(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = space0(input)?;

    // Parse gyro
    let (input, _) = tag("gyro:")(input)?;
    let (input, _) = space0(input)?;
    let (input, gyro) = parse_f32_3_list(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = space0(input)?;

    // Parse mag
    let (input, _) = tag("mag:")(input)?;
    let (input, _) = space0(input)?;
    let (input, mag) = parse_f32_3_list(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = space0(input)?;

    // Parse temperature
    let (input, _) = tag("temperature:")(input)?;
    let (input, _) = space0(input)?;
    let (input, temperature) = double(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = space0(input)?;

    // Parse timestamp
    let (input, _) = tag("timestamp:")(input)?;
    let (input, _) = space0(input)?;
    let (input, timestamp) = u64(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('}')(input)?;

    Ok((
        input,
        ImuMessage {
            euler_angles,
            quaternion,
            linear_acceleration,
            gravity,
            acceleration,
            gyro,
            mag,
            temperature: temperature as f32,
            timestamp,
        },
    ))
}

use core::str::FromStr;

impl FromStr for ImuMessage {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_imu_message(s).finish().map(|(_, res)| res).map_err(|_| ())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_imu_message() {
        let imu_message_str = "ImuMessage { euler_angles: [0.0, 0.0, 0.0], quaternion: [0.0, 0.0, 0.0, 0.0], linear_acceleration: [0.0, 0.0, 0.0], gravity: [0.0, 0.0, 0.0], acceleration: [0.0, 0.0, 0.0], gyro: [0.0, 0.0, 0.0], mag: [0.0, 0.0, 0.0], temperature: 0.0, timestamp: 0 }";

        let parsed = imu_message_str.parse::<ImuMessage>().unwrap();
        assert_eq!(parsed.euler_angles, [0.0, 0.0, 0.0]);
        assert_eq!(parsed.quaternion, [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(parsed.linear_acceleration, [0.0, 0.0, 0.0]);
        assert_eq!(parsed.gravity, [0.0, 0.0, 0.0]);
        assert_eq!(parsed.acceleration, [0.0, 0.0, 0.0]);
        assert_eq!(parsed.gyro, [0.0, 0.0, 0.0]);
        assert_eq!(parsed.mag, [0.0, 0.0, 0.0]);
        assert_eq!(parsed.temperature, 0.0);
        assert_eq!(parsed.timestamp, 0);
    }
}
