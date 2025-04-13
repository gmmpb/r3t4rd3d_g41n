use atomic_float::AtomicF32;
use nih_plug::prelude::{util, Editor};
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use crate::GainParams;

// Define custom colors for our theme
const BACKGROUND_COLOR: Color = Color::rgb(0x11, 0x11, 0x11);
const TEXT_COLOR: Color = Color::rgb(0xF2, 0xF2, 0xF2);
const ACCENT_COLOR: Color = Color::rgb(0xFF, 0x00, 0x66); // Hot pink
const SECONDARY_COLOR: Color = Color::rgb(0x00, 0xE6, 0xFF); // Cyan
const DARKER_BG: Color = Color::rgb(0x0A, 0x0A, 0x0A);

// Get version directly from Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Lens)]
struct Data {
    params: Arc<GainParams>,
    peak_meter: Arc<AtomicF32>,
}

impl Model for Data {}

// Makes sense to also define this here, makes it a bit easier to keep track of
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (320, 280)) // Slightly larger UI
}

pub(crate) fn create(
    params: Arc<GainParams>,
    peak_meter: Arc<AtomicF32>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        // Register custom fonts
        assets::register_noto_sans_thin(cx);
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_bold(cx);
        
        // Set the default font
        cx.set_default_font(&[assets::NOTO_SANS]);
        
        Data {
            params: params.clone(),
            peak_meter: peak_meter.clone(),
        }
        .build(cx);

        VStack::new(cx, |cx| {
            // Main background
            VStack::new(cx, |cx| {
                // Add plugin title with better styling
                Label::new(cx, "R3T4RD3D G41N")
                    .font_size(36.0)
                    .height(Pixels(50.0))
                    .color(TEXT_COLOR)
                    .child_space(Stretch(1.0));
                    
                Label::new(cx, "by VOIDLAB")
                    .font_size(16.0)
                    .height(Pixels(25.0))
                    .color(ACCENT_COLOR)
                    .bottom(Pixels(10.0))
                    .child_space(Stretch(1.0));
                
                // Create a better bordered control panel with more space
                // Apply styling to the VStack itself instead of the context
                VStack::new(cx, |cx| {
                    // Gain control with better spacing and alignment
                    Label::new(cx, "GAIN")
                        .font_size(18.0)
                        .color(SECONDARY_COLOR)
                        .child_space(Stretch(1.0))
                        .bottom(Pixels(8.0));
                        
                    // Make the slider larger and better positioned
                    HStack::new(cx, |cx| {
                        ParamSlider::new(cx, Data::params, |params| &params.gain)
                            .height(Pixels(45.0))
                            .background_color(SECONDARY_COLOR)
                            .border_color(ACCENT_COLOR);
                    })
                    .height(Pixels(50.0))
                    .bottom(Pixels(20.0));

                    // OUTPUT LEVEL section with better spacing
                    Label::new(cx, "OUTPUT LEVEL")
                        .font_size(18.0)
                        .color(ACCENT_COLOR)
                        .child_space(Stretch(1.0))
                        .bottom(Pixels(8.0));
                    
                    // Custom styled peak meter with proper hold time parameter
                    PeakMeter::new(
                        cx,
                        Data::peak_meter
                            .map(|peak_meter| util::gain_to_db(peak_meter.load(Ordering::Relaxed))),
                        Some(Duration::from_millis(600))
                    )
                    .height(Pixels(24.0));
                })
                .background_color(DARKER_BG)
                .border_color(ACCENT_COLOR)
                .border_width(Pixels(2.0))
                .border_radius(Pixels(15.0))
                .child_space(Pixels(25.0))
                .row_between(Pixels(10.0))
                .width(Percentage(80.0))
                .height(Percentage(55.0));
                
                // Add a better styled footer with version
                // Using the constant directly with separate labels
                HStack::new(cx, |cx| {
                    Label::new(cx, "VERSION: ")
                        .color(Color::rgba(200, 200, 200, 150))
                        .font_size(14.0);
                        
                    // Use the VERSION constant directly
                    Label::new(cx, VERSION)
                        .color(Color::rgba(200, 200, 200, 150))
                        .font_size(14.0);
                })
                .top(Pixels(15.0))
                .child_space(Stretch(1.0));
            })
            .background_color(BACKGROUND_COLOR)
            .row_between(Pixels(5.0))
            .child_left(Stretch(1.0))
            .child_right(Stretch(1.0));
        });

        ResizeHandle::new(cx);
    })
}