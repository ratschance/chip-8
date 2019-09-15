
struct Cpu {
    registers: Registers,
    memory: [u8; 4096],
}

struct Registers {
    v: [u8; 16],     // Vx where x is a hexadecimal digit 0..F
    i: u16,          // Generally used to store memory addresses
    delay_timer: u8,
    sound_timer: u8,
    pc: u16,         // Program Counter - used to store currently executing address
    sp: u8,          // Stack Pointer - used to point to the topmost level of the stack
    stack: [u16; 16],
}

impl Registers {
    fn initialize() -> Registers {
        Registers {
            v: [0; 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: 0,
            sp: 0,
            stack: [0; 16],
        }
    }
}

impl Cpu {
    fn initialize() -> Cpu {
        Cpu {
            registers: Registers::initialize(),
            memory: [0; 4096],
        }
    }

    fn process_opcode(&mut self, opcode: u16) {
        match opcode >> 12 {
            0x0 => {
                // 0xxx
                match opcode {
                    0x00e0 => {
                        // CLS - Clear display
                        //TODO: unimplemented
                    },
                    0x00ee => {
                        // RET - return from subroutine
                        if self.registers.sp == 0 {
                            panic!("Returned when stack pointer was already 0");
                        }
                        self.registers.pc = self.registers.stack[self.registers.sp as usize];
                        self.registers.sp -= 1;
                    },
                    _ => {
                        // Legacy routine, ignored
                    }
                }
            },
            0x1 => {
                // 1nnn - JP addr - Jump to location nnn
                self.registers.pc = opcode & 0xfff;
            },
            0x2 => {
                // 2nnn - CALL addr - Call subroutine at nnn
                self.registers.sp += 1;
                self.registers.stack[self.registers.sp as usize] = self.registers.pc;
                self.registers.pc = opcode & 0xfff;
            },
            0x3 => {
                // 3xkk - SE Vx, byte - Skip next instruction if Vx = kk
                let x = get_nibble(2, opcode);
                if self.registers.v[x] == (opcode & 0xff) as u8 {
                    self.registers.pc += 2;
                }
            },
            0x4 => {
                // 4xkk - SNE Vx, byte - Skip next instruction if Vx != kk
                let x = get_nibble(2, opcode);
                if self.registers.v[x] != (opcode & 0xff) as u8 {
                    self.registers.pc += 2;
                }
            },
            0x5 => {
                // 5xy0 - SE Vx, Vy - Skip next instruction if Vx = Vy
                let x = get_nibble(2, opcode);
                let y = get_nibble(1, opcode);
                if self.registers.v[x] == self.registers.v[y] {
                    self.registers.pc += 2;
                }
            },
            0x6 => {
                // 6xkk - LD Vx, byte - Set Vx := kk
                let x = get_nibble(2, opcode);
                self.registers.v[x] = (opcode & 0xff) as u8;
            },
            0x7 => {
                // 7xkk - ADD Vx, byte - Set Vx := Vx + kk
                let x = get_nibble(2, opcode);
                let val = self.registers.v[x] + (opcode & 0xff) as u8;
                self.registers.v[x] = val;
            },
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
            },
            0xa => {
                // Annn - LD I, addr - Set I := nnn
                self.registers.i = opcode & 0xfff;
            },
            0xb => {
                // Bnnn - JP V0, addr - Jump to location nnn + V0
                let loc = opcode & 0xfff;
                self.registers.pc = loc + self.registers.v[0] as u16;
            },
            0xc => {
                // Cxkk - RND Vx, byte - Set Vx := random byte AND kk
                let x = get_nibble(2, opcode);
                let val = opcode & 0xff;
                //TODO: unimplemented
            },
            0xd => {
                // Dxyn - DRW Vx, Vy, nibble - Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
                let x = get_nibble(2, opcode);
                let y = get_nibble(1, opcode);
                let n = get_nibble(0, opcode);
                //TODO: unimplemented
            },
            0xe => {
                // Exoo - Key operations with Vx depending on the value of oo
                let x = get_nibble(2, opcode);
                match opcode & 0xff {
                    0x9e => {
                        // Ex9E - SKP Vx - Skip next instruction if key with the value of Vx is pressed
                        //TODO: unimplemented
                    },
                    0xa1 => {
                        // ExA1 - SKNP Vx - Skip next instruction if key with value of Vx is not pressed
                        //TODO: unimplemented
                    },
                    _ => panic!("Invalid opcode: {}", opcode)
                }
            },
            0xf => {
                // Fxoo - Operations with Vx depending on the value of oo
                let x = get_nibble(2, opcode);
                let op = (opcode & 0xff) as u8;
                self.process_opcode_f(x, op);
            },
            _ => panic!("Unhandled opcode: {}", opcode)
        }
    }

