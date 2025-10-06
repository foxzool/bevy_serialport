# Contributing to bevy_serialport

Thank you for your interest in contributing to `bevy_serialport`! We welcome contributions from everyone, whether you're fixing bugs, adding features, improving documentation, or helping with testing.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Contributing Guidelines](#contributing-guidelines)
- [Code Style](#code-style)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Reporting Issues](#reporting-issues)
- [Feature Requests](#feature-requests)
- [Code of Conduct](#code-of-conduct)

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- Git
- A serial port device or virtual serial port for testing (optional but recommended)

### Development Setup

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/bevy_serialport.git
   cd bevy_serialport
   ```

3. **Add the upstream remote**:
   ```bash
   git remote add upstream https://github.com/foxzool/bevy_serialport.git
   ```

4. **Install dependencies and build**:
   ```bash
   cargo build
   ```

5. **Run tests** to ensure everything works:
   ```bash
   cargo test
   ```

## Contributing Guidelines

### Types of Contributions

We welcome several types of contributions:

- **🐛 Bug fixes**: Fix issues in the existing codebase
- **✨ New features**: Add new functionality to the library
- **📝 Documentation**: Improve README, API docs, or examples
- **🧪 Tests**: Add or improve test coverage
- **🔧 Refactoring**: Improve code quality or performance
- **💡 Examples**: Add new examples or improve existing ones

### Before You Start

1. **Check existing issues** to see if your contribution is already being worked on
2. **Open an issue** to discuss major changes before implementing them
3. **Keep changes focused** - one feature or bug fix per pull request
4. **Follow the existing code style** and conventions

## Code Style

### Rust Code Guidelines

We follow standard Rust conventions with some additional guidelines:

#### Formatting
- Use `cargo fmt` to format your code
- Maximum line length: 100 characters
- Use 4 spaces for indentation (no tabs)

#### Naming Conventions
- Functions and variables: `snake_case`
- Types and traits: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

#### Documentation
- All public APIs must have documentation comments (`///`)
- Include examples in documentation when helpful
- Use proper grammar and complete sentences

```rust
/// Opens a new serial port with the specified configuration.
/// 
/// # Arguments
/// 
/// * `port_name` - The name of the serial port (e.g., "COM1", "/dev/ttyUSB0")
/// * `baud_rate` - The communication speed in bits per second
/// 
/// # Examples
/// 
/// ```
/// use bevy_serialport::{SerialResource, SerialPortRuntime};
/// 
/// fn setup(mut serial_res: ResMut<SerialResource>, rt: Res<SerialPortRuntime>) {
///     serial_res.open(rt.clone(), "COM1", 115200).expect("Failed to open port");
/// }
/// ```
/// 
/// # Errors
/// 
/// Returns `SerialError` if:
/// - The port name is invalid or the port doesn't exist
/// - The port is already in use by another application
/// - The baud rate is not supported by the hardware
pub fn open(&mut self, task_pool: ArcRuntime, port: impl ToString, baud_rate: u32) -> Result<(), SerialError> {
    // Implementation...
}
```

#### Error Handling
- Use `thiserror` for custom error types
- Provide meaningful error messages with context
- Use `Result<T, E>` for fallible operations

```rust
#[derive(Error, Debug)]
pub enum SerialError {
    #[error("Failed to access serial port '{port}': {source}")]
    SerialPortError {
        port: String,
        #[source]
        source: serialport::Error,
    },
}
```

#### Comments
- Use `//` for implementation comments
- Use `///` for documentation comments
- Explain *why* not *what* in implementation comments

```rust
// Spawn separate tasks for send/receive to avoid blocking
// the main Bevy thread during serial I/O operations
self.spawn_handlers(task_pool, sender, reader, message_receiver, recv_queue.clone());
```

### Commit Messages

Use conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(api): add support for custom timeout configuration

fix(serial): handle port disconnection gracefully

docs(readme): update installation instructions

test(integration): add tests for multiple port scenarios
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests for a specific platform (Linux only)
cargo test --features linux-only
```

### Test Categories

1. **Unit Tests**: Test individual functions and components
2. **Integration Tests**: Test the library's integration with Bevy
3. **Hardware Tests**: Test with actual serial hardware (platform-specific)

### Writing Tests

#### Unit Tests
Place unit tests in the same file as the code being tested:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serial_port_setting_builder() {
        let setting = SerialPortSetting::new("COM1", 9600)
            .with_data_bits(DataBits::Eight)
            .with_parity(Parity::None);
            
        assert_eq!(setting.port_name, "COM1");
        assert_eq!(setting.baud_rate, 9600);
        assert_eq!(setting.data_bits, DataBits::Eight);
    }
}
```

#### Integration Tests
Place integration tests in the `tests/` directory:

```rust
// tests/integration_test.rs
use bevy::prelude::*;
use bevy_serialport::*;

#[test]
fn test_plugin_integration() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SerialPortPlugin));
    
    // Test that the plugin initializes correctly
    app.update();
    assert!(app.world().get_resource::<SerialResource>().is_some());
}
```

### Hardware Testing

For testing with actual hardware:

1. **Linux**: Tests use `socat` to create virtual serial port pairs
2. **Windows**: Manual testing with virtual COM ports
3. **CI**: Automated tests run on Linux with virtual ports

## Submitting Changes

### Pull Request Process

1. **Create a new branch** for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following the guidelines above

3. **Test your changes**:
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

4. **Commit your changes** with clear commit messages

5. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Create a Pull Request** on GitHub

### Pull Request Template

When creating a PR, please include:

- **Description**: What does this PR do?
- **Motivation**: Why is this change needed?
- **Testing**: How was this tested?
- **Breaking Changes**: Are there any breaking changes?
- **Checklist**: Complete the PR checklist

### Example PR Description

```markdown
## Description
Adds support for configurable timeouts when opening serial ports.

## Motivation
Users need to be able to control how long the library waits when attempting to open a port, especially when dealing with unreliable hardware connections.

## Changes
- Added `timeout` field to `SerialPortSetting`
- Updated `SerialPortWrap::new()` to respect timeout configuration
- Added builder method `with_timeout()` for fluent configuration
- Updated documentation and examples

## Testing
- Added unit tests for timeout configuration
- Tested manually with slow-responding hardware
- All existing tests still pass

## Breaking Changes
None - this is a backward-compatible addition.

## Checklist
- [x] Tests added/updated
- [x] Documentation updated
- [x] Changelog updated (if applicable)
- [x] Follows code style guidelines
```

## Reporting Issues

### Bug Reports

When reporting bugs, please include:

1. **Description**: Clear description of the issue
2. **Steps to Reproduce**: Minimal code example that reproduces the issue
3. **Expected Behavior**: What should happen
4. **Actual Behavior**: What actually happens
5. **Environment**: OS, Rust version, library version
6. **Hardware**: Serial port hardware details (if relevant)

### Issue Template

```markdown
**Bug Description**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Create a new Bevy app with...
2. Configure serial port with...
3. Send data...
4. See error

**Expected Behavior**
What you expected to happen.

**Actual Behavior**
What actually happened.

**Code Example**
```rust
// Minimal code example that reproduces the issue
```

**Environment**
- OS: [e.g., Windows 10, Ubuntu 20.04, macOS 12.0]
- Rust version: [e.g., 1.70.0]
- bevy_serialport version: [e.g., 0.10.0]
- Serial hardware: [e.g., Arduino Uno, USB-to-Serial adapter]

**Additional Context**
Any other context about the problem.
```

## Feature Requests

For new features:

1. **Search existing issues** to avoid duplicates
2. **Describe the use case** clearly
3. **Provide examples** of how the feature would be used
4. **Consider alternatives** and explain why this approach is best

## Code of Conduct

### Our Pledge

We are committed to making participation in this project a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, gender identity and expression, level of experience, nationality, personal appearance, race, religion, or sexual identity and orientation.

### Our Standards

Examples of behavior that contributes to creating a positive environment include:

- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

### Enforcement

Instances of abusive, harassing, or otherwise unacceptable behavior may be reported by contacting the project maintainers. All complaints will be reviewed and investigated promptly and fairly.

## Getting Help

If you need help:

1. **Check the documentation** first
2. **Search existing issues** for similar problems
3. **Ask questions** in GitHub Discussions
4. **Join the Bevy Discord** for general Bevy help

## Recognition

Contributors will be recognized in:

- The project's README
- Release notes for significant contributions
- The Git history (obviously!)

Thank you for contributing to `bevy_serialport`! 🚀

---

*This document is inspired by contributing guidelines from successful open-source projects and is designed to make contributing as smooth as possible.*