fn main() {
    // Memory
    let memory: [u8; 4000] = [0xff; 4000];

    // System memory map
    // 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
    // 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
    // 0x200-0xFFF - Program ROM and work RAM
    let interpreter_area: (u16, u16) = (0x000, 0x1ff);
    let font_set_area: (u16, u16) = (0x050, 0x0a0);
    let program_area: (u16, u16) = (0x200, 0xFff);

    // Stack
    let stack: [u16; 16] = [0xff; 16];
    let mut sp: u16;

    // Keypad
    let keypad: [u8; 16] = [0x00; 16];

    // Graphics
    let gfx: [u8; 64 * 32] = [0x00; 64 * 32];
    // CPU registers
    let V: [u8; 16] = [0xff; 16];
    // Index register
    let mut I: u16;
    // Program counter
    let mut pc: u16;
    // OpCode
    let mut op_code: u16;
    // Delay timer
    let mut delay_timer: u8;
    // Sound timer
    let mut sound_timer: u8;
}
