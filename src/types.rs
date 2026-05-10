#[cfg(doc)]
use crate::error::AS5600Error;
use crate::regs::regs::*;

/// A token required to perform permanent programming (burning) of the AS5600 chip.
///
/// This is a "Command Token" pattern that prevents accidental execution of irreversible
/// operations. Burning can only be done a limited number of times (ZMCO limit).
///
/// The AS5600 allows burning the zero and maximum positions up to 3 times,
/// while the configuration register can be burned only once.
#[derive(Debug, Clone, Copy)]
pub struct BurnToken {
    _priv: (),
}

impl BurnToken {
    /// Creates a new token, confirming the intent to permanently burn settings.
    ///
    /// # Safety Warning
    /// This operation is **irreversible**. Once burned, the settings cannot be changed
    /// back to the factory state. Ensure your configuration and positions are correct.
    pub fn confirm_permanent_burn() -> Self {
        Self { _priv: () }
    }
}

/// Power consumption modes of the AS5600.
///
/// Lower power modes reduce current consumption by increasing the sampling interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PowerMode {
    /// No power saving, continuous sampling. (Current: ~6.5mA)
    Nominal = 0b00,
    /// Low Power Mode 1 (Sampling: 1ms, Current: ~3.4mA)
    LPM1 = 0b01,
    /// Low Power Mode 2 (Sampling: 10ms, Current: ~1.8mA)
    LPM2 = 0b10,
    /// Low Power Mode 3 (Sampling: 100ms, Current: ~1.5mA)
    LPM3 = 0b11,
}

/// Hysteresis settings to suppress noise in the output.
///
/// Defines the number of Least Significant Bits (LSBs) the position must change
/// before the output is updated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Hysteresis {
    /// No hysteresis.
    Off = 0b00,
    /// 1 LSB hysteresis.
    Lsb1 = 0b01,
    /// 2 LSBs hysteresis.
    Lsb2 = 0b10,
    /// 3 LSBs hysteresis.
    Lsb3 = 0b11,
}

/// Output stage configuration for the OUT pin.
///
/// Determines whether the output pin behaves as an analog ratiometric voltage
/// or as a Pulse Width Modulated (PWM) signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OutputStage {
    /// Ratiometric analog output (0V to VDD).
    AnalogFull = 0b00,
    /// Ratiometric analog output (10% to 90% of VDD).
    AnalogReduced = 0b01,
    /// Pulse Width Modulation (PWM) output.
    PWM = 0b10,
}

/// PWM signal frequency when using PWM output stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PwmFrequency {
    /// 115 Hz PWM frequency.
    Hz115 = 0b00,
    /// 230 Hz PWM frequency.
    Hz230 = 0b01,
    /// 460 Hz PWM frequency.
    Hz460 = 0b10,
    /// 920 Hz PWM frequency.
    Hz920 = 0b11,
}

/// Slow filter settings for noise reduction.
///
/// Higher values mean more averaging and less noise, but higher step response time.
/// Averaging is calculated over N consecutive samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SlowFilter {
    /// 16x averaging.
    X16 = 0b00,
    /// 8x averaging.
    X8 = 0b01,
    /// 4x averaging.
    X4 = 0b10,
    /// 2x averaging.
    X2 = 0b11,
}

/// Fast filter threshold for adaptive filtering.
///
/// If the position change exceeds this threshold, the slow filter is bypassed
/// to provide a fast response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FastFilterThreshold {
    /// Fast filter disabled, only slow filter is used.
    SlowOnly = 0b000,
    /// 6 LSB threshold.
    Lsb6 = 0b001,
    /// 7 LSB threshold.
    Lsb7 = 0b010,
    /// 9 LSB threshold.
    Lsb9 = 0b011,
    /// 18 LSB threshold.
    Lsb18 = 0b100,
    /// 21 LSB threshold.
    Lsb21 = 0b101,
    /// 24 LSB threshold.
    Lsb24 = 0b110,
    /// 10 LSB threshold.
    Lsb10 = 0b111,
}

