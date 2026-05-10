use core::fmt;

/// Custom error type for the AS5600 driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AS5600Error<E> {
    /// Error originating from the underlying I2C communication.
    I2c(E),
    /// The maximum number of permanent burns (ZMCO) has been reached.
    ///
    /// The AS5600 allows burning ZPOS/MPOS settings up to 3 times,
    /// and the configuration register only once.
    OtpMaxBurnsReached,
    /// Provided parameter is out of the valid 12-bit range (0..4095).
    InvalidParameter,
    /// Magnetic field not detected (magnet is missing or misaligned).
    MagnetMissing,
    /// Magnetic field too weak (magnet is too far from the sensor).
    MagnetTooWeak,
    /// Magnetic field too strong (magnet is too close to the sensor).
    MagnetTooStrong,
    /// The angular travel (range) is too small.
    ///
    /// The AS5600 requires a minimum travel of 18 degrees (~205 counts).
    AngularTravelTooSmall,
    /// Provided I2C address is invalid (AS5600L only).
    /// Valid range is 0x08..0x77.
    InvalidAddress,
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
            AS5600Error::AngularTravelTooSmall => {
                write!(f, "Angular travel is too small (min 18°)")
            }
            AS5600Error::InvalidAddress => write!(f, "Invalid I2C address provided"),
        }
    }
}

#[cfg(any(feature = "std", test))]
impl<E: fmt::Debug> std::error::Error for AS5600Error<E> {}

impl<E> From<E> for AS5600Error<E> {
    #[inline]
    fn from(e: E) -> Self {
        AS5600Error::I2c(e)
    }
}
