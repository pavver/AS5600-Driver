use crate::regs::*;
use crate::traits::AS5600Interface;
use crate::types::*;
use crate::error::AS56Error;
use embedded_hal::i2c::{I2c, SevenBitAddress};

/// Main driver for the AS5600 sensor.
pub struct AS5600Driver<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C: I2c<SevenBitAddress>> AS5600Driver<I2C> {
    /// Creates a new driver instance with the default I2C address (0x36).
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            address: DEFAULT_ADDR,
        }
    }

    /// Creates a new driver instance with a custom I2C address.
    pub fn with_address(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }

    /// Internal helper to read a single byte from a register.
    fn read_u8(&mut self, reg: u8) -> Result<u8, AS56Error<I2C::Error>> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(self.address, &[reg], &mut buf)
            .map_err(AS56Error::I2c)?;
        Ok(buf[0])
    }

    /// Internal helper to write a single byte to a register.
    fn write_u8(&mut self, reg: u8, value: u8) -> Result<(), AS56Error<I2C::Error>> {
        self.i2c
            .write(self.address, &[reg, value])
            .map_err(AS56Error::I2c)?;
        Ok(())
    }

    /// Internal helper to read a 12-bit value from two consecutive registers.
    fn read_u16(&mut self, reg_hi: u8) -> Result<u16, AS56Error<I2C::Error>> {
        let mut buf = [0u8; 2];
        self.i2c
            .write_read(self.address, &[reg_hi], &mut buf)
            .map_err(AS56Error::I2c)?;
        Ok(u16::from_be_bytes(buf) & 0x0FFF)
    }

    /// Internal helper to write a 12-bit value to two consecutive registers.
    fn write_u16(&mut self, reg_hi: u8, value: u16) -> Result<(), AS56Error<I2C::Error>> {
        let bytes = value.to_be_bytes();
        self.i2c
            .write(self.address, &[reg_hi, bytes[0], bytes[1]])
            .map_err(AS56Error::I2c)?;
        Ok(())
    }

    /// **DANGER**: Permanently burns ZPOS and MPOS settings to the chip.
    pub unsafe fn danger_permanent_burn_settings(&mut self) -> Result<(), AS56Error<I2C::Error>> {
        self.i2c
            .write(self.address, &[regs::BURN, 0x80])
            .map_err(AS56Error::I2c)?;
        Ok(())
    }

    /// **DANGER**: Permanently burns Configuration settings to the chip.
    pub unsafe fn danger_permanent_burn_config(&mut self) -> Result<(), AS56Error<I2C::Error>> {
        self.i2c
            .write(self.address, &[regs::BURN, 0x40])
            .map_err(AS56Error::I2c)?;
        Ok(())
    }
}

impl<I2C: I2c<SevenBitAddress>> AS5600Interface for AS5600Driver<I2C> {
    type Error = I2C::Error;

    fn read_raw_angle(&mut self) -> Result<u16, AS56Error<Self::Error>> {
        self.read_u16(regs::RAW_ANGLE_HI)
    }

    fn read_angle(&mut self) -> Result<u16, AS56Error<Self::Error>> {
        self.read_u16(regs::ANGLE_HI)
    }

    fn get_burn_count(&mut self) -> Result<u8, AS56Error<Self::Error>> {
        Ok(self.read_u8(regs::ZMCO)? & 0x03)
    }

    fn get_status_raw(&mut self) -> Result<u8, AS56Error<Self::Error>> {
        self.read_u8(regs::STATUS)
    }

    fn get_magnet_status(&mut self) -> Result<MagnetStatus, AS56Error<Self::Error>> {
        let val = self.read_u8(regs::STATUS)?;
        Ok(MagnetStatus {
            detected: (val & 0x20) != 0,
            too_weak: (val & 0x10) != 0,
            too_strong: (val & 0x08) != 0,
        })
    }

