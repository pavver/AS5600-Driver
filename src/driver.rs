use crate::error::AS5600Error;
use crate::regs::*;
use crate::traits::AS5600Interface;
use crate::types::*;
use embedded_hal::i2c::{self, SevenBitAddress};

#[cfg(feature = "async")]
use crate::traits::AS5600AsyncInterface;

/// Main driver for the AS5600 sensor.
pub struct AS5600Driver<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C> AS5600Driver<I2C> {
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
}

macro_rules! define_as5600_methods {
    ($($is_async:ident)?) => {
        define_as5600_methods!(@meth $($is_async)?);
    };
    (@meth async) => {
        /// Reads the raw angle from the sensor (12-bit).
        async fn read_raw_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
            self.read_u16_internal_async().await
        }

        /// Reads the angle from the sensor (12-bit).
        async fn read_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
            self.read_u16_internal_reg_async(regs::ANGLE_HI).await
        }

        /// Gets the current burn count (ZMCO).
        async fn get_burn_count(&mut self) -> Result<u8, AS5600Error<Self::Error>> {
            let val = self.read_u8_internal_async(regs::ZMCO).await?;
            Ok(val & regs::ZMCO_MASK)
        }

        /// Reads the raw status register byte.
        async fn get_status_raw(&mut self) -> Result<u8, AS5600Error<Self::Error>> {
            self.read_u8_internal_async(regs::STATUS).await
        }

        /// Gets the magnet status (detected, weak, strong).
        async fn get_magnet_status(&mut self) -> Result<MagnetStatus, AS5600Error<Self::Error>> {
            let val = self.read_u8_internal_async(regs::STATUS).await?;
            Ok(MagnetStatus {
                detected: (val & regs::STATUS_MD_MASK) != 0,
                too_weak: (val & regs::STATUS_ML_MASK) != 0,
                too_strong: (val & regs::STATUS_MH_MASK) != 0,
            })
        }

        /// Gets the current magnitude of the magnetic field (12-bit).
        async fn get_magnitude(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
            self.read_u16_internal_reg_async(regs::MAGNITUDE_HI).await
        }

        /// Gets the current AGC value.
        async fn get_agc(&mut self) -> Result<u8, AS5600Error<Self::Error>> {
            self.read_u8_internal_async(regs::AGC).await
        }

        /// Reads the full configuration from the sensor.
        async fn get_config(&mut self) -> Result<Configuration, AS5600Error<Self::Error>> {
            let mut buf = [0u8; 2];
            self.i2c.write_read(self.address, &[regs::CONF_HI], &mut buf).await?;
            Ok(Configuration::from_bytes(buf[0], buf[1]))
        }

        /// Writes a complete configuration to the sensor.
        async fn set_config(&mut self, config: Configuration) -> Result<(), AS5600Error<Self::Error>> {
            let (hi, lo) = config.to_bytes();
            self.i2c.write(self.address, &[regs::CONF_HI, hi, lo]).await?;
            Ok(())
        }

        /// Applies configuration changes using a builder pattern.
        ///
        /// Only modified fields are updated. If a register is partially modified,
        /// its current value is read from the sensor before writing.
        async fn apply_config(&mut self, builder: ConfigurationBuilder) -> Result<(), AS5600Error<Self::Error>> {
            if builder.is_hi_dirty() {
                let hi_val = if builder.is_hi_complete() {
                    builder.calculate_hi(0)
                } else {
                    builder.calculate_hi(self.read_u8_internal_async(regs::CONF_HI).await?)
                };
                self.i2c.write(self.address, &[regs::CONF_HI, hi_val]).await?;
            }
            if builder.is_lo_dirty() {
                let lo_val = if builder.is_lo_complete() {
                    builder.calculate_lo(0)
                } else {
                    builder.calculate_lo(self.read_u8_internal_async(regs::CONF_LO).await?)
                };
                self.i2c.write(self.address, &[regs::CONF_LO, lo_val]).await?;
            }
            Ok(())
        }

        /// Gets the programmed zero position (ZPOS).
        async fn get_zero_position(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
            self.read_u16_internal_reg_async(regs::ZPOS_HI).await
        }

        /// Sets the zero position (ZPOS).
        async fn set_zero_position(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>> {
            self.write_u16_internal_async(regs::ZPOS_HI, angle).await
        }

        /// Gets the programmed max position (MPOS).
        async fn get_max_position(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
            self.read_u16_internal_reg_async(regs::MPOS_HI).await
        }

        /// Sets the max position (MPOS).
        async fn set_max_position(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>> {
            self.write_u16_internal_async(regs::MPOS_HI, angle).await
        }

        /// Gets the programmed max angle (MANG).
        async fn get_max_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
            self.read_u16_internal_reg_async(regs::MANG_HI).await
        }

        /// Sets the max angle (MANG).
        async fn set_max_angle(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>> {
            self.write_u16_internal_async(regs::MANG_HI, angle).await
        }

        /// Checks if the sensor is connected by attempting to read ZMCO.
        async fn is_connected(&mut self) -> bool {
            self.read_u8_internal_async(regs::ZMCO).await.is_ok()
        }

        /// Reads all diagnostic data in a single I2C transaction.
        async fn read_all_diagnostics(&mut self) -> Result<Diagnostics, AS5600Error<Self::Error>> {
            let mut buf = [0u8; 18];
            self.i2c.write_read(self.address, &[regs::STATUS], &mut buf).await?;
            let status_val = buf[0];
            let raw_angle = u16::from_be_bytes([buf[regs::RAW_ANGLE_OFFSET], buf[regs::RAW_ANGLE_OFFSET + 1]]) & regs::ANGLE_MASK;
            let angle = u16::from_be_bytes([buf[regs::ANGLE_OFFSET], buf[regs::ANGLE_OFFSET + 1]]) & regs::ANGLE_MASK;
            let agc = buf[regs::AGC_OFFSET];
            let magnitude = u16::from_be_bytes([buf[regs::MAGNITUDE_OFFSET], buf[regs::MAGNITUDE_OFFSET + 1]]) & regs::ANGLE_MASK;
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
    };
    (@meth) => {
        /// Reads the raw angle from the sensor (12-bit).
        fn read_raw_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
            self.read_u16_internal()
        }

        /// Reads the angle from the sensor (12-bit).
        fn read_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
            self.read_u16_internal_reg(regs::ANGLE_HI)
        }

        /// Gets the current burn count (ZMCO).
        fn get_burn_count(&mut self) -> Result<u8, AS5600Error<Self::Error>> {
            let val = self.read_u8_internal(regs::ZMCO)?;
            Ok(val & regs::ZMCO_MASK)
        }

        /// Reads the raw status register byte.
        fn get_status_raw(&mut self) -> Result<u8, AS5600Error<Self::Error>> {
            self.read_u8_internal(regs::STATUS)
        }

        /// Gets the magnet status (detected, weak, strong).
        fn get_magnet_status(&mut self) -> Result<MagnetStatus, AS5600Error<Self::Error>> {
            let val = self.read_u8_internal(regs::STATUS)?;
            Ok(MagnetStatus {
                detected: (val & regs::STATUS_MD_MASK) != 0,
                too_weak: (val & regs::STATUS_ML_MASK) != 0,
                too_strong: (val & regs::STATUS_MH_MASK) != 0,
            })
        }

        /// Gets the current magnitude of the magnetic field (12-bit).
        fn get_magnitude(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
            self.read_u16_internal_reg(regs::MAGNITUDE_HI)
        }

        /// Gets the current AGC value.
        fn get_agc(&mut self) -> Result<u8, AS5600Error<Self::Error>> {
            self.read_u8_internal(regs::AGC)
        }

        /// Reads the full configuration from the sensor.
        fn get_config(&mut self) -> Result<Configuration, AS5600Error<Self::Error>> {
            let mut buf = [0u8; 2];
            self.i2c.write_read(self.address, &[regs::CONF_HI], &mut buf)?;
            Ok(Configuration::from_bytes(buf[0], buf[1]))
        }

        /// Writes a complete configuration to the sensor.
        fn set_config(&mut self, config: Configuration) -> Result<(), AS5600Error<Self::Error>> {
            let (hi, lo) = config.to_bytes();
            self.i2c.write(self.address, &[regs::CONF_HI, hi, lo])?;
            Ok(())
        }

        /// Applies configuration changes using a builder pattern.
        ///
        /// Only modified fields are updated. If a register is partially modified,
        /// its current value is read from the sensor before writing.
        fn apply_config(&mut self, builder: ConfigurationBuilder) -> Result<(), AS5600Error<Self::Error>> {
            if builder.is_hi_dirty() {
                let hi_val = if builder.is_hi_complete() {
                    builder.calculate_hi(0)
                } else {
                    builder.calculate_hi(self.read_u8_internal(regs::CONF_HI)?)
                };
                self.i2c.write(self.address, &[regs::CONF_HI, hi_val])?;
            }
            if builder.is_lo_dirty() {
                let lo_val = if builder.is_lo_complete() {
                    builder.calculate_lo(0)
                } else {
                    builder.calculate_lo(self.read_u8_internal(regs::CONF_LO)?)
                };
                self.i2c.write(self.address, &[regs::CONF_LO, lo_val])?;
            }
            Ok(())
        }

        /// Gets the programmed zero position (ZPOS).
        fn get_zero_position(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
            self.read_u16_internal_reg(regs::ZPOS_HI)
        }

        /// Sets the zero position (ZPOS).
        fn set_zero_position(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>> {
            self.write_u16_internal(regs::ZPOS_HI, angle)
        }

        /// Gets the programmed max position (MPOS).
        fn get_max_position(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
            self.read_u16_internal_reg(regs::MPOS_HI)
        }

        /// Sets the max position (MPOS).
        fn set_max_position(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>> {
            self.write_u16_internal(regs::MPOS_HI, angle)
        }

        /// Gets the programmed max angle (MANG).
        fn get_max_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
            self.read_u16_internal_reg(regs::MANG_HI)
        }

        /// Sets the max angle (MANG).
        fn set_max_angle(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>> {
            self.write_u16_internal(regs::MANG_HI, angle)
        }

        /// Checks if the sensor is connected by attempting to read ZMCO.
        fn is_connected(&mut self) -> bool {
            self.read_u8_internal(regs::ZMCO).is_ok()
        }

        /// Reads all diagnostic data in a single I2C transaction.
        fn read_all_diagnostics(&mut self) -> Result<Diagnostics, AS5600Error<Self::Error>> {
            let mut buf = [0u8; 18];
            self.i2c.write_read(self.address, &[regs::STATUS], &mut buf)?;
            let status_val = buf[0];
            let raw_angle = u16::from_be_bytes([buf[regs::RAW_ANGLE_OFFSET], buf[regs::RAW_ANGLE_OFFSET + 1]]) & regs::ANGLE_MASK;
            let angle = u16::from_be_bytes([buf[regs::ANGLE_OFFSET], buf[regs::ANGLE_OFFSET + 1]]) & regs::ANGLE_MASK;
            let agc = buf[regs::AGC_OFFSET];
            let magnitude = u16::from_be_bytes([buf[regs::MAGNITUDE_OFFSET], buf[regs::MAGNITUDE_OFFSET + 1]]) & regs::ANGLE_MASK;
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
    };
}

macro_rules! define_internal_helpers {
    ($($is_async:ident)?) => {
        define_internal_helpers!(@meth $($is_async)?);
    };
    (@meth async) => {
        /// Internal helper to read a single byte from a register (async).
        async fn read_u8_internal_async(&mut self, reg: u8) -> Result<u8, AS5600Error<I2C::Error>> {
            let mut buf = [0u8; 1];
            self.i2c.write_read(self.address, &[reg], &mut buf).await?;
            Ok(buf[0])
        }

        /// Internal helper to read a 12-bit value from RAW_ANGLE_HI (async).
        async fn read_u16_internal_async(&mut self) -> Result<u16, AS5600Error<I2C::Error>> {
            let mut buf = [0u8; 2];
            self.i2c.write_read(self.address, &[regs::RAW_ANGLE_HI], &mut buf).await?;
            Ok(u16::from_be_bytes(buf) & regs::ANGLE_MASK)
        }

        /// Internal helper to read a 12-bit value from a custom register (async).
        async fn read_u16_internal_reg_async(&mut self, reg: u8) -> Result<u16, AS5600Error<I2C::Error>> {
            let mut buf = [0u8; 2];
            self.i2c.write_read(self.address, &[reg], &mut buf).await?;
            Ok(u16::from_be_bytes(buf) & regs::ANGLE_MASK)
        }

        /// Internal helper to write a 12-bit value to two consecutive registers (async).
        async fn write_u16_internal_async(&mut self, reg_hi: u8, value: u16) -> Result<(), AS5600Error<I2C::Error>> {
            if value > regs::ANGLE_MASK {
                return Err(AS5600Error::InvalidParameter);
            }
            let bytes = value.to_be_bytes();
            self.i2c.write(self.address, &[reg_hi, bytes[0], bytes[1]]).await?;
            Ok(())
        }
    };
    (@meth) => {
        /// Internal helper to read a single byte from a register.
        fn read_u8_internal(&mut self, reg: u8) -> Result<u8, AS5600Error<I2C::Error>> {
            let mut buf = [0u8; 1];
            self.i2c.write_read(self.address, &[reg], &mut buf)?;
            Ok(buf[0])
        }

        /// Internal helper to read a 12-bit value from RAW_ANGLE_HI.
        fn read_u16_internal(&mut self) -> Result<u16, AS5600Error<I2C::Error>> {
            let mut buf = [0u8; 2];
            self.i2c.write_read(self.address, &[regs::RAW_ANGLE_HI], &mut buf)?;
            Ok(u16::from_be_bytes(buf) & regs::ANGLE_MASK)
        }

        /// Internal helper to read a 12-bit value from a custom register.
        fn read_u16_internal_reg(&mut self, reg: u8) -> Result<u16, AS5600Error<I2C::Error>> {
            let mut buf = [0u8; 2];
            self.i2c.write_read(self.address, &[reg], &mut buf)?;
            Ok(u16::from_be_bytes(buf) & regs::ANGLE_MASK)
        }

        /// Internal helper to write a 12-bit value to two consecutive registers.
        fn write_u16_internal(&mut self, reg_hi: u8, value: u16) -> Result<(), AS5600Error<I2C::Error>> {
            if value > regs::ANGLE_MASK {
                return Err(AS5600Error::InvalidParameter);
            }
            let bytes = value.to_be_bytes();
            self.i2c.write(self.address, &[reg_hi, bytes[0], bytes[1]])?;
            Ok(())
        }
    };
}

impl<I2C: i2c::I2c<SevenBitAddress>> AS5600Interface for AS5600Driver<I2C> {
    type Error = I2C::Error;
    define_as5600_methods!();
}

impl<I2C: i2c::I2c<SevenBitAddress>> AS5600Driver<I2C> {
    define_internal_helpers!();

    /// Permanently burns ZPOS and MPOS settings to the chip.
    ///
    /// This requires a [`BurnToken`] to confirm the irreversible intent.
    /// The AS5600 allows burning ZPOS/MPOS settings up to 3 times (see ZMCO).
    pub fn permanent_burn_settings(
        &mut self,
        _token: BurnToken,
    ) -> Result<(), AS5600Error<I2C::Error>> {
        let count = AS5600Interface::get_burn_count(self)?;
        if count >= 3 {
            return Err(AS5600Error::OtpMaxBurnsReached);
        }
        self.i2c
            .write(self.address, &[regs::BURN, regs::BURN_SETTINGS_CMD])?;
        Ok(())
    }

    /// Permanently burns Configuration settings to the chip.
    ///
    /// This requires a [`BurnToken`] to confirm the irreversible intent.
    /// The AS5600 allows burning the configuration **ONLY ONCE**.
    pub fn permanent_burn_config(
        &mut self,
        _token: BurnToken,
    ) -> Result<(), AS5600Error<I2C::Error>> {
        self.i2c
            .write(self.address, &[regs::BURN, regs::BURN_CONFIG_CMD])?;
        Ok(())
    }
}

#[cfg(feature = "async")]
use embedded_hal_async::i2c as async_i2c;

#[cfg(feature = "async")]
impl<I2C: async_i2c::I2c<SevenBitAddress>> AS5600AsyncInterface for AS5600Driver<I2C> {
    type Error = I2C::Error;
    define_as5600_methods!(async);
}

#[cfg(feature = "async")]
impl<I2C: async_i2c::I2c<SevenBitAddress>> AS5600Driver<I2C> {
    define_internal_helpers!(async);
}

#[cfg(all(test, feature = "mock"))]
mod tests {
    use super::*;
    use crate::mock::{AS5600Mock, MockTransaction};
    use std::vec;

    #[test]
    fn test_read_angle() {
        let mock = AS5600Mock::new();
        mock.mock_set_raw_angle(1234);
        let mut driver = AS5600Driver::new(mock);
        assert_eq!(AS5600Interface::read_raw_angle(&mut driver).unwrap(), 1234);
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
        assert_eq!(
            AS5600Interface::get_magnet_status(&mut driver).unwrap(),
            status
        );
    }

    #[test]
    fn test_agc_magnitude() {
        let mock = AS5600Mock::new();
        mock.mock_set_agc(150);
        mock.mock_set_magnitude(2000);
        let mut driver = AS5600Driver::new(mock);
        assert_eq!(AS5600Interface::get_agc(&mut driver).unwrap(), 150);
        assert_eq!(AS5600Interface::get_magnitude(&mut driver).unwrap(), 2000);
    }

    #[test]
    fn test_invalid_parameter_validation() {
        let mock = AS5600Mock::new();
        let mut driver = AS5600Driver::new(mock);

        // Value 4096 is out of 12-bit range (0..4095)
        let result = AS5600Interface::set_zero_position(&mut driver, 4096);
        assert!(matches!(result, Err(AS5600Error::InvalidParameter)));

        let result = AS5600Interface::set_max_position(&mut driver, 5000);
        assert!(matches!(result, Err(AS5600Error::InvalidParameter)));

        let result = AS5600Interface::set_max_angle(&mut driver, 0xFFFF);
        assert!(matches!(result, Err(AS5600Error::InvalidParameter)));
    }

    #[test]
    fn test_set_position_correctness() {
        let mock = AS5600Mock::new();
        let mut driver = AS5600Driver::new(mock.clone());

        // Test ZPOS (Zero Position)
        AS5600Interface::set_zero_position(&mut driver, 0x0A23).unwrap();
        let log = mock.mock_get_log();
        // Expected: Write to ZPOS_HI (0x01) with data [0x0A, 0x23]
        assert_eq!(log.len(), 1);
        if let MockTransaction::Write(reg, data) = &log[0] {
            assert_eq!(*reg, regs::ZPOS_HI);
            assert_eq!(data, &vec![0x0A, 0x23]);
        } else {
            panic!("Expected Write transaction");
        }

        // Test MPOS (Max Position)
        AS5600Interface::set_max_position(&mut driver, 0x0FCD).unwrap();
        let log = mock.mock_get_log();
        assert_eq!(log.len(), 1);
        if let MockTransaction::Write(reg, data) = &log[0] {
            assert_eq!(*reg, regs::MPOS_HI);
            assert_eq!(data, &vec![0x0F, 0xCD]);
        } else {
            panic!("Expected Write transaction");
        }
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
    fn test_apply_config_smart_logic() {
        use crate::mock::MockTransaction;
        let mock = AS5600Mock::new();
        let mut driver = AS5600Driver::new(mock.clone());

        // --- Case 1: Partial update (only one field in CONF_HI) ---
        // Expect: 1 WriteRead (to get current) + 1 Write (to update)
        mock.mock_clear_log();
        AS5600Interface::apply_config(&mut driver, Configuration::builder().watchdog(false))
            .unwrap();
        let log = mock.mock_get_log();
        assert_eq!(log.len(), 2);
        assert!(matches!(
            log[0],
            MockTransaction::WriteRead(regs::CONF_HI, 1)
        ));
        assert!(matches!(log[1], MockTransaction::Write(regs::CONF_HI, _)));

        // --- Case 2: Complete update of one register (all fields in CONF_HI) ---
        // Expect: Only 1 Write (no Read needed)
        mock.mock_clear_log();
        AS5600Interface::apply_config(
            &mut driver,
            Configuration::builder()
                .watchdog(true)
                .fast_filter_threshold(FastFilterThreshold::Lsb6)
                .slow_filter(SlowFilter::X2),
        )
        .unwrap();
        let log = mock.mock_get_log();
        assert_eq!(log.len(), 1);
        assert!(matches!(log[0], MockTransaction::Write(regs::CONF_HI, _)));

        // --- Case 3: Update fields in different registers (CONF_HI and CONF_LO) ---
        // Expect: Operations for both registers
        mock.mock_clear_log();
        AS5600Interface::apply_config(
            &mut driver,
            Configuration::builder()
                .watchdog(true)
                .power_mode(PowerMode::LPM1),
        )
        .unwrap();
        let log = mock.mock_get_log();
        // 2 registers * (Read + Write) = 4 operations
        assert_eq!(log.len(), 4);

        // --- Case 4: No changes ---
        // Expect: 0 operations
        mock.mock_clear_log();
        AS5600Interface::apply_config(&mut driver, Configuration::builder()).unwrap();
        assert_eq!(mock.mock_get_log().len(), 0);
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

        AS5600Interface::set_config(&mut driver, config).unwrap();
        let read_config = AS5600Interface::get_config(&mut driver).unwrap();
        assert_eq!(read_config, config);
    }

    #[test]
    fn test_i2c_error_handling() {
        let mock = AS5600Mock::new();
        let mut driver = AS5600Driver::new(mock.clone());

        // Everything OK
        assert!(AS5600Interface::read_raw_angle(&mut driver).is_ok());

        // Simulate I2C failure
        mock.mock_set_error(Some(crate::mock::MockError::I2cError));
        let result = AS5600Interface::read_raw_angle(&mut driver);

        assert!(result.is_err());
        match result.unwrap_err() {
            AS5600Error::I2c(e) => assert_eq!(e, crate::mock::MockError::I2cError),
            _ => panic!("Expected I2C error"),
        }
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
        let diag = AS5600Interface::read_all_diagnostics(&mut driver).unwrap();

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
            assert_eq!(
                AS5600AsyncInterface::read_raw_angle(&mut driver)
                    .await
                    .unwrap(),
                2500
            );
        }

        #[tokio::test]
        async fn test_config_async() {
            let mock = AS5600Mock::new();
            let mut driver = AS5600Driver::new(mock);
            let config = Configuration::default();
            AS5600AsyncInterface::set_config(&mut driver, config)
                .await
                .unwrap();
            let read_config = AS5600AsyncInterface::get_config(&mut driver).await.unwrap();
            assert_eq!(read_config, config);
        }

        #[tokio::test]
        async fn test_apply_config_async() {
            let mock = AS5600Mock::new();
            let mut driver = AS5600Driver::new(mock.clone());

            // Partial update
            mock.mock_clear_log();
            AS5600AsyncInterface::apply_config(
                &mut driver,
                ConfigurationBuilder::new().watchdog(false),
            )
            .await
            .unwrap();
            assert_eq!(mock.mock_get_log().len(), 2);
        }

        #[tokio::test]
        async fn test_positions_async() {
            let mock = AS5600Mock::new();
            let mut driver = AS5600Driver::new(mock);

            AS5600AsyncInterface::set_zero_position(&mut driver, 100)
                .await
                .unwrap();
            assert_eq!(
                AS5600AsyncInterface::get_zero_position(&mut driver)
                    .await
                    .unwrap(),
                100
            );

            AS5600AsyncInterface::set_max_position(&mut driver, 200)
                .await
                .unwrap();
            assert_eq!(
                AS5600AsyncInterface::get_max_position(&mut driver)
                    .await
                    .unwrap(),
                200
            );

            AS5600AsyncInterface::set_max_angle(&mut driver, 300)
                .await
                .unwrap();
            assert_eq!(
                AS5600AsyncInterface::get_max_angle(&mut driver)
                    .await
                    .unwrap(),
                300
            );
        }

        #[tokio::test]
        async fn test_diagnostics_async() {
            let mock = AS5600Mock::new();
            mock.mock_set_raw_angle(3000);
            let mut driver = AS5600Driver::new(mock);
            let diag = AS5600AsyncInterface::read_all_diagnostics(&mut driver)
                .await
                .unwrap();
            assert_eq!(diag.raw_angle, 3000);

            assert_eq!(
                AS5600AsyncInterface::get_burn_count(&mut driver)
                    .await
                    .unwrap(),
                0
            );
            assert_eq!(
                AS5600AsyncInterface::get_status_raw(&mut driver)
                    .await
                    .unwrap(),
                regs::STATUS_MD_MASK
            );
            assert_eq!(
                AS5600AsyncInterface::get_agc(&mut driver).await.unwrap(),
                100
            );
        }
    }

    #[test]
    fn test_permanent_burn_safety() {
        let mock = AS5600Mock::new();
        let mut driver = AS5600Driver::new(mock.clone());
        let token = BurnToken::confirm_permanent_burn();

        // Test burning settings
        driver.permanent_burn_settings(token).unwrap();
        let log = mock.mock_get_log();
        assert_eq!(log.len(), 2); // 1 Read (ZMCO) + 1 Write (BURN)
        if let MockTransaction::Write(reg, data) = &log[1] {
            assert_eq!(*reg, regs::BURN);
            assert_eq!(data, &vec![regs::BURN_SETTINGS_CMD]);
        } else {
            panic!("Expected Write to BURN register");
        }

        // Test burning config
        driver.permanent_burn_config(token).unwrap();
        let log = mock.mock_get_log();
        assert_eq!(log.len(), 1);
        if let MockTransaction::Write(reg, data) = &log[0] {
            assert_eq!(*reg, regs::BURN);
            assert_eq!(data, &vec![regs::BURN_CONFIG_CMD]);
        }
    }
}
