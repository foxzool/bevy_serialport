use bevy::{app::ScheduleRunnerPlugin, log::LogPlugin};
use std::time::Duration;

use bevy::prelude::*;
use bytes::Bytes;
use clap::Parser;

use bevy_serialport::{
    DataBits, FlowControl, Parity, SerialData, SerialPortPlugin, SerialPortRuntime,
    SerialPortSetting, SerialResource, StopBits,
};

#[derive(Parser, Resource, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[clap(short, long, value_parser)]
    port: String,

    /// Number of times to greet
    #[clap(short, long, value_parser, default_value_t = 115_200)]
    rate: u32,
}

fn main() {
    let args = Args::parse();

    App::new()
        .add_plugins((
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            ))),
            LogPlugin::default(),
            SerialPortPlugin,
        ))
        .insert_resource(args)
        .add_systems(Startup, setup)
        .add_systems(Update, (receive, send_test_data))
        .run();
}

fn setup(cmd_args: Res<Args>, mut serial_res: ResMut<SerialResource>, rt: Res<SerialPortRuntime>) {
    let serial_setting = SerialPortSetting {
        port_name: cmd_args.port.clone(),
        baud_rate: cmd_args.rate,
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

fn send_test_data(mut serial_res: ResMut<SerialResource>, cmd_args: Res<Args>) {
    serial_res.send_message(&cmd_args.port, Bytes::from(&b"123457"[..]))
}
