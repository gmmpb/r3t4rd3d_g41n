use nih_plug::prelude::*;

use aaa_mark_vst::Gain;

fn main() {
    nih_export_standalone::<Gain>();
}