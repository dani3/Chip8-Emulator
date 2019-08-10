extern crate sdl2;

mod core;

use crate::core::Processor;
use crate::core::GraphicsDriver;

fn main() {
    let processor = Processor::new();

    let sdl_context = sdl2::init().unwrap();

    let graphics_drivers = GraphicsDriver::new(&sdl_context);

    let mut event_pump = sdl_context.event_pump().unwrap();
    loop {
        for _event in event_pump.poll_iter() {
            // handle user input here
        }
    }
}