    fn process_opcode_8(&mut self, x: usize, y: usize, op: u8) {
        match op {
            0x0 => {
                // 8xy0 - LD Vx, Vy - Set Vx := Vy
                self.registers.v[x] = self.registers.v[y];
            },
            0x1 => {
                // 8xy1 - OR Vx, Vy - Set Vx := Vx OR Vy
                self.registers.v[x] = self.registers.v[x] | self.registers.v[y];
            },
            0x2 => {
                // 8xy2 - AND Vx, Vy - Set Vx := Vx AND Vy
                self.registers.v[x] = self.registers.v[x] & self.registers.v[y];
            },
            0x3 => {
                // 8xy3 - XOR Vx, Vy - Set Vx := Vx XOR Vy
                self.registers.v[x] = self.registers.v[x] ^ self.registers.v[y];
            },
            0x4 => {
                // 8xy4 - ADD Vx, Vy - Set Vx := Vx + Vy, set VF := carry
                let (val, carry) = (self.registers.v[x]).overflowing_add(self.registers.v[y]);
                self.registers.v[x] = val;
                if carry {
                    self.registers.v[0xf] = 0x1;
                } else {
                    self.registers.v[0xf] = 0x0;
                }
            },
            0x5 => {
                // 8xy5 - SUB Vx, Vy - Set Vx := Vx - Vy, set VF := NOT borrow
                let (val, borrow) = (self.registers.v[x]).overflowing_sub(self.registers.v[y]);
                self.registers.v[x] = val;
                if borrow {
                    self.registers.v[0xf] = 0x1;
                } else {
                    self.registers.v[0xf] = 0x0;
                }
            },
            0x6 => {
                // 8xy6 - SHR Vx - Set Vx := Vx >> 1
                self.registers.v[0xf] = self.registers.v[x] & 0x1;
                self.registers.v[x] = self.registers.v[x] >> 1;
            },
            0x7 => {
                // 8xy7 - SUBN Vx, Vy - Set Vx := Vy - Vx, set VF := NOT borrow
                let (val, borrow) = (self.registers.v[y]).overflowing_sub(self.registers.v[x]);
                self.registers.v[x] = val;
                if borrow {
                    self.registers.v[0xf] = 0x1;
                } else {
                    self.registers.v[0xf] = 0x0;
                }
            },
            0xe => {
                // 8xyE - SHL Vx - Set Vx := Vx << 1
                self.registers.v[0xf] = self.registers.v[x] & 0x8;
                self.registers.v[x] = self.registers.v[x] << 1;
            },
            _ => panic!("Unhandled opcode: {}", op)
        }
    }

    fn process_opcode_f(&mut self, x: usize, op: u8) {
        match op {
            0x07 => {
                // Fx07 - LD Vx, DT - Set Vx := Delay timer value
                self.registers.v[x] = self.registers.delay_timer;
            },
            0x0a => {
                // Fx0A - LD Vx, K - Wait for a key press, store the value of the key in Vx
                //TODO: unimplemented
            },
            0x15 => {
                // Fx15 - LD DT, Vx - Set delay timer := Vx
                self.registers.delay_timer = self.registers.v[x];
            },
            0x18 => {
                // Fx18 - LD ST, Vx - Set sound timer := Vx
                self.registers.sound_timer = self.registers.v[x];
            },
            0x1e => {
                // Fx1E - ADD I, Vx - Set I := I + Vx
                self.registers.i = self.registers.i + self.registers.v[x] as u16;
            },
            0x29 => {
                // Fx29 - LD F, Vx - Set I := location of sprite for digit Vx
                //TODO: unimplemented
            },
            0x33 => {
                // Fx33 - LD B, Vx - Store BCD representation of Vx in memory locations I, I+1, and I+2
                //TODO: unimplemented
            },
            0x55 => {
                // Fx55 - LD [I], Vx - Store registers V0 through Vx, in memory starting at location I
                //TODO: unimplemented
            },
            0x65 => {
                // Fx65 - LD Vx, [I] - Read registers V0 through Vx from memory starting at location I
                //TODO: unimplemented
            },
            _ => panic!("Unhandled opcode: {}", op)
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
