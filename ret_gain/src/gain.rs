/// A simple gain processor module
pub struct GainProcessor {}

impl GainProcessor {
    /// Create a new gain processor
    pub fn new() -> Self {
        Self {}
    }
    
    /// Process a sample with gain
    pub fn process(&self, sample: f32, gain: f32) -> f32 {
        sample * gain
    }
}