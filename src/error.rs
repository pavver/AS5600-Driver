use core::fmt;

/// Custom error type for the AS5600 driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AS5600Error<E> {
    /// Error from the underlying I2C communication.
    I2c(E),
}

impl<E: fmt::Debug> fmt::Display for AS5600Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AS5600Error::I2c(e) => write!(f, "I2C error: {:?}", e),
        }
    }
}

#[cfg(feature = "std")]
impl<E: fmt::Debug> std::error::Error for AS5600Error<E> {}
