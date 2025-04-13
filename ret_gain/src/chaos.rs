use std::f32::consts::PI;

/// A chaotic audio effect based on the Lorenz attractor and other chaotic systems
pub struct ChaosAttractor {
    /// Amount of chaos to apply (0.0 to 1.0)
    chaos_amount: f32,
    
    /// Lorenz attractor state variables
    x: f32,
    y: f32,
    z: f32,
    
    /// Lorenz system parameters
    sigma: f32,
    rho: f32,
    beta: f32,
    
    /// Sample rate for time-based calculations
    sample_rate: f32,
    
    /// Time step for the simulation
    dt: f32,
    
    /// Phase accumulator for secondary modulation
    phase: f32,
    
    /// Counter for slow evolution of parameters
    evolution_counter: usize,
}

impl ChaosAttractor {
    /// Create a new chaos attractor effect with the given amount
    pub fn new(chaos_amount: f32) -> Self {
        // Initialize with standard Lorenz parameters
        let sigma = 10.0;
        let rho = 28.0;
        let beta = 8.0 / 3.0;
        
        Self {
            chaos_amount,
            // Start with non-zero values to avoid getting stuck at the origin
            x: 0.1,
            y: 0.1,
            z: 0.1,
            sigma,
            rho,
            beta,
            sample_rate: 44100.0, // Default, will be updated
            dt: 0.001, // Time step for numerical integration
            phase: 0.0,
            evolution_counter: 0,
        }
    }
    
    /// Set the sample rate for time-based calculations
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        // Adjust time step based on sample rate
        self.dt = 0.005 * (44100.0 / sample_rate);
    }
    
    /// Reset the chaotic system to initial conditions
    pub fn reset(&mut self) {
        self.x = 0.1;
        self.y = 0.1;
        self.z = 0.1;
        self.phase = 0.0;
        self.evolution_counter = 0;
    }
    
    /// Update the Lorenz attractor state
    fn update_lorenz(&mut self, input_influence: f32) {
        // Scale the system variables to keep them in a reasonable range
        let scale_factor = 0.1;
        let x_scaled = self.x * scale_factor;
        let y_scaled = self.y * scale_factor;
        let z_scaled = self.z * scale_factor;
        
        // Apply input signal influence to the rho parameter
        let rho_mod = self.rho + (input_influence * 5.0 * self.chaos_amount);
        
        // Calculate derivatives based on the Lorenz system
        let dx = self.sigma * (y_scaled - x_scaled);
        let dy = x_scaled * (rho_mod - z_scaled) - y_scaled;
        let dz = x_scaled * y_scaled - self.beta * z_scaled;
        
        // Apply Euler integration
        self.x += dx * self.dt;
        self.y += dy * self.dt;
        self.z += dz * self.dt;
        
        // Prevent extreme values by clamping
        self.x = self.x.clamp(-100.0, 100.0);
        self.y = self.y.clamp(-100.0, 100.0);
        self.z = self.z.clamp(-100.0, 100.0);
    }
    
    /// Get a normalized value from the Lorenz system (between -1.0 and 1.0)
    fn get_lorenz_output(&self) -> f32 {
        // Combine the three dimensions into a single signal
        // Normalize to -1.0 to 1.0 range
        let x_norm = (self.x / 30.0).tanh();
        let y_norm = (self.y / 30.0).tanh();
        let z_norm = (self.z / 50.0).tanh();
        
        // Mix the three components
        0.5 * x_norm + 0.3 * y_norm + 0.2 * z_norm
    }
    
    /// Slowly evolve the Lorenz parameters over time
    fn evolve_parameters(&mut self) {
        // Only update occasionally for performance
        if self.evolution_counter % 4000 == 0 {
            // Create slow LFOs for parameter evolution
            let time = (self.evolution_counter as f32) / (self.sample_rate * 120.0); // 2 minute cycle
            let sigma_mod = 0.5 * (time * 0.1 * PI).sin();
            let rho_mod = 0.5 * (time * 0.07 * PI).sin();
            let beta_mod = 0.3 * (time * 0.05 * PI).sin();
            
            // Modulate parameters around their standard values
            self.sigma = 10.0 + (sigma_mod * self.chaos_amount);
            self.rho = 28.0 + (rho_mod * 5.0 * self.chaos_amount);
            self.beta = (8.0 / 3.0) + (beta_mod * self.chaos_amount);
        }
        
        // Increment counter and wrap around at a large value
        self.evolution_counter = (self.evolution_counter + 1) % (self.sample_rate as usize * 600); // 10 minute cycle
    }
    
    /// Process a single sample through the chaos effect
    pub fn process(&mut self, sample: f32) -> f32 {
        if self.chaos_amount <= 0.001 {
            return sample; // Bypass if chaos amount is essentially zero
        }
        
        // Update the phase accumulator for secondary modulation
        self.phase += 0.001 * (440.0 / self.sample_rate); 
        if self.phase > 1.0 {
            self.phase -= 1.0;
        }
        
        // Update the chaotic system, using the input to influence it
        self.update_lorenz(sample);
        
        // Evolve parameters slowly over time
        self.evolve_parameters();
        
        // Get the chaotic output signal
        let chaos_signal = self.get_lorenz_output();
        
        // Combine the input with the chaotic signal in different ways
        
        // 1. Amplitude modulation
        let am = sample * (1.0 + chaos_signal * self.chaos_amount);
        
        // 2. Frequency modulation (via allpass filter with varying delay)
        let phase_mod = (self.phase + chaos_signal * 0.01 * self.chaos_amount) * 2.0 * PI;
        let fm = sample * phase_mod.cos() * 0.5;
        
        // 3. Direct addition of shaped chaos
        let shaped_chaos = chaos_signal.powf(3.0) * self.chaos_amount * 0.3;
        
        // Mix together based on chaos amount
        let result = sample * (1.0 - self.chaos_amount) + 
                     (am * 0.5 + fm * 0.3 + shaped_chaos) * self.chaos_amount;
        
        // Apply soft clipping to prevent extreme output values
        soft_clip(result)
    }
}

/// Soft clipping function to prevent output from exceeding [-1, 1] too harshly
fn soft_clip(input: f32) -> f32 {
    input.tanh()
}