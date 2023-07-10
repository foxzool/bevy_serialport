use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use clap::Parser;

use bevy_serialport::{SerialData, SerialPortPlugin, SerialPortRuntime, SerialResource};

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
        .add_systems(Update, receive)
        .run()
}

fn setup(cmd_args: Res<Args>, mut serial_res: ResMut<SerialResource>, rt: Res<SerialPortRuntime>) {
    serial_res
        .open(rt.clone(), &cmd_args.port, cmd_args.rate)
        .expect("open serial port error");
}

/// receive data and send back
fn receive(mut serial_res: ResMut<SerialResource>, mut serial_ev: EventReader<SerialData>) {
    for message in serial_ev.iter() {
        info!("receive {:?}", message);
        serial_res.send_message(&message.port, message.data.clone());
    }
}