/// Status of the magnetic system.
///
/// Provides information about magnet detection and field strength.
/// These values are read from the STATUS register.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MagnetStatus {
    /// True if a magnet is detected by the Hall sensors (MD bit).
    pub detected: bool,
    /// True if the magnetic field is too weak (ML bit).
    pub too_weak: bool,
    /// True if the magnetic field is too strong (MH bit).
    pub too_strong: bool,
}

/// Full configuration of the AS5600 chip.
///
/// This struct maps to the CONF_HI and CONF_LO registers.
/// It contains settings for power consumption, filtering, and output behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Configuration {
    /// Current power mode.
    pub power_mode: PowerMode,
    /// Hysteresis setting.
    pub hysteresis: Hysteresis,
    /// Output pin functionality.
    pub output_stage: OutputStage,
    /// Frequency for PWM output.
    pub pwm_frequency: PwmFrequency,
    /// Slow filter averaging factor.
    pub slow_filter: SlowFilter,
    /// Threshold for fast filter bypass.
    pub fast_filter_threshold: FastFilterThreshold,
    /// Enable/Disable the watchdog timer.
    ///
    /// If enabled, the device enters Low Power Mode 3 after 1 minute of inactivity
    /// (no I2C activity or position change).
    pub watchdog: bool,
}

impl Configuration {
    /// Returns a new configuration builder.
    pub fn builder() -> ConfigurationBuilder {
        ConfigurationBuilder::new()
    }

    /// Creates a configuration from raw CONF_HI and CONF_LO register bytes.
    pub fn from_bytes(hi: u8, lo: u8) -> Self {
        Self {
            power_mode: match lo & CONF_PM_MASK {
                0b01 => PowerMode::LPM1,
                0b10 => PowerMode::LPM2,
                0b11 => PowerMode::LPM3,
                _ => PowerMode::Nominal,
            },
            hysteresis: match (lo & CONF_HYST_MASK) >> 2 {
                0b01 => Hysteresis::Lsb1,
                0b10 => Hysteresis::Lsb2,
                0b11 => Hysteresis::Lsb3,
                _ => Hysteresis::Off,
            },
            output_stage: match (lo & CONF_OUTS_MASK) >> 4 {
                0b01 => OutputStage::AnalogReduced,
                0b10 => OutputStage::PWM,
                _ => OutputStage::AnalogFull,
            },
            pwm_frequency: match (lo & CONF_PWMF_MASK) >> 6 {
                0b01 => PwmFrequency::Hz230,
                0b10 => PwmFrequency::Hz460,
                0b11 => PwmFrequency::Hz920,
                _ => PwmFrequency::Hz115,
            },
            slow_filter: match hi & CONF_SF_MASK {
                0b01 => SlowFilter::X8,
                0b10 => SlowFilter::X4,
                0b11 => SlowFilter::X2,
                _ => SlowFilter::X16,
            },
            fast_filter_threshold: match (hi & CONF_FTH_MASK) >> 2 {
                0b001 => FastFilterThreshold::Lsb6,
                0b010 => FastFilterThreshold::Lsb7,
                0b011 => FastFilterThreshold::Lsb9,
                0b100 => FastFilterThreshold::Lsb18,
                0b101 => FastFilterThreshold::Lsb21,
                0b110 => FastFilterThreshold::Lsb24,
                0b111 => FastFilterThreshold::Lsb10,
                _ => FastFilterThreshold::SlowOnly,
            },
            watchdog: (hi & CONF_WD_MASK) != 0,
        }
    }

    /// Converts the configuration into raw (CONF_HI, CONF_LO) register bytes.
    pub fn to_bytes(&self) -> (u8, u8) {
        let hi = ((self.watchdog as u8) << 5)
            | ((self.fast_filter_threshold as u8) << 2)
            | (self.slow_filter as u8);
        let lo = ((self.pwm_frequency as u8) << 6)
            | ((self.output_stage as u8) << 4)
            | ((self.hysteresis as u8) << 2)
            | (self.power_mode as u8);
        (hi, lo)
    }
}

