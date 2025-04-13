use atomic_float::AtomicF32;
use nih_plug::prelude::{util, Editor};
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use crate::gain::GainParams;

// More refined color palette - professional but still distinctive
const BACKGROUND_COLOR: Color = Color::rgb(0x18, 0x18, 0x1E); // Dark background with slight blue tint
const TEXT_COLOR: Color = Color::rgb(0xE8, 0xE9, 0xF3); // Soft white
const ACCENT_COLOR: Color = Color::rgb(0xFF, 0x1A, 0x8C); // Softened pink
const SECONDARY_COLOR: Color = Color::rgb(0x0A, 0xD8, 0xE9); // Cyan for contrast
const MAGIC_COLOR: Color = Color::rgb(0x9B, 0x59, 0xB6); // Purple for the magic slider
const KNOB_BG_COLOR: Color = Color::rgb(0x22, 0x22, 0x2A); // Slight contrast for controls
const PANEL_BG: Color = Color::rgb(0x20, 0x20, 0x28); // Panel background

// Semi-transparent colors
const BORDER_COLOR: Color = Color::rgba(0xFF, 0x1A, 0x8C, 0x30); // Very subtle borders
const METER_BG_COLOR: Color = Color::rgba(0x0A, 0x0A, 0x10, 0x80); // Dark meter background
const TEXT_SECONDARY: Color = Color::rgba(0xE8, 0xE9, 0xF3, 0x70); // Secondary text

// Get version directly from Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Lens)]
struct Data {
    params: Arc<GainParams>,
    peak_meter: Arc<AtomicF32>,
}

impl Model for Data {}

// Adjusted window size to accommodate the new control
pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (380, 280)) // Increase height for new slider
}

pub(crate) fn create(
    params: Arc<GainParams>,
    peak_meter: Arc<AtomicF32>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        // Register fonts
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

        // Main container
        VStack::new(cx, |cx| {
            // Header with plugin name and brand - better aligned
            HStack::new(cx, |cx| {
                VStack::new(cx, |cx| {
                    Label::new(cx, "R3T4RD3D G41N")
                        .font_size(22.0)
                        .color(TEXT_COLOR);
                        
             
                })
                .child_space(Stretch(1.0));
            })
            .height(Pixels(42.0))
            .child_left(Pixels(15.0))
            .child_right(Pixels(15.0))
            .width(Percentage(95.0))
            .background_color(KNOB_BG_COLOR)
            .border_color(BORDER_COLOR)
            .border_width(Pixels(1.0))
            .border_radius(Pixels(4.0))
            .bottom(Pixels(10.0));
            
            // Main controls section with better organization
            VStack::new(cx, |cx| {
                // GAIN with better positioned elements
                HStack::new(cx, |cx| {
                    Label::new(cx, "GAIN")
                        .font_size(14.0)
                        .color(SECONDARY_COLOR)
                        .width(Percentage(15.0))
                        .child_space(Stretch(1.0));
                        
                    ParamSlider::new(cx, Data::params, |params| &params.gain)
                        .width(Percentage(75.0))
                        .height(Pixels(20.0))
                        .color(SECONDARY_COLOR)
                        .top(Pixels(5.0))
                        .font_size(13.0);
                })
                .height(Pixels(30.0))
                .child_left(Pixels(15.0))
                .child_right(Pixels(15.0))
                .width(Percentage(95.0))
                .background_color(PANEL_BG)
                .border_color(BORDER_COLOR)
                .border_width(Pixels(1.0))
                .border_radius(Pixels(4.0))
                .bottom(Pixels(8.0));
        
                // DISTORTION with better alignment
                HStack::new(cx, |cx| {
                    Label::new(cx, "DIST")
                        .font_size(14.0)
                        .color(ACCENT_COLOR)
                        .width(Percentage(15.0))
                        .child_space(Stretch(1.0));
                        
                    ParamSlider::new(cx, Data::params, |params| &params.drive)
                        .width(Percentage(75.0))
                        .height(Pixels(20.0))
                        .top(Pixels(5.0))
                        .color(ACCENT_COLOR)
                        .font_size(13.0);
                })
                .height(Pixels(30.0))
                .child_left(Pixels(15.0))
                .child_right(Pixels(15.0))
                .width(Percentage(95.0))
                .background_color(PANEL_BG)
                .border_color(BORDER_COLOR)
                .border_width(Pixels(1.0))
                .border_radius(Pixels(4.0))
                .bottom(Pixels(8.0));
                
                // MAGIC ONE - new slider for fractal algorithm
                HStack::new(cx, |cx| {
                    Label::new(cx, "MAGIC")
                        .font_size(14.0)
                        .color(MAGIC_COLOR)
                        .width(Percentage(15.0))
                        .child_space(Stretch(1.0));
                        
                    ParamSlider::new(cx, Data::params, |params| &params.magic)
                        .width(Percentage(75.0))
                        .height(Pixels(20.0))
                        .top(Pixels(5.0))
                        .color(MAGIC_COLOR)
                        .font_size(13.0);
                })
                .height(Pixels(30.0))
                .child_left(Pixels(15.0))
                .child_right(Pixels(15.0))
                .width(Percentage(95.0))
                .background_color(PANEL_BG)
                .border_color(BORDER_COLOR)
                .border_width(Pixels(1.0))
                .border_radius(Pixels(4.0))
                .bottom(Pixels(8.0));

                // OUTPUT METER with improved styling
                VStack::new(cx, |cx| {
                    Label::new(cx, "OUTPUT LEVEL")
                        .font_size(14.0)
                        .color(SECONDARY_COLOR)
                        .bottom(Pixels(4.0));
                    
                    // Improved peak meter
                    PeakMeter::new(
                        cx,
                        Data::peak_meter
                            .map(|peak_meter| util::gain_to_db(peak_meter.load(Ordering::Relaxed))),
                        Some(Duration::from_millis(600))
                    )
                    .height(Pixels(12.0))
                    .width(Percentage(90.0))
                    .background_color(METER_BG_COLOR);
                })
                .height(Pixels(48.0))
                .child_left(Pixels(15.0))
                .child_right(Pixels(15.0))
                .width(Percentage(95.0))
                .background_color(PANEL_BG)
                .border_color(BORDER_COLOR)
                .border_width(Pixels(1.0))
                .border_radius(Pixels(4.0));
            })
            .child_top(Pixels(0.0))
            .width(Percentage(100.0))
            .height(Pixels(170.0)); // Increased height for the new slider
            
            // Footer with version info
            HStack::new(cx, |cx| {
                Label::new(cx, &format!("v{VERSION}"))
                    .color(TEXT_SECONDARY)
                    .font_size(11.0);
                    
                Label::new(cx, "Mark Gemesi - weblabstudio.hu © 2025")
                    .left(Pixels(10.0))
                    .color(TEXT_SECONDARY)
                    .font_size(11.0);
            })
            .height(Pixels(24.0))
            .child_left(Pixels(15.0))
            .child_right(Pixels(15.0))
            .top(Pixels(8.0))
            .width(Percentage(100.0));
        })
        .background_color(BACKGROUND_COLOR)
        .child_top(Pixels(10.0))
        .child_bottom(Pixels(10.0))
        .child_left(Pixels(10.0))
        .child_right(Pixels(10.0));
    })
}