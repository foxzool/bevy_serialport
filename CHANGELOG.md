# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.12.0] - 2026-06-20

### Changed
- Upgrade to Bevy 0.19.0 (stable)
- Updated all Bevy dependencies to 0.19.0 compatible versions
  - bevy_app: 0.19.0
  - bevy_ecs: 0.19.0
  - bevy_log: 0.19.0
  - bevy_derive: 0.19.0
  - bevy_utils: 0.19.0
- Bump crate version to 0.12.0

### Compatibility
- **Bevy 0.18**: `bevy_serialport` 0.11.0
- **Bevy 0.19**: `bevy_serialport` 0.12.0

### Verified
- ✅ All unit tests pass
- ✅ `cargo build --all-features` passes
- ✅ `cargo clippy --all-targets --all-features -- -D warnings` passes
- ✅ Examples compile

## [0.11.0] - 2026-01-14

### Added
- Bevy 0.18 support
- Updated all Bevy dependencies from 0.17.0 to 0.18.0

### Changed
- `bevy_serialport` version bumped to 0.11.0 for Bevy 0.18 compatibility
- All core APIs remain compatible with Bevy 0.18

### Compatibility
- **Bevy 0.17**: `bevy_serialport` 0.10.x
- **Bevy 0.18**: `bevy_serialport` 0.11.0

### Verified
- ✅ All unit tests pass
- ✅ Integration tests pass
- ✅ Examples compile and run successfully
- ✅ Core APIs (Message, MessageReader, MessageWriter, Plugin, Resource, SystemParam) remain compatible

### Migration Notes
No breaking changes required for users upgrading from bevy_serialport 0.10.x to 0.11.0.
Simply update your `Cargo.toml`:

```toml
[dependencies]
bevy_serialport = "0.11"
bevy = "0.18"  # or your preferred features
```

The API remains fully compatible. See the [Bevy 0.18 Migration Guide](https://bevy.org/learn/migration-guides/0-17-to-0-18/) for Bevy-specific changes.
