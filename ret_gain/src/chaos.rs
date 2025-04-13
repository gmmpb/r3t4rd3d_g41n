// Import the PI constant from Rust's standard library
use std::f32::consts::PI;

/// A chaotic audio effect based on the Lorenz attractor and other chaotic systems
// This implements an effect based on chaos theory - specifically the Lorenz attractor
// The Lorenz attractor is a set of differential equations that create unpredictable but deterministic patterns
pub struct ChaosAttractor {
    /// Amount of chaos to apply (0.0 to 1.0)
    // Controls how much of the effect is applied to the signal
    chaos_amount: f32,
    
    /// Lorenz attractor state variables
    // These three variables represent the state of the Lorenz system in 3D space
    x: f32,  // x coordinate in the Lorenz system
    y: f32,  // y coordinate in the Lorenz system
    z: f32,  // z coordinate in the Lorenz system
    
    /// Lorenz system parameters
    // These parameters control the behavior of the Lorenz system
    // Different values create different chaotic behaviors
    sigma: f32,  // Controls how quickly the system reacts to differences in x and y
    rho: f32,    // Related to the onset of chaos (critical value around 24.74)
    beta: f32,   // Related to the size and twist of the Lorenz attractor
    
    /// Sample rate for time-based calculations
    // We need to know the sample rate for proper time-based effects
    sample_rate: f32,
    
    /// Time step for the simulation
    // Controls how much the Lorenz system advances with each sample
    // Smaller values give more accurate simulation but require more calculations
    dt: f32,
    
    /// Phase accumulator for secondary modulation
    // Keeps track of phase for additional modulation effects
    phase: f32,
    
    /// Counter for slow evolution of parameters
    // Allows the system parameters to evolve slowly over time for continual variation
    evolution_counter: usize,
}

impl ChaosAttractor {
    /// Create a new chaos attractor effect with the given amount
    // Constructor for the ChaosAttractor effect
    pub fn new(chaos_amount: f32) -> Self {
        // Initialize with standard Lorenz parameters 
        // These are the classic values that produce the butterfly-shaped attractor
        let sigma = 10.0;
        let rho = 28.0;
        let beta = 8.0 / 3.0;
        
        // Create and return a new ChaosAttractor with initial values
        Self {
            chaos_amount,  // Set the amount of chaos effect to apply
            // Start with non-zero values to avoid getting stuck at the origin
            // The origin (0,0,0) is an unstable equilibrium point in the Lorenz system
            x: 0.1,
            y: 0.1,
            z: 0.1,
            sigma,
            rho,
            beta,
            sample_rate: 44100.0, // Default sample rate, will be updated
            dt: 0.001, // Time step for numerical integration
            phase: 0.0, // Start with zero phase
            evolution_counter: 0, // Start counter at zero
        }
    }
    
