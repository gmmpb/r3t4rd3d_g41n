// Import the NIH-plug prelude for audio processing types and traits
use nih_plug::prelude::*;
// Import PI constant from the standard library
use std::f32::consts::PI;

/// A complex fractal-based audio effect that combines fractal patterns with non-linear wave-shaping
// This struct implements a creative effect based on fractal mathematics
pub struct FractalMagic {
    /// The amount of "magic" to apply (0.0 to 1.0)
    // Controls how much of the effect is applied to the signal
    magic_amount: f32,
    
    /// Internal state for creating evolving patterns
    // These track the state of our fractal calculation, similar to complex numbers
    // In fractal math, complex numbers (with real and imaginary parts) are common
    z_real: f32,  // Real part of our complex number z
    z_imag: f32,  // Imaginary part of our complex number z
    
    /// Sample rate for time-based calculations
    // We need to know the sample rate to create time-based effects properly
    sample_rate: f32,
    
    /// Sample counter for evolving patterns
    // Keeps track of how many samples we've processed for time-based evolution
    sample_counter: usize,  // usize is an unsigned integer sized for the platform (32 or 64 bit)
    
    /// Smoothing factor for release/decay
    // Controls how quickly the effect decays when input decreases
    release_smoothing: f32,
    
    /// Previous output value for smoothing
    // Used to create smooth transitions between processed samples
    prev_output: f32,
}

impl FractalMagic {
    /// Create a new fractal magic effect with the given amount
    // Constructor for the FractalMagic effect
    pub fn new(magic_amount: f32) -> Self {
        // Create and return a new instance with initial values
        Self {
            magic_amount,          // The amount of effect to apply
            z_real: 0.0,           // Start with a zero state
            z_imag: 0.0,           // Start with a zero state
            sample_rate: 44100.0,  // Default sample rate, will be updated later
            sample_counter: 0,     // Start with counter at 0
            release_smoothing: 0.9995, // High value for smooth release (close to 1.0)
            prev_output: 0.0,      // Start with previous output at 0
        }
    }

