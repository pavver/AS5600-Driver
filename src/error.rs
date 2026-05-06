use core::fmt;

/// Custom error type for the AS5600 driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AS5600Error<E> {
    /// Error from the underlying I2C communication.
    I2c(E),
    /// The maximum number of permanent burns (ZMCO) has been reached.
    ///
    /// For ZPOS/MPOS the limit is 3, for Configuration it is 1.
    OtpMaxBurnsReached,
    /// Provided parameter is out of valid range (e.g. angle > 4095).
    InvalidParameter,
    /// Magnetic field not detected (magnet missing).
    MagnetMissing,
    /// Magnetic field too weak (magnet too far).
    MagnetTooWeak,
    /// Magnetic field too strong (magnet too close).
    MagnetTooStrong,
}

impl<E: fmt::Debug> fmt::Display for AS5600Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AS5600Error::I2c(e) => write!(f, "I2C error: {:?}", e),
            AS5600Error::OtpMaxBurnsReached => write!(f, "OTP programming limit reached"),
            AS5600Error::InvalidParameter => write!(f, "Invalid parameter value provided"),
            AS5600Error::MagnetMissing => write!(f, "Magnet not detected (missing)"),
            AS5600Error::MagnetTooWeak => write!(f, "Magnetic field too weak"),
            AS5600Error::MagnetTooStrong => write!(f, "Magnetic field too strong"),
        }
    }
}

#[cfg(feature = "std")]
impl<E: fmt::Debug> std::error::Error for AS5600Error<E> {}

impl<E> From<E> for AS5600Error<E> {
    fn from(e: E) -> Self {
        AS5600Error::I2c(e)
    }
}
