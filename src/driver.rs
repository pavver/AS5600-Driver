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

    /// Internal helper to read a 12-bit value from two consecutive registers.
    fn read_u16(&mut self, reg_hi: u8) -> Result<u16, AS56Error<I2C::Error>> {
        let mut buf = [0u8; 2];
        self.i2c
            .write_read(self.address, &[reg_hi], &mut buf)
            .map_err(AS56Error::I2c)?;
        Ok(u16::from_be_bytes(buf) & regs::ANGLE_MASK)
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
            .write(self.address, &[regs::BURN, regs::BURN_SETTINGS_CMD])
            .map_err(AS56Error::I2c)?;
        Ok(())
    }

    /// **DANGER**: Permanently burns Configuration settings to the chip.
    pub unsafe fn danger_permanent_burn_config(&mut self) -> Result<(), AS56Error<I2C::Error>> {
        self.i2c
            .write(self.address, &[regs::BURN, regs::BURN_CONFIG_CMD])
            .map_err(AS56Error::I2c)?;
        Ok(())
    }
}

#[cfg(feature = "async")]
impl<I2C: embedded_hal_async::i2c::I2c<SevenBitAddress>> AS5600Driver<I2C> {
    /// Internal helper to read a single byte from a register (async).
    async fn read_u8_async(&mut self, reg: u8) -> Result<u8, AS56Error<I2C::Error>> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(self.address, &[reg], &mut buf)
            .await
            .map_err(AS56Error::I2c)?;
        Ok(buf[0])
    }

    /// Internal helper to read a 12-bit value from two consecutive registers (async).
    async fn read_u16_async(&mut self, reg_hi: u8) -> Result<u16, AS56Error<I2C::Error>> {
        let mut buf = [0u8; 2];
        self.i2c
            .write_read(self.address, &[reg_hi], &mut buf)
            .await
            .map_err(AS56Error::I2c)?;
        Ok(u16::from_be_bytes(buf) & regs::ANGLE_MASK)
    }

    /// Internal helper to write a 12-bit value to two consecutive registers (async).
    async fn write_u16_async(&mut self, reg_hi: u8, value: u16) -> Result<(), AS56Error<I2C::Error>> {
        let bytes = value.to_be_bytes();
        self.i2c
            .write(self.address, &[reg_hi, bytes[0], bytes[1]])
            .await
            .map_err(AS56Error::I2c)?;
        Ok(())
    }
}