/// A builder for the [`Configuration`] struct that tracks which fields were changed.
///
/// This allows performing partial updates to the sensor configuration, minimizing
/// I2C traffic by only writing to registers that have actually changed.
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ConfigurationBuilder {
    pub(crate) power_mode: Option<PowerMode>,
    pub(crate) hysteresis: Option<Hysteresis>,
    pub(crate) output_stage: Option<OutputStage>,
    pub(crate) pwm_frequency: Option<PwmFrequency>,
    pub(crate) slow_filter: Option<SlowFilter>,
    pub(crate) fast_filter_threshold: Option<FastFilterThreshold>,
    pub(crate) watchdog: Option<bool>,
}

impl ConfigurationBuilder {
    /// Creates a new builder with no fields set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the power consumption mode.
    pub fn power_mode(mut self, mode: PowerMode) -> Self {
        self.power_mode = Some(mode);
        self
    }

    /// Sets the hysteresis level.
    pub fn hysteresis(mut self, hysteresis: Hysteresis) -> Self {
        self.hysteresis = Some(hysteresis);
        self
    }

    /// Sets the output pin functionality.
    pub fn output_stage(mut self, output_stage: OutputStage) -> Self {
        self.output_stage = Some(output_stage);
        self
    }

    /// Sets the PWM signal frequency.
    pub fn pwm_frequency(mut self, frequency: PwmFrequency) -> Self {
        self.pwm_frequency = Some(frequency);
        self
    }

    /// Sets the slow filter averaging factor.
    pub fn slow_filter(mut self, filter: SlowFilter) -> Self {
        self.slow_filter = Some(filter);
        self
    }

    /// Sets the fast filter threshold.
    pub fn fast_filter_threshold(mut self, threshold: FastFilterThreshold) -> Self {
        self.fast_filter_threshold = Some(threshold);
        self
    }

    /// Enables or disables the watchdog timer.
    pub fn watchdog(mut self, enabled: bool) -> Self {
        self.watchdog = Some(enabled);
        self
    }

    /// Checks if any field in the CONF_HI register (WD, FTH, SF) was modified.
    pub fn is_hi_dirty(&self) -> bool {
        self.watchdog.is_some()
            || self.fast_filter_threshold.is_some()
            || self.slow_filter.is_some()
    }

    /// Checks if all fields in the CONF_HI register were modified (allowing direct write).
    pub fn is_hi_complete(&self) -> bool {
        self.watchdog.is_some()
            && self.fast_filter_threshold.is_some()
            && self.slow_filter.is_some()
    }

    /// Checks if any field in the CONF_LO register (PWMF, OUTS, HYST, PM) was modified.
    pub fn is_lo_dirty(&self) -> bool {
        self.pwm_frequency.is_some()
            || self.output_stage.is_some()
            || self.hysteresis.is_some()
            || self.power_mode.is_some()
    }

    /// Checks if all fields in the CONF_LO register were modified (allowing direct write).
    pub fn is_lo_complete(&self) -> bool {
        self.pwm_frequency.is_some()
            && self.output_stage.is_some()
            && self.hysteresis.is_some()
            && self.power_mode.is_some()
    }

    /// Builds a full configuration using default values for unset fields.
    pub fn build(self) -> Configuration {
        let d = Configuration::default();
        Configuration {
            power_mode: self.power_mode.unwrap_or(d.power_mode),
            hysteresis: self.hysteresis.unwrap_or(d.hysteresis),
            output_stage: self.output_stage.unwrap_or(d.output_stage),
            pwm_frequency: self.pwm_frequency.unwrap_or(d.pwm_frequency),
            slow_filter: self.slow_filter.unwrap_or(d.slow_filter),
            fast_filter_threshold: self
                .fast_filter_threshold
                .unwrap_or(d.fast_filter_threshold),
            watchdog: self.watchdog.unwrap_or(d.watchdog),
        }
    }

    /// Internal helper to calculate the byte for the CONF_HI register based on current value.
    pub(crate) fn calculate_hi(&self, current: u8) -> u8 {
        let mut val = current;
        if let Some(wd) = self.watchdog {
            val = (val & !CONF_WD_MASK) | ((wd as u8) << 5);
        }
        if let Some(fth) = self.fast_filter_threshold {
            val = (val & !CONF_FTH_MASK) | ((fth as u8) << 2);
        }
        if let Some(sf) = self.slow_filter {
            val = (val & !CONF_SF_MASK) | (sf as u8);
        }
        val
    }

