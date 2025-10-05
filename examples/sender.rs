use bevy::app::ScheduleRunnerPlugin;
use std::time::Duration;

use bevy::prelude::*;
use bevy_log::{error, info, LogPlugin};
use bevy_serialport::{
    DataBits, FlowControl, Parity, SerialData, SerialPortPlugin, SerialPortRuntime,
    SerialPortSetting, SerialResource, StopBits,
};
use clap::Parser;

#[derive(Parser, Resource, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Serial port name (e.g., COM1, /dev/ttyUSB0)
    #[clap(short, long, value_parser)]
    port: String,

    /// Baud rate for serial communication
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

/// Setup the serial port with custom configuration
fn setup(cmd_args: Res<Args>, mut serial_res: ResMut<SerialResource>, rt: Res<SerialPortRuntime>) {
    let serial_setting = SerialPortSetting::new(&cmd_args.port, cmd_args.rate)
        .with_data_bits(DataBits::Eight)
        .with_flow_control(FlowControl::None)
        .with_parity(Parity::None)
        .with_stop_bits(StopBits::One);

    match serial_res.open_with_setting(rt.clone(), serial_setting) {
        Ok(_) => info!("Successfully opened serial port: {}", cmd_args.port),
        Err(e) => {
            error!("Failed to open serial port: {}", e);
            std::process::exit(1);
        }
    }
}

/// Receive and log incoming data
fn receive(mut serial_ev: MessageReader<SerialData>) {
    for message in serial_ev.read() {
        info!(
            "Received from {}: {}",
            message.port,
            message.as_string_lossy()
        );
    }
}

/// Send test data periodically
fn send_test_data(
    mut serial_res: ResMut<SerialResource>,
    cmd_args: Res<Args>,
    time: Res<Time>,
    mut last_send: Local<f32>,
) {
    // Send test data every 2 seconds
    let elapsed = time.elapsed().as_secs_f32();
    if elapsed - *last_send > 2.0 {
        let test_message = format!("Test message at {:.2}s\n", elapsed);

        if let Err(e) = serial_res.send_string(&cmd_args.port, &test_message) {
            error!("Failed to send test message: {}", e);
        } else {
            info!("Sent test message: {}", test_message.trim());
        }

        *last_send = elapsed;
    }
}
