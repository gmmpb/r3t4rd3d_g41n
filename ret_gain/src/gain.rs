use atomic_float::AtomicF32;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::Arc;

use crate::editor;
use crate::distortion::Distortion;
use crate::fractal::FractalMagic;
use crate::chaos::ChaosAttractor;

/// The time it takes for the peak meter to decay by 12 dB after switching to complete silence.
const PEAK_METER_DECAY_MS: f64 = 150.0;

/// This is mostly identical to the gain example, minus some fluff, and with a GUI.
pub struct Gain {
    params: Arc<GainParams>,

    /// Needed to normalize the peak meter's response based on the sample rate.
    peak_meter_decay_weight: f32,
    /// The current data for the peak meter. This is stored as an [`Arc`] so we can share it between
    /// the GUI and the audio processing parts. If you have more state to share, then it's a good
    /// idea to put all of that in a struct behind a single `Arc`.
    ///
    /// This is stored as voltage gain.
    peak_meter: Arc<AtomicF32>,
    distortion: Distortion, // The distortion processor
    fractal_magic: FractalMagic, // The fractal magic processor
    chaos_attractor: ChaosAttractor, // The chaos attractor processor
}

#[derive(Params)]
pub struct GainParams {
    /// The editor state, saved together with the parameter state so the custom scaling can be
    /// restored.
    #[persist = "editor-state"]
    pub editor_state: Arc<ViziaState>,

    #[id = "gain"]
    pub gain: FloatParam,
    
    #[id = "drive"]
    pub drive: FloatParam, // The drive parameter
    
    #[id = "magic"]
    pub magic: FloatParam, // The magic parameter
    
    #[id = "chaos"]
    pub chaos: FloatParam, // Add the chaos parameter
}

impl Default for Gain {
    fn default() -> Self {
        Self {
            params: Arc::new(GainParams::default()),

            peak_meter_decay_weight: 1.0,
            peak_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),
            distortion: Distortion::new(1.0), // Initialize with no distortion
            fractal_magic: FractalMagic::new(0.0), // Initialize with no fractal magic
            chaos_attractor: ChaosAttractor::new(0.0), // Initialize with no chaos
        }
    }
}

impl Default for GainParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            
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

impl Plugin for Gain {
    const NAME: &'static str = "Weblab Studio - R3T4RD3D G41N";
    const VENDOR: &'static str = "Weblab Studio - weblabstudio.hu";
    const URL: &'static str = "";
    const EMAIL: &'static str = "hello@weblabstudio.hu";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");    

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.peak_meter.clone(),
            self.params.editor_state.clone(),
        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // After `PEAK_METER_DECAY_MS` milliseconds of pure silence, the peak meter's value should
        // have dropped by 12 dB
        self.peak_meter_decay_weight = 0.25f64
            .powf((buffer_config.sample_rate as f64 * PEAK_METER_DECAY_MS / 1000.0).recip())
            as f32;

        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Update the sample rates for time-based effects
        self.fractal_magic.set_sample_rate(context.transport().sample_rate as f32);
        self.chaos_attractor.set_sample_rate(context.transport().sample_rate as f32);
        
        for channel_samples in buffer.iter_samples() {
            let mut amplitude = 0.0;
            let num_samples = channel_samples.len();

            // Get the smoothed parameter values
            let gain = self.params.gain.smoothed.next();
            let drive = self.params.drive.smoothed.next();
            let magic = self.params.magic.smoothed.next();
            let chaos = self.params.chaos.smoothed.next();
            
            // Update the effect processors with current parameter values
            self.distortion = Distortion::new(drive);
            self.fractal_magic = FractalMagic::new(magic);
            self.chaos_attractor = ChaosAttractor::new(chaos);
            
            for sample in channel_samples {
                // Apply effects in sequence:
                
                // 1. First apply the distortion
                *sample = self.distortion.process(*sample);
                
                // 2. Then apply the fractal magic effect
                *sample = self.fractal_magic.process(*sample);
                
                // 3. Then apply the chaos attractor effect
                *sample = self.chaos_attractor.process(*sample);
                
                // 4. Finally apply the gain
                *sample *= gain;
                
                amplitude += *sample;
            }

            // To save resources, a plugin can (and probably should!) only perform expensive
            // calculations that are only displayed on the GUI while the GUI is open
            if self.params.editor_state.is_open() {
                amplitude = (amplitude / num_samples as f32).abs();
                let current_peak_meter = self.peak_meter.load(std::sync::atomic::Ordering::Relaxed);
                let new_peak_meter = if amplitude > current_peak_meter {
                    amplitude
                } else {
                    current_peak_meter * self.peak_meter_decay_weight
                        + amplitude * (1.0 - self.peak_meter_decay_weight)
                };

                self.peak_meter
                    .store(new_peak_meter, std::sync::atomic::Ordering::Relaxed)
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Gain {
    const CLAP_ID: &'static str = "com.moist-plugins-gmbh.gain-gui-vizia";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A smoothed gain parameter example plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for Gain {
    const VST3_CLASS_ID: [u8; 16] = *b"GainGuiVIIIZIAAA";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}