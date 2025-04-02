use chrono::NaiveTime;
use nmea::sentences::FixType;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
#[derive(Debug, Clone, Copy)]
pub struct GpsMessage {
    #[cfg_attr(feature = "defmt-03", defmt(Debug2Format))]
    pub fix_time: NaiveTime,
    #[cfg_attr(feature = "defmt-03", defmt(Debug2Format))]
    pub fix_type: FixType,
    /// Latitude in degrees.
    pub latitude: f64,
    /// Longitude in degrees.
    pub longitude: f64,
    /// MSL Altitude in meters
    pub altitude: f32,
    /// Number of satellites used for fix.
    pub num_of_fix_satellites: u8,
    /// Timestamp in microseconds.
    pub timestamp: u64,
}

use nom::{
    bytes::complete::{is_not, tag}, character::complete::{char, space0, u64, u8}, number::complete::{double}, sequence::delimited, Finish, IResult, Parser
};

// Helper parser to extract a quoted string.
fn parse_quoted_string(input: &str) -> IResult<&str, &str> {
    delimited(char('"'), is_not("\""), char('"')).parse(input)
}

fn parse_gps_message(input: &str) -> IResult<&str, GpsMessage> {
    let (input, _) = tag("GpsMessage")(input)?;
    let (input, _) = space0(input)?;
    let (input, _) = char('{')(input)?;
    let (input, _) = space0(input)?;

    // Parse fix_time: "00:00:00"
    let (input, _) = tag("fix_time:")(input)?;
    let (input, _) = space0(input)?;
    let (input, fix_time_str) = parse_quoted_string(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = space0(input)?;

    // Parse fix_type: "Invalid"
    let (input, _) = tag("fix_type:")(input)?;
    let (input, _) = space0(input)?;
    let (input, fix_type_str) = parse_quoted_string(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = space0(input)?;

    // Parse latitude: f64
    let (input, _) = tag("latitude:")(input)?;
    let (input, _) = space0(input)?;
    let (input, latitude) = double(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = space0(input)?;

    // Parse longitude: f64
    let (input, _) = tag("longitude:")(input)?;
    let (input, _) = space0(input)?;
    let (input, longitude) = double(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = space0(input)?;

    // Parse altitude: f32 (we parse as f64 then cast to f32)
    let (input, _) = tag("altitude:")(input)?;
    let (input, _) = space0(input)?;
    let (input, altitude) = double(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = space0(input)?;

    // Parse num_of_fix_satellites: u8
    let (input, _) = tag("num_of_fix_satellites:")(input)?;
    let (input, _) = space0(input)?;
    let (input, num_of_fix_satellites) = u8(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, _) = space0(input)?;

    // Parse timestamp: u64
    let (input, _) = tag("timestamp:")(input)?;
    let (input, _) = space0(input)?;
    let (input, timestamp) = u64(input)?;
    let (input, _) = space0(input)?;

    let (input, _) = char('}')(input)?;

    // Convert the parsed fix_time string into a NaiveTime.
    let fix_time = match NaiveTime::parse_from_str(fix_time_str, "%H:%M:%S") {
        Ok(fix_time) => fix_time,
        Err(_) => return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Fail))),
    };

    // Convert the parsed fix_type string into a FixType.
    let fix_type = match fix_type_str {
        "Invalid" => FixType::Invalid,
        "Gps" => FixType::Gps,
        "DGps" => FixType::DGps,
        "Pps" => FixType::Pps,
        "Rtk" => FixType::Rtk,
        "FloatRtk" => FixType::FloatRtk,
        "Estimated" => FixType::Estimated,
        "Manual" => FixType::Manual,
        "Simulation" => FixType::Simulation,
        _ => return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Fail))),
    };

    Ok((
        input,
        GpsMessage {
            fix_time,
            fix_type,
            latitude,
            longitude,
            altitude: altitude as f32,
            num_of_fix_satellites,
            timestamp,
        },
    ))
}

use core::str::FromStr;

impl FromStr for GpsMessage {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_gps_message(s).finish().map(|(_, res)| res).map_err(|_| ())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_gps_message() {
        let gps_message_str = "GpsMessage { fix_time: \"00:00:00\", fix_type: \"Invalid\", latitude: 0.0, longitude: 0.0, altitude: 0.0, num_of_fix_satellites: 0, timestamp: 0 }";

        let parsed = gps_message_str.parse::<GpsMessage>().unwrap();
        assert_eq!(parsed.fix_time, NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        assert_eq!(parsed.fix_type, FixType::Invalid);
        assert_eq!(parsed.latitude, 0.0);
        assert_eq!(parsed.longitude, 0.0);
        assert_eq!(parsed.altitude, 0.0);
        assert_eq!(parsed.num_of_fix_satellites, 0);
        assert_eq!(parsed.timestamp, 0);
    }
}
