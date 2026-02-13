# AS5600 Driver (Rust)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/AS5600-Driver.svg)](https://crates.io/crates/AS5600-Driver)

A comprehensive, low-level, platform-agnostic Rust driver for the **AS5600** magnetic rotary encoder (12-bit contactless potentiometer). Built on **`embedded-hal` 1.0**, it provides direct access to all device registers and OTP programming functions.

## 📌 Table of Contents
- [Features](#-key-features)
- [Installation](#-installation)
- [Usage Examples](#-usage-examples)
- [Interface Abstraction](#quick-start-decoupled-interface-traits)
- [I2C Bus Sharing](#-sharing-the-i2c-bus)
- [Safety Warning (OTP)](#️-safety-warning-otp-programming)
- [Support](#support-the-project--підтримати-проект)
- [License](#-license)

## 🚀 Key Features
- **no_std Support**: Ready for bare-metal microcontrollers (ESP32, STM32, nRF, etc.).
- **Full Register Map**: Complete coverage of ZPOS, MPOS, MANG, CONF, STATUS, RAW_ANGLE, ANGLE, AGC, and MAGNITUDE.
- **Hardware Configuration**: Support for Hysteresis, Power Modes, PWM settings, and Fast/Slow Filters.
- **Diagnostics**: Methods to monitor magnet detection, magnetic field strength, and Automatic Gain Control (AGC).
- **OTP Programming**: Secure methods for permanent burning of settings (marked `unsafe`).
- **Mocking Support**: Built-in hardware emulator for testing and simulation (behind the `mock` feature).
- **Trait-based Interface**: `AS5600Interface` trait allows easy swapping between real hardware and mocks.
- **Cross-Platform**: Support for Linux (SBCs like Raspberry Pi), ESP32 (std & no_std), and any other platform implementing `embedded-hal`.

## 📦 Installation
Add this to your `Cargo.toml`:
```toml
[dependencies]
# Basic no_std version
AS5600-Driver = "0.1.0"

# Or with std and anyhow support (recommended for ESP32/Linux)
AS5600-river = { version = "0.1.0", features = ["std", "anyhow"] }
```

### ⚙️ Features
- `std`: Enables standard library support.
- `anyhow`: Enables integration with `anyhow` crate for easier error handling (requires `std`).
- `mock`: Enables the hardware mock emulator (requires `std`).

## 🛠 Usage Examples

All examples provide a real-time monitoring dashboard as shown below:

![AS5600 Dashboard Preview](image.png)
*Typical real-time diagnostic output from the provided examples.*

We provide several ready-to-use examples for different environments:

- **[ESP32 Dashboard (std)](./example/esp-std)**: A real-time terminal dashboard for ESP32 using the `std` library and `esp-idf-hal`.
- **[ESP32 Dashboard (no_std)](./example/esp-no_std)**: Bare-metal implementation for ESP32 using `esp-hal` (no operating system).
- **[Linux Dashboard](./example/linux)**: Using the sensor on Linux-based SBCs (Raspberry Pi, etc.) via `/dev/i2c-x`.
- **[Mock Simulation](./example/mock)**: Hardware-free simulation for testing UI and logic on your PC.

### Quick Start: Decoupled Interface (Traits)
Using `AS5600Interface` allows your application logic to be independent of the specific I2C implementation.

```rust
use AS5600_Driver::AS5600Interface;

// This function works with ANY sensor implementation (Real or Mock)
fn run_diagnostic(encoder: &mut impl AS5600Interface) -> anyhow::Result<()> {
    let raw = encoder.read_raw_angle()?;
    let status = encoder.get_magnet_status()?;
    
    println!("Position: {}, Detected: {}", raw, status.detected);
    Ok(())
}
```

## 🔄 Sharing the I2C Bus
The AS5600 has a **fixed I2C address (0x36)**. 

### Multiple Sensors
To use multiple AS5600 sensors on the same bus, you must use an I2C multiplexer (e.g., TCA9548A).

### Shared Bus with Other Devices
To share the bus with other device types, use a bus manager like `embedded-hal-bus`:

```rust
// Using a reference (&mut) to the I2C bus
let mut bus = I2cDriver::new(...)?;
let mut encoder1 = AS5600Driver::new(&mut bus);
// Other sensors on the same bus must have different addresses
let mut other_sensor = OtherSensor::new(&mut bus, 0x42); 
```

## ⚠️ Safety Warning: OTP Programming
The AS5600 has One-Time Programmable (OTP) memory. These methods perform permanent, irreversible hardware changes:
- `danger_permanent_burn_settings()`: Programs ZPOS and MPOS. Max **3 times**.
- `danger_permanent_burn_config()`: Programs CONF register. **ONLY ONCE**.

## Support the Project / Підтримати проект

If you find this extension useful and want to support development or speed up new features:

**Donate via:**
- 🇺🇦 [Donatello](https://donatello.to/pavver) — Ukrainian service supporting:
  - 💳 Visa/Mastercard
  - 🪙 Cryptocurrency (USDT)
  - 🏦 Other payment methods
- 🌍 PayPal: pavvers1@gmail.com

Your support helps keep this project alive and growing. Thank you! / Дякую за підтримку! 💙💛

---

You can contact me via [telegram](https://t.me/pavver) or pavvers1@gmail.com.

## 📄 License
Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.
