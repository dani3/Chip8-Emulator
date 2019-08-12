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

            let x   = nibbles.1 as u8;
            let y   = nibbles.2 as u8;
            let kk  = (opcode & 0x00FF) as u8;
            let nnn = (opcode & 0x0FFF) as u16;

            print!("Opcode = {:x?} ", opcode);

            match nibbles {
                (0x0,0x0,0xe,0x0) => self.exec_cls(),
                (0x0,0x0,0xe,0xe) => self.exec_ret(),
                (0x0,_,_,_)       => self.exec_sys(nnn),
                (0x1,_,_,_)       => self.exec_jp(nnn),
                (0x2,_,_,_)       => self.exec_call(nnn),
                (0x3,_,_,_)       => self.exec_se_vx_byte(x, kk),
                (0x4,_,_,_)       => self.exec_sne_vx_byte(x, kk),
                (0x5,_,_,_)       => self.exec_se_vx_vy(x, y),
                (0x6,_,_,_)       => self.exec_ld_vx_byte(x, kk),
                (0x7,_,_,_)       => self.exec_add_vx_byte(x, kk),
                (0x8,_,_,0x0)     => self.exec_ld_vx_vy(x, y),
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
        println!("CLS");

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

        println!("SYS pc -> {:x?}", self.pc);
    }

    /// __1nnn - JP addr__
    /// Jump to location nnn.
    ///
    /// The interpreter sets the program counter to nnn.
    fn exec_jp(&mut self, nnn: u16) {
        self.jump(nnn);

        println!("JP pc -> {:x?}", self.pc);
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

        self.jump(nnn);

        println!("CALL pc -> {:x?} | sp -> {:x?} | stack -> {:x?}{:x?}"
            , self.pc
            , self.sp
            , self.stack[self.sp as usize]
            , self.stack[(self.sp + 1) as usize]);
    }

    /// __3xkk - SE Vx, byte__
    /// Skip next instruction if Vx = kk.
    ///
    /// The interpreter compares register Vx to kk, and if
    /// they are equal, increments the program counter by 2.
    fn exec_se_vx_byte(&mut self, x: u8, kk: u8) {
        if kk == self.v[x as usize] {
            self.skip();
        } else {
            self.increment_pc();
        }

        println!("SE V{:x?} byte {:x?} == {:x?}", x, kk, self.v[x as usize]);
    }

    /// __4xkk - SNE Vx, byte__
    /// Skip next instruction if Vx != kk.
    ///
    /// The interpreter compares register Vx to kk, and if
    /// they are not equal, increments the program counter by 2.
    fn exec_sne_vx_byte(&mut self, x: u8, kk: u8) {
        if kk != self.v[x as usize] {
            self.skip();
        } else {
            self.increment_pc();
        }

        println!("SNE V{:x?} byte {:x?} != {:x?}", x, kk, self.v[x as usize]);
    }

    /// __5xkk - SE Vx, Vy__
    /// Skip next instruction if Vx = Vy.
    ///
    /// The interpreter compares register Vx to register Vy, and if
    /// they are equal, increments the program counter by 2.
    fn exec_se_vx_vy(&mut self, x: u8, y: u8) {
        if self.v[x as usize] == self.v[y as usize] {
            self.skip();
        } else {
            self.increment_pc();
        }

        println!("SE V{:x?} byte {:x?} == {:x?}", x, self.v[x as usize], self.v[y as usize]);
    }

    /// __6xkk - LD Vx, byte__
    /// Set Vx = kk.
    ///
    /// The interpreter puts the value kk into register Vx.
    fn exec_ld_vx_byte(&mut self, x: u8, kk: u8) {
        self.v[x as usize] = kk;

        self.increment_pc();

        println!("LD V{:x?} -> {:x?}", x, self.v[x as usize]);
    }

    /// __7xkk - ADD Vx, byte__
    /// Set Vx = Vx + kk.
    ///
    /// Adds the value kk to the value of register Vx, then stores the result in Vx.
    fn exec_add_vx_byte(&mut self, x: u8, kk: u8) {
        self.v[x as usize] += kk;

        self.increment_pc();

        println!("ADD V{:x?} -> {:x?}", x, self.v[x as usize]);
    }

    /// __8xy0 - LD Vx, Vy__
    /// Set Vx = Vy.
    ///
    /// Stores the value of register Vy in register Vx.
    fn exec_ld_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] = self.v[y as usize];

        self.increment_pc();

        println!("LD V{:x?} -> V{:x?}", y, x);
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
