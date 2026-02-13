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
#[derive(Debug, Clone, Copy)]
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
    /// Enable/Disable the watchdog timer (auto-low-power after 1 minute of inactivity).
    pub watchdog: bool,
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
