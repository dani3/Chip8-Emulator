use sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

// Below you’ll find an example of the original keypad layout.

// Keypad                   Keyboard
// +-+-+-+-+                +-+-+-+-+
// |1|2|3|C|                |1|2|3|4|
// +-+-+-+-+                +-+-+-+-+
// |4|5|6|D|                |Q|W|E|R|
// +-+-+-+-+       =>       +-+-+-+-+
// |7|8|9|E|                |A|S|D|F|
// +-+-+-+-+                +-+-+-+-+
// |A|0|B|F|                |Z|X|C|V|
// +-+-+-+-+                +-+-+-+-+

pub struct InputDriver {
    event_pump: sdl2::EventPump
}

impl InputDriver {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        InputDriver {
            event_pump: sdl_context.event_pump().unwrap()
        }
    }

    pub fn poll(&mut self) -> Result<[bool; 16], ()> {
        for event in self.event_pump.poll_iter() {
            if let Event::Quit {..} = event {
                return Err(());
            }
        }

        let keys : Vec<Keycode> =
            self.event_pump.keyboard_state()
                           .pressed_scancodes()
                           .filter_map(Keycode::from_scancode)
                           .collect();

        let mut chip8_keys = [false; 16];

        for key in keys {
            let index = match key {
                Keycode::Num1 => Some(0x01),
                Keycode::Num2 => Some(0x02),
                Keycode::Num3 => Some(0x03),
                Keycode::Num4 => Some(0x0c),
                Keycode::Q    => Some(0x04),
                Keycode::W    => Some(0x05),
                Keycode::E    => Some(0x06),
                Keycode::R    => Some(0x0d),
                Keycode::A    => Some(0x07),
                Keycode::S    => Some(0x08),
                Keycode::D    => Some(0x09),
                Keycode::F    => Some(0x0e),
                Keycode::Z    => Some(0x0a),
                Keycode::X    => Some(0x00),
                Keycode::C    => Some(0x0b),
                Keycode::V    => Some(0x0f),
                _             => None
            };

            if let Some(i) = index {
                chip8_keys[i] = true;
            }
        }

        return Ok(chip8_keys);
    }
}