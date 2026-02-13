use crate::types::*;
use crate::error::AS56Error;

/// A common interface for any AS5600-compatible sensor (real or simulated).
///
/// This trait allows writing generic code that works with both the real
/// hardware driver and the mock implementation for testing.
pub trait AS5600Interface {
    /// The error type returned by the sensor methods.
    type Error;

    /// Reads the raw 12-bit angle from the Hall sensors.
    fn read_raw_angle(&mut self) -> Result<u16, AS56Error<Self::Error>>;

    /// Reads the 12-bit angle after applying all settings.
    fn read_angle(&mut self) -> Result<u16, AS56Error<Self::Error>>;

    /// Returns the current magnet status and field strength health.
    fn get_magnet_status(&mut self) -> Result<MagnetStatus, AS56Error<Self::Error>>;

    /// Returns the raw value of the status register.
    fn get_status_raw(&mut self) -> Result<u8, AS56Error<Self::Error>>;

    /// Returns the magnitude value from the Hall sensors.
    fn get_magnitude(&mut self) -> Result<u16, AS56Error<Self::Error>>;

    /// Returns the current Automatic Gain Control (AGC) value.
    fn get_agc(&mut self) -> Result<u8, AS56Error<Self::Error>>;

    /// Returns the number of times the settings have been permanently burned to the chip.
    fn get_burn_count(&mut self) -> Result<u8, AS56Error<Self::Error>>;

    /// Reads the current full configuration from the chip.
    fn get_config(&mut self) -> Result<Configuration, AS56Error<Self::Error>>;

    /// Writes a new configuration to the chip's volatile memory.
    fn set_config(&mut self, config: Configuration) -> Result<(), AS56Error<Self::Error>>;

    /// Gets the current zero position (ZPOS).
    fn get_zero_position(&mut self) -> Result<u16, AS56Error<Self::Error>>;

    /// Sets the zero position (ZPOS) in volatile memory.
    fn set_zero_position(&mut self, angle: u16) -> Result<(), AS56Error<Self::Error>>;

    /// Gets the current maximum position (MPOS).
    fn get_max_position(&mut self) -> Result<u16, AS56Error<Self::Error>>;

    /// Sets the maximum position (MPOS) in volatile memory.
    fn set_max_position(&mut self, angle: u16) -> Result<(), AS56Error<Self::Error>>;

    /// Gets the current maximum angle (MANG).
    fn get_max_angle(&mut self) -> Result<u16, AS56Error<Self::Error>>;

    /// Sets the maximum angle (MANG) in volatile memory.
    fn set_max_angle(&mut self, angle: u16) -> Result<(), AS56Error<Self::Error>>;
}
