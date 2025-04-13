use nih_plug::prelude::*;

use ret_gain::RetardedGain;

fn main() {
    nih_export_standalone::<RetardedGain>();
}