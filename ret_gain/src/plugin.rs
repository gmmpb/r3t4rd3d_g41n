// Import the AtomicF32 type from the atomic_float crate, which allows safe concurrent access to f32 values
// This is important for sharing data between audio thread and GUI thread
use atomic_float::AtomicF32;

// Import everything (*) from the nih_plug prelude module
// Preludes in Rust contain commonly used types and traits that a library author thinks you'll need
use nih_plug::prelude::*;

// Import the ViziaState struct for handling GUI state
use nih_plug_vizia::ViziaState;

// Import the standard library's Arc (Atomic Reference Counting) type
// Arc allows multiple ownership of the same data across different threads safely
use std::sync::Arc;

// Import our own modules with editor, effects, etc.
use crate::editor;  // 'crate' means "from the current crate (package)"
use crate::distortion::Distortion;  // Import the Distortion struct from distortion.rs
use crate::fractal::FractalMagic;  // Import the FractalMagic struct from fractal.rs
use crate::chaos::ChaosAttractor;  // Import the ChaosAttractor struct from chaos.rs
use crate::gain::GainProcessor;  // Import the GainProcessor struct from gain.rs

/// The main plugin structure combining all effects
// This struct is the central part of our plugin, containing all the data and effect processors
pub struct RetardedGain {
    // Arc<T> is like a shared pointer in C++ or a reference in JavaScript, but thread-safe
    // It allows multiple parts of the code to access the same data without copying it
    params: Arc<RetardedGainParams>,  // Hold all parameter data

    /// Needed to normalize the peak meter's response based on the sample rate.
    // This will be calculated based on the sample rate to make meters decay at a consistent rate
    peak_meter_decay_weight: f32,
    
    /// The current data for the peak meter. Shared between GUI and audio processing.
    // AtomicF32 allows both audio thread and GUI thread to safely access this value
    peak_meter: Arc<AtomicF32>,
    
    // The effect processors - each one handles a specific audio effect
    gain_processor: GainProcessor,  // Controls volume
    distortion: Distortion,  // Adds distortion/saturation 
    fractal_magic: FractalMagic,  // Applies fractal-based effects
    chaos_attractor: ChaosAttractor,  // Applies chaos theory algorithms to sound
}

// The #[derive(Params)] macro automatically implements the Params trait for our struct
// This is similar to decorators in Python or TypeScript - it adds functionality to our type
#[derive(Params)]
pub struct RetardedGainParams {
    /// The editor state, saved together with the parameter state
    // #[persist = "..."] is an attribute that tells the system this field should be saved
    #[persist = "editor-state"]
    pub editor_state: Arc<ViziaState>,  // Holds the GUI state

    // Parameter definitions - each gets a unique ID and stores a single value
    // Similar to props/state in React or properties in a Python class
    #[id = "gain"]  // Unique identifier for this parameter
    pub gain: FloatParam,  // FloatParam is a special type that handles parameter behaviors
    
    #[id = "drive"]
    pub drive: FloatParam,
    
    #[id = "magic"]
    pub magic: FloatParam,
    
    #[id = "chaos"]
    pub chaos: FloatParam,
}

// Implementation block for the RetardedGain struct
// Implements methods and behaviors for the RetardedGain type
// Similar to adding methods to a class in JavaScript or Python
impl Default for RetardedGain {
    // The Default trait provides a way to create a default value for a type
    // Similar to a default constructor in other languages
    fn default() -> Self {
        // Create the parameters with default values
        let params = Arc::new(RetardedGainParams::default());
        
        // Create and return a new RetardedGain instance
        // In Rust, the last expression without a semicolon is implicitly returned
        Self {
            // Clone the Arc to increment the reference count (not copying the actual data)
            params: params.clone(),
            peak_meter_decay_weight: 1.0,
            // Create a new atomic f32 with negative infinity dB as the initial value
            peak_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),
            // Create each effect processor
            gain_processor: GainProcessor::new(),
            // Initialize effects with the default parameter values
            distortion: Distortion::new(params.drive.default_plain_value()),
            fractal_magic: FractalMagic::new(params.magic.default_plain_value()),
            chaos_attractor: ChaosAttractor::new(params.chaos.default_plain_value()),
        }
    }
}

