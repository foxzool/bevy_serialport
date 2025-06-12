#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

use std::{collections::BTreeMap, sync::Arc};

use bevy_app::prelude::*;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_utils::default;
use bytes::Bytes;
use parking_lot::Mutex;
pub use serialport::{DataBits, FlowControl, Parity, StopBits};
use tokio::runtime::Builder;

pub use error::SerialError;
pub use serial_wrap::*;
pub use utils::*;

pub mod codec;
mod error;
mod serial_wrap;
pub mod utils;
/// Serial port plugin for Bevy
pub struct SerialPortPlugin;

impl Plugin for SerialPortPlugin {
    fn build(&self, app: &mut App) {
        // Create the Tokio runtime for async operations
        let tokio_rt = SerialPortRuntime(Arc::new(
            Builder::new_multi_thread()
                .enable_all()
                .thread_name("bevy-serialport")
                .build()
                .expect("Failed to create Tokio runtime for serial port operations"),
        ));

        app.insert_resource(tokio_rt)
            .init_resource::<SerialResource>()
            .add_event::<SerialData>()
            .add_systems(PreUpdate, broadcast_serial_message);
    }
}

/// Event containing serial port data received
#[derive(Debug, Event)]
pub struct SerialData {
    /// The port name that received the data
    pub port: String,
    /// The raw data received
    pub data: Bytes,
}

impl SerialData {
    /// Create a new SerialData event
    pub fn new(port: impl Into<String>, data: Bytes) -> Self {
        Self {
            port: port.into(),
            data,
        }
    }

    /// Get the data as a UTF-8 string, replacing invalid sequences
    pub fn as_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.data).to_string()
    }

    /// Try to get the data as a UTF-8 string
    pub fn as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.data.to_vec())
    }

    /// Get the data as a byte slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get the length of the data
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the data is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct SerialPortRuntime(Arc<tokio::runtime::Runtime>);
pub type ArcRuntime = Arc<tokio::runtime::Runtime>;
pub type RecvQueue = Arc<Mutex<Vec<Bytes>>>;

/// Serial port resource for managing multiple serial connections
#[derive(Default, Resource)]
pub struct SerialResource {
    pub ports: BTreeMap<String, SerialPortWrap>,
}

impl SerialResource {
    /// Open a serial port with basic configuration
    pub fn open(
        &mut self,
        task_pool: ArcRuntime,
        port: impl ToString,
        baud_rate: u32,
    ) -> Result<(), SerialError> {
        let port_str = port.to_string();
        let setting = SerialPortSetting {
            port_name: port_str.clone(),
            baud_rate,
            ..default()
        };

        self.open_with_setting(task_pool, setting)
    }

    /// Open a serial port with custom settings
    pub fn open_with_setting(
        &mut self,
        task_pool: ArcRuntime,
        setting: SerialPortSetting,
    ) -> Result<(), SerialError> {
        let port_name = setting.port_name.clone();

        // Validate configuration
        if port_name.is_empty() {
            return Err(SerialError::InvalidConfiguration {
                reason: "Port name cannot be empty".to_string(),
            });
        }

        if setting.baud_rate == 0 {
            return Err(SerialError::InvalidConfiguration {
                reason: "Baud rate must be greater than 0".to_string(),
            });
        }

        // Create serial port wrapper
        let serial_port = SerialPortWrap::new(task_pool, setting).map_err(|e| match e {
            SerialError::SerialPortError { source, .. } => {
                SerialError::serial_port_error(&port_name, source)
            }
            other => other,
        })?;

        self.ports.insert(port_name, serial_port);
        Ok(())
    }

    /// Send a message to a specific port
    pub fn send_message(&mut self, port: &str, message: Bytes) -> Result<(), SerialError> {
        match self.ports.get_mut(port) {
            Some(serial_wrap) => serial_wrap
                .msg_sender
                .lock()
                .send(message)
                .map_err(|_| SerialError::channel_closed(port)),
            None => Err(SerialError::port_not_found(port)),
        }
    }

    /// Send a string message to a specific port
    pub fn send_string(&mut self, port: &str, message: &str) -> Result<(), SerialError> {
        self.send_message(port, Bytes::from(message.as_bytes().to_vec()))
    }

    /// Check if a port is connected
    pub fn is_port_connected(&self, port: &str) -> bool {
        self.ports.contains_key(port)
    }

    /// Get list of connected ports
    pub fn connected_ports(&self) -> Vec<&String> {
        self.ports.keys().collect()
    }

    /// Close a specific port
    pub fn close_port(&mut self, port: &str) -> bool {
        self.ports.remove(port).is_some()
    }

    /// Close all ports
    pub fn close_all_ports(&mut self) {
        self.ports.clear();
    }
}

/// Broadcasts received serial messages as Bevy events
/// Optimized to avoid unnecessary allocations and clones
fn broadcast_serial_message(
    mut serial_res: ResMut<SerialResource>,
    mut message_ev: EventWriter<SerialData>,
) {
    // Collect all messages directly without intermediate vectors
    let messages: Vec<SerialData> = serial_res
        .ports
        .iter_mut()
        .flat_map(|(port_name, port_wrap)| {
            port_wrap.get_messages().into_iter().map(|data| SerialData {
                port: port_name.clone(),
                data,
            })
        })
        .collect();

    if !messages.is_empty() {
        message_ev.write_batch(messages);
    }
}

#[cfg(test)]
mod unit_tests {
    use bevy::prelude::{App, MinimalPlugins};

    use crate::SerialPortPlugin;

    /// This tests that we have properly set up the System parameters used in our systems, but
    /// doesn't test the 'real' functionality of the plugin.
    #[test]
    fn smoke_test_basic_integration_with_bevy_app() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, SerialPortPlugin));
        app.update();
        app.update();
    }
}
