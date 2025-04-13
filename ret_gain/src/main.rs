// Import everything from the nih_plug prelude module
// This gives us access to all the common types and functions we need
use nih_plug::prelude::*;

// Import the RetardedGain struct from our ret_gain crate
// This is the main plugin structure we defined in plugin.rs
use ret_gain::RetardedGain;

// Main function - the entry point for the standalone application
// Similar to main() in other languages like C/C++, Python, etc.
fn main() {
    // Call the nih_export_standalone macro to create a standalone version of our plugin
    // This allows the plugin to run as a normal desktop application
    // The ::<RetardedGain> part is using a "turbofish" syntax to specify the generic type
    nih_export_standalone::<RetardedGain>();
}