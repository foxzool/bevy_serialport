# bevy_serialport

[![Crates.io](https://img.shields.io/crates/v/bevy_serialport)](https://crates.io/crates/bevy_serialport)
[![Downloads](https://img.shields.io/crates/d/bevy_serialport)](https://crates.io/crates/bevy_serialport)
[![Documentation](https://docs.rs/bevy_serialport/badge.svg)](https://docs.rs/bevy_serialport)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/Seldom-SE/seldom_pixel#license)

`bevy_serialport` is a plugin for add async serial port support for bevy.

## Usage

``` no_run
use bevy::{app::ScheduleRunnerPlugin, log::LogPlugin};
use std::time::Duration;

use bevy::prelude::*;
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
    let serial_setting = SerialPortSetting {
        port_name: "COM1".to_string(),
        baud_rate: 115_200,
        data_bits: DataBits::Five,
        flow_control: FlowControl::None,
        parity: Parity::None,
        stop_bits: StopBits::One,
        timeout: Default::default(),
    };
    serial_res
        .open_with_setting(rt.clone(), serial_setting)
        .expect("open serial port error");
}

fn receive(mut serial_ev: EventReader<SerialData>) {
    for message in serial_ev.read() {
        info!("receive {:?}", message);
    }
}

fn send_test_data(mut serial_res: ResMut<SerialResource>) {
    serial_res.send_message("COM1", Bytes::from(&b"123457"[..]))
}

```

## Supported Versions

| bevy | bevy_serialport |
|------|-----------------|
| 0.14 | 0.7             |
| 0.13 | 0.6             |
| 0.12 | 0.5             |
| 0.11 | 0.4             |
| 0.10 | 0.3             |
| 0.9  | 0.2             |
| 0.8  | 0.1             |

## License

Dual-licensed under either

- MIT
- Apache 2.0