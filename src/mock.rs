use crate::regs::*;
use crate::types::*;
use std::sync::{Arc, Mutex};
use std::vec::Vec;

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
    transaction_log: Vec<MockTransaction>,
    error_state: Option<MockError>,
}

/// Represents a single I2C transaction recorded by the mock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockTransaction {
    /// A write operation: (register, data)
    Write(u8, Vec<u8>),
    /// A write-read operation (typical for register reading): (register, bytes_read)
    WriteRead(u8, usize),
}

/// A mock I2C device that emulates AS5600 behavior.
///
/// This mock allows you to test your application logic without real hardware.
/// It implements `embedded-hal` I2C traits, so it can be passed to the `AS5600Driver`.
///
/// It also provides a "backdoor" API (`mock_set_*` methods) to change sensor values
/// from other threads or from your test code, and a way to inspect I2C traffic.
#[derive(Clone)]
pub struct AS5600Mock {
    state: Arc<Mutex<MockState>>,
}

impl AS5600Mock {
    /// Creates a new mock with a healthy default state.
    /// - Magnet detected
    /// - AGC at 100
    /// - Watchdog enabled
    pub fn new() -> Self {
        let mut registers = [0u8; 256];
        // Default healthy state
        registers[regs::STATUS as usize] = regs::STATUS_MD_MASK; // Detected
        registers[regs::AGC as usize] = 100;
        registers[regs::CONF_HI as usize] = regs::CONF_WD_MASK; // Watchdog ON

        Self {
            state: Arc::new(Mutex::new(MockState { 
                registers,
                transaction_log: Vec::new(),
                error_state: None,
            })),
        }
    }

    /// Forces the mock to return an error on all subsequent I2C operations.
    pub fn mock_set_error(&self, error: Option<MockError>) {
        let mut state = self.state.lock().unwrap();
        state.error_state = error;
    }

    /// Returns the recorded transaction log and clears it.
    pub fn mock_get_log(&self) -> Vec<MockTransaction> {
        let mut state = self.state.lock().unwrap();
        std::mem::take(&mut state.transaction_log)
    }

    /// Resets the transaction log.
    pub fn mock_clear_log(&self) {
        let mut state = self.state.lock().unwrap();
        state.transaction_log.clear();
    }

    // --- Simulation Controller API ---

    /// Sets the raw angle that the mock will report.
    pub fn mock_set_raw_angle(&self, angle: u16) {
        let mut state = self.state.lock().unwrap();
        let bytes = (angle & regs::ANGLE_MASK).to_be_bytes();
        state.registers[regs::RAW_ANGLE_HI as usize] = bytes[0];
        state.registers[regs::RAW_ANGLE_LO as usize] = bytes[1];
    }

    /// Sets the magnet status that the mock will report.
    pub fn mock_set_status(&self, status: MagnetStatus) {
        let mut state = self.state.lock().unwrap();
        let mut val = 0u8;
        if status.detected {
            val |= regs::STATUS_MD_MASK;
        }
        if status.too_weak {
            val |= regs::STATUS_ML_MASK;
        }
        if status.too_strong {
            val |= regs::STATUS_MH_MASK;
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
        let bytes = (magnitude & regs::ANGLE_MASK).to_be_bytes();
        state.registers[regs::MAGNITUDE_HI as usize] = bytes[0];
        state.registers[regs::MAGNITUDE_LO as usize] = bytes[1];
    }
}

impl embedded_hal::i2c::ErrorType for AS5600Mock {
    type Error = MockError;
}

impl embedded_hal::i2c::I2c<embedded_hal::i2c::SevenBitAddress> for AS5600Mock {
    fn read(&mut self, _address: u8, _read: &mut [u8]) -> Result<(), Self::Error> {
        let state = self.state.lock().unwrap();
        if let Some(err) = state.error_state {
            return Err(err);
        }
        // Simple read from the last register is not fully implemented in this mock
        // as the AS5600 driver always uses write_read for register access.
        Ok(())
    }

    fn write(&mut self, _address: u8, write: &[u8]) -> Result<(), Self::Error> {
        let mut state = self.state.lock().unwrap();
        if let Some(err) = state.error_state {
            return Err(err);
        }
        if !write.is_empty() {
            state.transaction_log.push(MockTransaction::Write(write[0], write[1..].to_vec()));
        }
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
        let mut state = self.state.lock().unwrap();
        if let Some(err) = state.error_state {
            return Err(err);
        }
        if !write.is_empty() {
            state.transaction_log.push(MockTransaction::WriteRead(write[0], read.len()));
        }
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

#[cfg(feature = "async")]
impl embedded_hal_async::i2c::I2c<embedded_hal::i2c::SevenBitAddress> for AS5600Mock {
    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        embedded_hal::i2c::I2c::read(self, address, read)
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        embedded_hal::i2c::I2c::write(self, address, write)
    }

    async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        embedded_hal::i2c::I2c::write_read(self, address, write, read)
    }

    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        embedded_hal::i2c::I2c::transaction(self, address, operations)
    }
}
