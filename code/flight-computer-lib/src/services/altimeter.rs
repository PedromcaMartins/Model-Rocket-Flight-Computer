use telemetry_messages::uom::si::{length::meter, pressure::pascal};
use telemetry_messages::{Altitude, Pressure};

#[must_use]
pub fn altitude_from_pressure(pressure: Pressure) -> Altitude {
    #[allow(unused_imports)]
    use telemetry_messages::uom::num_traits::Float;

    let pressure = pressure.get::<pascal>();
    let p0 = 101_325.0; // ISA sea level standard pressure in pascal
    let exponent = 0.190_284;
    let scale = 44_330.0;

    let pressure_ratio = pressure / p0;
    let altitude_m = scale * (1.0 - pressure_ratio.powf(exponent));

    Altitude::new::<meter>(altitude_m)
}

#[test]
fn test_altitude_from_pressure() {
    let pressure = Pressure::new::<pascal>(101_325.0);
    let altitude = altitude_from_pressure(pressure);
    assert_eq!(altitude.get::<meter>(), 0.0);

    let pressure = Pressure::new::<pascal>(50_000.0);
    let altitude = altitude_from_pressure(pressure);
    assert!(altitude.get::<meter>() > 0.0);
}
