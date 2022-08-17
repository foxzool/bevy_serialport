use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use futures::{
    stream::{SplitSink, SplitStream, StreamExt},
    SinkExt,
};
use parking_lot::Mutex;
use serialport::{DataBits, FlowControl, Parity, StopBits};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::codec::{Decoder, Framed};

use crate::{codec::RawCodec, error, error::SerialError, RecvQueue, Runtime};

/// settings for initialize serial port
#[derive(Debug)]
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
            port_name: "".to_string(),
            baud_rate: 115_200,
            data_bits: DataBits::Eight,
            flow_control: FlowControl::None,
            parity: Parity::None,
            stop_bits: StopBits::One,
            timeout: Duration::from_millis(0),
        }
    }
}

pub struct SerialPortWrap {
    pub msg_sender: Arc<Mutex<UnboundedSender<Bytes>>>,
    pub recv_queue: RecvQueue,
}

impl SerialPortWrap {
    pub fn new(task_pool: Runtime, setting: SerialPortSetting) -> Result<Self, SerialError> {
        let recv_queue = Arc::new(Mutex::new(Vec::new()));

        let pool_clone = task_pool.clone();

        let (message_sender, mut message_receiver) = unbounded_channel::<Bytes>();
        let recv_queue_2 = recv_queue.clone();
        let (mut sender, mut reader) = pool_clone.block_on(async move {
            let serial_port = tokio_serial::new(setting.port_name, setting.baud_rate)
                .data_bits(setting.data_bits)
                .flow_control(setting.flow_control)
                .parity(setting.parity)
                .stop_bits(setting.stop_bits)
                .open_native_async()?;

            let stream = RawCodec.framed(serial_port);
            Ok::<
                (
                    SplitSink<Framed<SerialStream, RawCodec>, Bytes>,
                    SplitStream<Framed<SerialStream, RawCodec>>,
                ),
                serialport::Error,
            >(stream.split())
        })?;
        task_pool.clone().spawn(async move {
            task_pool.spawn(async move {
                while let Some(message) = message_receiver.recv().await {
                    match sender.send(message).await {
                        Ok(_) => {}
                        Err(err) => {
                            error!("{:?}", err);
                        }
                    }
                }
            });

            task_pool.spawn(async move {
                while let Some(Ok(recv_message)) = reader.next().await {
                    recv_queue_2.lock().push(recv_message);
                }
            });

            Ok::<(), SerialError>(())
        });

        Ok(Self {
            msg_sender: Arc::new(Mutex::new(message_sender)),
            recv_queue,
        })
    }

    pub fn get_messages(&mut self) -> Vec<Bytes> {
        self.recv_queue.clone().lock().drain(..).collect()
    }
}
