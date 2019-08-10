use crate::core::CHIP8_HEIGHT;
use crate::core::CHIP8_WIDTH;

const MEMORY_SIZE:            usize = 4096;
const STACK_SIZE:             usize = 16;
const KEYPAD_SIZE:            usize = 16;
const NUM_REGISTERS:          usize = 16;
const INTERPRETER_AREA_START:   u16 = 0x000;
const INTERPRETER_AREA_END:     u16 = 0x1ff;
const FONT_AREA_START:          u16 = 0x050;
const FONT_AREA_END:            u16 = 0x0a0;
const PROGRAM_AREA_START:       u16 = 0x200;
const PROGRAM_AREA_END:         u16 = 0xfff;

pub struct Processor {
    // Memory
    memory: [u8; MEMORY_SIZE],
    // System memory map
    // 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
    // 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
    // 0x200-0xFFF - Program ROM and work RAM
    interpreter_area: (u16, u16),
    font_set_area: (u16, u16),
    program_area: (u16, u16),
    // Stack
    stack: [u16; STACK_SIZE],
    // Stack pointer
    sp: u16,
    // Keypad
    keypad: [u8; KEYPAD_SIZE],
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
        Processor {
            memory: [0xff; MEMORY_SIZE],
            interpreter_area: (INTERPRETER_AREA_START, INTERPRETER_AREA_END),
            font_set_area: (FONT_AREA_START, FONT_AREA_END),
            program_area: (PROGRAM_AREA_START, PROGRAM_AREA_END),
            stack: [0xff; STACK_SIZE],
            sp: 0,
            keypad: [0x00; KEYPAD_SIZE],
            vram: [0x00; CHIP8_WIDTH * CHIP8_HEIGHT],
            V: [0x00; NUM_REGISTERS],
            I: 0,
            pc: 0,
            op_code: 0,
            delay_timer: 0,
            sound_timer: 0
        }
    }
}