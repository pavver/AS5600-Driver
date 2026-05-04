use core::time;
use esp_idf_svc::hal::i2c::*;
use esp_idf_svc::hal::prelude::*;
use std::thread::sleep;

use AS5600_Driver::{AS5600Driver, AS5600Interface};

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    // Initialize I2C0 with pins 21 (SDA) and 22 (SCL)
    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio21, // SDA
        peripherals.pins.gpio22, // SCL
        &I2cConfig::new().baudrate(400.kHz().into()),
    )?;

    let mut encoder = AS5600Driver::new(i2c);

    loop {
        // Clear screen using ANSI escape codes
        print!("{}[2J{}[1;1H", 27 as char, 27 as char);

        match encoder.read_raw_angle() {
            Ok(_) => {
                if let Err(e) = render_dashboard(&mut encoder) {
                    println!("❌ Dashboard Error: {:?}", e);
                }
            }
            Err(_) => {
                println!("╔══════════════════════════════════════════════════════════════╗");
                println!("║                ⚠️  AS5600 DISCONNECTED!  ⚠️                  ║");
                println!("╚══════════════════════════════════════════════════════════════╝");
            }
        };

        sleep(time::Duration::from_millis(500));
    }
}

fn render_dashboard<I>(encoder: &mut I) -> anyhow::Result<()>
where
    I: AS5600Interface,
    I::Error: std::fmt::Debug + Send + Sync + 'static,
{
    // --- OPTION 1: Traditional individual reading ---
    let raw = encoder.read_raw_angle()?;
    let filtered = encoder.read_angle()?;
    let status = encoder.get_magnet_status()?;
    let status_raw = encoder.get_status_raw()?;
    let magnitude = encoder.get_magnitude()?;
    let agc = encoder.get_agc()?;
    
    // --- OPTION 2: Optimized batch reading (Commented out) ---
    /*
    let diag = encoder.read_all_diagnostics()?;
    let raw = diag.raw_angle;
    let filtered = diag.angle;
    let status = diag.magnet_status;
    let status_raw = encoder.get_status_raw()?; // Not in diag struct
    let magnitude = diag.magnitude;
    let agc = diag.agc;
    */

    let burn_count = encoder.get_burn_count()?;
    let conf = encoder.get_config()?;
    
    // Read limits and ranges
    let zpos = encoder.get_zero_position()?;
    let mpos = encoder.get_max_position()?;
    let mang = encoder.get_max_angle()?;

    // High-precision progress bar calculation
    let bar_size = 27;
    let total_fractions = (raw as f32 / 4095.0 * (bar_size as f32 * 8.0)) as usize;
    let full_blocks = total_fractions / 8;
    let fraction = total_fractions % 8;
    
    let sub_blocks = [" ", "▏", "▎", "▍", "▌", "▋", "▊", "▉"];
    let mut bar = "█".repeat(full_blocks);
    if full_blocks < bar_size {
        bar.push_str(sub_blocks[fraction]);
        bar.push_str(&" ".repeat(bar_size - full_blocks - 1));
    }
    
    let percent1 = (raw as f32 / 4095.0 * 100.0) as usize;
    let percent2 = (filtered as f32 / 4095.0 * 100.0) as usize;

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║             🛰️  AS5600 FULL REGISTER MONITOR                 ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    
    // 1. Position Section
    println!("║ 📍 POSITION DATA               ╭───────────────────────────╮ ║");
    println!("║    Raw Angle: {:>4} / 4095 {:>3}% │{:<27}│ ║", raw, percent1, bar);
    println!("║    Filtered:  {:>4} / 4095 {:>3}% ╰───────────────────────────╯ ║", filtered, percent2);

    // 2. Magnet Status Section
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ 🧲 MAGNET DIAGNOSTICS                                        ║");
    
    let det_sym = if status.detected { "✅ YES" } else { "❌ NO " };
    let low_sym = if status.too_weak { "⚠️ LOW " } else { "✅ OK  " };
    let high_sym = if status.too_strong { "⚠️ HIGH" } else { "✅ OK  " };

    println!("║    Detected:       {:<8}Field Status:    0x{:02X} Raw        ║", det_sym, status_raw);
    println!("║    Too Weak:       {:}  Too Strong:      {}         ║", low_sym, high_sym);
    println!("║    Magnitude:      {:<8} AGC Value:       {:<3}             ║", magnitude, agc);

    // 3. Configuration Section (Detailed)
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ ⚙️ CHIP CONFIGURATION (CONF)                                 ║");

    let wd_status = if conf.watchdog { "⚡ ON " } else { "💤 OFF" };
    let pm_str = format!("{:?}", conf.power_mode);
    let hyst_str = format!("{:?}", conf.hysteresis);
    let out_str = format!("{:?}", conf.output_stage);
    let pwm_str = format!("{:?}", conf.pwm_frequency);
    let slow_str = format!("{:?}", conf.slow_filter);
    let fast_str = format!("{:?}", conf.fast_filter_threshold);

    println!("║    Watchdog:       {:<6}  Power Mode:      {:<13}   ║", wd_status, pm_str);
    println!("║    Hysteresis:     {:<8} Output Stage:    {:<13}   ║", hyst_str, out_str);
    println!("║    PWM Frequency:  {:<8} Slow Filter:     {:<13}   ║", pwm_str, slow_str);
    println!("║    Fast Threshold: {:<40}  ║", fast_str);

    // 4. Memory & Ranges Section
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║ 💾 MEMORY & OPERATING RANGES                                 ║");
    println!("║    Burn Cycles:    {}/3      Zero Pos (ZPOS): {:<4}            ║", burn_count, zpos);
    println!("║    Max Pos (MPOS): {:<4}     Max Ang (MANG):  {:<4}            ║", mpos, mang);
    println!("╚══════════════════════════════════════════════════════════════╝");

    Ok(())
}
