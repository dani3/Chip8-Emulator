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
const FONT_AREA_START:        usize = 0x000;
const FONT_AREA_END:          usize = 0x050;
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

        if (self.cpu_flags & UPDATE_VRAM_BIT) == UPDATE_VRAM_BIT {
            self.cpu_flags = 0;
        }

        let mut beep_request = false;

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
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }

            if self.sound_timer > 0 {
                self.sound_timer -= 1;

                beep_request = self.sound_timer == 0;
            }

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
                (0xf,_,0x1,0x5)   => self.exec_ld_dt_vx(x),
                (0xf,_,0x1,0x8)   => self.exec_ld_st_vx(x),
                (0xf,_,0x1,0xe)   => self.exec_add_i_vx(x),
                (0xf,_,0x2,0x9)   => self.exec_ld_f_vx(x),
                (0xf,_,0x3,0x3)   => self.exec_ld_b_vx(x),
                (0xf,_,0x5,0x5)   => self.exec_ld_i_vx(x),
                (0xf,_,0x6,0x5)   => self.exec_ld_vx_i(x),
                (_,_,_,_)         => self.increment_pc()
            }
        }

        Ok(Output {
            vram_changed: ((self.cpu_flags & UPDATE_VRAM_BIT) == UPDATE_VRAM_BIT),
            beep_request,
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
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
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
        self.stack[self.sp as usize] = self.pc + OPCODE_SIZE;

        self.sp += 1;

        self.jump(nnn);
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
    }

    /// __6xkk - LD Vx, byte__
    /// Set Vx = kk.
    ///
    /// The interpreter puts the value kk into register Vx.
    fn exec_ld_vx_byte(&mut self, x: u8, kk: u8) {
        self.v[x as usize] = kk;

        self.increment_pc();
    }

    /// __7xkk - ADD Vx, byte__
    /// Set Vx = Vx + kk.
    ///
    /// Adds the value kk to the value of register Vx, then stores the result in Vx.
    fn exec_add_vx_byte(&mut self, x: u8, kk: u8) {
        let i = kk as u16;
        let j = self.v[x as usize] as u16;

        let sum = i + j;

        self.v[x as usize] = sum as u8;

        self.increment_pc();
    }

    /// __8xy0 - LD Vx, Vy__
    /// Set Vx = Vy.
    ///
    /// Stores the value of register Vy in register Vx.
    fn exec_ld_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] = self.v[y as usize];

        self.increment_pc();
    }

    /// __8xy1 - OR Vx, Vy__
    /// Set Vx = Vx OR Vy.
    ///
    /// Performs a bitwise OR on the values of Vx and Vy, then
    /// stores the result in Vx.
    fn exec_or_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] |= self.v[y as usize];

        self.increment_pc();
    }

    /// __8xy2 - AND Vx, Vy__
    /// Set Vx = Vx AND Vy.
    ///
    /// Performs a bitwise AND on the values of Vx and Vy, then
    /// stores the result in Vx.
    fn exec_and_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] &= self.v[y as usize];

        self.increment_pc();
    }

    /// __8xy3 - XOR Vx, Vy__
    /// Set Vx = Vx XOR Vy.
    ///
    /// Performs a bitwise exclusive OR on the values of Vx and Vy, then
    /// stores the result in Vx.
    fn exec_xor_vx_vy(&mut self, x: u8, y: u8) {
        self.v[x as usize] ^= self.v[y as usize];

        self.increment_pc();
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
    }

    /// __8xye - SHL Vx {, Vy}__
    /// Set Vx = Vx SHL 1.
    ///
    /// If the most-significant bit of Vx is 1, then VF is
    /// set to 1, otherwise 0. Then Vx is multiplied by 2.
    fn exec_shl_vx_vy(&mut self, x: u8) {
        self.v[0xf] = (self.v[x as usize] & 0x80) >> 7;

        self.v[x as usize] <<= 1;

        self.increment_pc();
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
    }

    /// __annn - LD I, addr__
    /// Set I = nnn.
    ///
    /// The value of register I is set to nnn.
    fn exec_ld_i(&mut self, nnn: u16) {
        self.i = nnn;

        self.increment_pc();
    }

    /// __bnnn - JP V0, addr__
    /// Jump to location nnn + V0.
    ///
    /// The program counter is set to nnn plus the value of V0.
    fn exec_jp_v0(&mut self, nnn: u16) {
        self.pc = nnn + self.v[0x0] as u16;
    }

    /// __cxkk - RND Vx, byte__
    /// Set Vx = random byte AND kk.
    ///
    /// The interpreter generates a random number from 0 to 255, which is
    /// then ANDed with the value kk. The results are stored in Vx.
    fn exec_rnd(&mut self, x: u8, kk: u8) {
        let mut rng = rand::thread_rng();
        self.v[x as usize] = rng.gen_range(0, 255) & kk;

        self.increment_pc();
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
        self.v[0x0f] = 0;
        for byte in 0..n {
            let y = (self.v[y as usize] as usize + byte as usize) % CHIP8_HEIGHT;
            for bit in 0..8 {
                let x = (self.v[x as usize] as usize + bit) % CHIP8_WIDTH;
                let color = (self.memory[self.i as usize + byte as usize] >> (7 - bit)) & 1;

                self.v[0x0f] |= color & self.vram[y][x];
                self.vram[y][x] ^= color;
            }
        }

        self.cpu_flags |= UPDATE_VRAM_BIT;
        self.increment_pc();
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
    }

    /// __fx07 - LD Vx, DT__
    /// Set Vx = delay timer value.
    ///
    /// The value of DT is placed into Vx.
    fn exec_ld_vx_dt(&mut self, x: u8) {
        self.v[x as usize] = self.delay_timer;

        self.increment_pc();
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
    }

    /// __fx15 - LD DT, Vc__
    /// Set delay timer = Vx.
    ///
    /// DT is set equal to the value of Vx.
    fn exec_ld_dt_vx(&mut self, x: u8) {
        self.delay_timer = self.v[x as usize];

        self.increment_pc();
    }

    /// __fx18 - LD ST, Vc__
    /// Set sound timer = Vx.
    ///
    /// ST is set equal to the value of Vx.
    fn exec_ld_st_vx(&mut self, x: u8) {
        self.sound_timer = self.v[x as usize];

        self.increment_pc();
    }

    /// __fx1e - ADD I, Vx__
    /// Set I = I + Vx.
    ///
    /// The values of I and Vx are added, and the results are stored in I.
    fn exec_add_i_vx(&mut self, x: u8) {
        self.i += self.v[x as usize] as u16;
        self.v[0xf] = if self.i > 0x0F00 { 1 } else { 0 };

        self.increment_pc();
    }

    /// __fx29 - LD F, Vx__
    /// Set I = location of sprite for digit Vx.
    ///
    /// The value of I is set to the location for the
    /// hexadecimal sprite corresponding to the value of Vx.
    fn exec_ld_f_vx(&mut self, x: u8) {
        self.i = (self.v[x as usize] as u16) * 5;

        self.increment_pc();
    }

    /// __fx33 - LD B, Vx__
    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    ///
    /// The interpreter takes the decimal value of Vx, and places
    /// the hundreds digit in memory at location in I, the tens
    /// digit at location I+1, and the ones digit at location I+2.
    fn exec_ld_b_vx(&mut self, x: u8) {
        let value = self.v[x as usize];

        let hundreds = value / 100;
        let tens = (value % 100) / 10;
        let ones = value % 10;

        self.memory[self.i as usize]     = hundreds;
        self.memory[self.i as usize + 1] = tens;
        self.memory[self.i as usize + 2] = ones;

        self.increment_pc();
    }

    /// __fx55 - LD [I], Vx__
    /// Store registers V0 through Vx in memory starting at location I.
    ///
    /// The interpreter copies the values of registers V0 through
    /// Vx into memory, starting at the address in I.
    fn exec_ld_i_vx(&mut self, x: u8) {
        let limit = x as usize;
        for i in 0 ..= limit {
            self.memory[self.i as usize + i] = self.v[i];
        }

        self.increment_pc();
    }

    /// __fx65 - LD Vx, [I]__
    /// Read registers V0 through Vx from memory starting at location I.
    ///
    /// The interpreter reads values from memory starting at location I
    /// into registers V0 through Vx.
    fn exec_ld_vx_i(&mut self, x: u8) {
        let limit = x as usize;
        for i in 0 ..= limit {
            self.v[i] = self.memory[self.i as usize + i];
        }

        self.increment_pc();
    }

    /// Return the opcode currently pointed from the program counter.
    fn read_opcode(&self) -> u16 {
        ((self.memory[self.pc as usize] as u16) << 8) |
          self.memory[(self.pc + 1) as usize] as u16
    }

    /// Increment the program counter.
    fn increment_pc(&mut self) {
        self.pc += 2;
        println!("PC = {:x?}", self.pc);
    }

    /// Jump to the specified address.
    ///
    /// # Arguments
    ///
    /// * `addr` - u16 containing the target address.
    fn jump(&mut self, addr: u16) {
        self.pc = addr;
        println!("PC = {:x?}", self.pc);
    }

    /// Skip the next opcode.
    fn skip(&mut self) {
        self.pc += 2 * OPCODE_SIZE;
        println!("PC = {:x?}", self.pc);
    }
}