    /// Internal helper to calculate the byte for the CONF_LO register based on current value.
    pub(crate) fn calculate_lo(&self, current: u8) -> u8 {
        let mut val = current;
        if let Some(pwmf) = self.pwm_frequency {
            val = (val & !CONF_PWMF_MASK) | ((pwmf as u8) << 6);
        }
        if let Some(outs) = self.output_stage {
            val = (val & !CONF_OUTS_MASK) | ((outs as u8) << 4);
        }
        if let Some(hyst) = self.hysteresis {
            val = (val & !CONF_HYST_MASK) | ((hyst as u8) << 2);
        }
        if let Some(pm) = self.power_mode {
            val = (val & !CONF_PM_MASK) | (pm as u8);
        }
        val
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            power_mode: PowerMode::Nominal,
            hysteresis: Hysteresis::Lsb1,
            output_stage: OutputStage::AnalogFull,
            pwm_frequency: PwmFrequency::Hz115,
            slow_filter: SlowFilter::X16,
            fast_filter_threshold: FastFilterThreshold::SlowOnly,
            watchdog: true,
        }
    }
}

/// Information about the current angle and magnet status.
///
/// This structure is returned by the optimized `read_angle_with_status` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AngleWithStatus {
    /// The current 12-bit angle after all filters.
    pub angle: u16,
    /// Current health status of the magnetic system.
    pub status: MagnetStatus,
}

impl AngleWithStatus {
    /// Checks if a magnet is detected.
    ///
    /// # Errors
    /// Returns `Err(AS5600Error::MagnetMissing)` if the sensor does not detect a magnet.
    pub fn check_magnet_detected<E>(self) -> Result<Self, crate::error::AS5600Error<E>> {
        if !self.status.detected {
            return Err(crate::error::AS5600Error::MagnetMissing);
        }
        Ok(self)
    }

    /// Checks if the magnetic field is not too weak.
    ///
    /// # Errors
    /// Returns `Err(AS5600Error::MagnetTooWeak)` if the magnetic field strength is below the threshold.
    pub fn check_magnet_not_too_weak<E>(self) -> Result<Self, crate::error::AS5600Error<E>> {
        if self.status.too_weak {
            return Err(crate::error::AS5600Error::MagnetTooWeak);
        }
        Ok(self)
    }

    /// Checks if the magnetic field is not too strong.
    ///
    /// # Errors
    /// Returns `Err(AS5600Error::MagnetTooStrong)` if the magnetic field strength is above the threshold.
    pub fn check_magnet_not_too_strong<E>(self) -> Result<Self, crate::error::AS5600Error<E>> {
        if self.status.too_strong {
            return Err(crate::error::AS5600Error::MagnetTooStrong);
        }
        Ok(self)
    }

    /// Performs all magnet health checks in one call.
    ///
    /// Verifies that the magnet is detected and that the field strength is within
    /// the recommended range (neither too weak nor too strong).
    ///
    /// # Errors
    /// Returns the first detected error from:
    /// - [`AS5600Error::MagnetMissing`]
    /// - [`AS5600Error::MagnetTooWeak`]
    /// - [`AS5600Error::MagnetTooStrong`]
    pub fn check_magnet_all<E>(self) -> Result<Self, crate::error::AS5600Error<E>> {
        self.check_magnet_detected()?
            .check_magnet_not_too_weak()?
            .check_magnet_not_too_strong()
    }
}

/// A comprehensive snapshot of the sensor status and readings.
///
/// This structure is used for optimized batch reading of all diagnostic
/// information in a single I2C transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Diagnostics {
    /// The current 12-bit angle after all filters.
    pub angle: u16,
    /// The 12-bit raw angle directly from the sensors.
    pub raw_angle: u16,
    /// Current health status of the magnetic system.
    pub magnet_status: MagnetStatus,
    /// Current Automatic Gain Control value (0..255).
    ///
    /// Lower values indicate a stronger magnetic field.
    pub agc: u8,
    /// Current magnitude of the magnetic field (12-bit).
    pub magnitude: u16,
}
