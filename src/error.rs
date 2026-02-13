use core::fmt;

/// Custom error type for the AS5600 driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AS56Error<E> {
    /// Error from the underlying I2C communication.
    I2c(E),
}

impl<E: fmt::Debug> fmt::Display for AS56Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AS56Error::I2c(e) => write!(f, "I2C error: {:?}", e),
        }
    }
}

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
impl<E: fmt::Debug> std::error::Error for AS56Error<E> {}