// Default implementation for parameters
// This defines how parameters should be initialized
impl Default for RetardedGainParams {
    fn default() -> Self {
        Self {
            // Get the default editor state
            editor_state: editor::default_state(),

            // Define the gain parameter
            gain: FloatParam::new(
                "Gain",  // Display name
                util::db_to_gain(0.0),  // Default value (0 dB converted to gain ratio)
                // Define the range and behavior of the parameter
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),  // Minimum value (-30 dB)
                    max: util::db_to_gain(30.0),   // Maximum value (+30 dB)
                    // Make the slider movement more natural for dB values
                    // Skewing makes the slider movement more intuitive for logarithmic values
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            // Add a smoother to prevent clicks/pops when parameter changes
            // This is like adding interpolation to parameter changes
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            // Add a unit suffix to the displayed value
            .with_unit(" dB")
            // Convert internal values to display strings
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            // Convert user-entered strings to internal values
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            
            // Define the drive parameter
            drive: FloatParam::new(
                "Drive",
                1.0, // Default value (no distortion)
                FloatRange::Skewed {
                    min: 1.0,    // No distortion
                    max: 50.0,   // Maximum distortion
                    factor: 0.5, // Skew towards lower values for more precise control
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit("x")
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
            
            // Define the magic parameter for fractal effects
            magic: FloatParam::new(
                "Magic One",
                0.0, // Default value (no effect)
                FloatRange::Linear {
                    min: 0.0,    // No effect
                    max: 1.0,    // Full effect
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit("")
            .with_value_to_string(formatters::v2s_f32_percentage(2)),
            
            // Define the chaos parameter
            chaos: FloatParam::new(
                "Chaos",
                0.0, // Default value (no effect)
                FloatRange::Linear {
                    min: 0.0,    // No effect
                    max: 1.0,    // Full effect
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(1)),
        }
    }
}

// Implement the Plugin trait - this defines how our plugin behaves in a host
// Similar to implementing an interface in TypeScript or a protocol in Swift
impl Plugin for RetardedGain {
    // Static constants that define metadata about the plugin
    const NAME: &'static str = "Weblab Studio - R3T4RD3D G41N";
    const VENDOR: &'static str = "Weblab Studio - weblabstudio.hu";
    const URL: &'static str = "";
    const EMAIL: &'static str = "hello@weblabstudio.hu";

    // Get version from Cargo.toml using environment variable
    // env! is a macro that accesses environment variables at compile time
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");    

    // Define the audio input/output configurations supported by this plugin
    // We support both mono and stereo processing
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        // Stereo configuration
        AudioIOLayout {
            // NonZeroU32 is a type that guarantees the value is not zero
            // new() returns an Option, which is like null/undefined but safer
            main_input_channels: NonZeroU32::new(2),  // 2 input channels
            main_output_channels: NonZeroU32::new(2), // 2 output channels
            // The .. syntax means "all other fields keep their default values"
            ..AudioIOLayout::const_default()
        },
        // Mono configuration
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),  // 1 input channel
            main_output_channels: NonZeroU32::new(1), // 1 output channel
            ..AudioIOLayout::const_default()
        },
    ];

    // Whether the plugin can handle sample-accurate automation
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // Associated types (a bit like generics in TypeScript)
    // These are placeholders for types that will be used with this trait
    type SysExMessage = ();  // The () type is like void or None - we don't use SysEx
    type BackgroundTask = (); // We don't use background tasks

    // Return the parameters of this plugin
    // This method gives the host access to the plugin's parameters
    fn params(&self) -> Arc<dyn Params> {
        self.params.clone() // Return a cloned Arc to our parameters
    }

    // Create the editor (GUI) for this plugin
    // Returns an Option, which is like null/undefined but type-safe
    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        // Call the create function from the editor module to create the GUI
        editor::create(
            self.params.clone(),
            self.peak_meter.clone(),
            self.params.editor_state.clone(),
        )
    }

    // Initialize the plugin - called when the plugin is first loaded
    // Returns true if initialization was successful
    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Set a faster decay for the peak meter
        self.peak_meter_decay_weight = 0.5;
        true // Return true to indicate successful initialization
    }

