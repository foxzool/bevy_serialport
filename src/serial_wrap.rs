use std::sync::Arc;

use bevy::log::info;
use bytes::Bytes;
use futures::{
    stream::{SplitSink, SplitStream, StreamExt},
    SinkExt,
};
use parking_lot::Mutex;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::codec::{Decoder, Framed};

use crate::{codec::RawCodec, error, error::SerialError, RecvQueue, Runtime};

/// 串口设置
pub struct SerialPortWrap {
    /// 串口名
    pub port: String,
    /// 波特率
    pub baud_rate: u32,
    pub msg_sender: Arc<Mutex<UnboundedSender<Bytes>>>,
    pub recv_queue: RecvQueue,
}

impl SerialPortWrap {
    pub fn new(task_pool: Runtime, port: &str, baud_rate: u32) -> Result<Self, SerialError> {
        let recv_queue = Arc::new(Mutex::new(Vec::new()));

        let port = port.to_string();
        let port_2 = port.clone();
        let rt_2 = task_pool.clone();

        let (message_sender, mut message_receiver) = unbounded_channel::<Bytes>();
        let recv_queue_2 = recv_queue.clone();
        let (mut sender, mut reader) = rt_2.block_on(async move {
            let serial_port = tokio_serial::new(&port_2, baud_rate).open_native_async()?;

            info!("打开串口 {} ", port_2);

            let stream = RawCodec.framed(serial_port);
            Ok::<
                (
                    SplitSink<Framed<SerialStream, RawCodec>, bytes::Bytes>,
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
            port,
            baud_rate,
            msg_sender: Arc::new(Mutex::new(message_sender)),
            recv_queue,
        })
    }

    pub fn get_messages(&mut self) -> Vec<Bytes> {
        self.recv_queue.clone().lock().drain(..).collect()
    }
}
