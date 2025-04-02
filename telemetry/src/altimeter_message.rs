#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct AltimeterMessage {
    /// Pressure in Pascal.
    pub pressure: f64,
    /// Altitude in meters.
    pub altitude: f32,
    /// Temperature in Celsius degrees.
    pub temperature: f32,
    /// Timestamp in microseconds.
    pub timestamp: u64,
}

use nom::{
    bytes::complete::tag, character::complete::{char, space0, u64}, number::complete::double, Finish, IResult
};

fn parse_altimeter_message(input: &str) -> IResult<&str, AltimeterMessage> {
    let (input, _) = tag("AltimeterMessage")(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('{')(input)?;
    let (input, _) = space0(input)?;

    // Parse the "pressure" field.
    let (input, _) = tag("pressure:")(input)?;
    let (input, _) = space0(input)?;
    let (input, pressure) = double(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = space0(input)?;

    // Parse the "altitude" field.
    let (input, _) = tag("altitude:")(input)?;
    let (input, _) = space0(input)?;
    let (input, altitude) = double(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = space0(input)?;

    // Parse the "temperature" field.
    let (input, _) = tag("temperature:")(input)?;
    let (input, _) = space0(input)?;
    let (input, temperature) = double(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = space0(input)?;

    // Parse the "timestamp" field.
    let (input, _) = tag("timestamp:")(input)?;
    let (input, _) = space0(input)?;
    let (input, timestamp) = u64(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('}')(input)?;

    Ok((
        input,
        AltimeterMessage {
            pressure,
            altitude: altitude as f32,
            temperature: temperature as f32,
            timestamp,
        },
    ))
}

use core::str::FromStr;

impl FromStr for AltimeterMessage {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_altimeter_message(s).finish().map(|(_, res)| res).map_err(|_| ())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_imu_message() {
        let imu_message_str = "AltimeterMessage { pressure: 0.0, altitude: 0.0, temperature: 0.0, timestamp: 0 }";
        let parsed = imu_message_str.parse::<AltimeterMessage>().unwrap();
        assert_eq!(parsed.pressure, 0.0);
        assert_eq!(parsed.altitude, 0.0);
        assert_eq!(parsed.temperature, 0.0);
        assert_eq!(parsed.timestamp, 0);
    }
}
