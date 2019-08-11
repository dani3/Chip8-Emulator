use crate::core::CHIP8_HEIGHT;
use crate::core::CHIP8_WIDTH;
use crate::core::FONTSET;

const MEMORY_SIZE:            usize = 4096;
const STACK_SIZE:             usize = 16;
const KEYPAD_SIZE:            usize = 16;
const NUM_REGISTERS:          usize = 16;
const INTERPRETER_AREA_START: usize = 0x000;
const INTERPRETER_AREA_END:   usize = 0x1ff;
const FONT_AREA_START:        usize = 0x050;
const FONT_AREA_END:          usize = 0x0a0;
const PROGRAM_AREA_START:     usize = 0x200;
const PROGRAM_AREA_END:       usize = 0xfff;

pub struct Processor {
    // System memory map
    // 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
    // 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
    // 0x200-0xFFF - Program ROM and work RAM
    memory: [u8; MEMORY_SIZE],
    // Stack
    stack: [u16; STACK_SIZE],
    // Stack pointer
    sp: u16,
    // Keypad
    keypad: [bool; KEYPAD_SIZE],
    // Graphics
    vram: [u8; CHIP8_WIDTH * CHIP8_HEIGHT],
    // CPU registers
    V: [u8; NUM_REGISTERS],
    // Index register
    I: u16,
    // Program counter
    pc: u16,
    // OpCode
    op_code: u16,
    // Delay timer
    delay_timer: u8,
    // Sound timer
    sound_timer: u8
}

impl Processor {
    pub fn new() -> Self {
        let mut memory: [u8; MEMORY_SIZE] = [0xff; MEMORY_SIZE];
        memory[FONT_AREA_START..FONT_AREA_END].copy_from_slice(&FONTSET);

        Processor {
            // Clear memory
            memory,
            // Clear stack
            stack: [0xff; STACK_SIZE],
            sp: 0,
            keypad: [false; KEYPAD_SIZE],
            // Cleary display
            vram: [0x00; CHIP8_WIDTH * CHIP8_HEIGHT],
            // Clear registers
            V: [0x00; NUM_REGISTERS],
            // Clear index
            I: 0,
            // Program counter starts at 0x200
            pc: 0x200,
            // Reset OpCode
            op_code: 0,
            // Reset timers
            delay_timer: 0,
            sound_timer: 0
        }
    }

    pub fn load(&mut self, game: &Vec<u8>) {
        for i in 0..game.len() {
            self.memory[PROGRAM_AREA_START + i] = game[i];
        }
    }
}