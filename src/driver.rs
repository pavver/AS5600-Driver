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
    #[inline]
    pub fn new(i2c: I2C) -> Self {
        Self {
            i2c,
            address: DEFAULT_ADDR,
        }
    }

    /// Creates a new driver instance with a custom I2C address.
    #[inline]
    pub fn with_address(i2c: I2C, address: u8) -> Self {
        Self { i2c, address }
    }
}

/// Internal macro to handle optional await.
macro_rules! maybe_await {
    (async, $e:expr) => {
        $e.await
    };
    (sync, $e:expr) => {
        $e
    };
}

/// Internal macro to define either a sync or async function.
macro_rules! define_method {
    (async, $(#[$attr:meta])* $name:ident($($args:tt)*) -> $ret:ty { $($body:tt)* }) => {
        $(#[$attr])*
        #[inline]
        async fn $name($($args)*) -> $ret { $($body)* }
    };
    (sync, $(#[$attr:meta])* $name:ident($($args:tt)*) -> $ret:ty { $($body:tt)* }) => {
        $(#[$attr])*
        #[inline]
        fn $name($($args)*) -> $ret { $($body)* }
    };
}

/// Unified logic for all AS5600 methods.
macro_rules! define_as5600_logic {
    ($mode:ident, $u8:ident, $u16:ident, $u16_reg:ident, $write_u16:ident) => {
        define_method!($mode,
            /// Reads the raw angle from the sensor (12-bit).
            read_raw_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
                maybe_await!($mode, self.$u16())
            }
        );

        define_method!($mode,
            /// Reads the angle from the sensor (12-bit).
            read_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
                maybe_await!($mode, self.$u16_reg(regs::ANGLE_HI))
            }
        );

        define_method!($mode,
            /// Gets the current burn count (ZMCO).
            get_burn_count(&mut self) -> Result<u8, AS5600Error<Self::Error>> {
                let val = maybe_await!($mode, self.$u8(regs::ZMCO))?;
                Ok(val & regs::ZMCO_MASK)
            }
        );

        define_method!($mode,
            /// Reads the raw status register byte.
            get_status_raw(&mut self) -> Result<u8, AS5600Error<Self::Error>> {
                maybe_await!($mode, self.$u8(regs::STATUS))
            }
        );

        define_method!($mode,
            /// Gets the magnet status (detected, weak, strong).
            get_magnet_status(&mut self) -> Result<MagnetStatus, AS5600Error<Self::Error>> {
                let val = maybe_await!($mode, self.$u8(regs::STATUS))?;
                Ok(MagnetStatus {
                    detected: (val & regs::STATUS_MD_MASK) != 0,
                    too_weak: (val & regs::STATUS_ML_MASK) != 0,
                    too_strong: (val & regs::STATUS_MH_MASK) != 0,
                })
            }
        );

        define_method!($mode,
            /// Gets the current magnitude of the magnetic field (12-bit).
            get_magnitude(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
                maybe_await!($mode, self.$u16_reg(regs::MAGNITUDE_HI))
            }
        );

        define_method!($mode,
            /// Gets the current AGC value.
            get_agc(&mut self) -> Result<u8, AS5600Error<Self::Error>> {
                maybe_await!($mode, self.$u8(regs::AGC))
            }
        );

        define_method!($mode,
            /// Reads the full configuration from the sensor.
            get_config(&mut self) -> Result<Configuration, AS5600Error<Self::Error>> {
                let mut buf = [0u8; 2];
                maybe_await!($mode, self.i2c.write_read(self.address, &[regs::CONF_HI], &mut buf))?;
                Ok(Configuration::from_bytes(buf[0], buf[1]))
            }
        );

        define_method!($mode,
            /// Writes a complete configuration to the sensor.
            set_config(&mut self, config: Configuration) -> Result<(), AS5600Error<Self::Error>> {
                let (hi, lo) = config.to_bytes();
                maybe_await!($mode, self.i2c.write(self.address, &[regs::CONF_HI, hi, lo]))?;
                Ok(())
            }
        );

        define_method!($mode,
            /// Applies configuration changes using a builder pattern.
            apply_config(&mut self, builder: ConfigurationBuilder) -> Result<(), AS5600Error<Self::Error>> {
                if builder.is_hi_dirty() {
                    let hi_val = if builder.is_hi_complete() {
                        builder.calculate_hi(0)
                    } else {
                        builder.calculate_hi(maybe_await!($mode, self.$u8(regs::CONF_HI))?)
                    };
                    maybe_await!($mode, self.i2c.write(self.address, &[regs::CONF_HI, hi_val]))?;
                }
                if builder.is_lo_dirty() {
                    let lo_val = if builder.is_lo_complete() {
                        builder.calculate_lo(0)
                    } else {
                        builder.calculate_lo(maybe_await!($mode, self.$u8(regs::CONF_LO))?)
                    };
                    maybe_await!($mode, self.i2c.write(self.address, &[regs::CONF_LO, lo_val]))?;
                }
                Ok(())
            }
        );

        define_method!($mode,
            /// Gets the programmed zero position (ZPOS).
            get_zero_position(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
                maybe_await!($mode, self.$u16_reg(regs::ZPOS_HI))
            }
        );

        define_method!($mode,
            /// Sets the zero position (ZPOS).
            set_zero_position(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>> {
                maybe_await!($mode, self.$write_u16(regs::ZPOS_HI, angle))
            }
        );

        define_method!($mode,
            /// Gets the programmed max position (MPOS).
            get_max_position(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
                maybe_await!($mode, self.$u16_reg(regs::MPOS_HI))
            }
        );

        define_method!($mode,
            /// Sets the max position (MPOS).
            set_max_position(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>> {
                maybe_await!($mode, self.$write_u16(regs::MPOS_HI, angle))
            }
        );

        define_method!($mode,
            /// Gets the programmed max angle (MANG).
            get_max_angle(&mut self) -> Result<u16, AS5600Error<Self::Error>> {
                maybe_await!($mode, self.$u16_reg(regs::MANG_HI))
            }
        );

        define_method!($mode,
            /// Sets the max angle (MANG).
            set_max_angle(&mut self, angle: u16) -> Result<(), AS5600Error<Self::Error>> {
                maybe_await!($mode, self.$write_u16(regs::MANG_HI, angle))
            }
        );

        define_method!($mode,
            /// Checks if the sensor is connected by attempting to read ZMCO.
            is_connected(&mut self) -> bool {
                maybe_await!($mode, self.$u8(regs::ZMCO)).is_ok()
            }
        );

        define_method!($mode,
            /// Reads both the 12-bit angle and magnet status in one optimized I2C transaction.
            ///
            /// This is the most efficient way to read the angle while simultaneously
            /// verifying that the magnet is present and within range.
            ///
            /// # Returns
            /// An [`AngleWithStatus`] structure containing the filtered angle and magnet health.
            read_angle_with_status(&mut self) -> Result<AngleWithStatus, AS5600Error<Self::Error>> {
                let mut buf = [0u8; 5];
                maybe_await!($mode, self.i2c.write_read(self.address, &[regs::STATUS], &mut buf))?;
                let status_val = buf[0];
                let angle = u16::from_be_bytes([buf[3], buf[4]]) & regs::ANGLE_MASK;
                Ok(AngleWithStatus {
                    angle,
                    status: MagnetStatus {
                        detected: (status_val & regs::STATUS_MD_MASK) != 0,
                        too_weak: (status_val & regs::STATUS_ML_MASK) != 0,
                        too_strong: (status_val & regs::STATUS_MH_MASK) != 0,
                    },
                })
            }
        );

        define_method!($mode,
            /// Reads all diagnostic and position data in a single I2C transaction.
            ///
            /// Fetches status, raw angle, filtered angle, AGC, and magnitude
            /// (18 bytes total) from the sensor.
            read_all_diagnostics(&mut self) -> Result<Diagnostics, AS5600Error<Self::Error>> {
                let mut buf = [0u8; 18];
                maybe_await!($mode, self.i2c.write_read(self.address, &[regs::STATUS], &mut buf))?;
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
        );

        define_method!($mode,
            /// Permanently burns ZPOS and MPOS settings to the chip.
            ///
            /// # Errors
            /// - Returns [`AS5600Error::OtpMaxBurnsReached`] if the burn count (ZMCO) is already 3.
            /// - Returns [`AS5600Error::MagnetMissing`] if no magnet is detected (required by hardware).
            /// - Returns [`AS5600Error::AngularTravelTooSmall`] if the range between ZPOS and MPOS is < 18°.
            permanent_burn_settings(
                &mut self,
                _token: BurnToken,
            ) -> Result<(), AS5600Error<Self::Error>> {
                let count = maybe_await!($mode, self.$u8(regs::ZMCO))? & regs::ZMCO_MASK;
                if count >= 3 {
                    return Err(AS5600Error::OtpMaxBurnsReached);
                }

                // According to datasheet, MD bit must be 1 for Burn_Angle to work.
                let status = maybe_await!($mode, self.$u8(regs::STATUS))?;
                if (status & regs::STATUS_MD_MASK) == 0 {
                    return Err(AS5600Error::MagnetMissing);
                }

                // Minimum angular travel check (18 degrees ~ 205 counts)
                let mpos = maybe_await!($mode, self.$u16_reg(regs::MPOS_HI))?;
                if mpos != 0 {
                    let zpos = maybe_await!($mode, self.$u16_reg(regs::ZPOS_HI))?;
                    let diff = if mpos >= zpos {
                        mpos - zpos
                    } else {
                        (4096 - zpos) + mpos
                    };

                    if diff < 205 {
                        return Err(AS5600Error::AngularTravelTooSmall);
                    }
                } else {
                    // If MPOS is 0, MANG is used. Check current MANG.
                    let mang = maybe_await!($mode, self.$u16_reg(regs::MANG_HI))?;
                    if mang > 0 && mang < 205 {
                        return Err(AS5600Error::AngularTravelTooSmall);
                    }
                }

                maybe_await!($mode, self.i2c.write(self.address, &[regs::BURN, regs::BURN_SETTINGS_CMD]))?;
                Ok(())
            }
        );

        define_method!($mode,
            /// Permanently burns Configuration settings to the chip.
            ///
            /// # Errors
            /// - Returns [`AS5600Error::OtpMaxBurnsReached`] if ZMCO is not 0.
            /// - Returns [`AS5600Error::AngularTravelTooSmall`] if MANG is set to < 18°.
            ///
            /// # Safety Warning
            /// According to the datasheet, this command is ONLY executed if ZMCO = 00.
            permanent_burn_config(
                &mut self,
                _token: BurnToken,
            ) -> Result<(), AS5600Error<Self::Error>> {
                let count = maybe_await!($mode, self.$u8(regs::ZMCO))? & regs::ZMCO_MASK;
                if count != 0 {
                    return Err(AS5600Error::OtpMaxBurnsReached);
                }

                // MANG (Maximum Angle) must also be at least 18 degrees.
                // Default MANG is 0, which means 360 degrees (OK).
                let mang = maybe_await!($mode, self.$u16_reg(regs::MANG_HI))?;
                if mang > 0 && mang < 205 {
                    return Err(AS5600Error::AngularTravelTooSmall);
                }

                maybe_await!($mode, self.i2c.write(self.address, &[regs::BURN, regs::BURN_CONFIG_CMD]))?;
                Ok(())
            }
        );
    };
}

/// Internal helpers definition logic.
macro_rules! define_internal_helpers_logic {
    ($mode:ident, $u8:ident, $u16:ident, $u16_reg:ident, $write_u16:ident) => {
        define_method!($mode,
            /// Internal helper to read a single byte from a register.
            $u8(&mut self, reg: u8) -> Result<u8, AS5600Error<I2C::Error>> {
                let mut buf = [0u8; 1];
                maybe_await!($mode, self.i2c.write_read(self.address, &[reg], &mut buf))?;
                Ok(buf[0])
            }
        );
        define_method!($mode,
            /// Internal helper to read a 12-bit value from RAW_ANGLE_HI.
            $u16(&mut self) -> Result<u16, AS5600Error<I2C::Error>> {
                let mut buf = [0u8; 2];
                maybe_await!($mode, self.i2c.write_read(self.address, &[regs::RAW_ANGLE_HI], &mut buf))?;
                Ok(u16::from_be_bytes(buf) & regs::ANGLE_MASK)
            }
        );
        define_method!($mode,
            /// Internal helper to read a 12-bit value from a custom register.
            $u16_reg(&mut self, reg: u8) -> Result<u16, AS5600Error<I2C::Error>> {
                let mut buf = [0u8; 2];
                maybe_await!($mode, self.i2c.write_read(self.address, &[reg], &mut buf))?;
                Ok(u16::from_be_bytes(buf) & regs::ANGLE_MASK)
            }
        );
        define_method!($mode,
            /// Internal helper to write a 12-bit value to two consecutive registers.
            $write_u16(&mut self, reg_hi: u8, value: u16) -> Result<(), AS5600Error<I2C::Error>> {
                if value > regs::ANGLE_MASK {
                    return Err(AS5600Error::InvalidParameter);
                }
                let bytes = value.to_be_bytes();
                maybe_await!($mode, self.i2c.write(self.address, &[reg_hi, bytes[0], bytes[1]]))?;
                Ok(())
            }
        );
    };
}

// Generate Synchronous implementation
impl<I2C: i2c::I2c<SevenBitAddress>> AS5600Interface for AS5600Driver<I2C> {
    type Error = I2C::Error;
    define_as5600_logic!(
        sync,
        read_u8_internal,
        read_u16_internal,
        read_u16_internal_reg,
        write_u16_internal
    );
}

impl<I2C: i2c::I2c<SevenBitAddress>> AS5600Driver<I2C> {
    define_internal_helpers_logic!(
        sync,
        read_u8_internal,
        read_u16_internal,
        read_u16_internal_reg,
        write_u16_internal
    );
}

// Generate Asynchronous implementation
#[cfg(feature = "async")]
use embedded_hal_async::i2c as async_i2c;

#[cfg(feature = "async")]
impl<I2C: async_i2c::I2c<SevenBitAddress>> AS5600AsyncInterface for AS5600Driver<I2C> {
    type Error = I2C::Error;
    define_as5600_logic!(
        async,
        read_u8_internal_async,
        read_u16_internal_async,
        read_u16_internal_reg_async,
        write_u16_internal_async
    );
}

#[cfg(feature = "async")]
impl<I2C: async_i2c::I2c<SevenBitAddress>> AS5600Driver<I2C> {
    define_internal_helpers_logic!(
        async,
        read_u8_internal_async,
        read_u16_internal_async,
        read_u16_internal_reg_async,
        write_u16_internal_async
    );
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

    #[test]
    fn test_read_angle_with_status() {
        let mock = AS5600Mock::new();
        mock.mock_set_angle(2048);
        let status = MagnetStatus {
            detected: true,
            too_weak: true,
            too_strong: false,
        };
        mock.mock_set_status(status);

        let mut driver = AS5600Driver::new(mock);
        let result = AS5600Interface::read_angle_with_status(&mut driver).unwrap();

        assert_eq!(result.angle, 2048);
        assert_eq!(result.status, status);

        // Test fluent API checks
        assert!(matches!(result.check_magnet_detected::<()>(), Ok(_)));
        assert!(matches!(
            result.check_magnet_not_too_weak::<()>(),
            Err(AS5600Error::MagnetTooWeak)
        ));
        assert!(matches!(
            result.check_magnet_all::<()>(),
            Err(AS5600Error::MagnetTooWeak)
        ));
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

        #[tokio::test]
        async fn test_permanent_burn_safety_async() {
            let mock = AS5600Mock::new();
            // Set healthy range for tests
            mock.mock_set_zpos(0);
            mock.mock_set_mpos(2000);
            mock.mock_set_mang(0);

            let mut driver = AS5600Driver::new(mock.clone());
            let token = BurnToken::confirm_permanent_burn();

            AS5600AsyncInterface::permanent_burn_settings(&mut driver, token)
                .await
                .unwrap();
            let log = mock.mock_get_log();
            // 1 Read (ZMCO) + 1 Read (STATUS) + 1 Read (ZPOS) + 1 Read (MPOS) + 1 Write (BURN) = 5 operations
            assert_eq!(log.len(), 5);

            AS5600AsyncInterface::permanent_burn_config(&mut driver, token)
                .await
                .unwrap();
            // 1 Read (ZMCO) + 1 Read (MANG) + 1 Write (BURN) = 3 operations
            assert_eq!(mock.mock_get_log().len(), 3);
        }

        #[tokio::test]
        async fn test_permanent_burn_no_magnet_async() {
            let mock = AS5600Mock::new();
            mock.mock_set_status(MagnetStatus {
                detected: false,
                too_weak: false,
                too_strong: false,
            });
            let mut driver = AS5600Driver::new(mock);
            let token = BurnToken::confirm_permanent_burn();

            let result = AS5600AsyncInterface::permanent_burn_settings(&mut driver, token).await;
            assert!(matches!(result, Err(AS5600Error::MagnetMissing)));
        }

        #[tokio::test]
        async fn test_permanent_burn_angle_too_small_async() {
            let mock = AS5600Mock::new();
            let mut driver = AS5600Driver::new(mock.clone());
            let token = BurnToken::confirm_permanent_burn();

            // 100 counts is approx 8.7 degrees (< 18)
            mock.mock_set_zpos(0);
            mock.mock_set_mpos(100);

            let result = AS5600AsyncInterface::permanent_burn_settings(&mut driver, token).await;
            assert!(matches!(result, Err(AS5600Error::AngularTravelTooSmall)));
        }
        #[tokio::test]
        async fn test_permanent_burn_angle_wrap_around_valid_async() {
            let mock = AS5600Mock::new();
            let mut driver = AS5600Driver::new(mock.clone());
            let token = BurnToken::confirm_permanent_burn();

            // From 4000 to 100: (4096-4000) + 100 = 196
            // 196 is still < 205 (~17.2°), so it should fail.
            mock.mock_set_zpos(4000);
            mock.mock_set_mpos(100);
            let result = AS5600AsyncInterface::permanent_burn_settings(&mut driver, token).await;
            assert!(matches!(result, Err(AS5600Error::AngularTravelTooSmall)));

            // From 4000 to 200: (4096-4000) + 200 = 296
            // 296 is > 205, so it should pass.
            mock.mock_set_mpos(200);
            let result = AS5600AsyncInterface::permanent_burn_settings(&mut driver, token).await;
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_permanent_burn_safety() {
        let mock = AS5600Mock::new();
        // Set healthy range for tests
        mock.mock_set_zpos(0);
        mock.mock_set_mpos(2000);
        mock.mock_set_mang(0);

        let mut driver = AS5600Driver::new(mock.clone());
        let token = BurnToken::confirm_permanent_burn();

        // Test burning settings
        AS5600Interface::permanent_burn_settings(&mut driver, token).unwrap();
        let log = mock.mock_get_log();
        // 1 Read (ZMCO) + 1 Read (STATUS) + 1 Read (ZPOS) + 1 Read (MPOS) + 1 Write (BURN) = 5
        assert_eq!(log.len(), 5);
        if let MockTransaction::Write(reg, data) = &log[4] {
            assert_eq!(*reg, regs::BURN);
            assert_eq!(data, &vec![regs::BURN_SETTINGS_CMD]);
        } else {
            panic!("Expected Write to BURN register");
        }

        // Test burning config
        AS5600Interface::permanent_burn_config(&mut driver, token).unwrap();
        let log = mock.mock_get_log();
        // 1 Read (ZMCO) + 1 Read (MANG) + 1 Write (BURN) = 3
        assert_eq!(log.len(), 3);
        if let MockTransaction::Write(reg, data) = &log[2] {
            assert_eq!(*reg, regs::BURN);
            assert_eq!(data, &vec![regs::BURN_CONFIG_CMD]);
        }
    }

    #[test]
    fn test_permanent_burn_config_forbidden_if_zmco_not_zero() {
        let mock = AS5600Mock::new();
        let mut driver = AS5600Driver::new(mock.clone());
        let token = BurnToken::confirm_permanent_burn();

        // Simulate that settings were already burned once (ZMCO = 1)
        mock.mock_set_zmco(1);

        // Attempt to burn config should fail
        let result = AS5600Interface::permanent_burn_config(&mut driver, token);
        assert!(matches!(result, Err(AS5600Error::OtpMaxBurnsReached)));
    }

    #[test]
    fn test_permanent_burn_mang_too_small() {
        let mock = AS5600Mock::new();
        let mut driver = AS5600Driver::new(mock.clone());
        let token = BurnToken::confirm_permanent_burn();

        // 50 counts is approx 4.4 degrees (< 18)
        mock.mock_set_mang(50);

        let result = AS5600Interface::permanent_burn_config(&mut driver, token);
        assert!(matches!(result, Err(AS5600Error::AngularTravelTooSmall)));
    }
}
