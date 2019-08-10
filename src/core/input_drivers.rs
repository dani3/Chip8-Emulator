use sdl2::Sdl;
use sdl2::EventPump;
use sdl2::event::Event;

pub struct InputDrivers {
    event_pump: EventPump
}

impl InputDrivers {
    pub fn new(sdl_context: &Sdl) -> Self {
        InputDrivers {
            event_pump: sdl_context.event_pump().unwrap()
        }
    }

    pub fn poll(&mut self) -> Result<[bool; 16], ()> {
        for event in self.event_pump.poll_iter() {
            if let Event::Quit {..} = event {
                return Err(());
            }
        }

        return Ok([false; 16]);
    }
}