use sdl2::Sdl;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::rect::Rect;
use sdl2::video::Window;

use crate::core::CHIP8_HEIGHT;
use crate::core::CHIP8_WIDTH;

const SCALE_FACTOR: u32 = 16;

const SCREEN_HEIGHT: u32 = (CHIP8_HEIGHT as u32) * SCALE_FACTOR;
const SCREEN_WIDTH: u32 = (CHIP8_WIDTH as u32) * SCALE_FACTOR;

pub struct GraphicsDriver {
    canvas: Canvas<Window>
}

impl GraphicsDriver {
    pub fn new(sdl_context: &Sdl) -> Self {
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("Chip-8 Emulator", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        GraphicsDriver {
            canvas
        }
    }

    pub fn draw(&mut self, vram: &[[u8; CHIP8_WIDTH]; CHIP8_HEIGHT]) {
        for (y, row) in vram.iter().enumerate() {
            for (x, &pixel) in row.iter().enumerate() {
                let xpos = x * SCALE_FACTOR as usize;
                let ypos = y * SCALE_FACTOR as usize;

                self.canvas.set_draw_color(self.create_color(pixel == 1));
                let _ =
                    self.canvas.fill_rect(
                        Rect::new(xpos as i32, ypos as i32, SCALE_FACTOR, SCALE_FACTOR));
            }
        }

        self.canvas.present();
    }

    fn create_color(&self, is_set: bool) -> Color {
        if is_set {
            Color::RGB(255, 255, 255)
        } else {
            Color::RGB(0, 0, 0)
        }
    }
}
