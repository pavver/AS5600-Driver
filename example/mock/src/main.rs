use core::time;
use std::thread::sleep;

// Import all necessary items from the driver, including the Mock
use AS5600_Driver::{
    AS5600Driver, AS5600Interface, AS56Mock, 
    Configuration, PowerMode, Hysteresis, OutputStage, 
    PwmFrequency, SlowFilter, FastFilterThreshold, MagnetStatus
};

fn main() -> anyhow::Result<()> {
    // In mock mode, we initialize the virtual device
    let mock_i2c = AS56Mock::new();
    let mut encoder = AS5600Driver::new(mock_i2c.clone());

    println!("Starting simulation mode...");
    sleep(time::Duration::from_secs(1));

    let mut angle: u16 = 0;
    let mut step: usize = 0;

    loop {
        // 1. Clear screen
        print!("{}[2J{}[1;1H", 27 as char, 27 as char);

        // 2. Simulate sensor data changes
        angle = (angle + 120) % 4096;
        mock_i2c.mock_set_raw_angle(angle);
        mock_i2c.mock_set_magnitude(angle / 2 + 100);
        mock_i2c.mock_set_agc((angle / 16) as u8);

        // 3. Simulate configuration changes (to test layout)
        let conf = match step % 4 {
            0 => Configuration {
                power_mode: PowerMode::Nominal,
                hysteresis: Hysteresis::Off,
                output_stage: OutputStage::AnalogFull,
                pwm_frequency: PwmFrequency::Hz115,
                slow_filter: SlowFilter::X16,
                fast_filter_threshold: FastFilterThreshold::SlowOnly,
                watchdog: true,
            },
            1 => Configuration {
                power_mode: PowerMode::LPM1,
                hysteresis: Hysteresis::Lsb3,
                output_stage: OutputStage::AnalogReduced,
                pwm_frequency: PwmFrequency::Hz920,
                slow_filter: SlowFilter::X2,
                fast_filter_threshold: FastFilterThreshold::Lsb24,
                watchdog: false,
            },
            2 => Configuration {
                power_mode: PowerMode::LPM3,
                hysteresis: Hysteresis::Lsb1,
                output_stage: OutputStage::PWM,
                pwm_frequency: PwmFrequency::Hz460,
                slow_filter: SlowFilter::X4,
                fast_filter_threshold: FastFilterThreshold::Lsb10,
                watchdog: true,
            },
            _ => Configuration {
                power_mode: PowerMode::LPM2,
                hysteresis: Hysteresis::Lsb2,
                output_stage: OutputStage::AnalogFull,
                pwm_frequency: PwmFrequency::Hz230,
                slow_filter: SlowFilter::X8,
                fast_filter_threshold: FastFilterThreshold::Lsb6,
                watchdog: false,
            },
        };
        // Update config via driver (this updates registers in the mock)
        encoder.set_config(conf)?;

        // 4. Simulate magnet status changes
        let status = match (step / 2) % 4 {
            0 => MagnetStatus { detected: true, too_weak: false, too_strong: false },
            1 => MagnetStatus { detected: true, too_weak: true, too_strong: false },
            2 => MagnetStatus { detected: true, too_weak: false, too_strong: true },
            _ => MagnetStatus { detected: false, too_weak: false, too_strong: false },
        };
        mock_i2c.mock_set_status(status);

        // 5. Call the dashboard rendering function
        if let Err(e) = render_dashboard(&mut encoder) {
            println!("❌ Dashboard Error: {:?}", e);
        }

        println!("\n[ SIMULATION STEP: {} ]", step);
        println!("Press Ctrl+C to stop.");

        step += 1;
        sleep(time::Duration::from_millis(800));
    }
}

// DASHBOARD RENDERING FUNCTION
fn render_dashboard<I>(encoder: &mut I) -> anyhow::Result<()>
where
    I: AS5600Interface,
    I::Error: std::fmt::Debug + Send + Sync + 'static,
{
    // Reading ALL data
    let raw = encoder.read_raw_angle()?;
    let filtered = encoder.read_angle()?;
    let status = encoder.get_magnet_status()?;
    let status_raw = encoder.get_status_raw()?;
    let magnitude = encoder.get_magnitude()?;
    let agc = encoder.get_agc()?;
    let burn_count = encoder.get_burn_count()?;
    let conf = encoder.get_config()?;
    
    // Limits and Ranges
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
