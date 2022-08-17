use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerialError {
    #[error("serial port error")]
    SerialPortError(#[from] serialport::Error),
    #[error("tokio join error")]
    JoinError(#[from] tokio::task::JoinError),
}