    /// Set the sample rate for time-based calculations
    // Updates the sample rate and adjusts dependent parameters
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        // Adjust time step based on sample rate to ensure consistent behavior
        // Higher sample rates need smaller time steps for equivalent simulation speed
        self.dt = 0.005 * (44100.0 / sample_rate);
    }
    
    /// Reset the chaotic system to initial conditions
    // Resets the state of the Lorenz system to avoid getting stuck or blowing up
    pub fn reset(&mut self) {
        // Reset to slightly off-center initial conditions
        self.x = 0.1;
        self.y = 0.1;
        self.z = 0.1;
        self.phase = 0.0;
        self.evolution_counter = 0;
    }
    
    /// Update the Lorenz attractor state
    // This is the heart of the chaos effect - it computes one step of the Lorenz equations
    // The Lorenz equations are a simplified model of atmospheric convection
    fn update_lorenz(&mut self, input_influence: f32) {
        // Scale the system variables to keep them in a reasonable range
        // Without scaling, the Lorenz system can produce very large values
        let scale_factor = 0.1;
        let x_scaled = self.x * scale_factor;
        let y_scaled = self.y * scale_factor;
        let z_scaled = self.z * scale_factor;
        
        // Apply input signal influence to the rho parameter
        // This makes the chaos system responsive to the input audio
        let rho_mod = self.rho + (input_influence * 5.0 * self.chaos_amount);
        
        // Calculate derivatives based on the Lorenz system equations
        // These are the three differential equations that define the Lorenz attractor:
        let dx = self.sigma * (y_scaled - x_scaled);  // Rate of change for x
        let dy = x_scaled * (rho_mod - z_scaled) - y_scaled;  // Rate of change for y
        let dz = x_scaled * y_scaled - self.beta * z_scaled;  // Rate of change for z
        
        // Apply Euler integration to update the state
        // Euler integration: new_value = old_value + (rate_of_change * time_step)
        // This is the simplest numerical method for solving differential equations
        self.x += dx * self.dt;
        self.y += dy * self.dt;
        self.z += dz * self.dt;
        
        // Prevent extreme values by clamping
        // This keeps the system stable and prevents digital clipping
        self.x = self.x.clamp(-100.0, 100.0);  // clamp limits a value to a specified range
        self.y = self.y.clamp(-100.0, 100.0);
        self.z = self.z.clamp(-100.0, 100.0);
    }
    
    /// Get a normalized value from the Lorenz system (between -1.0 and 1.0)
    // Converts the 3D Lorenz state into a single audio signal value
    fn get_lorenz_output(&self) -> f32 {
        // Combine the three dimensions into a single signal
        // Normalize each dimension to approximately -1.0 to 1.0 range using tanh
        // tanh naturally limits values to [-1, 1] with a smooth curve
        let x_norm = (self.x / 30.0).tanh();
        let y_norm = (self.y / 30.0).tanh();
        let z_norm = (self.z / 50.0).tanh();
        
        // Mix the three components with different weights
        // This creates a more interesting signal than using just one dimension
        0.5 * x_norm + 0.3 * y_norm + 0.2 * z_norm
    }
    
    /// Slowly evolve the Lorenz parameters over time
    // This prevents the effect from sounding the same over long periods
    fn evolve_parameters(&mut self) {
        // Only update occasionally for performance
        // We don't need to update parameters for every sample
        if self.evolution_counter % 4000 == 0 {
            // Create slow LFOs (Low Frequency Oscillators) for parameter evolution
            // These create slow, cyclic variations in the parameters
            let time = (self.evolution_counter as f32) / (self.sample_rate * 120.0); // 2 minute cycle
            
            // Generate three different slowly varying oscillations with different frequencies
            let sigma_mod = 0.5 * (time * 0.1 * PI).sin();
            let rho_mod = 0.5 * (time * 0.07 * PI).sin();
            let beta_mod = 0.3 * (time * 0.05 * PI).sin();
            
            // Modulate parameters around their standard values
            // The chaos_amount scales how much variation is applied
            self.sigma = 10.0 + (sigma_mod * self.chaos_amount);
            self.rho = 28.0 + (rho_mod * 5.0 * self.chaos_amount);
            self.beta = (8.0 / 3.0) + (beta_mod * self.chaos_amount);
        }
        
        // Increment counter and wrap around at a large value
        // This prevents the counter from overflowing
        self.evolution_counter = (self.evolution_counter + 1) % (self.sample_rate as usize * 600); // 10 minute cycle
    }
    
    /// Process a single sample through the chaos effect
    // This is the main processing function that applies the chaos effect to an audio sample
    pub fn process(&mut self, sample: f32) -> f32 {
        // Early exit if the effect is turned off (optimization)
        if self.chaos_amount <= 0.001 {
            return sample; // Bypass if chaos amount is essentially zero
        }
        
        // Update the phase accumulator for secondary modulation
        // This creates an additional oscillation for modulation effects
        self.phase += 0.001 * (440.0 / self.sample_rate); 
        if self.phase > 1.0 {
            self.phase -= 1.0;  // Wrap phase when it exceeds 1.0
        }
        
        // Update the chaotic system, using the input to influence it
        // This makes the chaos responsive to the input audio
        self.update_lorenz(sample);
        
        // Evolve parameters slowly over time for continual variation
        self.evolve_parameters();
        
        // Get the chaotic output signal from the Lorenz system
        let chaos_signal = self.get_lorenz_output();
        
        // Combine the input with the chaotic signal in different ways
        
        // 1. Amplitude modulation (AM) - varies the volume based on the chaos signal
        // Multiplying signals creates amplitude modulation, producing sidebands
        let am = sample * (1.0 + chaos_signal * self.chaos_amount);
        
        // 2. Frequency modulation (FM) via allpass filter with varying delay
        // This creates frequency modulation effects by varying the phase
        let phase_mod = (self.phase + chaos_signal * 0.01 * self.chaos_amount) * 2.0 * PI;
        let fm = sample * phase_mod.cos() * 0.5;
        
        // 3. Direct addition of shaped chaos
        // Raising to the power of 3 (cubic) adds harmonic content
        let shaped_chaos = chaos_signal.powf(3.0) * self.chaos_amount * 0.3;
        
        // Mix together based on chaos amount
        // Blend the original signal with the processed signal based on chaos_amount
        let result = sample * (1.0 - self.chaos_amount) +  // Original (dry) signal
                     (am * 0.5 + fm * 0.3 + shaped_chaos) * self.chaos_amount;  // Processed (wet) signal
        
        // Apply soft clipping to prevent extreme output values
        // This prevents the output from getting too loud or distorted
        soft_clip(result)
    }
}

/// Soft clipping function to prevent output from exceeding [-1, 1] too harshly
// This limits the signal in a musical way to prevent digital distortion
fn soft_clip(input: f32) -> f32 {
    // The hyperbolic tangent (tanh) function naturally limits values to [-1, 1]
    // It has a smooth S-curve shape that sounds more natural than hard clipping
    input.tanh()
}