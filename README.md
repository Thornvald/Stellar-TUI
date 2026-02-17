# Stellar TUI

Terminal UI for managing and building Unreal Engine projects.

![Stellar TUI screenshot](./Stellar-TUI.png)

## Features

- Project list with persistent project selection
- Unreal Engine path selection (auto-detect + manual)
- Build controls with live output log panel
- Keyboard-first navigation across Projects, Engine, Build, and Logs

## Keyboard Controls

- `Left` / `Right` or `Tab` / `Shift+Tab`: move between UI sections
- `Up` / `Down`: move within the current section (in Logs: `Up` shows older lines, `Down` returns to follow-latest)
- `Enter`: activate/select focused item
- `a`: add project using manual path input
- `f`: add project using file picker dialog
- `d` or `Delete`: remove focused project (with confirmation)
- `e`: set engine path / open engine picker
- `r`: re-detect engine installs
- `b`: start build
- `n`: clean rebuild (remove temp files, regenerate project files, then build)
- `c`: cancel build
- `x`: clear logs
- `y`: copy logs to clipboard
- `?`: help
- `q`: quit

## Build Locally

Prerequisites:

- Rust toolchain (stable): https://rustup.rs

Build debug binary:

```bash
cargo build
```

Build release binary:

```bash
cargo build --release
```

Outputs:

- Windows: `target/release/stellar.exe`
- Linux/macOS: `target/release/stellar`

## Run

```bash
cargo run
```

or run the built binary directly from `target/release`.

## Install and Run as `stellar`

If `stellar` is on your system `PATH`, users can open a terminal and run:

```bash
stellar
```

For non-technical users, the most reliable approach is to ship installers that set up `PATH` automatically:

- Windows: MSI/EXE installer (or Winget package)
- macOS: PKG installer (or Homebrew tap)
- Linux: native packages (`.deb`/`.rpm`) or package managers

Portable fallback (manual): place the binary in a folder already on `PATH`.

### Windows Installer

The repository includes an Inno Setup installer script at `packaging/windows/stellar.iss`.

- Installs Stellar to `Program Files\Stellar`
- Adds the install folder to user `PATH` automatically
- Creates Start Menu entries

After installation, users can open a new terminal and run:

```bash
stellar
```

## Install Options

Two install options are supported:

1. GitHub Release download
   - `stellar-setup-<version>.exe` (recommended for most users)
   - `stellar-windows-x64.exe` (portable executable)
2. Winget package
   - `winget install Thornvald.StellarTUI`

Winget becomes available after the package submission PR is merged in `microsoft/winget-pkgs` and index updates propagate. If it is not found yet, use the GitHub Release installer first.

## Cross-Platform Distribution

A binary built on one OS is generally for that OS only. Recommended release process:

1. Build native binaries on Windows, Linux, and macOS.
2. Publish all binaries in a GitHub Release.
3. Users download the binary for their platform.

This repository includes a GitHub Actions workflow at `.github/workflows/release.yml` that:

- builds release binaries on Windows, Linux, and macOS
- uploads CI artifacts
- when you push a tag like `v1.0.0`, publishes those binaries to a GitHub Release

## Repository

- Source and releases: `https://github.com/Thornvald/Stellar-TUI`

## Creating a Release

Tag and push:

```bash
git tag v0.90.0
git push origin v0.90.0
```

The workflow will publish release assets automatically.
