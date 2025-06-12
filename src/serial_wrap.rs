use bevy_log::error;
use bytes::Bytes;
use futures::{
    stream::{SplitSink, SplitStream, StreamExt},
    SinkExt,
};
use parking_lot::Mutex;
use serialport::{DataBits, FlowControl, Parity, StopBits};
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::codec::{Decoder, Framed};

use crate::{codec::RawCodec, error::SerialError, ArcRuntime, RecvQueue};

/// Configuration settings for initializing a serial port
#[derive(Debug, Clone)]
pub struct SerialPortSetting {
    /// The port name, usually the device path
    pub port_name: String,
    /// The baud rate in symbols-per-second
    pub baud_rate: u32,
    /// Number of bits used to represent a character sent on the line
    pub data_bits: DataBits,
    /// The type of signalling to use for controlling data transfer
    pub flow_control: FlowControl,
    /// The type of parity to use for error checking
    pub parity: Parity,
    /// Number of bits to use to signal the end of a character
    pub stop_bits: StopBits,
    /// Amount of time to wait to receive data before timing out
    pub timeout: Duration,
}

impl Default for SerialPortSetting {
    fn default() -> Self {
        Self {
            port_name: String::new(),
            baud_rate: 115_200,
            data_bits: DataBits::Eight,
            flow_control: FlowControl::None,
            parity: Parity::None,
            stop_bits: StopBits::One,
            timeout: Duration::from_millis(0),
        }
    }
}

impl SerialPortSetting {
    /// Create a new setting with port name and baud rate
    pub fn new(port_name: impl Into<String>, baud_rate: u32) -> Self {
        Self {
            port_name: port_name.into(),
            baud_rate,
            ..Default::default()
        }
    }

    /// Set the data bits
    pub fn with_data_bits(mut self, data_bits: DataBits) -> Self {
        self.data_bits = data_bits;
        self
    }

    /// Set the flow control
    pub fn with_flow_control(mut self, flow_control: FlowControl) -> Self {
        self.flow_control = flow_control;
        self
    }

    /// Set the parity
    pub fn with_parity(mut self, parity: Parity) -> Self {
        self.parity = parity;
        self
    }

    /// Set the stop bits
    pub fn with_stop_bits(mut self, stop_bits: StopBits) -> Self {
        self.stop_bits = stop_bits;
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), SerialError> {
        if self.port_name.is_empty() {
            return Err(SerialError::InvalidConfiguration {
                reason: "Port name cannot be empty".to_string(),
            });
        }

        if self.baud_rate == 0 {
            return Err(SerialError::InvalidConfiguration {
                reason: "Baud rate must be greater than 0".to_string(),
            });
        }

        Ok(())
    }
}

/// Wrapper for serial port operations with async handling
pub struct SerialPortWrap {
    pub msg_sender: Arc<Mutex<UnboundedSender<Bytes>>>,
    pub recv_queue: RecvQueue,
}

impl SerialPortWrap {
    /// Create a new serial port wrapper with the given settings
    pub fn new(task_pool: ArcRuntime, setting: SerialPortSetting) -> Result<Self, SerialError> {
        let recv_queue = Arc::new(Mutex::new(Vec::new()));
        let (message_sender, message_receiver) = unbounded_channel::<Bytes>();

        // Initialize serial port connection
        let (sender, reader) = Self::initialize_serial_port(&task_pool, &setting)?;

        // Spawn async tasks for handling send/receive operations
        Self::spawn_handlers(
            task_pool,
            sender,
            reader,
            message_receiver,
            recv_queue.clone(),
        );

        Ok(Self {
            msg_sender: Arc::new(Mutex::new(message_sender)),
            recv_queue,
        })
    }

    /// Initialize the serial port and return split sender/receiver
    fn initialize_serial_port(
        task_pool: &ArcRuntime,
        setting: &SerialPortSetting,
    ) -> Result<
        (
            SplitSink<Framed<SerialStream, RawCodec>, Bytes>,
            SplitStream<Framed<SerialStream, RawCodec>>,
        ),
        SerialError,
    > {
        task_pool.block_on(async {
            let serial_port = tokio_serial::new(&setting.port_name, setting.baud_rate)
                .data_bits(setting.data_bits)
                .flow_control(setting.flow_control)
                .parity(setting.parity)
                .stop_bits(setting.stop_bits)
                .open_native_async()
                .map_err(|e| SerialError::SerialPortError {
                    port: setting.port_name.clone(),
                    source: e,
                })?;

            let stream = RawCodec.framed(serial_port);
            Ok(stream.split())
        })
    }

    /// Spawn async handlers for send and receive operations
    fn spawn_handlers(
        task_pool: ArcRuntime,
        mut sender: SplitSink<Framed<SerialStream, RawCodec>, Bytes>,
        mut reader: SplitStream<Framed<SerialStream, RawCodec>>,
        mut message_receiver: tokio::sync::mpsc::UnboundedReceiver<Bytes>,
        recv_queue: RecvQueue,
    ) {
        // Spawn sender task
        task_pool.spawn(async move {
            while let Some(message) = message_receiver.recv().await {
                if let Err(err) = sender.send(message).await {
                    error!("Failed to send message: {:?}", err);
                    break;
                }
            }
        });

        // Spawn receiver task
        task_pool.spawn(async move {
            while let Some(result) = reader.next().await {
                match result {
                    Ok(message) => {
                        recv_queue.lock().push(message);
                    }
                    Err(err) => {
                        error!("Failed to receive message: {:?}", err);
                        break;
                    }
                }
            }
        });
    }

    /// Get all pending messages and clear the queue
    pub fn get_messages(&mut self) -> Vec<Bytes> {
        self.recv_queue.lock().drain(..).collect()
    }
}
