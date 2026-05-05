use crate::regs::*;
use crate::traits::AS5600Interface;
use crate::types::*;
use crate::error::AS5600Error;
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
    fn read_u8(&mut self, reg: u8) -> Result<u8, AS5600Error<I2C::Error>> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(self.address, &[reg], &mut buf)
            .map_err(AS5600Error::I2c)?;
        Ok(buf[0])
    }

    /// Internal helper to read a 12-bit value from two consecutive registers.
    fn read_u16(&mut self, reg_hi: u8) -> Result<u16, AS5600Error<I2C::Error>> {
        let mut buf = [0u8; 2];
        self.i2c
            .write_read(self.address, &[reg_hi], &mut buf)
            .map_err(AS5600Error::I2c)?;
        Ok(u16::from_be_bytes(buf) & regs::ANGLE_MASK)
    }

    /// Internal helper to write a 12-bit value to two consecutive registers.
    fn write_u16(&mut self, reg_hi: u8, value: u16) -> Result<(), AS5600Error<I2C::Error>> {
        let bytes = value.to_be_bytes();
        self.i2c
            .write(self.address, &[reg_hi, bytes[0], bytes[1]])
            .map_err(AS5600Error::I2c)?;
        Ok(())
    }

    /// **DANGER**: Permanently burns ZPOS and MPOS settings to the chip.
    pub unsafe fn danger_permanent_burn_settings(&mut self) -> Result<(), AS5600Error<I2C::Error>> {
        self.i2c
            .write(self.address, &[regs::BURN, regs::BURN_SETTINGS_CMD])
            .map_err(AS5600Error::I2c)?;
        Ok(())
    }

    /// **DANGER**: Permanently burns Configuration settings to the chip.
    pub unsafe fn danger_permanent_burn_config(&mut self) -> Result<(), AS5600Error<I2C::Error>> {
        self.i2c
            .write(self.address, &[regs::BURN, regs::BURN_CONFIG_CMD])
            .map_err(AS5600Error::I2c)?;
        Ok(())
    }
}

#[cfg(feature = "async")]
impl<I2C: embedded_hal_async::i2c::I2c<SevenBitAddress>> AS5600Driver<I2C> {
    /// Internal helper to read a single byte from a register (async).
    async fn read_u8_async(&mut self, reg: u8) -> Result<u8, AS5600Error<I2C::Error>> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(self.address, &[reg], &mut buf)
            .await
            .map_err(AS5600Error::I2c)?;
        Ok(buf[0])
    }

    /// Internal helper to read a 12-bit value from two consecutive registers (async).
    async fn read_u16_async(&mut self, reg_hi: u8) -> Result<u16, AS5600Error<I2C::Error>> {
        let mut buf = [0u8; 2];
        self.i2c
            .write_read(self.address, &[reg_hi], &mut buf)
            .await
            .map_err(AS5600Error::I2c)?;
        Ok(u16::from_be_bytes(buf) & regs::ANGLE_MASK)
    }

    /// Internal helper to write a 12-bit value to two consecutive registers (async).
    async fn write_u16_async(&mut self, reg_hi: u8, value: u16) -> Result<(), AS5600Error<I2C::Error>> {
        let bytes = value.to_be_bytes();
        self.i2c
            .write(self.address, &[reg_hi, bytes[0], bytes[1]])
            .await
            .map_err(AS5600Error::I2c)?;
        Ok(())
    }
}

