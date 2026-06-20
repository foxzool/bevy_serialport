# bevy_serialport

[![Crates.io](https://img.shields.io/crates/v/bevy_serialport)](https://crates.io/crates/bevy_serialport)
[![Downloads](https://img.shields.io/crates/d/bevy_serialport)](https://crates.io/crates/bevy_serialport)
[![Documentation](https://docs.rs/bevy_serialport/badge.svg)](https://docs.rs/bevy_serialport)
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/foxzool/bevy_serialport#license)
[![CI](https://github.com/foxzool/bevy_serialport/workflows/CI/badge.svg)](https://github.com/foxzool/bevy_serialport/actions)

Async serial port plugin for the [Bevy](https://bevyengine.org/) game engine, providing seamless integration between serial communication and Bevy's ECS system.

## Features

- ✨ **Async Serial Communication**: Non-blocking serial I/O using Tokio
- 🎯 **Event-Driven Architecture**: Receive serial data through Bevy events
- 🔧 **Comprehensive Configuration**: Full control over serial port settings
- 🛡️ **Enhanced Error Handling**: Detailed error types with context
- 🚀 **High Performance**: Optimized for minimal overhead
- 🎮 **Multiple Port Support**: Manage multiple serial connections simultaneously
- 🔌 **Cross-Platform**: Works on Windows, macOS, and Linux
- 📦 **Utility Functions**: Built-in helpers for port discovery and validation

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
bevy_serialport = "0.12"
bevy = "0.19"
```

## Basic Usage

```rust
use bevy::prelude::*;
use bevy_serialport::{SerialData, SerialPortPlugin, SerialResource, SerialPortRuntime};
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            SerialPortPlugin,
        ))
        .add_systems(Startup, setup_serial)
        .add_systems(Update, handle_serial_data)
        .run();
}

fn setup_serial(
    mut serial_res: ResMut<SerialResource>, 
    rt: Res<SerialPortRuntime>
) {
    // Open a serial port with default settings (115200 baud, 8N1)
    if let Err(e) = serial_res.open(rt.clone(), "COM1", 115_200) {
        error!("Failed to open serial port: {}", e);
    }
}

fn handle_serial_data(
    mut serial_events: MessageReader<SerialData>,
    mut serial_res: ResMut<SerialResource>,
) {
    for event in serial_events.read() {
        info!("Received from {}: {}", event.port, event.as_string_lossy());
        
        // Echo the data back
        let _ = serial_res.send_string(&event.port, "OK\n");
    }
}
```

## Advanced Configuration

```rust
use bevy_serialport::{
    SerialPortSetting, DataBits, FlowControl, Parity, StopBits
};
use std::time::Duration;

fn setup_advanced_serial(
    mut serial_res: ResMut<SerialResource>,
    rt: Res<SerialPortRuntime>
) {
    let settings = SerialPortSetting::new("/dev/ttyUSB0", 9600)
        .with_data_bits(DataBits::Seven)
        .with_parity(Parity::Even)
        .with_stop_bits(StopBits::Two)
        .with_flow_control(FlowControl::Software)
        .with_timeout(Duration::from_millis(100));

    match serial_res.open_with_setting(rt.clone(), settings) {
        Ok(_) => info!("Serial port configured successfully"),
        Err(e) => error!("Configuration failed: {}", e),
    }
}
```

## Port Discovery

```rust
use bevy_serialport::utils::*;

fn discover_ports() {
    match list_available_ports() {
        Ok(ports) => {
            info!("Available ports:");
            for port in ports {
                info!("  - {}", port);
            }
        }
        Err(e) => error!("Failed to list ports: {}", e),
    }
}
```

## Multiple Ports

```rust
fn setup_multiple_ports(
    mut serial_res: ResMut<SerialResource>,
    rt: Res<SerialPortRuntime>
) {
    let ports = vec![
        ("sensor_port", "/dev/ttyUSB0", 9600),
        ("gps_port", "/dev/ttyUSB1", 4800),
        ("debug_port", "COM3", 115200),
    ];

    for (name, port, baud) in ports {
        if let Err(e) = serial_res.open(rt.clone(), port, baud) {
            error!("Failed to open {}: {}", name, e);
        } else {
            info!("Opened {} on {}", name, port);
        }
    }
}

fn handle_multiple_ports(mut serial_events: MessageReader<SerialData>) {
    for event in serial_events.read() {
        match event.port.as_str() {
            "/dev/ttyUSB0" => handle_sensor_data(&event),
            "/dev/ttyUSB1" => handle_gps_data(&event),
            "COM3" => handle_debug_data(&event),
            _ => warn!("Unknown port: {}", event.port),
        }
    }
}
```

## Error Handling

The library provides comprehensive error handling:

```rust
use bevy_serialport::SerialError;

fn robust_serial_setup(
    mut serial_res: ResMut<SerialResource>,
    rt: Res<SerialPortRuntime>
) {
    match serial_res.open(rt.clone(), "COM1", 115200) {
        Ok(_) => info!("Port opened successfully"),
        Err(SerialError::SerialPortError { port, source }) => {
            error!("Hardware error on {}: {}", port, source);
        }
        Err(SerialError::InvalidConfiguration { reason }) => {
            error!("Configuration error: {}", reason);
        }
        Err(SerialError::PortNotFound { port }) => {
            error!("Port {} not found", port);
        }
        Err(e) => error!("Other error: {}", e),
    }
}
```

## SerialData API

The `SerialData` event provides convenient methods for data access:

```rust
fn process_serial_data(mut events: MessageReader<SerialData>) {
    for event in events.read() {
        // Get data as string (lossy conversion)
        let text = event.as_string_lossy();
        
        // Try to get data as valid UTF-8 string
        match event.as_string() {
            Ok(valid_text) => info!("Valid UTF-8: {}", valid_text),
            Err(_) => warn!("Invalid UTF-8 data received"),
        }
        
        // Access raw bytes
        let bytes = event.as_bytes();
        info!("Received {} bytes from {}", bytes.len(), event.port);
        
        // Check if empty
        if event.is_empty() {
            warn!("Empty message received");
        }
    }
}
```

## Examples

The repository includes several examples:

- **Basic Receiver**: `cargo run --example serial_receiver -- --port COM1`
- **Basic Sender**: `cargo run --example serial_sender -- --port COM1`
- **Advanced Usage**: `cargo run --example advanced_usage`

## Supported Platforms

| Platform | Status | Notes |
|----------|--------|-------|
| Windows  | ✅     | COM ports (COM1, COM2, etc.) |
| Linux    | ✅     | USB/UART devices (/dev/ttyUSB0, /dev/ttyACM0, etc.) |
| macOS    | ✅     | USB/UART devices (/dev/cu.*, /dev/tty.*) |

## Bevy Compatibility

| Bevy Version | bevy_serialport Version |
|--------------|-------------------------|
| 0.19         | 0.12                    |
| 0.18         | 0.11                    |
| 0.17         | 0.10                    |
| 0.16         | 0.9                     |
| 0.15         | 0.8                     |
| 0.14         | 0.7                     |
| 0.13         | 0.6                     |
| 0.12         | 0.5                     |
| 0.11         | 0.4                     |
| 0.10         | 0.3                     |
| 0.9          | 0.2                     |
| 0.8          | 0.1                     |

## Common Use Cases

- 🔬 **Scientific Instruments**: Communicate with sensors and measurement devices
- 🤖 **Robotics**: Control motors, read sensors, and communicate with microcontrollers
- 🎮 **Game Controllers**: Interface with custom hardware controllers
- 📡 **IoT Integration**: Connect with embedded devices and sensors
- 🔧 **Development Tools**: Debug interfaces and diagnostic tools

## Performance Considerations

- Uses Tokio's async runtime for non-blocking I/O
- Minimal memory allocations in the hot path
- Efficient message batching for high-throughput scenarios
- Thread-safe design with minimal locking overhead

## Troubleshooting

### Port Not Found
```bash
# Linux: Check available ports
ls /dev/tty*

# Windows: Use Device Manager or PowerShell
Get-WmiObject -Class Win32_SerialPort | Select-Object Name,DeviceID
```

### Permission Denied (Linux)
```bash
# Add user to dialout group
sudo usermod -a -G dialout $USER
# Then logout and login again
```

### Port Already in Use
Ensure no other applications are using the serial port. On Linux, you can check with:
```bash
lsof /dev/ttyUSB0
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is dual-licensed under either:

- [MIT License](LICENSE-MIT) ([http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
- [Apache License 2.0](LICENSE-APACHE) ([http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

At your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Acknowledgments

- Built on top of [tokio-serial](https://github.com/berkowski/tokio-serial) for async serial communication
- Inspired by the Bevy community's commitment to ergonomic game development
- Thanks to all contributors and users who have helped improve this library

---

*For more information, visit the [documentation](https://docs.rs/bevy_serialport) or check out the [repository](https://github.com/foxzool/bevy_serialport).*