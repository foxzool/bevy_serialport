use std::collections::BTreeMap;
use std::sync::Arc;

use bevy::prelude::*;
use bytes::Bytes;
use parking_lot::Mutex;
use tokio::runtime::Builder;

pub mod codec;
mod error;
mod serial_wrap;
pub use error::SerialError;
pub use serial_wrap::*;
pub use serialport::{DataBits, FlowControl, Parity, StopBits};

pub struct SerialPortPlugin;

impl Plugin for SerialPortPlugin {
    fn build(&self, app: &mut App) {
        let tokio_rt = Arc::new(Builder::new_multi_thread().enable_all().build().unwrap());
        app.insert_resource(tokio_rt)
            .init_resource::<SerialResource>()
            .add_event::<SerialData>();

        app.add_system_to_stage(CoreStage::PreUpdate, broadcast_serial_message);
    }
}

#[derive(Debug)]
pub struct SerialData {
    pub port: String,
    pub data: Bytes,
}

pub type Runtime = Arc<tokio::runtime::Runtime>;
pub type RecvQueue = Arc<Mutex<Vec<Bytes>>>;

/// serial port resource
#[derive(Default)]
pub struct SerialResource {
    pub ports: BTreeMap<String, SerialPortWrap>,
}

impl SerialResource {
    pub fn open(
        &mut self,
        task_pool: Runtime,
        port: impl ToString,
        baud_rate: u32,
    ) -> Result<(), SerialError> {
        let setting = SerialPortSetting {
            port_name: port.to_string(),
            baud_rate,
            ..default()
        };
        let client = SerialPortWrap::new(task_pool, setting)?;

        self.ports.insert(port.to_string(), client);

        Ok(())
    }

    pub fn open_with_setting(
        &mut self,
        task_pool: Runtime,
        setting: SerialPortSetting,
    ) -> Result<(), SerialError> {
        let port_name = setting.port_name.clone();
        let serial_port = SerialPortWrap::new(task_pool, setting)?;

        self.ports.insert(port_name, serial_port);

        Ok(())
    }

    pub fn send_message(&mut self, port: &str, message: Bytes) {
        if let Some(serial_wrap) = self.ports.get_mut(port) {
            if let Err(err) = serial_wrap.msg_sender.lock().send(message) {
                error!("send data to {} error {:?}", port, err)
            }
        }
    }
}

fn broadcast_serial_message(
    mut serial_res: ResMut<SerialResource>,
    mut message_ev: EventWriter<SerialData>,
) {
    let mut messages: Vec<SerialData> = Vec::new();

    for (port_name, port_wrap) in serial_res.ports.iter_mut() {
        let mut serial_messages: Vec<SerialData> = port_wrap
            .get_messages()
            .into_iter()
            .map(|m| SerialData {
                port: port_name.clone(),
                data: m,
            })
            .collect();

        messages.append(&mut serial_messages);
    }

    message_ev.send_batch(messages.into_iter());
}