    fn get_magnitude(&mut self) -> Result<u16, AS56Error<Self::Error>> {
        self.read_u16(regs::MAGNITUDE_HI)
    }

    fn get_agc(&mut self) -> Result<u8, AS56Error<Self::Error>> {
        self.read_u8(regs::AGC)
    }

    fn get_config(&mut self) -> Result<Configuration, AS56Error<Self::Error>> {
        let hi = self.read_u8(regs::CONF_HI)?;
        let lo = self.read_u8(regs::CONF_LO)?;

        Ok(Configuration {
            power_mode: match lo & 0x03 {
                0b01 => PowerMode::LPM1,
                0b10 => PowerMode::LPM2,
                0b11 => PowerMode::LPM3,
                _ => PowerMode::Nominal,
            },
            hysteresis: match (lo >> 2) & 0x03 {
                0b01 => Hysteresis::Lsb1,
                0b10 => Hysteresis::Lsb2,
                0b11 => Hysteresis::Lsb3,
                _ => Hysteresis::Off,
            },
            output_stage: match (lo >> 4) & 0x03 {
                0b01 => OutputStage::AnalogReduced,
                0b10 => OutputStage::PWM,
                _ => OutputStage::AnalogFull,
            },
            pwm_frequency: match (lo >> 6) & 0x03 {
                0b01 => PwmFrequency::Hz230,
                0b10 => PwmFrequency::Hz460,
                0b11 => PwmFrequency::Hz920,
                _ => PwmFrequency::Hz115,
            },
            slow_filter: match hi & 0x03 {
                0b01 => SlowFilter::X8,
                0b10 => SlowFilter::X4,
                0b11 => SlowFilter::X2,
                _ => SlowFilter::X16,
            },
            fast_filter_threshold: match (hi >> 2) & 0x07 {
                0b001 => FastFilterThreshold::Lsb6,
                0b010 => FastFilterThreshold::Lsb7,
                0b011 => FastFilterThreshold::Lsb9,
                0b100 => FastFilterThreshold::Lsb18,
                0b101 => FastFilterThreshold::Lsb21,
                0b110 => FastFilterThreshold::Lsb24,
                0b111 => FastFilterThreshold::Lsb10,
                _ => FastFilterThreshold::SlowOnly,
            },
            watchdog: (hi & 0x20) != 0,
        })
    }

    fn set_config(&mut self, config: Configuration) -> Result<(), AS56Error<Self::Error>> {
        let hi = ((config.watchdog as u8) << 5)
            | ((config.fast_filter_threshold as u8) << 2)
            | (config.slow_filter as u8);

        let lo = ((config.pwm_frequency as u8) << 6)
            | ((config.output_stage as u8) << 4)
            | ((config.hysteresis as u8) << 2)
            | (config.power_mode as u8);

        self.write_u8(regs::CONF_HI, hi)?;
        self.write_u8(regs::CONF_LO, lo)?;
        Ok(())
    }

    fn get_zero_position(&mut self) -> Result<u16, AS56Error<Self::Error>> {
        self.read_u16(regs::ZPOS_HI)
    }

    fn set_zero_position(&mut self, angle: u16) -> Result<(), AS56Error<Self::Error>> {
        self.write_u16(regs::ZPOS_HI, angle & 0x0FFF)
    }

    fn get_max_position(&mut self) -> Result<u16, AS56Error<Self::Error>> {
        self.read_u16(regs::MPOS_HI)
    }

    fn set_max_position(&mut self, angle: u16) -> Result<(), AS56Error<Self::Error>> {
        self.write_u16(regs::MPOS_HI, angle & 0x0FFF)
    }

    fn get_max_angle(&mut self) -> Result<u16, AS56Error<Self::Error>> {
        self.read_u16(regs::MANG_HI)
    }

    fn set_max_angle(&mut self, angle: u16) -> Result<(), AS56Error<Self::Error>> {
        self.write_u16(regs::MANG_HI, angle & 0x0FFF)
    }
}
