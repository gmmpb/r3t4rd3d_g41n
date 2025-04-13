use nih_plug::prelude::*;
use std::f32::consts::PI;

/// A complex fractal-based audio effect that combines fractal patterns with non-linear wave-shaping
pub struct FractalMagic {
    /// The amount of "magic" to apply (0.0 to 1.0)
    magic_amount: f32,
    /// Internal state for creating evolving patterns
    z_real: f32,
    z_imag: f32,
    /// Sample rate for time-based calculations
    sample_rate: f32,
    /// Sample counter for evolving patterns
    sample_counter: usize,
}

impl FractalMagic {
    /// Create a new fractal magic effect with the given amount
    pub fn new(magic_amount: f32) -> Self {
        Self {
            magic_amount,
            z_real: 0.0,
            z_imag: 0.0,
            sample_rate: 44100.0, // Default, will be updated
            sample_counter: 0,
        }
    }

    /// Set the sample rate for time-based calculations
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    /// Reset the internal state
    pub fn reset(&mut self) {
        self.z_real = 0.0;
        self.z_imag = 0.0;
        self.sample_counter = 0;
    }
    
    /// Process a single sample through the fractal magic algorithm
    pub fn process(&mut self, sample: f32) -> f32 {
        if self.magic_amount <= 0.001 {
            return sample; // Bypass if magic amount is essentially zero
        }

        // Scale the magic amount for different aspects of the effect
        let fractal_strength = self.magic_amount * 2.5;
        let fold_strength = self.magic_amount * 3.0;
        let feedback_amount = self.magic_amount * 0.7;
        
        // Update the fractal state - using a modified Julia set iteration
        // The input sample modulates the fractal parameters
        let c_real = 0.285 + 0.01 * (sample * fractal_strength).sin();
        let c_imag = 0.01 + 0.01 * (sample * fractal_strength).cos();
        
        // Update z based on previous z and the current sample
        let temp_real = self.z_real;
        let temp_imag = self.z_imag;
        
        // z = z² + c + sample_influence
        self.z_real = temp_real * temp_real - temp_imag * temp_imag + c_real + sample * 0.1;
        self.z_imag = 2.0 * temp_real * temp_imag + c_imag;
        
        // Prevent explosions by clamping
        if self.z_real.abs() > 4.0 || self.z_imag.abs() > 4.0 {
            self.z_real *= 0.5;
            self.z_imag *= 0.5;
        }
        
        // Add slow LFO modulation based on sample count
        let lfo_freq = 0.1; // Very slow modulation
        let lfo_phase = (self.sample_counter as f32 / self.sample_rate) * lfo_freq * 2.0 * PI;
        let lfo_value = lfo_phase.sin() * 0.2;
        
        // Wave folding for harmonic richness
        let folded = wave_fold(sample + lfo_value, fold_strength);
        
        // Combine original, fractal modulation, and folded signal
        let result = sample * (1.0 - self.magic_amount) +
                     (self.z_real * 0.2 * fractal_strength + folded) * self.magic_amount;
        
        // Apply feedback with tanh limiting
        let with_feedback = result + feedback_amount * self.z_real.tanh();
        
        // Increment counter for time-based modulation
        self.sample_counter = (self.sample_counter + 1) % (self.sample_rate as usize * 60); // Reset after 1 minute
        
        with_feedback
    }
    
    /// Process a buffer of samples through the fractal magic effect
    pub fn process_buffer(&mut self, buffer: &mut Buffer) {
        for channel_samples in buffer.iter_samples() {
            for sample in channel_samples {
                *sample = self.process(*sample);
            }
        }
    }
}

/// Wave folding function that creates harmonic content when the signal exceeds a threshold
fn wave_fold(input: f32, fold_amount: f32) -> f32 {
    if fold_amount <= 0.0 {
        return input;
    }
    
    // The threshold after which folding begins to occur
    let threshold = 1.0 / fold_amount;
    
    // Simple folding algorithm that reflects the signal when it exceeds the threshold
    if input > threshold {
        return 2.0 * threshold - input;
    } else if input < -threshold {
        return -2.0 * threshold - input;
    }
    
    input
}