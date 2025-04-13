// Re-export modules
mod editor;
mod gain;
mod distortion;
mod fractal;
mod chaos;
mod plugin;

// Re-export main types for use in main.rs and elsewhere
pub use plugin::RetardedGain;
pub use gain::GainProcessor;
pub use distortion::Distortion;
pub use fractal::FractalMagic;
pub use chaos::ChaosAttractor;

// Export the plugin into the proper formats
use nih_plug::prelude::*;
nih_export_clap!(plugin::RetardedGain);
nih_export_vst3!(plugin::RetardedGain);