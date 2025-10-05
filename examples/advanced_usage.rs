use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_log::{error, info, warn, LogPlugin};
use bevy_serialport::{
    list_available_ports, port_exists, DataBits, FlowControl, Parity, SerialData, SerialPortPlugin,
    SerialPortRuntime, SerialPortSetting, SerialResource, StopBits,
};
use std::time::Duration;

/// Advanced example showing enhanced API features
fn main() {
    // List available ports before starting
    match list_available_ports() {
        Ok(ports) => {
            if ports.is_empty() {
                println!("No serial ports found on this system");
                return;
            }
            println!("Available serial ports: {:?}", ports);
        }
        Err(e) => {
            eprintln!("Failed to list serial ports: {}", e);
            return;
        }
    }

    App::new()
        .add_plugins((
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            ))),
            LogPlugin::default(),
            SerialPortPlugin,
        ))
        .add_systems(Startup, setup_multiple_ports)
        .add_systems(Update, (handle_serial_data, send_periodic_data))
        .run();
}

/// Setup multiple serial ports with different configurations
fn setup_multiple_ports(mut serial_res: ResMut<SerialResource>, rt: Res<SerialPortRuntime>) {
    // Try to open a high-speed port
    let high_speed_config = SerialPortSetting::new("/dev/ttyUSB0", 115200)
        .with_data_bits(DataBits::Eight)
        .with_parity(Parity::None)
        .with_stop_bits(StopBits::One)
        .with_flow_control(FlowControl::None);

    match serial_res.open_with_setting(rt.clone(), high_speed_config) {
        Ok(_) => info!("High-speed port opened successfully"),
        Err(e) => warn!("Failed to open high-speed port: {}", e),
    }

    // Try to open a low-speed port for sensors
    if port_exists("/dev/ttyUSB1") {
        let sensor_config = SerialPortSetting::new("/dev/ttyUSB1", 9600)
            .with_data_bits(DataBits::Eight)
            .with_parity(Parity::Even)
            .with_stop_bits(StopBits::One);

        match serial_res.open_with_setting(rt.clone(), sensor_config) {
            Ok(_) => info!("Sensor port opened successfully"),
            Err(e) => warn!("Failed to open sensor port: {}", e),
        }
    }

    // Show connected ports
    let connected = serial_res.connected_ports();
    info!("Connected to {} ports: {:?}", connected.len(), connected);
}

/// Handle incoming serial data with enhanced processing
fn handle_serial_data(mut serial_ev: MessageReader<SerialData>) {
    for data in serial_ev.read() {
        info!("Port {}: received {} bytes", data.port, data.len());

        // Try to parse as UTF-8 string
        match data.as_string() {
            Ok(text) => {
                info!("Text data: '{}'", text.trim());

                // Parse simple commands
                match text.trim() {
                    "PING" => info!("Received ping from {}", data.port),
                    "STATUS" => info!("Status request from {}", data.port),
                    cmd if cmd.starts_with("SET:") => {
                        info!("Configuration command: {}", cmd);
                    }
                    _ => info!("Unknown text command: {}", text.trim()),
                }
            }
            Err(_) => {
                // Handle binary data
                info!("Binary data: {:?}", data.as_bytes());
            }
        }
    }
}

/// Send periodic data to connected ports
fn send_periodic_data(
    mut serial_res: ResMut<SerialResource>,
    time: Res<Time>,
    mut last_send: Local<f32>,
) {
    // Send data every 5 seconds
    let elapsed = time.elapsed().as_secs_f32();
    if elapsed - *last_send > 5.0 {
        let connected_ports: Vec<String> =
            serial_res.connected_ports().into_iter().cloned().collect();
        for port_name in connected_ports {
            let message = format!("Heartbeat from Bevy at {:.1}s\n", elapsed);

            match serial_res.send_string(&port_name, &message) {
                Ok(_) => info!("Sent heartbeat to {}", port_name),
                Err(e) => error!("Failed to send to {}: {}", port_name, e),
            }
        }

        *last_send = elapsed;
    }
}