    // Process audio - this is where the actual audio processing happens
    // Called repeatedly by the host with new audio buffers
    fn process(
        &mut self,
        buffer: &mut Buffer,  // The audio buffer with input/output samples
        _aux: &mut AuxiliaryBuffers,  // Additional buffers (not used here)
        context: &mut impl ProcessContext<Self>,  // Context with timing, transport info, etc.
    ) -> ProcessStatus {
        // Update the sample rates for time-based effects
        // Getting the sample rate from the transport info
        self.fractal_magic.set_sample_rate(context.transport().sample_rate as f32);
        self.chaos_attractor.set_sample_rate(context.transport().sample_rate as f32);
        
        // Variable to track the maximum peak value in this processing block
        let mut max_peak: f32 = 0.0;
        
        // Process each set of samples
        // buffer.iter_samples() gives access to all channels of each sample at once
        for channel_samples in buffer.iter_samples() {
            // Get the smoothed parameter values
            // Smoothing prevents clicks/pops when changing parameters
            let gain = self.params.gain.smoothed.next();
            let drive = self.params.drive.smoothed.next();
            let magic = self.params.magic.smoothed.next();
            let chaos = self.params.chaos.smoothed.next();
            
            // Update the effect processors with current parameter values
            self.distortion = Distortion::new(drive);
            self.fractal_magic = FractalMagic::new(magic);
            self.chaos_attractor = ChaosAttractor::new(chaos);
            
            // Process each sample across all channels
            for sample in channel_samples {
                // Apply effects in sequence
                // Each effect processes the output of the previous effect
                *sample = self.distortion.process(*sample);    // Apply distortion
                *sample = self.fractal_magic.process(*sample); // Apply fractal effect
                *sample = self.chaos_attractor.process(*sample); // Apply chaos effect
                *sample = self.gain_processor.process(*sample, gain); // Apply gain
                
                // Track the peak level for the meter
                // abs() gets the absolute value, and max() compares with the current max
                max_peak = max_peak.max(sample.abs());
            }
        }
        
        // Update the peak meter with smoothing/decay
        // First, load the current meter value
        let current_meter = self.peak_meter.load(std::sync::atomic::Ordering::Relaxed);
        
        // Calculate the new meter value:
        // If the new peak is higher, jump to it
        // Otherwise apply decay to the current value
        let new_meter = if max_peak > current_meter {
            max_peak // Jump to new peak if higher
        } else {
            current_meter * self.peak_meter_decay_weight // Apply decay
        };
        
        // Store the new meter value atomically
        // Atomic operations ensure data is safely shared between threads
        self.peak_meter.store(new_meter, std::sync::atomic::Ordering::Relaxed);

        // Return normal status to indicate processing completed successfully
        ProcessStatus::Normal
    }
}

// Implementation for CLAP plugin format support
// CLAP is a newer plugin format with modern features
impl ClapPlugin for RetardedGain {
    // Define the unique ID for this plugin in CLAP hosts
    const CLAP_ID: &'static str = "com.weblabstudio.retarded-gain";
    
    // Description shown in CLAP hosts
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A multi-effect audio plugin with gain, distortion, fractal and chaos effects");
    
    // URL for manual/documentation
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    
    // URL for support
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    
    // Features/capabilities of this plugin in CLAP format
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect, // It's an audio effect
        ClapFeature::Stereo,      // Supports stereo processing
        ClapFeature::Mono,        // Supports mono processing
        ClapFeature::Utility,     // It's a utility plugin
    ];
}

// Implementation for VST3 plugin format support
// VST3 is a widely-used plugin format by Steinberg
impl Vst3Plugin for RetardedGain {
    // Unique class ID for VST3 - must be 16 bytes
    // The *b prefix creates a byte array from a string
    const VST3_CLASS_ID: [u8; 16] = *b"R3T4RD3DG41NWSHU";
    
    // Categories for this plugin in VST3 hosts
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}