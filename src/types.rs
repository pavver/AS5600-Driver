use crate::regs::regs::*;

/// Power consumption modes of the AS5600.
///
/// Lower power modes reduce current consumption by increasing the sampling interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerMode {
    /// No power saving, continuous sampling. (Current: ~6.5mA)
    Nominal = 0b00,
    /// Low Power Mode 1 (Sampling: 1ms)
    LPM1 = 0b01,
    /// Low Power Mode 2 (Sampling: 10ms)
    LPM2 = 0b10,
    /// Low Power Mode 3 (Sampling: 100ms)
    LPM3 = 0b11,
}

/// Hysteresis settings to suppress noise in the output.
///
/// Defines the number of LSBs the position must change before the output is updated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MagnetStatus {
    /// True if a magnet is detected by the Hall sensors.
    pub detected: bool,
    /// True if the magnetic field is too weak (magnet too far).
    pub too_weak: bool,
    /// True if the magnetic field is too strong (magnet too close).
    pub too_strong: bool,
}

/// Full configuration of the AS5600 chip.
///
/// This struct maps to the CONF_HI and CONF_LO registers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Configuration {
    pub power_mode: PowerMode,
    pub hysteresis: Hysteresis,
    pub output_stage: OutputStage,
    pub pwm_frequency: PwmFrequency,
    pub slow_filter: SlowFilter,
    pub fast_filter_threshold: FastFilterThreshold,
    pub watchdog: bool,
}

impl Configuration {
    pub fn builder() -> ConfigurationBuilder {
        ConfigurationBuilder::new()
    }

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
#[derive(Debug, Clone, Copy, Default)]
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn power_mode(mut self, mode: PowerMode) -> Self {
        self.power_mode = Some(mode);
        self
    }

    pub fn hysteresis(mut self, hysteresis: Hysteresis) -> Self {
        self.hysteresis = Some(hysteresis);
        self
    }

    pub fn output_stage(mut self, output_stage: OutputStage) -> Self {
        self.output_stage = Some(output_stage);
        self
    }

    pub fn pwm_frequency(mut self, frequency: PwmFrequency) -> Self {
        self.pwm_frequency = Some(frequency);
        self
    }

    pub fn slow_filter(mut self, filter: SlowFilter) -> Self {
        self.slow_filter = Some(filter);
        self
    }

    pub fn fast_filter_threshold(mut self, threshold: FastFilterThreshold) -> Self {
        self.fast_filter_threshold = Some(threshold);
        self
    }

    pub fn watchdog(mut self, enabled: bool) -> Self {
        self.watchdog = Some(enabled);
        self
    }

    /// Checks if any field in the CONF_HI register (WD, FTH, SF) was modified.
    pub fn is_hi_dirty(&self) -> bool {
        self.watchdog.is_some() || self.fast_filter_threshold.is_some() || self.slow_filter.is_some()
    }

    /// Checks if all fields in the CONF_HI register were modified (allowing direct write).
    pub fn is_hi_complete(&self) -> bool {
        self.watchdog.is_some() && self.fast_filter_threshold.is_some() && self.slow_filter.is_some()
    }

    /// Checks if any field in the CONF_LO register (PWMF, OUTS, HYST, PM) was modified.
    pub fn is_lo_dirty(&self) -> bool {
        self.pwm_frequency.is_some() || self.output_stage.is_some() || self.hysteresis.is_some() || self.power_mode.is_some()
    }

    /// Checks if all fields in the CONF_LO register were modified (allowing direct write).
    pub fn is_lo_complete(&self) -> bool {
        self.pwm_frequency.is_some() && self.output_stage.is_some() && self.hysteresis.is_some() && self.power_mode.is_some()
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
            fast_filter_threshold: self.fast_filter_threshold.unwrap_or(d.fast_filter_threshold),
            watchdog: self.watchdog.unwrap_or(d.watchdog),
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Diagnostics {
    pub angle: u16,
    pub raw_angle: u16,
    pub magnet_status: MagnetStatus,
    pub agc: u8,
    pub magnitude: u16,
}
