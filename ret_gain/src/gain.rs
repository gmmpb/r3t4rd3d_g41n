/// A simple gain processor module
// This struct is responsible for a single effect: adjusting the volume (gain) of audio samples
pub struct GainProcessor {}

impl GainProcessor {
    /// Create a new gain processor
    // Constructor method that creates a new instance of GainProcessor
    // In Rust, constructors are just regular methods (usually named "new") that return Self
    pub fn new() -> Self {
        // The empty curly braces {} create a new instance of the struct
        // Since our struct has no fields, this is an empty struct
        Self {}
    }
    
    /// Process a sample with gain
    // This method applies gain (volume adjustment) to a single audio sample
    // &self means this method doesn't modify the GainProcessor instance
    // The -> f32 indicates that this method returns a 32-bit floating point number
    pub fn process(&self, sample: f32, gain: f32) -> f32 {
        // Multiply the input sample by the gain factor and return the result
        // In audio, gain is a multiplicative effect (volume adjustment)
        // This is the entire DSP (Digital Signal Processing) algorithm for gain!
        sample * gain
    }
}