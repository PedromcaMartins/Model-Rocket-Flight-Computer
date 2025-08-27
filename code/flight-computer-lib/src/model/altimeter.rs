use uom::si::{length::meter, pressure::pascal, quantities::{Length, Pressure}};

#[must_use]
pub fn altitude_from_pressure(pressure: Pressure<f64>) -> Length<f64> {
    #[allow(unused_imports)]
    use uom::num_traits::Float;

    #[allow(clippy::cast_possible_truncation)]
    let pressure = pressure.get::<pascal>() as f32;
    let p0 = 101_325.0_f32; // ISA sea level standard pressure in pascal
    let exponent = 0.190_284_f32;
    let scale = 44_330.0_f32;

    let pressure_ratio = pressure / p0;
    let altitude_m = scale * (1.0 - pressure_ratio.powf(exponent));

    Length::new::<meter>(altitude_m.into())
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
