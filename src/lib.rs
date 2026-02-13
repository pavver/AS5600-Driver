//! # AS5600 Driver
//!
//! A platform-agnostic Rust driver for the AS5600 magnetic rotary encoder,
//! based on the `embedded-hal` traits.
//!
//! The AS5600 is a contactless magnetic rotary encoder with high-resolution 12-bit
//! contactless on-axis angular position measurement over a full turn of 360°.
//!
//! ## Features
//! - Read raw and filtered angle (12-bit resolution)
//! - Configure power modes, hysteresis, and filters
//! - Read magnet status (detected, too weak, too strong)
//! - Automatic Gain Control (AGC) and Magnitude reading
//! - Programming support (ZPOS, MPOS, MANG, and permanent BURN)
//! - Mock driver for testing and simulation
//!
//! ## Example (ESP32)
//! ```rust,ignore
//! use AS5600_Driver::{AS5600Driver, AS5600Interface};
//!
//! // Setup I2C from your HAL
//! let i2c = ...;
//! let mut sensor = AS5600Driver::new(i2c);
//!
//! match sensor.read_angle() {
//!     Ok(angle) => println!("Angle: {}", angle),
//!     Err(e) => eprintln!("Error: {:?}", e),
//! }
//! ```

#![no_std]
#![allow(non_snake_case)]

#[cfg(feature = "std")]
extern crate std;

pub mod driver;
pub mod error;
pub mod regs;
pub mod traits;
pub mod types;

#[cfg(feature = "mock")]
pub mod mock;

// Re-exports for convenience
pub use driver::AS5600Driver;
pub use error::AS56Error;
pub use regs::*;
pub use traits::AS5600Interface;
pub use types::*;

#[cfg(feature = "mock")]
pub use mock::AS56Mock;
