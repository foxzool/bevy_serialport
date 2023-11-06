[![crates.io](https://img.shields.io/crates/v/bevy_serialport)](https://crates.io/crates/bevy_serialport)

# bevy_serialport
`bevy_serialport` is a plugin for add async serial port support for bevy.

## Usage

``` rust
use std::time::Duration;

use bevy::{app::ScheduleRunnerSettings, log::LogPlugin, prelude::*};
use bytes::Bytes;

use bevy_serialport::{
    DataBits, FlowControl, Parity, Runtime, SerialData, SerialPortPlugin, SerialPortSetting,
    SerialResource, StopBits,
};


fn main() {
    App::new()
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_millis(10)))
        .add_plugin(LogPlugin)
        .add_plugins(MinimalPlugins)
        .add_plugin(SerialPortPlugin)
        .add_startup_system(setup)
        .add_system(receive)
        .add_system(send_test_data)
        .run()
}

fn setup(cmd_args: Res<Args>, mut serial_res: ResMut<SerialResource>, rt: Res<Runtime>) {
    let serial_setting = SerialPortSetting {
        port_name: "COM1".to_string(),
        baud_rate: 115200,
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
    for message in serial_ev.iter() {
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
| 0.12 | 0.5             |
| 0.11 | 0.4             |
| 0.10 | 0.3             |
| 0.9  | 0.2             |
| 0.8  | 0.1             |

## License

Dual-licensed under either

- MIT
- Apache 2.0