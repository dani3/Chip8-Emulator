extern crate sdl2;

mod core;

use crate::core::Processor;
use crate::core::GraphicsDriver;
use crate::core::InputDriver;

fn main() {
    let sdl_context = sdl2::init().unwrap();

    // Initialize graphics drivers
    let mut graphics_drivers = GraphicsDriver::new(&sdl_context);
    // Initialize the input drivers
    let mut input_drivers = InputDriver::new(&sdl_context);
    // Create the VM
    let mut processor = Processor::new();

    // VM loop
    while let Ok(keypad) = input_drivers.poll() {

    }
}
