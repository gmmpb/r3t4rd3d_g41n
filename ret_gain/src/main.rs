use nih_plug::prelude::*;

use ret_gain::Gain;

fn main() {
    nih_export_standalone::<Gain>();
}