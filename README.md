# bevy_serialport

[![Crates.io](https://img.shields.io/crates/v/bevy_serialport)](https://crates.io/crates/bevy_serialport)
[![Downloads](https://img.shields.io/crates/d/bevy_serialport)](https://crates.io/crates/bevy_serialport)
[![Documentation](https://docs.rs/bevy_serialport/badge.svg)](https://docs.rs/bevy_serialport)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/Seldom-SE/seldom_pixel#license)

`bevy_serialport` is a plugin for add async serial port support for bevy.

## Usage

```rust,no_run
use bevy::prelude::*;
use bevy_app::ScheduleRunnerPlugin;
use bevy_log::{LogPlugin, info, error};
use std::time::Duration;
use bytes::Bytes;

use bevy_serialport::{
    DataBits, FlowControl, Parity, SerialData, SerialPortPlugin, SerialPortRuntime,
    SerialPortSetting, SerialResource, StopBits,
};

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            ))),
            LogPlugin::default(),
            SerialPortPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (receive, send_test_data))
        .run();
}

fn setup(mut serial_res: ResMut<SerialResource>, rt: Res<SerialPortRuntime>) {
    // Using builder pattern for cleaner configuration
    let serial_setting = SerialPortSetting::new("COM1", 115_200)
        .with_data_bits(DataBits::Eight)
        .with_flow_control(FlowControl::None)
        .with_parity(Parity::None)
        .with_stop_bits(StopBits::One);
    
    match serial_res.open_with_setting(rt.clone(), serial_setting) {
        Ok(_) => info!("Successfully opened serial port"),
        Err(e) => error!("Failed to open serial port: {}", e),
    }
}

fn receive(mut serial_ev: EventReader<SerialData>) {
    for message in serial_ev.read() {
        // Enhanced API with convenient string conversion
        info!("Received from {}: {}", message.port, message.as_string_lossy());
    }
}

fn send_test_data(mut serial_res: ResMut<SerialResource>) {
    // Better error handling and convenient string method
    if let Err(e) = serial_res.send_string("COM1", "Hello, Serial!") {
        error!("Failed to send message: {}", e);
    }
}
```

## Supported Versions

| bevy | bevy_serialport |
|------|-----------------|
| 0.16 | 0.9             |
| 0.15 | 0.8             |
| 0.14 | 0.7             |
| 0.13 | 0.6             |
| 0.12 | 0.5             |
| 0.11 | 0.4             |
| 0.10 | 0.3             |
| 0.9  | 0.2             |
| 0.8  | 0.1             |

## License

Dual-licensed under either:

- [`MIT`](LICENSE-MIT): [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT)
- [`Apache 2.0`](LICENSE-APACHE): [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0)

At your option. This means that when using this crate in your game, you may choose which license to use.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dually licensed as above, without any additional terms or conditions.