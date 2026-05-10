use crate::error::AS5600Error;
use crate::types::*;

/// Internal macro to define the AS5600 interface methods.
///
/// This avoids duplication between synchronous and asynchronous traits.
macro_rules! define_as5600_trait_methods {
    ($($async:ident)?) => {
        /// The error type returned by the underlying I2C implementation.
        type Error;

        /// Reads the raw 12-bit angle from the Hall sensors.
        ///
        /// This value is not affected by zero position (ZPOS) or maximum position (MPOS) settings.
        $($async)? fn read_raw_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>>;

        /// Reads the 12-bit angle after applying ZPOS/MPOS settings and filtering.
        $($async)? fn read_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>>;

        /// Returns the current magnet status and field strength health.
        $($async)? fn get_magnet_status(&mut self) -> Result<MagnetStatus, AS5600Error<Self::Error>>;

        /// Returns the raw value of the status register.
        $($async)? fn get_status_raw(&mut self) -> Result<u8, AS5600Error<Self::Error>>;

        /// Returns the magnitude value from the Hall sensors.
        ///
        /// Higher values indicate a stronger magnetic field.
        $($async)? fn get_magnitude(&mut self) -> Result<u16, AS5600Error<Self::Error>>;

        /// Returns the current Automatic Gain Control (AGC) value (0..255).
        $($async)? fn get_agc(&mut self) -> Result<u8, AS5600Error<Self::Error>>;

        /// Returns the number of times the settings have been permanently burned to the chip.
        ///
        /// Max value is 3.
        $($async)? fn get_burn_count(&mut self) -> Result<u8, AS5600Error<Self::Error>>;

        /// Reads the current full configuration from the chip.
        $($async)? fn get_config(&mut self) -> Result<Configuration, AS5600Error<Self::Error>>;

        /// Writes a new configuration to the chip's volatile memory.
        $($async)? fn set_config(&mut self, config: Configuration) -> Result<(), AS5600Error<Self::Error>>;

        /// Applies a partial configuration update, minimizing I2C transactions.
        ///
        /// This method only writes to registers that were explicitly set in the builder.
        $($async)? fn apply_config(
            &mut self,
            builder: ConfigurationBuilder,
        ) -> Result<(), AS5600Error<Self::Error>>;

        /// Gets the current programmed zero position (ZPOS).
        $($async)? fn get_zero_position(&mut self) -> Result<u16, AS5600Error<Self::Error>>;

        /// Sets the zero position (ZPOS) in volatile memory.
        ///
        /// # Errors
        /// Returns [`AS5600Error::InvalidParameter`] if the angle is greater than 4095.
        $($async)? fn set_zero_position(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>>;

        /// Gets the current programmed maximum position (MPOS).
        $($async)? fn get_max_position(&mut self) -> Result<u16, AS5600Error<Self::Error>>;

        /// Sets the maximum position (MPOS) in volatile memory.
        ///
        /// # Errors
        /// Returns [`AS5600Error::InvalidParameter`] if the angle is greater than 4095.
        $($async)? fn set_max_position(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>>;

        /// Gets the current programmed maximum angle (MANG).
        $($async)? fn get_max_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>>;

        /// Sets the maximum angle (MANG) in volatile memory.
        ///
        /// # Errors
        /// Returns [`AS5600Error::InvalidParameter`] if the angle is greater than 4095.
        $($async)? fn set_max_angle(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>>;

        /// Checks if the sensor is connected and responding on the I2C bus.
        $($async)? fn is_connected(&mut self) -> bool;

        /// Reads both the 12-bit angle and magnet status in one optimized I2C transaction.
        $($async)? fn read_angle_with_status(&mut self) -> Result<AngleWithStatus, AS5600Error<Self::Error>>;

        /// Reads all diagnostic data (Angle, Raw Angle, Status, AGC, Magnitude) in one optimized transaction.
        $($async)? fn read_all_diagnostics(&mut self) -> Result<Diagnostics, AS5600Error<Self::Error>>;
    };
}

/// A common interface for any AS5600-compatible sensor (real or simulated).
///
/// This trait allows writing generic application logic that works with both the real
/// hardware driver and the mock implementation used for testing.
pub trait AS5600Interface {
    define_as5600_trait_methods!();
}

/// Asynchronous interface for the AS5600 sensor.
///
/// This trait provides the same functionality as [`AS5600Interface`] but uses
/// asynchronous methods compatible with `embedded-hal-async`.
#[cfg(feature = "async")]
#[allow(async_fn_in_trait)]
pub trait AS5600AsyncInterface {
    define_as5600_trait_methods!(async);
}
