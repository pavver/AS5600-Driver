# AS5600 Mock Simulation Example

This example demonstrates how to use the **AS5600 Mock** to simulate a real magnetic sensor in a pure Rust environment (Windows, Linux, macOS). 

It is ideal for:
- Developing and testing UI/Dashboards without physical hardware.
- Stress-testing logic with various sensor states and error conditions.
- Running CI/CD tests for your peripheral logic.

## 📸 Preview
![AS5600 Dashboard](../../image.png)

## ✨ Features
- **Hardware-free Development**: Runs on any platform supported by Rust.
- **Dynamic Simulation**: Automatically cycles through all possible configuration states and magnet positions.
- **Shared State**: Uses `Arc<Mutex<...>>` to allow thread-safe manipulation of virtual registers.

## 🛠 Project Structure
- `src/main.rs`: A simulation loop that updates a `AS56Mock` and renders the dashboard.
- `Cargo.toml`: Enables the `mock` feature of the `AS5600-Driver` crate.

## 💡 Key Concept: The Simulation Controller
The `AS56Mock` provides special "backdoor" methods starting with `mock_set_*`. These are **only** available when the `mock` feature is enabled and allow you to simulate hardware events that would normally be read-only:

```rust
let mock_i2c = AS56Mock::new();
mock_i2c.mock_set_raw_angle(2048); // Force a position
mock_i2c.mock_set_status(MagnetStatus { detected: false, ... }); // Simulate magnet loss
```
