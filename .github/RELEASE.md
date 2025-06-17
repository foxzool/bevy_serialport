# Release Process

This document describes how to release a new version of `bevy_serialport`.

## Prerequisites

Before releasing, ensure that:

1. **CRATES_TOKEN Secret**: Set up the `CRATES_TOKEN` secret in your GitHub repository settings:
   - Go to your repository's Settings → Secrets and variables → Actions
   - Add a new repository secret named `CRATES_TOKEN`
   - Value should be your crates.io API token (get it from https://crates.io/me)

2. **Repository Permissions**: The GitHub Actions workflow requires `contents: write` permission to create releases.

## Release Steps

1. **Update Version**: Update the version in `Cargo.toml`
2. **Commit Changes**: Commit any final changes
3. **Create and Push Tag**: Create a tag with the version number
   ```bash
   git tag v0.9.2
   git push origin v0.9.2
   ```

## Automated Process

When you push a tag (e.g., `v0.9.2`), the GitHub Actions workflow will:

1. **Run Tests**: Execute the full test suite to ensure code quality
2. **Create GitHub Release**: 
   - Generate a changelog from commit messages
   - Create a new GitHub release with the changelog
   - Mark as prerelease if tag contains `-` (e.g., `v1.0.0-beta.1`)
3. **Publish to crates.io**:
   - Verify the tag version matches `Cargo.toml`
   - Package and verify the crate
   - Publish to crates.io

## Tag Naming Convention

- **Stable releases**: `v1.0.0`, `v1.0.1`, etc.
- **Pre-releases**: `v1.0.0-alpha.1`, `v1.0.0-beta.1`, `v1.0.0-rc.1`, etc.

Pre-release tags (containing `-`) will be marked as pre-releases on GitHub.

## Troubleshooting

- **Failed publication**: Check that the `CRATES_TOKEN` secret is set correctly
- **Version mismatch**: Ensure the git tag version matches the version in `Cargo.toml`
- **Test failures**: The release will not proceed if tests fail

## Manual Release (if needed)

If you need to release manually:

```bash
# Build and test
cargo build --all-features
cargo test --all-features

# Package and publish
cargo package
cargo publish
```