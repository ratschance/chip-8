use rand::Rng;

pub const C8_WIDTH: usize = 64;
pub const C8_HEIGHT: usize = 32;

pub struct Cpu {
    registers: Registers,
    memory: [u8; 4096],
    display: [[bool; C8_WIDTH]; C8_HEIGHT],
    key_state: [bool; 16],
}

struct Registers {
    v: [u8; 16], // Vx where x is a hexadecimal digit 0..F
    i: u16,      // Generally used to store memory addresses
    delay_timer: u8,
    sound_timer: u8,
    pc: u16, // Program Counter - used to store currently executing address
    sp: u8,  // Stack Pointer - used to point to the topmost level of the stack
    stack: [u16; 16],
}

impl Registers {
    fn initialize() -> Registers {
        Registers {
            v: [0; 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: 512,
            sp: 0,
            stack: [0; 16],
        }
    }
}

impl Cpu {
    pub fn initialize() -> Cpu {
        Cpu {
            registers: Registers::initialize(),
            memory: [0; 4096],
            display: [[false; C8_WIDTH]; C8_HEIGHT],
            key_state: [false; 16],
        }
    }

    pub fn load_rom(&mut self, path: &str) {
        use std::fs::File;
        use std::io::prelude::*;

        let mut rom = File::open(path).expect("Unable to open ROM");

        let _ = rom
            .read(&mut self.memory[512..])
            .expect("Unable to read ROM into memory");
    }

    pub fn load_sprites(&mut self) {
        let sprites = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];
        self.memory[..sprites.len()].copy_from_slice(&sprites[..])
    }

    pub fn tick(&mut self) {
        if self.registers.delay_timer > 0 {
            self.registers.delay_timer -= 1;
        }

        if self.registers.sound_timer > 0 {
            //TODO: Make sound
            self.registers.sound_timer -= 1;
        }

        let pc = self.registers.pc as usize;
        self.registers.pc += 2;
        self.process_opcode((self.memory[pc] as u16) << 8 | self.memory[pc + 1] as u16);
    }

    pub fn view_display(&mut self) -> &[[bool; C8_WIDTH]; C8_HEIGHT] {
        &self.display
    }

    pub fn set_key_pressed(&mut self, key: usize) {
        self.key_state[key] = true;
    }

    pub fn set_key_released(&mut self, key: usize) {
        self.key_state[key] = false;
    }

