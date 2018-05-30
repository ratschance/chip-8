
struct Cpu {
    registers: Registers,
    memory: [u8; 4096],
}

struct Registers {
    v: [u8; 16],     // Vx where x is a hexidecimal digit 0..F
    i: u16,          // Generally used to store memory addresses
    delay_timer: u8,
    sound_timer: u8,
    pc: u16,         // Program Counter - used to store currently executing address
    sp: u8,          // Stack Pointer - used to point to the topmost level of the stack
    stack: [u16; 16],
}

fn decode_opcode(opcode: u16) {
    match opcode >> 12 {
        0x0 => {
            // 0xxx
            match opcode {
                0x00e0 => {
                    // CLS - Clear display
                },
                0x00ee => {
                    // RET - return from subroutine
                },
                _ => {
                    // Legacy routine, ignored
                }
            }
        },
        0x1 => {
            // 1nnn - JP addr - Jump to location nnn
            let loc = opcode & 0xfff;
        },
        0x2 => {
            // 2nnn - CALL addr - Call subroutine at nnn
            let loc = opcode & 0xfff;
        },
        0x3 => {
            // 3xkk - SE Vx, byte - Skip next instruction if Vx = kk
            let x = get_nibble(2, opcode);
            let val = opcode & 0xff;
        },
        0x4 => {
            // 4xkk - SNE Vx, byte - Skip next instruction if Vx != kk
            let x = get_nibble(2, opcode);
            let val = opcode & 0xff;
        },
        0x5 => {
            // 5xy0 - SE Vx, Vy - Skip next instruction if Vx = Vy
            let x = get_nibble(2, opcode);
            let y = get_nibble(1, opcode);
        },
        0x6 => {
            // 6xkk - LD Vx, byte - Set Vx := kk
            let x = get_nibble(2, opcode);
            let val = opcode & 0xff;
        },
        0x7 => {
            // 7xkk - ADD Vx, byte - Set Vx := Vx + kk
            let x = get_nibble(2, opcode);
            let val = opcode & 0xff;
        },
        0x8 => {
            // 8xyo - Operations between Vx and Vy depending on the value of o
            let x = get_nibble(2, opcode);
            let y = get_nibble(1, opcode);
            let op = get_nibble(0, opcode);
            process_opcode_8(x, y, op);
        }
        0x9 => {
            // 9xy0 - SNE Vx, Vy - Skip next instruction if Vx != Vy
            let x = get_nibble(2, opcode);
            let y = get_nibble(1, opcode);
        },
        0xa => {
            // Annn - LD I, addr - Set I := nnn
            let val = opcode & 0xfff;
        },
        0xb => {
            // Bnnn - JP V0, addr - Jump to location nnn + V0
            let loc = opcode & 0xfff;
        },
        0xc => {
            // Cxkk - RND Vx, byte - Set Vx := random byte AND kk
            let x = get_nibble(2, opcode);
            let val = opcode & 0xff;
        },
        0xd => {
            // Dxyn - DRW Vx, Vy, nibble - Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
            let x = get_nibble(2, opcode);
            let y = get_nibble(1, opcode);
            let n = get_nibble(0, opcode);
        },
        0xe => {
            // Exoo - Key operations with Vx depending on the value of oo
            let x = get_nibble(2, opcode);
            match opcode & 0xff {
                0x9e => {
                    // Ex9E - SKP Vx - Skip next instruction if key with the value of Vx is pressed
                },
                0xa1 => {
                    // ExA1 - SKNP Vx - Skip next instruction if ket with value of Vx is not pressed
                },
                _ => panic!("Invalid opcode: {}", opcode)
            }
        },
        0xf => {
            // Fxoo - Operations with Vx depending on the value of oo
            let x = get_nibble(2, opcode);
            let op = (opcode & 0xff) as u8;
            process_opcode_f(x, op);
        }
        _ => panic!("Unhandled opcode: {}", opcode)
    }
}

fn process_opcode_8(x: u8, y: u8, op: u8) {
    //TODO: Add operation matching for 0x8nnn opcodes
}

fn process_opcode_f(x: u8, op: u8) {
    //TODO: Add operation matching for 0xFnnn opcodes
}

/// Gets the nibble corresponding to the zero-based index of the set of four bits in the u16.
/// Indexes are laid out as 3333_2222_1111_0000 where the least siginificant four bits are 0 and
/// the most significan four bits are 3.
fn get_nibble(index: u8, value: u16) -> u8 {
    let offset = index * 4;
    ((value & (0xf << offset)) >> offset) as u8
}
