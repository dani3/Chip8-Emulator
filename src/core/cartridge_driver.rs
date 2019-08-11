use std::io;
use std::fs::File;
use std::io::prelude::*;

pub struct CartridgeDriver {
    cartridge_content: Vec<u8>
}

impl CartridgeDriver {
    pub fn new(filename: &String) -> Result<Self, io::Error> {
        let mut buffer = Vec::new();
        let mut file = File::open(filename)?;

        file.read_to_end(&mut buffer)?;

        Ok(CartridgeDriver
        {
            cartridge_content: buffer
        })
    }

    pub fn get(self) -> Vec<u8> {
        self.cartridge_content
    }
}