    fn process_opcode(&mut self, opcode: u16) {
        match opcode >> 12 {
            0x0 => {
                // 0xxx
                match opcode {
                    0x00e0 => {
                        // CLS - Clear display
                        for i in 0..self.display.len() {
                            for j in 0..self.display[i].len() {
                                self.display[i][j] = false;
                            }
                        }
                    }
                    0x00ee => {
                        // RET - return from subroutine
                        if self.registers.sp == 0 {
                            panic!("Returned when stack pointer was already 0");
                        }
                        self.registers.pc = self.registers.stack[self.registers.sp as usize];
                        self.registers.sp -= 1;
                    }
                    _ => {
                        // Legacy routine, ignored
                    }
                }
            }
            0x1 => {
                // 1nnn - JP addr - Jump to location nnn
                self.registers.pc = opcode & 0xfff;
            }
            0x2 => {
                // 2nnn - CALL addr - Call subroutine at nnn
                self.registers.sp += 1;
                self.registers.stack[self.registers.sp as usize] = self.registers.pc;
                self.registers.pc = opcode & 0xfff;
            }
            0x3 => {
                // 3xkk - SE Vx, byte - Skip next instruction if Vx = kk
                let x = get_nibble(2, opcode);
                if self.registers.v[x] == (opcode & 0xff) as u8 {
                    self.registers.pc += 2;
                }
            }
            0x4 => {
                // 4xkk - SNE Vx, byte - Skip next instruction if Vx != kk
                let x = get_nibble(2, opcode);
                if self.registers.v[x] != (opcode & 0xff) as u8 {
                    self.registers.pc += 2;
                }
            }
            0x5 => {
                // 5xy0 - SE Vx, Vy - Skip next instruction if Vx = Vy
                let x = get_nibble(2, opcode);
                let y = get_nibble(1, opcode);
                if self.registers.v[x] == self.registers.v[y] {
                    self.registers.pc += 2;
                }
            }
            0x6 => {
                // 6xkk - LD Vx, byte - Set Vx := kk
                let x = get_nibble(2, opcode);
                self.registers.v[x] = (opcode & 0xff) as u8;
            }
            0x7 => {
                // 7xkk - ADD Vx, byte - Set Vx := Vx + kk
                let x = get_nibble(2, opcode);
                let val = self.registers.v[x].wrapping_add((opcode & 0xff) as u8);
                self.registers.v[x] = val;
            }
            0x8 => {
                // 8xyo - Operations between Vx and Vy depending on the value of o
                let x = get_nibble(2, opcode);
                let y = get_nibble(1, opcode);
                let op = get_nibble(0, opcode) as u8;
                self.process_opcode_8(x, y, op);
            }
            0x9 => {
                // 9xy0 - SNE Vx, Vy - Skip next instruction if Vx != Vy
                let x = get_nibble(2, opcode);
                let y = get_nibble(1, opcode);
                if self.registers.v[x] != self.registers.v[y] {
                    self.registers.pc += 2;
                }
            }
            0xa => {
                // Annn - LD I, addr - Set I := nnn
                self.registers.i = opcode & 0xfff;
            }
            0xb => {
                // Bnnn - JP V0, addr - Jump to location nnn + V0
                let loc = opcode & 0xfff;
                self.registers.pc = loc + self.registers.v[0] as u16;
            }
            0xc => {
                // Cxkk - RND Vx, byte - Set Vx := random byte AND kk
                let x = get_nibble(2, opcode);
                let kk = (opcode & 0xff) as u8;
                let mut rng = rand::thread_rng();
                self.registers.v[x] = rng.gen::<u8>() & kk;
            }
            0xd => {
                // Dxyn - DRW Vx, Vy, nibble - Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
                let x = get_nibble(2, opcode);
                let y = get_nibble(1, opcode);
                let n = get_nibble(0, opcode);
                for i in 0..n {
                    let i_offset = self.registers.v[y] as usize + i;
                    let sprite = self.memory[self.registers.i as usize + i];
                    for j in 0..8 {
                        let j_offset = self.registers.v[x] as usize + j;
                        let pixel = (sprite >> (7 - j)) & 0x1;

                        if j_offset < C8_WIDTH && i_offset < C8_HEIGHT && pixel == 0x1 {
                            if self.display[i_offset][j_offset] {
                                self.registers.v[0xF] = 1;
                            }
                            self.display[i_offset][j_offset] ^= true;
                        }
                    }
                }
            }
            0xe => {
                // Exoo - Key operations with Vx depending on the value of oo
                let x = get_nibble(2, opcode);
                match opcode & 0xff {
                    0x9e => {
                        // Ex9E - SKP Vx - Skip next instruction if key with the value of Vx is pressed
                        if self.key_state[self.registers.v[x] as usize] {
                            self.registers.pc += 2;
                        }
                    }
                    0xa1 => {
                        // ExA1 - SKNP Vx - Skip next instruction if key with value of Vx is not pressed
                        if !self.key_state[self.registers.v[x] as usize] {
                            self.registers.pc += 2;
                        }
                    }
                    _ => panic!("Invalid opcode: {}", opcode),
                }
            }
            0xf => {
                // Fxoo - Operations with Vx depending on the value of oo
                let x = get_nibble(2, opcode);
                let op = (opcode & 0xff) as u8;
                self.process_opcode_f(x, op);
            }
            _ => panic!("Unhandled opcode: {}", opcode),
        }
    }