    /// Set the sample rate for time-based calculations
    // This method updates the sample rate and recalculates dependent values
    // &mut self means this method can modify the struct (mutable reference)
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        
        // Adjust release smoothing based on sample rate
        // This ensures the effect behaves consistently at different sample rates
        // powf raises the base number to the specified power
        self.release_smoothing = 0.9995f32.powf(44100.0 / sample_rate);
    }

    /// Reset the internal state
    // Clears the internal state of the effect
    pub fn reset(&mut self) {
        self.z_real = 0.0;
        self.z_imag = 0.0;
        self.sample_counter = 0;
        self.prev_output = 0.0;
    }
    
    /// Process a single sample through the fractal magic algorithm
    // This is where the magic happens! The main DSP method.
    pub fn process(&mut self, sample: f32) -> f32 {
        // Early exit if the effect is turned off (optimization)
        if self.magic_amount <= 0.001 {
            return sample; // Bypass if magic amount is essentially zero
        }

        // Scale the magic amount for different aspects of the effect
        // Each aspect of the effect responds differently to the magic amount
        let fractal_strength = self.magic_amount * 2.0; // Reduced from 2.5
        let fold_strength = self.magic_amount * 2.5;    // Reduced from 3.0
        let feedback_amount = self.magic_amount * 0.4;  // Reduced from 0.7
        
        // Update the fractal state - using a modified Julia set iteration
        // The Julia set is a famous fractal in mathematics
        // The input sample modulates the fractal parameters for audio-responsive behavior
        let c_real = 0.285 + 0.01 * (sample * fractal_strength).sin();
        let c_imag = 0.01 + 0.01 * (sample * fractal_strength).cos();
        
        // Store the current z values temporarily
        let temp_real = self.z_real;
        let temp_imag = self.z_imag;
        
        // z = z² + c + sample_influence
        // This is the core of the Julia set fractal formula, with audio input
        // For complex number z², we calculate (a+bi)² = a² - b² + 2abi
        self.z_real = temp_real * temp_real - temp_imag * temp_imag + c_real + sample * 0.1;
        self.z_imag = 2.0 * temp_real * temp_imag + c_imag;
        
        // Better state management to prevent explosions
        // If the values get too large, scale them back to prevent the effect from getting out of control
        if self.z_real.abs() > 2.0 || self.z_imag.abs() > 2.0 {
            self.z_real *= 0.5;
            self.z_imag *= 0.5;
        }
        
        // Add slow LFO modulation based on sample count
        // LFO = Low Frequency Oscillator - adds movement to the sound
        let lfo_freq = 0.1; // Very slow modulation - 0.1 Hz
        
        // Calculate the phase of the LFO based on sample count and rate
        // This converts our sample counter to a phase angle for the sine wave
        let lfo_phase = (self.sample_counter as f32 / self.sample_rate) * lfo_freq * 2.0 * PI;
        
        // Calculate the actual LFO value using sine
        let lfo_value = lfo_phase.sin() * 0.1; // Reduced amplitude from 0.2
        
        // Wave folding for harmonic richness
        // Wave folding is a technique that "folds" the waveform back on itself,
        // creating interesting harmonics (frequencies not in the original sound)
        let folded = wave_fold(sample + lfo_value, fold_strength);
        
        // Combine original, fractal modulation, and folded signal
        // This blends the dry signal with the processed signal based on magic_amount
        let result = sample * (1.0 - self.magic_amount) +  // Dry signal
                     (self.z_real * 0.2 * fractal_strength + folded) * self.magic_amount; // Wet signal
        
        // Apply feedback with tanh limiting and reduced feedback
        // Feedback means feeding part of the output back into the algorithm
        // tanh limits the feedback to prevent it from growing out of control
        let with_feedback = result + feedback_amount * self.z_real.tanh();
        
        // Apply smoothing for better release behavior
        // Fast attack, slow release is a common pattern in audio effects
        let smoothed = if with_feedback.abs() > self.prev_output.abs() {
            // Fast attack - immediately jump to new value when it's larger
            with_feedback
        } else {
            // Smooth release - gradually decrease when value gets smaller
            // This is a weighted average between new and previous values
            with_feedback * (1.0 - self.release_smoothing) + self.prev_output * self.release_smoothing
        };
        
        // Hard limit to ensure output stays in bounds
        // This prevents the effect from producing samples that are too loud
        let limited = soft_clip(smoothed);
        
        // Increment counter for time-based modulation
        // The modulo (%) operator ensures the counter wraps around after 1 minute
        self.sample_counter = (self.sample_counter + 1) % (self.sample_rate as usize * 60); // Reset after 1 minute
        
        // Store for next iteration - this is used for smoothing
        self.prev_output = limited;
        
        // Return the processed sample
        limited
    }
    
    /// Process a buffer of samples through the fractal magic effect
    // Convenience method to process an entire buffer at once
    pub fn process_buffer(&mut self, buffer: &mut Buffer) {
        // Iterate through each set of samples across all channels
        for channel_samples in buffer.iter_samples() {
            // For each sample in the current frame
            for sample in channel_samples {
                // Process the sample and write back to the buffer in-place
                *sample = self.process(*sample);
            }
        }
    }
}

/// Wave folding function that creates harmonic content when the signal exceeds a threshold
// This is a separate function (not a method) that implements the wavefolder algorithm
// In Rust, functions don't need to be part of a struct/class
fn wave_fold(input: f32, fold_amount: f32) -> f32 {
    // Early exit if no folding is needed
    if fold_amount <= 0.0 {
        return input;
    }
    
    // The threshold after which folding begins to occur
    // As fold_amount increases, the threshold decreases, creating more folding
    let threshold = 1.0 / fold_amount;
    
    // Simple folding algorithm that reflects the signal when it exceeds the threshold
    // This creates a "fold" in the waveform, adding harmonics
    if input > threshold {
        // Reflect around the threshold line (like a mirror)
        return 2.0 * threshold - input;
    } else if input < -threshold {
        // Reflect around the negative threshold line
        return -2.0 * threshold - input;
    }
    
    // If within thresholds, return the original input
    input
}

/// Soft clipper to ensure output stays within reasonable bounds
// Another utility function that prevents the output from getting too loud
fn soft_clip(input: f32) -> f32 {
    // Hyperbolic tangent provides a smooth, musical limiting
    // tanh approaches ±1 as the input approaches ±∞, creating a smooth curve
    // This sounds more musical than hard clipping, which would create harsh distortion
    input.tanh()
}