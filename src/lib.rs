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

pub struct SerialPortPlugin;

impl Plugin for SerialPortPlugin {
    fn build(&self, app: &mut App) {
        let tokio_rt = Arc::new(Builder::new_multi_thread().enable_all().build().unwrap());
        app.insert_resource(tokio_rt);

        app.init_resource::<SerialResource>();
    }
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
        port: &str,
        baud_rate: u32,
    ) -> Result<(), SerialError> {
        let client = SerialPortWrap::new(task_pool, port, baud_rate)?;

        self.ports.insert(port.to_string(), client);

        Ok(())
    }

    pub fn view_messages(&mut self, port: &str) -> Vec<Bytes> {
        let mut messages = Vec::new();
        if let Some(serial_wrap) = self.ports.get_mut(port) {
            let mut serial_messages: Vec<Bytes> = serial_wrap.get_messages();
            messages.append(&mut serial_messages);
        }

        messages
    }

    pub fn send_message(&mut self, port: &str, message: Bytes) {
        if let Some(serial_wrap) = self.ports.get_mut(port) {
            if let Err(err) = serial_wrap.msg_sender.lock().send(message) {
                error!("send data to {} error {:?}", port, err)
            }
        }
    }
}
