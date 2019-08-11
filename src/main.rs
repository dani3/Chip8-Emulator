use std::env;
use colored::*;

mod core;

use crate::core::*;

fn main() {
    // Read the game name
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        println!("{}: no game specified", "Error".red());
        return;
    }

    let game = &args[1];

    println!("{} Chip-8 emulator", "Initializing".green());

    println!("{} SDL2", "Initializing".green());
    let sdl_context = sdl2::init().unwrap();

    // Create the VM
    let mut processor = Processor::new();

    println!("{} drivers", "Initializing".green());
    // Initialize graphics drivers
    let mut graphics_drivers = GraphicsDriver::new(&sdl_context);
    // Initialize the input drivers
    let mut input_drivers = InputDriver::new(&sdl_context);

    // Create the cartridge driver
    println!("{} cartridge", "Reading".green());
    let cartridge_driver = CartridgeDriver::new(&game).unwrap();

    println!("{} {}", "Loading".green(), &game);
    processor.load(&cartridge_driver.get());

    // VM loop
    while let Ok(keypad) = input_drivers.poll() {
        processor.tick(keypad);
    }
}
