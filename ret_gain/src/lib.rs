// Re-export modules
// These "mod" statements tell Rust to include these files as modules in our crate
// Similar to JavaScript imports or Python imports, but they define the module structure
mod editor;      // The GUI editor implementation
mod gain;        // The gain effect processor
mod distortion;  // The distortion effect processor
mod fractal;     // The fractal-based effect processor
mod chaos;       // The chaos/lorenz attractor effect
mod plugin;      // The main plugin structure that combines all effects

// Re-export main types for use in main.rs and elsewhere
// These "pub use" statements make the specified items available to users of our crate
// This is like "export" in JavaScript/TypeScript modules - exposing our public API
pub use plugin::RetardedGain;      // Export the main plugin struct
pub use gain::GainProcessor;       // Export the gain processor
pub use distortion::Distortion;    // Export the distortion processor
pub use fractal::FractalMagic;     // Export the fractal effect
pub use chaos::ChaosAttractor;     // Export the chaos effect

// Export the plugin into the proper formats
// These are macro invocations that generate the necessary code for VST3 and CLAP plugin formats
use nih_plug::prelude::*;          // Import the NIH-plug framework items
nih_export_clap!(plugin::RetardedGain);  // Generate CLAP plugin export code for RetardedGain
nih_export_vst3!(plugin::RetardedGain);  // Generate VST3 plugin export code for RetardedGain