# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build and Test
- `cargo build` - Build the library
- `cargo test` - Run all tests
- `cargo test test_bevy_integration` - Run specific integration test
- `cargo run --example serial_receiver -- --port COM1 --rate 115200` - Run receiver example
- `cargo run --example serial_sender -- --port COM1 --rate 115200` - Run sender example

### Development
- `cargo check` - Fast compilation check
- `cargo clippy` - Linting
- `cargo fmt` - Code formatting
- `cargo run --example advanced_usage` - Run advanced usage example

## Architecture

This is a Bevy plugin that provides async serial port communication using Tokio. The architecture consists of:

### Core Components
- **SerialPortPlugin**: Main plugin that sets up the Tokio runtime and resources
- **SerialResource**: Resource managing multiple serial port connections with enhanced error handling and convenience methods
- **SerialPortWrap**: Wrapper handling individual serial port connections with separate send/receive channels
- **SerialData**: Event fired when data is received, with built-in string conversion utilities
- **SerialPortSetting**: Builder pattern configuration for serial port parameters

### Key Design Patterns
- Uses Tokio runtime embedded within Bevy's ECS for async operations
- Split architecture: separate sender/receiver channels for each port using tokio_util::codec
- Message passing via unbounded channels between Bevy systems and Tokio tasks
- Events broadcast to all systems listening for SerialData

### Data Flow
1. Serial ports opened via SerialResource::open() or open_with_setting() with validation
2. Each port spawns two tasks: one for sending, one for receiving
3. Received data queued in SerialPortWrap::recv_queue (Arc<Mutex<Vec<Bytes>>>)
4. broadcast_serial_message system drains queues and fires SerialData events
5. Send data via SerialResource::send_message() or send_string() with proper error handling

### Enhanced APIs
- **Builder Pattern**: SerialPortSetting::new().with_data_bits().with_parity()
- **String Utilities**: SerialData::as_string_lossy() and as_string()
- **Error Handling**: Comprehensive error types with proper context
- **Convenience Methods**: send_string(), is_port_connected(), connected_ports()

### Codec System
- Uses RawCodec (in codec.rs) for frame delimiting
- Built on tokio_util::codec::Decoder for stream parsing
- Frames serial data into Bytes for processing

The plugin is designed to handle multiple serial ports simultaneously while integrating cleanly with Bevy's ECS system.