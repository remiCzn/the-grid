use grid_engine::driver::DriverError;

#[test]
fn test_driver_error_display() {
    let err = DriverError::NoOutputDevice;
    assert_eq!(format!("{err}"), "Aucun périphérique de sortie audio trouvé");
}