#[cfg(feature = "async")]
impl<I2C: embedded_hal_async::i2c::I2c<SevenBitAddress>> crate::traits::AS5600AsyncInterface
    for AS5600Driver<I2C>
{
    type Error = I2C::Error;

    async fn read_raw_angle(&mut self) -> Result<u16, AS56Error<Self::Error>> {
        self.read_u16_async(regs::RAW_ANGLE_HI).await
    }

    async fn read_angle(&mut self) -> Result<u16, AS56Error<Self::Error>> {
        self.read_u16_async(regs::ANGLE_HI).await
    }

    async fn get_burn_count(&mut self) -> Result<u8, AS56Error<Self::Error>> {
        Ok(self.read_u8_async(regs::ZMCO).await? & regs::ZMCO_MASK)
    }

    async fn get_status_raw(&mut self) -> Result<u8, AS56Error<Self::Error>> {
        self.read_u8_async(regs::STATUS).await
    }

    async fn get_magnet_status(&mut self) -> Result<MagnetStatus, AS56Error<Self::Error>> {
        let val = self.read_u8_async(regs::STATUS).await?;
        Ok(MagnetStatus {
            detected: (val & regs::STATUS_MD_MASK) != 0,
            too_weak: (val & regs::STATUS_ML_MASK) != 0,
            too_strong: (val & regs::STATUS_MH_MASK) != 0,
        })
    }

    async fn get_magnitude(&mut self) -> Result<u16, AS56Error<Self::Error>> {
        self.read_u16_async(regs::MAGNITUDE_HI).await
    }

    async fn get_agc(&mut self) -> Result<u8, AS56Error<Self::Error>> {
        self.read_u8_async(regs::AGC).await
    }

    async fn get_config(&mut self) -> Result<Configuration, AS56Error<Self::Error>> {
        let mut buf = [0u8; 2];
        self.i2c
            .write_read(self.address, &[regs::CONF_HI], &mut buf)
            .await
            .map_err(AS56Error::I2c)?;
        let hi = buf[0];
        let lo = buf[1];

        Ok(Configuration {
            power_mode: match lo & regs::CONF_PM_MASK {
                0b01 => PowerMode::LPM1,
                0b10 => PowerMode::LPM2,
                0b11 => PowerMode::LPM3,
                _ => PowerMode::Nominal,
            },
            hysteresis: match (lo & regs::CONF_HYST_MASK) >> 2 {
                0b01 => Hysteresis::Lsb1,
                0b10 => Hysteresis::Lsb2,
                0b11 => Hysteresis::Lsb3,
                _ => Hysteresis::Off,
            },
            output_stage: match (lo & regs::CONF_OUTS_MASK) >> 4 {
                0b01 => OutputStage::AnalogReduced,
                0b10 => OutputStage::PWM,
                _ => OutputStage::AnalogFull,
            },
            pwm_frequency: match (lo & regs::CONF_PWMF_MASK) >> 6 {
                0b01 => PwmFrequency::Hz230,
                0b10 => PwmFrequency::Hz460,
                0b11 => PwmFrequency::Hz920,
                _ => PwmFrequency::Hz115,
            },
            slow_filter: match hi & regs::CONF_SF_MASK {
                0b01 => SlowFilter::X8,
                0b10 => SlowFilter::X4,
                0b11 => SlowFilter::X2,
                _ => SlowFilter::X16,
            },
            fast_filter_threshold: match (hi & regs::CONF_FTH_MASK) >> 2 {
                0b001 => FastFilterThreshold::Lsb6,
                0b010 => FastFilterThreshold::Lsb7,
                0b011 => FastFilterThreshold::Lsb9,
                0b100 => FastFilterThreshold::Lsb18,
                0b101 => FastFilterThreshold::Lsb21,
                0b110 => FastFilterThreshold::Lsb24,
                0b111 => FastFilterThreshold::Lsb10,
                _ => FastFilterThreshold::SlowOnly,
            },
            watchdog: (hi & regs::CONF_WD_MASK) != 0,
        })
    }

    async fn set_config(&mut self, config: Configuration) -> Result<(), AS56Error<Self::Error>> {
        let hi = ((config.watchdog as u8) << 5)
            | ((config.fast_filter_threshold as u8) << 2)
            | (config.slow_filter as u8);

        let lo = ((config.pwm_frequency as u8) << 6)
            | ((config.output_stage as u8) << 4)
            | ((config.hysteresis as u8) << 2)
            | (config.power_mode as u8);

        self.i2c
            .write(self.address, &[regs::CONF_HI, hi, lo])
            .await
            .map_err(AS56Error::I2c)?;
        Ok(())
    }

    async fn get_zero_position(&mut self) -> Result<u16, AS56Error<Self::Error>> {
        self.read_u16_async(regs::ZPOS_HI).await
    }

    async fn set_zero_position(&mut self, angle: u16) -> Result<(), AS56Error<Self::Error>> {
        self.write_u16_async(regs::ZPOS_HI, angle & regs::ZPOS_MASK).await
    }

    async fn get_max_position(&mut self) -> Result<u16, AS56Error<Self::Error>> {
        self.read_u16_async(regs::MPOS_HI).await
    }

    async fn set_max_position(&mut self, angle: u16) -> Result<(), AS56Error<Self::Error>> {
        self.write_u16_async(regs::MPOS_HI, angle & regs::MPOS_MASK).await
    }

    async fn get_max_angle(&mut self) -> Result<u16, AS56Error<Self::Error>> {
        self.read_u16_async(regs::MANG_HI).await
    }

    async fn set_max_angle(&mut self, angle: u16) -> Result<(), AS56Error<Self::Error>> {
        self.write_u16_async(regs::MANG_HI, angle & regs::MANG_MASK).await
    }

    async fn read_all_diagnostics(&mut self) -> Result<Diagnostics, AS56Error<Self::Error>> {
        let mut buf = [0u8; 18];
        self.i2c
            .write_read(self.address, &[regs::STATUS], &mut buf)
            .await
            .map_err(AS56Error::I2c)?;

        let status_val = buf[0];
        let raw_angle = u16::from_be_bytes([buf[1], buf[2]]) & regs::ANGLE_MASK;
        let angle = u16::from_be_bytes([buf[3], buf[4]]) & regs::ANGLE_MASK;
        let agc = buf[15];
        let magnitude = u16::from_be_bytes([buf[16], buf[17]]) & regs::ANGLE_MASK;

        Ok(Diagnostics {
            angle,
            raw_angle,
            magnet_status: MagnetStatus {
                detected: (status_val & regs::STATUS_MD_MASK) != 0,
                too_weak: (status_val & regs::STATUS_ML_MASK) != 0,
                too_strong: (status_val & regs::STATUS_MH_MASK) != 0,
            },
            agc,
            magnitude,
        })
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
        Ok(self.read_u8(regs::ZMCO)? & regs::ZMCO_MASK)
    }

    fn get_status_raw(&mut self) -> Result<u8, AS56Error<Self::Error>> {
        self.read_u8(regs::STATUS)
    }

    fn get_magnet_status(&mut self) -> Result<MagnetStatus, AS56Error<Self::Error>> {
        let val = self.read_u8(regs::STATUS)?;
        Ok(MagnetStatus {
            detected: (val & regs::STATUS_MD_MASK) != 0,
            too_weak: (val & regs::STATUS_ML_MASK) != 0,
            too_strong: (val & regs::STATUS_MH_MASK) != 0,
        })
    }

    fn get_magnitude(&mut self) -> Result<u16, AS56Error<Self::Error>> {
        self.read_u16(regs::MAGNITUDE_HI)
    }

    fn get_agc(&mut self) -> Result<u8, AS56Error<Self::Error>> {
        self.read_u8(regs::AGC)
    }

    fn get_config(&mut self) -> Result<Configuration, AS56Error<Self::Error>> {
        let mut buf = [0u8; 2];
        self.i2c
            .write_read(self.address, &[regs::CONF_HI], &mut buf)
            .map_err(AS56Error::I2c)?;
        let hi = buf[0];
        let lo = buf[1];

        Ok(Configuration {
            power_mode: match lo & regs::CONF_PM_MASK {
                0b01 => PowerMode::LPM1,
                0b10 => PowerMode::LPM2,
                0b11 => PowerMode::LPM3,
                _ => PowerMode::Nominal,
            },
            hysteresis: match (lo & regs::CONF_HYST_MASK) >> 2 {
                0b01 => Hysteresis::Lsb1,
                0b10 => Hysteresis::Lsb2,
                0b11 => Hysteresis::Lsb3,
                _ => Hysteresis::Off,
            },
            output_stage: match (lo & regs::CONF_OUTS_MASK) >> 4 {
                0b01 => OutputStage::AnalogReduced,
                0b10 => OutputStage::PWM,
                _ => OutputStage::AnalogFull,
            },
            pwm_frequency: match (lo & regs::CONF_PWMF_MASK) >> 6 {
                0b01 => PwmFrequency::Hz230,
                0b10 => PwmFrequency::Hz460,
                0b11 => PwmFrequency::Hz920,
                _ => PwmFrequency::Hz115,
            },
            slow_filter: match hi & regs::CONF_SF_MASK {
                0b01 => SlowFilter::X8,
                0b10 => SlowFilter::X4,
                0b11 => SlowFilter::X2,
                _ => SlowFilter::X16,
            },
            fast_filter_threshold: match (hi & regs::CONF_FTH_MASK) >> 2 {
                0b001 => FastFilterThreshold::Lsb6,
                0b010 => FastFilterThreshold::Lsb7,
                0b011 => FastFilterThreshold::Lsb9,
                0b100 => FastFilterThreshold::Lsb18,
                0b101 => FastFilterThreshold::Lsb21,
                0b110 => FastFilterThreshold::Lsb24,
                0b111 => FastFilterThreshold::Lsb10,
                _ => FastFilterThreshold::SlowOnly,
            },
            watchdog: (hi & regs::CONF_WD_MASK) != 0,
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

        self.i2c
            .write(self.address, &[regs::CONF_HI, hi, lo])
            .map_err(AS56Error::I2c)?;
        Ok(())
    }

    fn get_zero_position(&mut self) -> Result<u16, AS56Error<Self::Error>> {
        self.read_u16(regs::ZPOS_HI)
    }

    fn set_zero_position(&mut self, angle: u16) -> Result<(), AS56Error<Self::Error>> {
        self.write_u16(regs::ZPOS_HI, angle & regs::ZPOS_MASK)
    }

    fn get_max_position(&mut self) -> Result<u16, AS56Error<Self::Error>> {
        self.read_u16(regs::MPOS_HI)
    }

    fn set_max_position(&mut self, angle: u16) -> Result<(), AS56Error<Self::Error>> {
        self.write_u16(regs::MPOS_HI, angle & regs::MPOS_MASK)
    }

    fn get_max_angle(&mut self) -> Result<u16, AS56Error<Self::Error>> {
        self.read_u16(regs::MANG_HI)
    }

    fn set_max_angle(&mut self, angle: u16) -> Result<(), AS56Error<Self::Error>> {
        self.write_u16(regs::MANG_HI, angle & regs::MANG_MASK)
    }

    fn read_all_diagnostics(&mut self) -> Result<Diagnostics, AS56Error<Self::Error>> {
        // Read from STATUS (0x0B) to MAGNITUDE_LO (0x1C) = 18 bytes
        let mut buf = [0u8; 18];
        self.i2c
            .write_read(self.address, &[regs::STATUS], &mut buf)
            .map_err(AS56Error::I2c)?;

        let status_val = buf[0];
        let raw_angle = u16::from_be_bytes([buf[1], buf[2]]) & regs::ANGLE_MASK;
        let angle = u16::from_be_bytes([buf[3], buf[4]]) & regs::ANGLE_MASK;
        let agc = buf[15]; // AGC is at 0x1A, which is 0x0B + 15
        let magnitude = u16::from_be_bytes([buf[16], buf[17]]) & regs::ANGLE_MASK;

        Ok(Diagnostics {
            angle,
            raw_angle,
            magnet_status: MagnetStatus {
                detected: (status_val & regs::STATUS_MD_MASK) != 0,
                too_weak: (status_val & regs::STATUS_ML_MASK) != 0,
                too_strong: (status_val & regs::STATUS_MH_MASK) != 0,
            },
            agc,
            magnitude,
        })
    }
}
