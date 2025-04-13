use atomic_float::AtomicF32;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use std::sync::Arc;

use crate::editor;
use crate::distortion::Distortion;
use crate::fractal::FractalMagic;
use crate::chaos::ChaosAttractor;
use crate::gain::GainProcessor;

/// The main plugin structure combining all effects
pub struct RetardedGain {
    params: Arc<RetardedGainParams>,

    /// Needed to normalize the peak meter's response based on the sample rate.
    peak_meter_decay_weight: f32,
    /// The current data for the peak meter. Shared between GUI and audio processing.
    peak_meter: Arc<AtomicF32>,
    
    // The effect processors
    gain_processor: GainProcessor,
    distortion: Distortion,
    fractal_magic: FractalMagic,
    chaos_attractor: ChaosAttractor,
}

#[derive(Params)]
pub struct RetardedGainParams {
    /// The editor state, saved together with the parameter state
    #[persist = "editor-state"]
    pub editor_state: Arc<ViziaState>,

    #[id = "gain"]
    pub gain: FloatParam,
    
    #[id = "drive"]
    pub drive: FloatParam,
    
    #[id = "magic"]
    pub magic: FloatParam,
    
    #[id = "chaos"]
    pub chaos: FloatParam,
}

impl Default for RetardedGain {
    fn default() -> Self {
        let params = Arc::new(RetardedGainParams::default());
        
        Self {
            params: params.clone(),
            peak_meter_decay_weight: 1.0,
            peak_meter: Arc::new(AtomicF32::new(util::MINUS_INFINITY_DB)),
            gain_processor: GainProcessor::new(),
            distortion: Distortion::new(params.drive.default_plain_value()),
            fractal_magic: FractalMagic::new(params.magic.default_plain_value()),
            chaos_attractor: ChaosAttractor::new(params.chaos.default_plain_value()),
        }
    }
}

impl Default for RetardedGainParams {
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

impl Plugin for RetardedGain {
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
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Use a faster decay for the peak meter
        self.peak_meter_decay_weight = 0.5;
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
        
        // Peak meter tracking
        let mut max_peak: f32 = 0.0;
        
        for channel_samples in buffer.iter_samples() {
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
                // Apply effects in sequence
                *sample = self.distortion.process(*sample);
                *sample = self.fractal_magic.process(*sample);
                *sample = self.chaos_attractor.process(*sample);
                *sample = self.gain_processor.process(*sample, gain);
                
                // Track peak level
                max_peak = max_peak.max(sample.abs());
            }
        }
        
        // Update peak meter with decay
        let current_meter = self.peak_meter.load(std::sync::atomic::Ordering::Relaxed);
        let new_meter = if max_peak > current_meter {
            max_peak // Jump to new peak if higher
        } else {
            current_meter * self.peak_meter_decay_weight // Apply decay
        };
        
        self.peak_meter.store(new_meter, std::sync::atomic::Ordering::Relaxed);

        ProcessStatus::Normal
    }
}

impl ClapPlugin for RetardedGain {
    const CLAP_ID: &'static str = "com.weblabstudio.retarded-gain";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A multi-effect audio plugin with gain, distortion, fractal and chaos effects");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for RetardedGain {
    const VST3_CLASS_ID: [u8; 16] = *b"R3T4RD3DG41NWSHU";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}