// Import the NIH-plug prelude for audio processing types and traits
use nih_plug::prelude::*;

/// A simple distortion effect
// This struct implements a basic waveshaping distortion effect
pub struct Distortion {
    /// The amount of distortion to apply (1.0 = none, >1.0 = more distortion)
    // The drive parameter controls how much the signal is pushed before distortion
    // Higher values create more harmonics and a more aggressive sound
    drive: f32,
}

impl Distortion {
    /// Create a new distortion effect with the given drive amount
    // Constructor for the Distortion effect
    // Notice how in Rust we use 'Self' (capital S) to refer to the current type within an impl block
    pub fn new(drive: f32) -> Self {
        // Create a new instance with the specified drive amount
        // This syntax is creating a struct with named fields
        Self { drive }  // Shorthand for drive: drive
    }

    
    /// Process a single sample through the distortion algorithm
    // This is where the actual distortion effect happens
    // &self means this method takes an immutable reference to the struct instance
    pub fn process(&self, sample: f32) -> f32 {
        // Simple tanh distortion with drive control
        // 1. Multiply the input sample by the drive amount (makes signal stronger)
        // 2. Apply the hyperbolic tangent function (tanh) which "clips" the signal in a smooth way
        // This creates a "soft clipping" effect - a key part of many distortion/overdrive effects
        (sample * self.drive).tanh()
    }
    
    /// Process a buffer of samples through the distortion effect
    // This method processes an entire buffer of audio at once
    // This is a convenience method for processing multiple samples
    pub fn process_buffer(&self, buffer: &mut Buffer) {
        // Iterate through each set of samples across all channels
        for channel_samples in buffer.iter_samples() {
            // For each sample in the current frame
            for sample in channel_samples {
                // Apply distortion and write the result back to the same location
                // The * before sample is dereferencing the pointer to modify the original value
                // This is a key difference from JavaScript/Python - we're modifying the original data
                *sample = self.process(*sample);
            }
        }
    }
}