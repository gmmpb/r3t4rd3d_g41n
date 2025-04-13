# R3T4RD3D G41N - Rust Audio Plugin

A VST3/CLAP audio effect plugin built with Rust using the NIH-plug framework.

## Features

- Gain control with dB unit conversion (-30dB to +30dB)
- Adjustable distortion drive parameter
- Real-time peak meter visualization
- Cross-platform compatibility (Windows, macOS, Linux)
- GUI interface built with VIZIA
- Available in VST3 and CLAP formats

## Project Structure

- `ret_gain/`: Main plugin code
  - `src/distortion.rs`: Distortion processing implementation
  - `src/editor.rs`: GUI editor implementation
  - `src/gain.rs`: Main plugin gain processing logic
  - `src/lib.rs`: Plugin exports (VST3/CLAP)
  - `src/main.rs`: Standalone application entry point
- `xtask/`: Build utilities and automation scripts

## Building the Plugin

### Prerequisites

- Rust toolchain (latest stable version recommended)
- Cargo build system

### Build Commands

```bash
# Clone the repository
git clone https://github.com/your-username/rust-vst.git
cd rust-vst

# Build the plugin in debug mode
cargo build

# Build the plugin in release mode
cargo build --release

# Bundle the plugin for distribution
cargo xtask bundle ret_gain --release
```

The built plugins will be available in:

- Debug: `target/debug/`
- Release: `target/bundled/`

## Installation

Copy the built `.vst3` or `.clap` files to your system's VST plugin directory:

- Windows: `C:\Program Files\Common Files\VST3\`
- macOS: `/Library/Audio/Plug-Ins/VST3/`
- Linux: `/usr/lib/vst3/`

## Usage

Load the plugin in any compatible DAW (Digital Audio Workstation) that supports VST3 or CLAP plugins.

### Parameters

- **Gain**: Adjusts the output level of the audio (-30dB to +30dB)
- **Drive**: Controls the amount of distortion (1.0 to 50.0)

## Development

This project uses the [NIH-plug](https://github.com/robbert-vdh/nih-plug) framework for audio plugin development in Rust.

## License

[MIT License](LICENSE)

## Credits

Developed by Weblab Studio (weblabstudio.hu)
