use proto::uom::si::{length::meter, pressure::pascal};
use proto::sensor_data::{Altitude, Pressure};

#[must_use]
pub fn altitude_from_pressure(pressure: Pressure) -> Altitude {
    #[allow(unused_imports)]
    use proto::uom::num_traits::Float;

    let pressure = pressure.get::<pascal>();
    let p0 = 101_325.0; // ISA sea level standard pressure in pascal
    let exponent = 0.190_284;
    let scale = 44_330.0;

    let pressure_ratio = pressure / p0;
    let altitude_m = scale * (1.0 - pressure_ratio.powf(exponent));

    Altitude::new::<meter>(altitude_m)
}

#[cfg(test)]
mod tests {
    use proto::uom::si::pressure::millibar;

    use super::*;

    #[rstest::rstest]
    #[case(1013.25, 0.0)]
    #[case(1007.2, 50.0)]
    #[case(1001.2, 100.0)]
    #[case(995.35,  150.0)]
    #[case(989.45,  200.0)]
    #[case(983.57,  250.0)]
    #[case(977.72,  300.0)]
    #[case(971.90,  350.0)]
    #[case(966.11,  400.0)]
    #[case(960.34,  450.0)]
    #[case(954.61,  500.0)]
    #[trace]
    fn matches_isa_altitude_table(
        #[case] pressure: f32, 
        #[case] expected_altitude: f32, 
        #[values(1.0)] error_margin: f32
    ) {
        let pressure = Pressure::new::<millibar>(pressure);
        let altitude = altitude_from_pressure(pressure);
        let altitude = altitude.get::<meter>();

        assert!((altitude - expected_altitude).abs() < error_margin, "Expected altitude: {expected_altitude}, Calculated altitude: {altitude}");
    }
}
