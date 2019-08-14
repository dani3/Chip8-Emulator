use colored::*;
use rand::Rng;

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
    // Delay timer
    delay_timer: u8,
    // Sound timer
    sound_timer: u8,
    // Flags
    cpu_flags: u8,
    // Selected register
    selected_v: u8
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
            // Reset timers
            delay_timer: 0,
            sound_timer: 0,
            // Clear flags
            cpu_flags: 0,
            selected_v: 0
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

        for i in 0 .. game.len() {
            self.memory[PROGRAM_AREA_START + i] = game[i];
        }
    }

    pub fn tick(&mut self, keypad: [bool; KEYPAD_SIZE]) -> Result<Output, ()> {
        self.keypad = keypad;
        self.cpu_flags = 0;

        // If the program is waiting for a key
        if self.cpu_flags & WAITING_FOR_INPUT_BIT == 1 {
            for i in 0 .. KEYPAD_SIZE {
                if self.keypad[i] {
                    // Clear the flag
                    self.cpu_flags = 0;
                    // And store what key has been pressed.
                    self.v[self.selected_v as usize] = i as u8;
                }
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
            let n   = nibbles.3 as u8;
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
                (0x8,_,_,0x1)     => self.exec_or_vx_vy(x, y),
                (0x8,_,_,0x2)     => self.exec_and_vx_vy(x, y),
                (0x8,_,_,0x3)     => self.exec_xor_vx_vy(x, y),
                (0x8,_,_,0x4)     => self.exec_add_vx_vy(x, y),
                (0x8,_,_,0x5)     => self.exec_sub_vx_vy(x, y),
                (0x8,_,_,0x6)     => self.exec_shr_vx_vy(x),
                (0x8,_,_,0x7)     => self.exec_subn_vx_vy(x, y),
                (0x8,_,_,0xe)     => self.exec_shl_vx_vy(x),
                (0x9,_,_,0x0)     => self.exec_sne_vx_vy(x, y),
                (0xa,_,_,_)       => self.exec_ld_i(nnn),
                (0xb,_,_,_)       => self.exec_jp_v0(nnn),
                (0xc,_,_,_)       => self.exec_rnd(x, kk),
                (0xd,_,_,_)       => self.exec_drw(x, y, n),
                (0xe,_,0x9,0xe)   => self.exec_skp(x),
                (0xe,_,0xa,0x1)   => self.exec_sknp(x),
                (0xf,_,0x0,0x7)   => self.exec_ld_vx_dt(x),
                (0xf,_,0x0,0xa)   => self.exec_ld_vx_k(x),
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

    /// __8xy1 - OR Vx, Vy__
    /// Set Vx = Vx OR Vy.
    ///
    /// Performs a bitwise OR on the values of Vx and Vy, then
    /// stores the result in Vx.
    fn exec_or_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] |= self.v[y as usize];

        self.increment_pc();

        println!("OR V{:x?} -> V{:x?}", y, x);
    }

    /// __8xy2 - AND Vx, Vy__
    /// Set Vx = Vx AND Vy.
    ///
    /// Performs a bitwise AND on the values of Vx and Vy, then
    /// stores the result in Vx.
    fn exec_and_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] &= self.v[y as usize];

        self.increment_pc();

        println!("AND V{:x?} -> V{:x?}", y, x);
    }

    /// __8xy3 - XOR Vx, Vy__
    /// Set Vx = Vx XOR Vy.
    ///
    /// Performs a bitwise exclusive OR on the values of Vx and Vy, then
    /// stores the result in Vx.
    fn exec_xor_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] ^= self.v[y as usize];

        self.increment_pc();

        println!("XOR V{:x?} -> V{:x?}", y, x);
    }

    /// __8xy4 - ADD Vx, Vy__
    /// Set Vx = Vx + Vy. Set VF = carry.
    ///
    /// The values of Vx and Vy are added together. If the result
    /// is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0.
    /// Only the lowest 8 bits of the result are kept, and stored in Vx.
    fn exec_add_vx_vy(&mut self, x: u8, y: u8) {
        let sum = self.v[x as usize] as u16 + self.v[y as usize] as u16;
        let carry = sum > 255;
        if carry {
            self.v[0xf] = 1;
        } else {
            self.v[0xf] = 0;
        }

        self.v[x as usize] = sum as u8;

        self.increment_pc();

        println!("ADD V{:x?} -> V{:x?} (VF = {:x?})", y, x, self.v[0xf]);
    }

    /// __8xy5 - SUB Vx, Vy__
    /// Set Vx = Vx - Vy. Set VF = NOT borrow.
    ///
    /// If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted
    /// from Vx, and the results stored in Vx.
    fn exec_sub_vx_vy(&mut self, x: u8, y: u8) {
        if self.v[y as usize] < self.v[x as usize] {
            self.v[0xf] = 1;
        } else {
            self.v[0xf] = 0;
        }

        self.v[x as usize] = self.v[x as usize].wrapping_sub(self.v[y as usize]);

        self.increment_pc();

        println!("SUB V{:x?} -> V{:x?} (VF = {:x?})", y, x, self.v[0xf]);
    }

    /// __8xy6 - SHR Vx {, Vy}__
    /// Set Vx = Vx SHR 1.
    ///
    /// If the least-significant bit of Vx is 1, then VF is
    /// set to 1, otherwise 0. Then Vx is divided by 2.
    fn exec_shr_vx_vy(&mut self, x: u8) {
        self.v[0xf] = self.v[x as usize] & 0x01;

        self.v[x as usize] >>= 1;

        self.increment_pc();

        println!("SHR V{:x?} -> {:x?}", x, self.v[x as usize]);
    }

    /// __8xy7 - SUBN Vx, Vy__
    /// Set Vx = Vy- Vx. Set VF = NOT borrow.
    ///
    /// If Vy > Vx, then VF is set to 1, otherwise 0. Then Vy is subtracted
    /// from Vx, and the results stored in Vx.
    fn exec_subn_vx_vy(&mut self, x: u8, y: u8) {
        if self.v[x as usize] < self.v[y as usize] {
            self.v[0xf] = 1;
        } else {
            self.v[0xf] = 0;
        }

        self.v[x as usize] = self.v[y as usize].wrapping_sub(self.v[x as usize]);

        self.increment_pc();

        println!("SUBN V{:x?} -> V{:x?} (VF = {:x?})", y, x, self.v[0xf]);
    }

    /// __8xye - SHL Vx {, Vy}__
    /// Set Vx = Vx SHL 1.
    ///
    /// If the most-significant bit of Vx is 1, then VF is
    /// set to 1, otherwise 0. Then Vx is multiplied by 2.
    fn exec_shl_vx_vy(&mut self, x: u8) {
        self.v[0xf] = self.v[x as usize] & 0x80;

        self.v[x as usize] <<= 1;

        self.increment_pc();

        println!("SHL V{:x?} -> {:x?}", x, self.v[x as usize]);
    }

    /// __9xy0 - SNE Vx, Vy__
    /// Skip next instruction if Vx != Vy.
    ///
    /// The values of Vx and Vy are compared, and if they are not
    /// equal, the program counter is increased by 2.
    fn exec_sne_vx_vy(&mut self, x: u8, y: u8) {
        if self.v[x as usize] != self.v[y as usize] {
            self.skip();
        } else {
            self.increment_pc();
        }

        println!("SNE {:x?} != {:x?}", self.v[x as usize], self.v[y as usize]);
    }

    /// __annn - LD I, addr__
    /// Set I = nnn.
    ///
    /// The value of register I is set to nnn.
    fn exec_ld_i(&mut self, nnn: u16) {
        self.i = nnn;

        self.increment_pc();

        println!("LD I -> {:x?}", self.i);
    }

    /// __bnnn - JP V0, addr__
    /// Jump to location nnn + V0.
    ///
    /// The program counter is set to nnn plus the value of V0.
    fn exec_jp_v0(&mut self, nnn: u16) {
        self.pc = nnn + self.v[0x0] as u16;

        println!("JP V0 pc -> {:x?}", self.pc);
    }

    /// __cxkk - RND Vx, byte__
    /// Set Vx = random byte AND kk.
    ///
    /// The interpreter generates a random number from 0 to 255, which is
    /// then ANDed with the value kk. The results are stored in Vx.
    fn exec_rnd(&mut self, x: u8, kk: u8) {
        let mut rng = rand::thread_rng();
        self.v[x as usize] = rng.gen_range(0, 255) & kk;

        println!("RND rnd -> {:x?}", self.v[x as usize]);
    }

    /// __dxyn - DRW Vx, Vy, nibble__
    /// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    ///
    /// The interpreter reads n bytes from memory, starting at the
    /// address stored in I. These bytes are then displayed as sprites
    /// on screen at coordinates (Vx, Vy). Sprites are XORed onto the
    /// existing screen. If this causes any pixels to be erased, VF is
    /// set to 1, otherwise it is set to 0. If the sprite is positioned
    /// so part of it is outside the coordinates of the display, it wraps
    /// around to the opposite side of the screen.
    fn exec_drw(&mut self, x: u8, y: u8, n: u8) {
        let xpos = self.v[x as usize] as usize;
        let ypos = self.v[y as usize] as usize;

        let mut sprite: Vec<u8> = Vec::new();
        for i in self.i .. (self.i + n as u16) {
            sprite.push(self.memory[i as usize]);
        }

        self.v[0xf] = 0;

        for yline in 0 .. sprite.len() {
            let pixel = sprite[yline];
            for xline in 0 .. 8 {
                if (pixel & (0x80 >> xline)) != 0 {
                    if self.vram[xpos + xline][ypos + yline] == 1 {
                        self.v[0xf] = 1;
                    }

                    self.vram[xpos + xline][ypos + yline] ^= 1;
                }
            }
        }

        self.cpu_flags |= UPDATE_VRAM_BIT;
        self.increment_pc();

        println!("DRW");
    }

    /// __ex9e - SKP Vx__
    /// Skip next instruction if key with the value of Vx is pressed.
    ///
    /// Checks the keyboard, and if the key corresponding to the value of Vx is
    /// currently in the down position, PC is increased by 2.
    fn exec_skp(&mut self, x: u8) {
        if self.keypad[self.v[x as usize] as usize] {
            self.skip();
        } else {
            self.increment_pc();
        }

        println!("SKP Vx = {:x?} : {}", self.v[x as usize], self.keypad[self.v[x as usize] as usize]);
    }

    /// __exa1 - SKNP Vx__
    /// Skip next instruction if key with the value of Vx is not pressed.
    ///
    /// Checks the keyboard, and if the key corresponding to the value of Vx is
    /// currently in the up position, PC is increased by 2.
    fn exec_sknp(&mut self, x: u8) {
        if !(self.keypad[self.v[x as usize] as usize]) {
            self.skip();
        } else {
            self.increment_pc();
        }

        println!("SKNP Vx = {:x?} : {}", self.v[x as usize], self.keypad[self.v[x as usize] as usize]);
    }

    /// __fx07 - LD Vx, DT__
    /// Set Vx = delay timer value.
    ///
    /// The value of DT is placed into Vx.
    fn exec_ld_vx_dt(&mut self, x: u8) {
        self.delay_timer = self.v[x as usize];

        self.increment_pc();

        println!("LD V{:x?} -> DT = {:x?}", x, self.delay_timer);
    }

    /// __fx0a - LD Vx, K__
    /// Wait for a key press, store the value of the key in Vx
    ///
    /// All execution stops until a key is pressed, then the value
    /// of that key is stored in Vx.
    fn exec_ld_vx_k(&mut self, x: u8) {
        self.cpu_flags |= WAITING_FOR_INPUT_BIT;
        self.selected_v = x;

        self.increment_pc();

        println!("LD V{:x?}, K", x);
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
