use std::time::Duration;

use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_log::{error, info, LogPlugin};
use clap::Parser;

use bevy_serialport::{SerialData, SerialPortPlugin, SerialPortRuntime, SerialResource};

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
        .add_systems(Update, receive)
        .run();
}

/// Setup the serial port connection
fn setup(cmd_args: Res<Args>, mut serial_res: ResMut<SerialResource>, rt: Res<SerialPortRuntime>) {
    match serial_res.open(rt.clone(), &cmd_args.port, cmd_args.rate) {
        Ok(_) => info!("Successfully opened serial port: {}", cmd_args.port),
        Err(e) => {
            error!("Failed to open serial port: {}", e);
            std::process::exit(1);
        }
    }
}

/// Receive data and echo it back
fn receive(mut serial_res: ResMut<SerialResource>, mut serial_ev: EventReader<SerialData>) {
    for message in serial_ev.read() {
        info!(
            "Received from {}: {:?}",
            message.port,
            message.as_string_lossy()
        );

        // Echo the received data back
        if let Err(e) = serial_res.send_message(&message.port, message.data.clone()) {
            error!("Failed to send echo: {}", e);
        }
    }
}
