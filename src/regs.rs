/// Standard I2C address for the AS5600 (fixed by manufacturer).
pub const DEFAULT_ADDR: u8 = 0x36;

/// Register map for the AS5600 according to ams datasheet.
///
/// Registers are mostly 12-bit values spread across two 8-bit registers (HI/LO).
pub mod regs {
    /// Zero setting multi-cycle counter.
    ///
    /// Indicates how many times the `BURN_SETTINGS` command has been executed (max 3).
    pub const ZMCO: u8 = 0x00;
    /// Mask for the ZMCO bits in the ZMCO register.
    pub const ZMCO_MASK: u8 = 0x03;

    /// Start position (ZPOS) - HI register.
    /// Defines the 0 degree point.
    pub const ZPOS_HI: u8 = 0x01;
    /// Start position (ZPOS) - LO register.
    pub const ZPOS_LO: u8 = 0x02;
    /// Mask for the 12-bit ZPOS value.
    pub const ZPOS_MASK: u16 = 0x0FFF;

    /// Stop position (MPOS) - HI register.
    /// Defines the end point of the measuring range.
    pub const MPOS_HI: u8 = 0x03;
    /// Stop position (MPOS) - LO register.
    pub const MPOS_LO: u8 = 0x04;
    /// Mask for the 12-bit MPOS value.
    pub const MPOS_MASK: u16 = 0x0FFF;

    /// Maximum angle (MANG) - HI register.
    /// Defines the full range angle (if ZPOS/MPOS are not set manually).
    pub const MANG_HI: u8 = 0x05;
    /// Maximum angle (MANG) - LO register.
    pub const MANG_LO: u8 = 0x06;
    /// Mask for the 12-bit MANG value.
    pub const MANG_MASK: u16 = 0x0FFF;

    /// Configuration register - HI byte.
    pub const CONF_HI: u8 = 0x07;
    /// Configuration register - LO byte.
    pub const CONF_LO: u8 = 0x08;

    /// Configuration Masks (HI Byte)
    pub const CONF_WD_MASK: u8 = 0x20;
    pub const CONF_FTH_MASK: u8 = 0x1C;
    pub const CONF_SF_MASK: u8 = 0x03;

    /// Configuration Masks (LO Byte)
    pub const CONF_PWMF_MASK: u8 = 0xC0;
    pub const CONF_OUTS_MASK: u8 = 0x30;
    pub const CONF_HYST_MASK: u8 = 0x0C;
    pub const CONF_PM_MASK: u8 = 0x03;

    /// Status register.
    /// Contains magnet detection flags (MH, ML, MD).
    pub const STATUS: u8 = 0x0B;
    /// Magnet detected (MD) bit.
    pub const STATUS_MD_MASK: u8 = 0x20;
    /// Magnet too weak (ML) bit.
    pub const STATUS_ML_MASK: u8 = 0x10;
    /// Magnet too strong (MH) bit.
    pub const STATUS_MH_MASK: u8 = 0x08;

    /// Raw Angle - HI register.
    /// The direct 12-bit value from the Hall sensors.
    pub const RAW_ANGLE_HI: u8 = 0x0C;
    /// Raw Angle - LO register.
    pub const RAW_ANGLE_LO: u8 = 0x0D;
    /// Mask for the 12-bit angle value.
    pub const ANGLE_MASK: u16 = 0x0FFF;

    /// Angle - HI register.
    /// The 12-bit value after applying Zero Position, Maximum Position and filters.
    pub const ANGLE_HI: u8 = 0x0E;
    /// Angle - LO register.
    pub const ANGLE_LO: u8 = 0x0F;

    /// Automatic Gain Control.
    /// Returns 0..255 indicating the magnetic field stability.
    pub const AGC: u8 = 0x1A;

    /// Magnitude - HI register.
    /// Internal representation of the magnetic field strength.
    pub const MAGNITUDE_HI: u8 = 0x1B;
    /// Magnitude - LO register.
    pub const MAGNITUDE_LO: u8 = 0x1C;

    /// Programming register.
    /// Used for `BURN_SETTINGS` (0x80) and `BURN_ANGLE` (0x40).
    pub const BURN: u8 = 0xFF;
    /// Command to burn ZPOS and MPOS permanently.
    pub const BURN_SETTINGS_CMD: u8 = 0x80;
    /// Command to burn Configuration permanently.
    pub const BURN_CONFIG_CMD: u8 = 0x40;

    /// Offset from STATUS (0x0B) to other diagnostic registers.
    pub const RAW_ANGLE_OFFSET: usize = (RAW_ANGLE_HI - STATUS) as usize;
    pub const ANGLE_OFFSET: usize = (ANGLE_HI - STATUS) as usize;
    pub const AGC_OFFSET: usize = (AGC - STATUS) as usize;
    pub const MAGNITUDE_OFFSET: usize = (MAGNITUDE_HI - STATUS) as usize;
}
