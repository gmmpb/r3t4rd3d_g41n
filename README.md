# R3T4RD3D G41N - Rust Audio Plugin

A VST3/CLAP audio effect plugin built with Rust using the NIH-plug framework.

## Features

- Gain control with dB unit conversion (-30dB to +30dB)
- Adjustable distortion drive parameter
- "Magic One" fractal-based audio effect with non-linear wave-shaping
- "Chaos" parameter using Lorenz attractor for organic, unpredictable modulation
- Real-time peak meter visualization
- Cross-platform compatibility (Windows, macOS, Linux)
- GUI interface built with VIZIA
- Available in VST3 and CLAP formats

## Project Structure

- `ret_gain/`: Main plugin code
  - `src/distortion.rs`: Distortion processing implementation
  - `src/fractal.rs`: Fractal-based audio algorithm implementation
  - `src/chaos.rs`: Lorenz attractor chaotic system implementation
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
- **Magic One**: Controls the fractal-based audio effect that creates complex, evolving textures using wave-shaping (0-100%)
- **Chaos**: Controls the Lorenz attractor system that creates organic and unpredictable but musical modulations (0-100%)

## Technical Implementation

### Magic One Effect

The Magic One slider controls a fractal-based algorithm that combines mathematics with audio processing. It implements:

- Fractal patterns derived from Julia set iterations to modulate the audio
- Non-linear wave-shaping through wave folding for harmonic complexity
- Time-based modulation with an LFO for continuous evolution
- Internal feedback paths for complex texture creation

### Chaos Effect

The Chaos slider implements a Lorenz attractor system, a well-known mathematical model of chaotic behavior. It features:

- Real-time calculation of the Lorenz differential equations (dx/dt, dy/dt, dz/dt)
- Input-influenced parameters where your audio directly affects the chaotic system
- Multiple modulation techniques (amplitude modulation, frequency modulation)
- Slowly evolving system parameters for continuously changing effects

Both effects are implemented with sample-accurate processing and optimized for real-time audio applications.

## Development

This project uses the [NIH-plug](https://github.com/robbert-vdh/nih-plug) framework for audio plugin development in Rust.

## License

[MIT License](LICENSE)

## Credits

Developed by Weblab Studio (weblabstudio.hu)