#[cfg(feature = "async")]
impl<I2C: embedded_hal_async::i2c::I2c<SevenBitAddress>> crate::traits::AS5600AsyncInterface
    for AS5600Driver<I2C>
{
    type Error = I2C::Error;

    async fn read_raw_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
        self.read_u16_async(regs::RAW_ANGLE_HI).await
    }

    async fn read_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
        self.read_u16_async(regs::ANGLE_HI).await
    }

    async fn get_burn_count(&mut self) -> Result<u8, AS5600Error<Self::Error>> {
        Ok(self.read_u8_async(regs::ZMCO).await? & regs::ZMCO_MASK)
    }

    async fn get_status_raw(&mut self) -> Result<u8, AS5600Error<Self::Error>> {
        self.read_u8_async(regs::STATUS).await
    }

    async fn get_magnet_status(&mut self) -> Result<MagnetStatus, AS5600Error<Self::Error>> {
        let val = self.read_u8_async(regs::STATUS).await?;
        Ok(MagnetStatus {
            detected: (val & regs::STATUS_MD_MASK) != 0,
            too_weak: (val & regs::STATUS_ML_MASK) != 0,
            too_strong: (val & regs::STATUS_MH_MASK) != 0,
        })
    }

    async fn get_magnitude(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
        self.read_u16_async(regs::MAGNITUDE_HI).await
    }

    async fn get_agc(&mut self) -> Result<u8, AS5600Error<Self::Error>> {
        self.read_u8_async(regs::AGC).await
    }

    async fn get_config(&mut self) -> Result<Configuration, AS5600Error<Self::Error>> {
        let mut buf = [0u8; 2];
        self.i2c
            .write_read(self.address, &[regs::CONF_HI], &mut buf)
            .await
            .map_err(AS5600Error::I2c)?;
        Ok(Configuration::from_bytes(buf[0], buf[1]))
    }

    async fn set_config(&mut self, config: Configuration) -> Result<(), AS5600Error<Self::Error>> {
        let (hi, lo) = config.to_bytes();
        self.i2c
            .write(self.address, &[regs::CONF_HI, hi, lo])
            .await
            .map_err(AS5600Error::I2c)?;
        Ok(())
    }

    async fn get_zero_position(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
        self.read_u16_async(regs::ZPOS_HI).await
    }

    async fn set_zero_position(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>> {
        self.write_u16_async(regs::ZPOS_HI, angle & regs::ZPOS_MASK).await
    }

    async fn get_max_position(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
        self.read_u16_async(regs::MPOS_HI).await
    }

    async fn set_max_position(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>> {
        self.write_u16_async(regs::MPOS_HI, angle & regs::MPOS_MASK).await
    }

    async fn get_max_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
        self.read_u16_async(regs::MANG_HI).await
    }

    async fn set_max_angle(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>> {
        self.write_u16_async(regs::MANG_HI, angle & regs::MANG_MASK).await
    }

    async fn read_all_diagnostics(&mut self) -> Result<Diagnostics, AS5600Error<Self::Error>> {
        let mut buf = [0u8; 18];
        self.i2c
            .write_read(self.address, &[regs::STATUS], &mut buf)
            .await
            .map_err(AS5600Error::I2c)?;

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

    fn read_raw_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
        self.read_u16(regs::RAW_ANGLE_HI)
    }

    fn read_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
        self.read_u16(regs::ANGLE_HI)
    }

    fn get_burn_count(&mut self) -> Result<u8, AS5600Error<Self::Error>> {
        Ok(self.read_u8(regs::ZMCO)? & regs::ZMCO_MASK)
    }

    fn get_status_raw(&mut self) -> Result<u8, AS5600Error<Self::Error>> {
        self.read_u8(regs::STATUS)
    }

    fn get_magnet_status(&mut self) -> Result<MagnetStatus, AS5600Error<Self::Error>> {
        let val = self.read_u8(regs::STATUS)?;
        Ok(MagnetStatus {
            detected: (val & regs::STATUS_MD_MASK) != 0,
            too_weak: (val & regs::STATUS_ML_MASK) != 0,
            too_strong: (val & regs::STATUS_MH_MASK) != 0,
        })
    }

    fn get_magnitude(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
        self.read_u16(regs::MAGNITUDE_HI)
    }

    fn get_agc(&mut self) -> Result<u8, AS5600Error<Self::Error>> {
        self.read_u8(regs::AGC)
    }

    fn get_config(&mut self) -> Result<Configuration, AS5600Error<Self::Error>> {
        let mut buf = [0u8; 2];
        self.i2c
            .write_read(self.address, &[regs::CONF_HI], &mut buf)
            .map_err(AS5600Error::I2c)?;
        Ok(Configuration::from_bytes(buf[0], buf[1]))
    }

    fn set_config(&mut self, config: Configuration) -> Result<(), AS5600Error<Self::Error>> {
        let (hi, lo) = config.to_bytes();
        self.i2c
            .write(self.address, &[regs::CONF_HI, hi, lo])
            .map_err(AS5600Error::I2c)?;
        Ok(())
    }

    fn get_zero_position(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
        self.read_u16(regs::ZPOS_HI)
    }

    fn set_zero_position(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>> {
        self.write_u16(regs::ZPOS_HI, angle & regs::ZPOS_MASK)
    }

    fn get_max_position(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
        self.read_u16(regs::MPOS_HI)
    }

    fn set_max_position(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>> {
        self.write_u16(regs::MPOS_HI, angle & regs::MPOS_MASK)
    }

    fn get_max_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
        self.read_u16(regs::MANG_HI)
    }

    fn set_max_angle(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>> {
        self.write_u16(regs::MANG_HI, angle & regs::MANG_MASK)
    }

    fn read_all_diagnostics(&mut self) -> Result<Diagnostics, AS5600Error<Self::Error>> {
        // Read from STATUS (0x0B) to MAGNITUDE_LO (0x1C) = 18 bytes
        let mut buf = [0u8; 18];
        self.i2c
            .write_read(self.address, &[regs::STATUS], &mut buf)
            .map_err(AS5600Error::I2c)?;

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

#[cfg(all(test, feature = "mock"))]
mod tests {
    use super::*;
    use crate::mock::AS5600Mock;

    #[test]
    fn test_read_angle() {
        let mock = AS5600Mock::new();
        mock.mock_set_raw_angle(1234);
        let mut driver = AS5600Driver::new(mock);
        assert_eq!(driver.read_raw_angle().unwrap(), 1234);
    }

    #[test]
    fn test_magnet_status() {
        let mock = AS5600Mock::new();
        let status = MagnetStatus {
            detected: true,
            too_weak: false,
            too_strong: true,
        };
        mock.mock_set_status(status);
        let mut driver = AS5600Driver::new(mock);
        assert_eq!(driver.get_magnet_status().unwrap(), status);
    }

    #[test]
    fn test_agc_magnitude() {
        let mock = AS5600Mock::new();
        mock.mock_set_agc(150);
        mock.mock_set_magnitude(2000);
        let mut driver = AS5600Driver::new(mock);
        assert_eq!(driver.get_agc().unwrap(), 150);
        assert_eq!(driver.get_magnitude().unwrap(), 2000);
    }

    #[test]
    fn test_config_builder() {
        let config = Configuration::builder()
            .power_mode(PowerMode::LPM3)
            .watchdog(false)
            .hysteresis(Hysteresis::Off)
            .build();
        
        assert_eq!(config.power_mode, PowerMode::LPM3);
        assert_eq!(config.watchdog, false);
        assert_eq!(config.hysteresis, Hysteresis::Off);
        // Default values for other fields
        assert_eq!(config.slow_filter, SlowFilter::X16);
    }

    #[test]
    fn test_config_read_write() {
        let mock = AS5600Mock::new();
        let mut driver = AS5600Driver::new(mock);
        
        let config = Configuration {
            power_mode: PowerMode::LPM2,
            hysteresis: Hysteresis::Lsb3,
            output_stage: OutputStage::PWM,
            pwm_frequency: PwmFrequency::Hz460,
            slow_filter: SlowFilter::X2,
            fast_filter_threshold: FastFilterThreshold::Lsb18,
            watchdog: false,
        };

        driver.set_config(config).unwrap();
        let read_config = driver.get_config().unwrap();
        assert_eq!(read_config, config);
    }

    #[test]
    fn test_diagnostics() {
        let mock = AS5600Mock::new();
        mock.mock_set_raw_angle(1000);
        mock.mock_set_agc(50);
        mock.mock_set_magnitude(3000);
        mock.mock_set_status(MagnetStatus {
            detected: true,
            too_weak: false,
            too_strong: false,
        });

        let mut driver = AS5600Driver::new(mock);
        let diag = driver.read_all_diagnostics().unwrap();

        assert_eq!(diag.raw_angle, 1000);
        assert_eq!(diag.agc, 50);
        assert_eq!(diag.magnitude, 3000);
        assert_eq!(diag.magnet_status.detected, true);
    }

    #[cfg(feature = "async")]
    mod async_tests {
        use super::*;
        use crate::traits::AS5600AsyncInterface;

        #[tokio::test]
        async fn test_read_angle_async() {
            let mock = AS5600Mock::new();
            mock.mock_set_raw_angle(2500);
            let mut driver = AS5600Driver::new(mock);
            assert_eq!(AS5600AsyncInterface::read_raw_angle(&mut driver).await.unwrap(), 2500);
        }

        #[tokio::test]
        async fn test_config_async() {
            let mock = AS5600Mock::new();
            let mut driver = AS5600Driver::new(mock);
            let config = Configuration::default();
            AS5600AsyncInterface::set_config(&mut driver, config).await.unwrap();
            let read_config = AS5600AsyncInterface::get_config(&mut driver).await.unwrap();
            assert_eq!(read_config, config);
        }

        #[tokio::test]
        async fn test_diagnostics_async() {
            let mock = AS5600Mock::new();
            mock.mock_set_raw_angle(3000);
            let mut driver = AS5600Driver::new(mock);
            let diag = AS5600AsyncInterface::read_all_diagnostics(&mut driver).await.unwrap();
            assert_eq!(diag.raw_angle, 3000);
        }
    }
}
