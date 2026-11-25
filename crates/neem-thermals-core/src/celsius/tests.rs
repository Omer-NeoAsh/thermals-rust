use super::Celsius;

#[test]
fn rejects_junk() {
    for degrees in [f32::NAN, f32::INFINITY, -1.5, 0.0, 150.1] {
        assert_eq!(Celsius::new(degrees), None, "{degrees} should be rejected.");
    }
}

#[test]
fn accepts_range_edges() {
    assert_eq!(Celsius::new(0.1).map(Celsius::degrees), Some(0.1));
    assert_eq!(Celsius::new(150.0).map(Celsius::degrees), Some(150.0));
}

#[test]
fn displays_one_decimal_with_unit() {
    let celsius = Celsius::new(48.5).unwrap();
    assert_eq!(celsius.to_string(), "48.5 °C");
}
