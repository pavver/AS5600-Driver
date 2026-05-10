//! # AS5600 Driver
//!
//! A platform-agnostic Rust driver for the AS5600 magnetic rotary encoder,
//! compatible with `embedded-hal` 1.0 and `embedded-hal-async`.
//!
//! The AS5600 is a high-resolution 12-bit contactless magnetic rotary encoder.
//! It can measure angular position over a full 360° turn and provides digital
//! output via I2C, as well as programmable PWM or analog output.
//!
//! ## Core Features
//!
//! * **Flexible IO**: Supports both synchronous (`AS5600Interface`) and asynchronous (`AS5600AsyncInterface`) I2C communication.
//! * **Optimized Reads**: Fetch both angle and magnet status in a single I2C transaction via `read_angle_with_status`.
//! * **Rich Configuration**: Full control over power modes, hysteresis, filter settings, and output stages.
//! * **Permanent Programming**: Supports one-time-programmable (OTP) burning of configuration and zero/max positions with safety tokens.
//! * **Diagnostics**: Detailed reporting of magnetic field strength (too weak, too strong) and AGC levels.
//! * **Testing**: Includes a comprehensive `AS5600Mock` for simulation and unit testing.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use AS5600_Driver::{AS5600Driver, AS5600Interface, Configuration, PowerMode};
//!
//! // Provide your I2C peripheral from a HAL
//! let i2c = ...;
//! let mut encoder = AS5600Driver::new(i2c);
//!
//! // Simple angle reading with health check
//! let result = encoder.read_angle_with_status()?
//!     .check_magnet_all()?;
//!
//! println!("Current angle: {}", result.angle);
//!
//! // Dynamic configuration
//! encoder.apply_config(
//!     Configuration::builder()
//!         .power_mode(PowerMode::LPM1)
//!         .watchdog(false)
//! )?;
//! ```

#![no_std]
#![allow(non_snake_case)]

#[cfg(any(feature = "std", test))]
extern crate std;

pub mod driver;
pub mod error;
pub mod regs;
pub mod traits;
pub mod types;

#[cfg(any(feature = "mock", test))]
pub mod mock;

// Re-exports for convenience
pub use driver::AS5600Driver;
pub use error::AS5600Error;
pub use regs::*;
#[cfg(feature = "async")]
pub use traits::AS5600AsyncInterface;
pub use traits::AS5600Interface;
pub use types::*;

#[cfg(any(feature = "mock", test))]
pub use mock::AS5600Mock;
