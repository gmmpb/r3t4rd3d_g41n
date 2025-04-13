// Re-export modules
mod editor;
mod gain;
mod distortion;
mod fractal;

// Re-export main types for use in main.rs and elsewhere
pub use gain::Gain;
pub use distortion::Distortion;
pub use fractal::FractalMagic;

// Export the plugin into the proper formats
use nih_plug::prelude::*;
nih_export_clap!(gain::Gain);
nih_export_vst3!(gain::Gain);