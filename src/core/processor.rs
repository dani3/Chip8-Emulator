use colored::*;

use crate::core::CHIP8_HEIGHT;
use crate::core::CHIP8_WIDTH;
use crate::core::FONTSET;

// System memory map
// 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
// 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
// 0x200-0xFFF - Program ROM and work RAM

const MEMORY_SIZE:            usize = 4096;
const STACK_SIZE:             usize = 16;
const KEYPAD_SIZE:            usize = 16;
const OPCODE_SIZE:              u16 = 2;
const NUM_REGISTERS:          usize = 16;
const INTERPRETER_AREA_START: usize = 0x000;
const INTERPRETER_AREA_END:   usize = 0x1ff;
const FONT_AREA_START:        usize = 0x050;
const FONT_AREA_END:          usize = 0x0a0;
const PROGRAM_AREA_START:     usize = 0x200;
const PROGRAM_AREA_END:       usize = 0xfff;

const WAITING_FOR_INPUT_BIT:     u8 = 0x01;
const UPDATE_VRAM_BIT:           u8 = 0x02;

pub struct Output {
    pub vram_changed: bool,
    pub beep_request: bool,
    pub vram: [[u8; CHIP8_WIDTH]; CHIP8_HEIGHT]
}

/// The Chip-8 virtual machine is represented here
pub struct Processor {
    // Memory
    memory: [u8; MEMORY_SIZE],
    // Stack
    stack: [u16; STACK_SIZE],
    // Stack pointer
    sp: u16,
    // Keypad
    keypad: [bool; KEYPAD_SIZE],
    // Graphics
    vram: [[u8; CHIP8_WIDTH]; CHIP8_HEIGHT],
    // CPU registers
    v: [u8; NUM_REGISTERS],
    // Index register
    i: u16,
    // Program counter
    pc: u16,
    // OpCode
    op_code: u16,
    // Delay timer
    delay_timer: u8,
    // Sound timer
    sound_timer: u8,
    // Flags
    cpu_flags: u8
}

impl Processor {
    /// Initializes the virtual machine
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
            vram: [[0x00; CHIP8_WIDTH]; CHIP8_HEIGHT],
            // Clear registers
            v: [0x00; NUM_REGISTERS],
            // Clear index
            i: 0,
            // Program counter starts at 0x200
            pc: PROGRAM_AREA_START as u16,
            // Reset OpCode
            op_code: 0,
            // Reset timers
            delay_timer: 0,
            sound_timer: 0,
            // Clear flags
            cpu_flags: 0,
        }
    }

    /// Load the game into memory
    ///
    /// # Arguments
    ///
    /// * `game` - A buffer containing the opcodes of the game
    pub fn load(&mut self, game: &Vec<u8>) {
        if game.len() > (PROGRAM_AREA_END - PROGRAM_AREA_START) {
            panic!("Game is too big");
        }

        for i in 0..game.len() {
            self.memory[PROGRAM_AREA_START + i] = game[i];
        }
    }

    pub fn tick(&mut self, keypad: [bool; KEYPAD_SIZE]) -> Result<Output, ()> {
        self.keypad = keypad;
        self.cpu_flags = 0;

        // If the program is waiting for a key
        if self.cpu_flags & WAITING_FOR_INPUT_BIT == 1 {
            for i in 0..KEYPAD_SIZE {

            }
        }
        else {
            let opcode = self.read_opcode();
            let nibbles = (
                (opcode & 0xF000) >> 12 as u8,
                (opcode & 0x0F00) >> 8  as u8,
                (opcode & 0x00F0) >> 4  as u8,
                (opcode & 0x000F)       as u8
            );

            let x = nibbles.1;
            let y = nibbles.2;
            let kk = nibbles.2 << 4 | nibbles.3 as u16;
            let nnn = nibbles.1 << 8 | nibbles.2 << 4 | nibbles.3 as u16;

            match nibbles {
                (0x0,0x0,0xe,0x0) => self.exec_cls(),
                (0x0,0x0,0xe,0xe) => self.exec_ret(),
                (0x0,_,_,_)       => self.exec_sys(nnn),
                (0x1,_,_,_)       => self.exec_jp(nnn),
                (0x2,_,_,_)       => self.exec_call(nnn),
                (_,_,_,_)         => ()
            }
        }

        Ok(Output {
            vram_changed: ((self.cpu_flags & UPDATE_VRAM_BIT) == UPDATE_VRAM_BIT),
            beep_request: false,
            vram: self.vram
        })
    }

    /// __00E0 - CLS__
    /// Clear the display.
    fn exec_cls(&mut self) {
        self.vram = [[0x00; CHIP8_WIDTH]; CHIP8_HEIGHT];

        self.cpu_flags |= UPDATE_VRAM_BIT;

        self.increment_pc();
    }

    /// __00EE - RET__
    /// Return from a subroutine
    ///
    /// The interpreter sets the program counter to the address
    /// at the top of the stack, then subtracts 1 from the stack pointer.
    fn exec_ret(&mut self) {

    }

    /// __0nnn - SYS addr__
    /// Jump to a machine code routine at nnn.
    fn exec_sys(&mut self, nnn: u16) {
        self.jump(nnn);
    }

    /// __1nnn - JP addr__
    /// Jump to location nnn.
    ///
    /// The interpreter sets the program counter to nnn.
    fn exec_jp(&mut self, nnn: u16) {
        self.jump(nnn);
    }

    /// __2nnn - CALL addr__
    /// Call subroutine at nnn.
    ///
    /// The interpreter increments the stack pointer, then puts
    /// the current PC on the top of the stack. The PC is then set to nnn.
    fn exec_call(&mut self, nnn: u16) {
        self.sp += OPCODE_SIZE;

        self.stack[self.sp as usize]     = self.pc & 0xFF;
        self.stack[self.sp as usize + 1] = self.pc >> 8;

        self.pc = nnn;
    }

    /// Return the opcode currently pointed from the program counter.
    fn read_opcode(&self) -> u16 {
        ((self.memory[self.pc as usize] as u16) << 8) |
          self.memory[(self.pc + 1) as usize] as u16
    }

    /// Increment the program counter.
    fn increment_pc(&mut self) {
        self.pc += 2;
    }

    /// Jump to the specified address.
    ///
    /// # Arguments
    ///
    /// * `addr` - u16 containing the target address.
    fn jump(&mut self, addr: u16) {
        self.pc = addr;
    }

    /// Skip the next opcode.
    fn skip(&mut self) {
        self.pc += 2 * OPCODE_SIZE;
    }
}