    fn process_opcode_8(&mut self, x: usize, y: usize, op: u8) {
        match op {
            0x0 => {
                // 8xy0 - LD Vx, Vy - Set Vx := Vy
                self.registers.v[x] = self.registers.v[y];
            }
            0x1 => {
                // 8xy1 - OR Vx, Vy - Set Vx := Vx OR Vy
                self.registers.v[x] |= self.registers.v[y];
            }
            0x2 => {
                // 8xy2 - AND Vx, Vy - Set Vx := Vx AND Vy
                self.registers.v[x] &= self.registers.v[y];
            }
            0x3 => {
                // 8xy3 - XOR Vx, Vy - Set Vx := Vx XOR Vy
                self.registers.v[x] ^= self.registers.v[y];
            }
            0x4 => {
                // 8xy4 - ADD Vx, Vy - Set Vx := Vx + Vy, set VF := carry
                let (val, carry) = (self.registers.v[x]).overflowing_add(self.registers.v[y]);
                self.registers.v[x] = val;
                if carry {
                    self.registers.v[0xf] = 0x1;
                } else {
                    self.registers.v[0xf] = 0x0;
                }
            }
            0x5 => {
                // 8xy5 - SUB Vx, Vy - Set Vx := Vx - Vy, set VF := NOT borrow
                let (val, borrow) = (self.registers.v[x]).overflowing_sub(self.registers.v[y]);
                self.registers.v[x] = val;
                if borrow {
                    self.registers.v[0xf] = 0x1;
                } else {
                    self.registers.v[0xf] = 0x0;
                }
            }
            0x6 => {
                // 8xy6 - SHR Vx - Set Vx := Vx >> 1
                self.registers.v[0xf] = self.registers.v[x] & 0x1;
                self.registers.v[x] >>= 1;
            }
            0x7 => {
                // 8xy7 - SUBN Vx, Vy - Set Vx := Vy - Vx, set VF := NOT borrow
                let (val, borrow) = (self.registers.v[y]).overflowing_sub(self.registers.v[x]);
                self.registers.v[x] = val;
                if borrow {
                    self.registers.v[0xf] = 0x1;
                } else {
                    self.registers.v[0xf] = 0x0;
                }
            }
            0xe => {
                // 8xyE - SHL Vx - Set Vx := Vx << 1
                self.registers.v[0xf] = self.registers.v[x] & 0x8;
                self.registers.v[x] <<= 1;
            }
            _ => panic!("Unhandled opcode_8: {}", op),
        }
    }

    fn process_opcode_f(&mut self, x: usize, op: u8) {
        match op {
            0x07 => {
                // Fx07 - LD Vx, DT - Set Vx := Delay timer value
                self.registers.v[x] = self.registers.delay_timer;
            }
            0x0a => {
                // Fx0A - LD Vx, K - Wait for a key press, store the value of the key in Vx
                let mut key_pressed = false;
                for i in 0..16 {
                    if self.key_state[i] {
                        key_pressed = true;
                        self.registers.v[x] = i as u8;
                        break;
                    }
                }
                if !key_pressed {
                    // Decrement PC to keep the emulation on the same instruction
                    self.registers.pc -= 2;
                }
            }
            0x15 => {
                // Fx15 - LD DT, Vx - Set delay timer := Vx
                self.registers.delay_timer = self.registers.v[x];
            }
            0x18 => {
                // Fx18 - LD ST, Vx - Set sound timer := Vx
                self.registers.sound_timer = self.registers.v[x];
            }
            0x1e => {
                // Fx1E - ADD I, Vx - Set I := I + Vx
                self.registers.i += self.registers.v[x] as u16;
            }
            0x29 => {
                // Fx29 - LD F, Vx - Set I := location of sprite for digit Vx
                self.registers.i = self.registers.v[x] as u16 * 5;
            }
            0x33 => {
                // Fx33 - LD B, Vx - Store BCD representation of Vx in memory locations I, I+1, and I+2
                let mut val = self.registers.v[x];
                let mut out = [0u8; 3]; // 0 - hundreds, 1 - tens, 2 - ones
                for i in (0..2).rev() {
                    if val != 0 {
                        out[i] = (val % 10) as u8;
                    }
                    val /= 10;
                }
                let addr = self.registers.i as usize;
                self.memory[addr..addr + 3].copy_from_slice(&out[..]);
            }
            0x55 => {
                // Fx55 - LD [I], Vx - Store registers V0 through Vx, in memory starting at location I
                for i in 0..=x {
                    self.memory[self.registers.i as usize + i] = self.registers.v[i];
                }
            }
            0x65 => {
                // Fx65 - LD Vx, [I] - Read registers V0 through Vx from memory starting at location I
                for i in 0..=x {
                    self.registers.v[i] = self.memory[self.registers.i as usize + i];
                }
            }
            _ => panic!("Unhandled opcode_f: {}", op),
        }
    }
}

/// Gets the nibble corresponding to the zero-based index of the set of four bits in the u16.
///
/// Indexes are laid out as 3333_2222_1111_0000 where the least siginificant four bits are 0 and
/// the most significan four bits are 3.
fn get_nibble(index: u8, value: u16) -> usize {
    let offset = index * 4;
    ((value & (0xf << offset)) >> offset) as usize
}
