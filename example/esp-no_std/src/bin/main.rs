#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::main;
use esp_println::{print, println};

use AS5600_Driver::{AS5600Driver, AS5600Interface};

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    // Initialize I2C0 with pins 21 (SDA) and 22 (SCL)
    let i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default(),
    )
    .expect("Failed to initialize I2C")
    .with_sda(peripherals.GPIO21)
    .with_scl(peripherals.GPIO22);

    let mut encoder = AS5600Driver::new(i2c);
    let delay = Delay::new();

    loop {
        // Clear screen using ANSI escape codes
        print!("{}[2J{}[1;1H", 27 as char, 27 as char);

        match encoder.read_raw_angle() {
            Ok(_) => {
                if let Err(_e) = render_dashboard(&mut encoder) {
                    println!("❌ Dashboard Error");
                }
            }
            Err(_) => {
                println!("╔══════════════════════════════════════════════════════════════╗");
                println!("║                ⚠️  AS5600 DISCONNECTED!  ⚠️                  ║");
                println!("╚══════════════════════════════════════════════════════════════╝");
            }
        };

        delay.delay_millis(500);
    }
}

fn render_dashboard(encoder: &mut impl AS5600Interface) -> core::result::Result<(), core::fmt::Error> {
    // --- OPTION 1: Traditional individual reading ---
    let raw = encoder.read_raw_angle().map_err(|_| core::fmt::Error)?;
    let filtered = encoder.read_angle().map_err(|_| core::fmt::Error)?;
    let status = encoder.get_magnet_status().map_err(|_| core::fmt::Error)?;
    let status_raw = encoder.get_status_raw().map_err(|_| core::fmt::Error)?;
    let magnitude = encoder.get_magnitude().map_err(|_| core::fmt::Error)?;
    let agc = encoder.get_agc().map_err(|_| core::fmt::Error)?;
    
    // --- OPTION 2: Optimized batch reading (Commented out) ---
    /*
    let diag = encoder.read_all_diagnostics().map_err(|_| core::fmt::Error)?;
    let raw = diag.raw_angle;
    let filtered = diag.angle;
    let status = diag.magnet_status;
    let status_raw = encoder.get_status_raw().map_err(|_| core::fmt::Error)?; // Not in diag struct
    let magnitude = diag.magnitude;
    let agc = diag.agc;
    */

    let burn_count = encoder.get_burn_count().map_err(|_| core::fmt::Error)?;
    let conf = encoder.get_config().map_err(|_| core::fmt::Error)?;
    
    // Read limits and ranges
    let zpos = encoder.get_zero_position().map_err(|_| core::fmt::Error)?;
    let mpos = encoder.get_max_position().map_err(|_| core::fmt::Error)?;
    let mang = encoder.get_max_angle().map_err(|_| core::fmt::Error)?;

    // High-precision progress bar calculation
    let bar_size = 27;
    let total_fractions = (raw as f32 / 4095.0 * (bar_size as f32 * 8.0)) as usize;
    let full_blocks = total_fractions / 8;
    let fraction = total_fractions % 8;
    
    let sub_blocks = [" ", "▏", "▎", "▍", "▌", "▋", "▊", "▉"];
    
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║             🛰️  AS5600 FULL REGISTER MONITOR                 ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    
    // 1. Position Section
    println!("║ 📍 POSITION DATA               ╭───────────────────────────╮ ║");
    print!("║    Raw Angle: {:>4} / 4095 {:>3}% │", raw, (raw as f32 / 4095.0 * 100.0) as usize);
    
    for _ in 0..full_blocks {
        print!("█");
    }
    if full_blocks < bar_size {
        print!("{}", sub_blocks[fraction]);
        for _ in 0..(bar_size - full_blocks - 1) {
            print!(" ");
        }
    }
    println!("│ ║");
    println!("║    Filtered:  {:>4} / 4095 {:>3}% ╰───────────────────────────╯ ║", filtered, (filtered as f32 / 4095.0 * 100.0) as usize);

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
    
    // Перетворюємо в String для коректного вирівнювання
    let pm_str = alloc::format!("{:?}", conf.power_mode);
    let hyst_str = alloc::format!("{:?}", conf.hysteresis);
    let out_str = alloc::format!("{:?}", conf.output_stage);
    let pwm_str = alloc::format!("{:?}", conf.pwm_frequency);
    let slow_str = alloc::format!("{:?}", conf.slow_filter);
    let fast_str = alloc::format!("{:?}", conf.fast_filter_threshold);

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
