use nih_plug::prelude::*;

/// A simple distortion effect
pub struct Distortion {
    /// The amount of distortion to apply (1.0 = none, >1.0 = more distortion)
    drive: f32,
}

impl Distortion {
    /// Create a new distortion effect with the given drive amount
    pub fn new(drive: f32) -> Self {
        Self { drive }
    }
    
    /// Process a single sample through the distortion algorithm
    pub fn process(&self, sample: f32) -> f32 {
        // Simple tanh distortion with drive control
        (sample * self.drive).tanh()
    }
    
    /// Process a buffer of samples through the distortion effect
    pub fn process_buffer(&self, buffer: &mut Buffer) {
        for channel_samples in buffer.iter_samples() {
            for sample in channel_samples {
                *sample = self.process(*sample);
            }
        }
    }
}