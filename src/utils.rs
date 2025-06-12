/// Utility functions for serial port operations
use crate::SerialError;

/// List available serial ports on the system
pub fn list_available_ports() -> Result<Vec<String>, SerialError> {
    serialport::available_ports()
        .map_err(|e| SerialError::SerialPortError {
            port: "unknown".to_string(),
            source: e,
        })
        .map(|ports| ports.into_iter().map(|port| port.port_name).collect())
}

/// Check if a specific port exists on the system
pub fn port_exists(port_name: &str) -> bool {
    list_available_ports()
        .map(|ports| ports.iter().any(|p| p == port_name))
        .unwrap_or(false)
}

/// Get information about available ports with details
pub fn get_port_info() -> Result<Vec<serialport::SerialPortInfo>, SerialError> {
    serialport::available_ports().map_err(|e| SerialError::SerialPortError {
        port: "unknown".to_string(),
        source: e,
    })
}

/// Validate a baud rate value
pub fn is_valid_baud_rate(baud_rate: u32) -> bool {
    matches!(
        baud_rate,
        110 | 300
            | 600
            | 1200
            | 2400
            | 4800
            | 9600
            | 14400
            | 19200
            | 38400
            | 57600
            | 115200
            | 128000
            | 256000
    )
}

/// Get common baud rates
pub fn common_baud_rates() -> Vec<u32> {
    vec![9600, 14400, 19200, 38400, 57600, 115200, 128000, 256000]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baud_rate_validation() {
        assert!(is_valid_baud_rate(115200));
        assert!(is_valid_baud_rate(9600));
        assert!(!is_valid_baud_rate(123456));
    }

    #[test]
    fn test_common_baud_rates() {
        let rates = common_baud_rates();
        assert!(!rates.is_empty());
        assert!(rates.contains(&115200));
        assert!(rates.contains(&9600));
    }
}
