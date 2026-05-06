use crate::error::AS5600Error;
use crate::types::*;

/// A common interface for any AS5600-compatible sensor (real or simulated).
///
/// This trait allows writing generic code that works with both the real
/// hardware driver and the mock implementation for testing.
pub trait AS5600Interface {
    /// The error type returned by the sensor methods.
    type Error;

    /// Reads the raw 12-bit angle from the Hall sensors.
    fn read_raw_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>>;

    /// Reads the 12-bit angle after applying all settings.
    fn read_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>>;

    /// Returns the current magnet status and field strength health.
    fn get_magnet_status(&mut self) -> Result<MagnetStatus, AS5600Error<Self::Error>>;

    /// Returns the raw value of the status register.
    fn get_status_raw(&mut self) -> Result<u8, AS5600Error<Self::Error>>;

    /// Returns the magnitude value from the Hall sensors.
    fn get_magnitude(&mut self) -> Result<u16, AS5600Error<Self::Error>>;

    /// Returns the current Automatic Gain Control (AGC) value.
    fn get_agc(&mut self) -> Result<u8, AS5600Error<Self::Error>>;

    /// Returns the number of times the settings have been permanently burned to the chip.
    fn get_burn_count(&mut self) -> Result<u8, AS5600Error<Self::Error>>;

    /// Reads the current full configuration from the chip.
    fn get_config(&mut self) -> Result<Configuration, AS5600Error<Self::Error>>;

    /// Writes a new configuration to the chip's volatile memory.
    fn set_config(&mut self, config: Configuration) -> Result<(), AS5600Error<Self::Error>>;

    /// Applies a partial configuration update, minimizing I2C transactions.
    fn apply_config(
        &mut self,
        builder: ConfigurationBuilder,
    ) -> Result<(), AS5600Error<Self::Error>>;

    /// Gets the current zero position (ZPOS).
    fn get_zero_position(&mut self) -> Result<u16, AS5600Error<Self::Error>>;

    /// Sets the zero position (ZPOS) in volatile memory.
    fn set_zero_position(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>>;

    /// Gets the current maximum position (MPOS).
    fn get_max_position(&mut self) -> Result<u16, AS5600Error<Self::Error>>;

    /// Sets the maximum position (MPOS) in volatile memory.
    fn set_max_position(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>>;

    /// Gets the current maximum angle (MANG).
    fn get_max_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>>;

    /// Sets the maximum angle (MANG) in volatile memory.
    fn set_max_angle(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>>;

    /// Checks if the sensor is connected and responding on the I2C bus.
    fn is_connected(&mut self) -> bool;

    /// Reads all diagnostic data (Angle, Raw Angle, Status, AGC, Magnitude) in one optimized transaction.
    fn read_all_diagnostics(&mut self) -> Result<Diagnostics, AS5600Error<Self::Error>>;
}

/// Asynchronous interface for the AS5600 sensor.
#[cfg(feature = "async")]
#[allow(async_fn_in_trait)]
pub trait AS5600AsyncInterface {
    type Error;

    /// Checks if the sensor is connected and responding on the I2C bus.
    async fn is_connected(&mut self) -> bool;

    async fn read_raw_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>>;
    async fn read_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>>;
    async fn get_config(&mut self) -> Result<Configuration, AS5600Error<Self::Error>>;
    async fn set_config(&mut self, config: Configuration) -> Result<(), AS5600Error<Self::Error>>;
    async fn apply_config(
        &mut self,
        builder: ConfigurationBuilder,
    ) -> Result<(), AS5600Error<Self::Error>>;
    async fn get_magnet_status(&mut self) -> Result<MagnetStatus, AS5600Error<Self::Error>>;
    async fn get_status_raw(&mut self) -> Result<u8, AS5600Error<Self::Error>>;
    async fn get_agc(&mut self) -> Result<u8, AS5600Error<Self::Error>>;
    async fn get_magnitude(&mut self) -> Result<u16, AS5600Error<Self::Error>>;
    async fn get_zero_position(&mut self) -> Result<u16, AS5600Error<Self::Error>>;
    async fn set_zero_position(&mut self, position: u16) -> Result<(), AS5600Error<Self::Error>>;
    async fn get_max_position(&mut self) -> Result<u16, AS5600Error<Self::Error>>;
    async fn set_max_position(&mut self, position: u16) -> Result<(), AS5600Error<Self::Error>>;
    async fn get_max_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>>;
    async fn set_max_angle(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>>;
    async fn get_burn_count(&mut self) -> Result<u8, AS5600Error<Self::Error>>;
    async fn read_all_diagnostics(&mut self) -> Result<Diagnostics, AS5600Error<Self::Error>>;
}
