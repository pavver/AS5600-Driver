# AS5600 ESP32 Dashboard Example

This example demonstrates a professional-grade real-time monitoring dashboard for the **AS5600** magnetic rotary encoder on an **ESP32** using the `esp-idf-hal`.

## 📸 Preview
![AS5600 Dashboard](../../image.png)

## ✨ Features
- **High-Precision Progress Bar**: Uses Unicode sub-block characters (`▏` to `█`) for smooth visual feedback of the 12-bit angle.
- **Full Register Monitoring**: Displays real-time data from all internal registers:
    - Raw and Filtered Angle.
    - Magnet Status (Detected, Too Weak, Too Strong).
    - Field Strength (Magnitude) and AGC (Automatic Gain Control) quality.
- **Chip Configuration**: Shows current hardware settings (Power Mode, Hysteresis, PWM, Filters, Watchdog).
- **Memory Status**: Displays OTP (One-Time Programmable) burn counts and operating ranges (ZPOS, MPOS, MANG).

## 🔌 Wiring (ESP32)

| AS5600 Pin | ESP32 Pin | Note |
|------------|-----------|------|
| **VCC**    | 3.3V / 5V | See Power Note below |
| **GND**    | GND       | Common ground |
| **SDA**    | GPIO 21   | I2C Data |
| **SCL**    | GPIO 22   | I2C Clock |

> **⚠️ Power Note**: If using 3.3V, ensure the `VDD3V3` and `VDD5V` pins on the AS5600 module are connected. If using 5V, only connect to `VDD5V`.

## 🚀 How to Run

1.  **Install prerequisites**: Ensure you have the [Rust ESP32 toolchain](https://github.com/esp-rs/espup) installed.
2.  **Flash and Monitor**:
    ```bash
    cargo run
    ```
    *This will compile the project, flash it to your ESP32, and open the monitor to show the dashboard.*

## 🛠 Project Structure
- `src/main.rs`: Contains the main loop and the `render_dashboard` function using the `AS5600Interface` trait.
- `Cargo.toml`: Configured for ESP-IDF v5.x and the local `AS5600-Driver` crate.
