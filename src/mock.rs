use crate::regs::*;
use crate::types::*;
use std::sync::{Arc, Mutex};

/// Errors that can occur when using the mock driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockError {
    /// Simulated I2C communication error.
    I2cError,
}

impl embedded_hal::i2c::Error for MockError {
    fn kind(&self) -> embedded_hal::i2c::ErrorKind {
        embedded_hal::i2c::ErrorKind::Other
    }
}

/// Internal state shared between the mock I2C implementation and the controller.
struct MockState {
    registers: [u8; 256],
}

/// A mock I2C device that emulates AS5600 behavior.
///
/// This mock allows you to test your application logic without real hardware.
/// It implements `embedded-hal` I2C traits, so it can be passed to the `AS5600Driver`.
///
/// It also provides a "backdoor" API (`mock_set_*` methods) to change sensor values
/// from other threads or from your test code.
#[derive(Clone)]
pub struct AS56Mock {
    state: Arc<Mutex<MockState>>,
}

impl AS56Mock {
    /// Creates a new mock with a healthy default state.
    /// - Magnet detected
    /// - AGC at 100
    /// - Watchdog enabled
    pub fn new() -> Self {
        let mut registers = [0u8; 256];
        // Default healthy state
        registers[regs::STATUS as usize] = 0x20; // Detected
        registers[regs::AGC as usize] = 100;
        registers[regs::CONF_HI as usize] = 0x20; // Watchdog ON

        Self {
            state: Arc::new(Mutex::new(MockState { registers })),
        }
    }

    // --- Simulation Controller API ---

    /// Sets the raw angle that the mock will report.
    pub fn mock_set_raw_angle(&self, angle: u16) {
        let mut state = self.state.lock().unwrap();
        let bytes = (angle & 0x0FFF).to_be_bytes();
        state.registers[regs::RAW_ANGLE_HI as usize] = bytes[0];
        state.registers[regs::RAW_ANGLE_LO as usize] = bytes[1];
    }

    /// Sets the magnet status that the mock will report.
    pub fn mock_set_status(&self, status: MagnetStatus) {
        let mut state = self.state.lock().unwrap();
        let mut val = 0u8;
        if status.detected {
            val |= 0x20;
        }
        if status.too_weak {
            val |= 0x10;
        }
        if status.too_strong {
            val |= 0x08;
        }
        state.registers[regs::STATUS as usize] = val;
    }

    /// Sets the Automatic Gain Control (AGC) value.
    pub fn mock_set_agc(&self, agc: u8) {
        let mut state = self.state.lock().unwrap();
        state.registers[regs::AGC as usize] = agc;
    }

    /// Sets the internal magnitude value.
    pub fn mock_set_magnitude(&self, magnitude: u16) {
        let mut state = self.state.lock().unwrap();
        let bytes = magnitude.to_be_bytes();
        state.registers[regs::MAGNITUDE_HI as usize] = bytes[0];
        state.registers[regs::MAGNITUDE_LO as usize] = bytes[1];
    }
}

impl embedded_hal::i2c::ErrorType for AS56Mock {
    type Error = MockError;
}

impl embedded_hal::i2c::I2c<embedded_hal::i2c::SevenBitAddress> for AS56Mock {
    fn read(&mut self, _address: u8, _read: &mut [u8]) -> Result<(), Self::Error> {
        // Simple read from the last register is not fully implemented in this mock
        // as the AS5600 driver always uses write_read for register access.
        Ok(())
    }

    fn write(&mut self, _address: u8, write: &[u8]) -> Result<(), Self::Error> {
        let mut state = self.state.lock().unwrap();
        if write.len() >= 2 {
            let reg = write[0] as usize;
            for (i, val) in write.iter().skip(1).enumerate() {
                if reg + i < 256 {
                    state.registers[reg + i] = *val;
                }
            }
        }
        Ok(())
    }

    fn write_read(
        &mut self,
        _address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        let state = self.state.lock().unwrap();
        let reg = write[0] as usize;
        for i in 0..read.len() {
            if reg + i < 256 {
                read[i] = state.registers[reg + i];
            }
        }
        Ok(())
    }

    fn transaction(
        &mut self,
        _address: u8,
        _operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        unimplemented!("Full I2C transactions are not implemented in this mock")
    }
}
