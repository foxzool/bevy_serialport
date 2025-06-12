use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerialError {
    #[error("Failed to access serial port '{port}': {source}")]
    SerialPortError {
        port: String,
        #[source]
        source: serialport::Error,
    },

    #[error("Tokio task join error: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Failed to send message to port '{port}': channel closed")]
    ChannelClosed { port: String },

    #[error("Port '{port}' not found in active connections")]
    PortNotFound { port: String },

    #[error("Invalid port configuration: {reason}")]
    InvalidConfiguration { reason: String },
}

impl SerialError {
    /// Create a new serial port error with context
    pub fn serial_port_error(port: impl Into<String>, source: serialport::Error) -> Self {
        Self::SerialPortError {
            port: port.into(),
            source,
        }
    }

    /// Create a channel closed error
    pub fn channel_closed(port: impl Into<String>) -> Self {
        Self::ChannelClosed { port: port.into() }
    }

    /// Create a port not found error
    pub fn port_not_found(port: impl Into<String>) -> Self {
        Self::PortNotFound { port: port.into() }
    }
}
