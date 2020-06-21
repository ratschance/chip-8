use rand::Rng;

pub const C8_WIDTH: usize = 64;
pub const C8_HEIGHT: usize = 32;

pub struct Cpu {
    registers: Registers,
    memory: [u8; 4096],
    display: [[bool; C8_WIDTH]; C8_HEIGHT],
    key_state: [bool; 16],
    waiting: Option<usize>,
    has_disp_update: bool,
    cycle_count: usize,
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

struct Opcode {
    a: u8,
    kk: u8,
    n: u8,
    nnn: u16,
    x: usize,
    y: usize,
}

impl Opcode {
    pub fn from_op(op: u16) -> Self {
        Opcode {
            a: (op >> 12 & 0xf) as u8,
            kk: (op & 0xff) as u8,
            n: (op & 0xf) as u8,
            nnn: op & 0xfff,
            x: (op >> 8 & 0xf) as usize,
            y: (op >> 4 & 0xf) as usize,
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
            waiting: None,
            has_disp_update: false,
            cycle_count: 0,
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
        if self.waiting == None {
            self.has_disp_update = false;

            let pc = self.registers.pc as usize;
            self.registers.pc += 2;
            self.process_opcode((self.memory[pc] as u16) << 8 | self.memory[pc + 1] as u16);
        }

        if self.cycle_count % 8 == 0 {
            if self.registers.delay_timer > 0 {
                self.registers.delay_timer -= 1;
            }

            if self.registers.sound_timer > 0 {
                //TODO: Make sound
                self.registers.sound_timer -= 1;
            }
        }
        self.cycle_count += 1;
    }

    pub fn view_display(&mut self) -> &[[bool; C8_WIDTH]; C8_HEIGHT] {
        &self.display
    }

    pub fn set_key_pressed(&mut self, key: usize) {
        self.key_state[key] = true;
        if let Some(x) = self.waiting {
            self.registers.v[x] = key as u8;
            self.waiting = None;
        }
    }

    pub fn set_key_released(&mut self, key: usize) {
        self.key_state[key] = false;
    }

    pub fn has_disp_update(&self) -> bool {
        self.has_disp_update
    }

    fn process_opcode(&mut self, opcode: u16) {
        let op = Opcode::from_op(opcode);
        match (op.a, op.x, op.y, op.n) {
            (0x0, 0x0, 0xE, 0x0) => self.cls(),
            (0x0, 0x0, 0xE, 0xE) => self.ret(),
            (0x0, _, _, _) => self.sys(op.nnn),
            (0x1, _, _, _) => self.jp(op.nnn),
            (0x2, _, _, _) => self.call(op.nnn),
            (0x3, _, _, _) => self.sec(op.x, op.kk),
            (0x4, _, _, _) => self.snec(op.x, op.kk),
            (0x5, _, _, 0x0) => self.se(op.x, op.y),
            (0x6, _, _, _) => self.ldc(op.x, op.kk),
            (0x7, _, _, _) => self.addc(op.x, op.kk),
            (0x8, _, _, 0x0) => self.ld(op.x, op.y),
            (0x8, _, _, 0x1) => self.or(op.x, op.y),
            (0x8, _, _, 0x2) => self.and(op.x, op.y),
            (0x8, _, _, 0x3) => self.xor(op.x, op.y),
            (0x8, _, _, 0x4) => self.add(op.x, op.y),
            (0x8, _, _, 0x5) => self.sub(op.x, op.y),
            (0x8, _, _, 0x6) => self.shr(op.x),
            (0x8, _, _, 0x7) => self.subn(op.x, op.y),
            (0x8, _, _, 0xE) => self.shl(op.x),
            (0x9, _, _, 0x0) => self.sne(op.x, op.y),
            (0xA, _, _, _) => self.ldi(op.nnn),
            (0xB, _, _, _) => self.jp0(op.nnn),
            (0xC, _, _, _) => self.rnd(op.x, op.kk),
            (0xD, _, _, _) => self.drw(op.x, op.y, op.n),
            (0xE, _, 0x9, 0xE) => self.skp(op.x),
            (0xE, _, 0xA, 0x1) => self.sknp(op.x),
            (0xF, _, 0x0, 0x7) => self.ldxdt(op.x),
            (0xF, _, 0x0, 0xA) => self.ldxk(op.x),
            (0xF, _, 0x1, 0x5) => self.lddtx(op.x),
            (0xF, _, 0x1, 0x8) => self.ldstx(op.x),
            (0xF, _, 0x1, 0xE) => self.addi(op.x),
            (0xF, _, 0x2, 0x9) => self.ldf(op.x),
            (0xF, _, 0x3, 0x3) => self.ldb(op.x),
            (0xF, _, 0x5, 0x5) => self.ldix(op.x),
            (0xF, _, 0x6, 0x5) => self.ldxi(op.x),
            (_, _, _, _) => panic!(
                "Unidentified opcode: {:X} {:X} {:X} {:X}",
                op.a, op.x, op.y, op.n
            ),
        }
    }

    /// CLS - Clear display
    fn cls(&mut self) {
        for i in 0..self.display.len() {
            for j in 0..self.display[i].len() {
                self.display[i][j] = false;
            }
        }
        self.has_disp_update = true;
    }

    /// RET - return from subroutine
    fn ret(&mut self) {
        if self.registers.sp == 0 {
            panic!("Returned when stack pointer was already 0");
        }
        self.registers.pc = self.registers.stack[self.registers.sp as usize];
        self.registers.sp -= 1;
    }

    /// Legacy routine, ignored
    fn sys(&mut self, _nnn: u16) {}

    /// 1nnn - JP addr - Jump to location nnn
    fn jp(&mut self, nnn: u16) {
        self.registers.pc = nnn;
    }

    /// 2nnn - CALL addr - Call subroutine at nnn
    fn call(&mut self, nnn: u16) {
        self.registers.sp += 1;
        self.registers.stack[self.registers.sp as usize] = self.registers.pc;
        self.registers.pc = nnn;
    }

    /// 3xkk - SE Vx, byte - Skip next instruction if Vx = kk
    fn sec(&mut self, x: usize, kk: u8) {
        if self.registers.v[x] == kk {
            self.registers.pc += 2;
        }
    }

    /// 4xkk - SNE Vx, byte - Skip next instruction if Vx != kk
    fn snec(&mut self, x: usize, kk: u8) {
        if self.registers.v[x] != kk {
            self.registers.pc += 2;
        }
    }

    /// 5xy0 - SE Vx, Vy - Skip next instruction if Vx = Vy
    fn se(&mut self, x: usize, y: usize) {
        if self.registers.v[x] == self.registers.v[y] {
            self.registers.pc += 2;
        }
    }

    /// 6xkk - LD Vx, byte - Set Vx := kk
    fn ldc(&mut self, x: usize, kk: u8) {
        self.registers.v[x] = kk;
    }

    /// 7xkk - ADD Vx, byte - Set Vx := Vx + kk
    fn addc(&mut self, x: usize, kk: u8) {
        let val = self.registers.v[x].wrapping_add(kk);
        self.registers.v[x] = val;
    }

    /// 8xy0 - LD Vx, Vy - Set Vx := Vy
    fn ld(&mut self, x: usize, y: usize) {
        self.registers.v[x] = self.registers.v[y];
    }

    /// 8xy1 - OR Vx, Vy - Set Vx := Vx OR Vy
    fn or(&mut self, x: usize, y: usize) {
        self.registers.v[x] |= self.registers.v[y];
    }

    /// 8xy2 - AND Vx, Vy - Set Vx := Vx AND Vy
    fn and(&mut self, x: usize, y: usize) {
        self.registers.v[x] &= self.registers.v[y];
    }

    /// 8xy3 - XOR Vx, Vy - Set Vx := Vx XOR Vy
    fn xor(&mut self, x: usize, y: usize) {
        self.registers.v[x] ^= self.registers.v[y];
    }

    /// 8xy4 - ADD Vx, Vy - Set Vx := Vx + Vy, set VF := carry
    fn add(&mut self, x: usize, y: usize) {
        let (val, carry) = (self.registers.v[x]).overflowing_add(self.registers.v[y]);
        self.registers.v[x] = val;
        if carry {
            self.registers.v[0xf] = 0x1;
        } else {
            self.registers.v[0xf] = 0x0;
        }
    }

    /// 8xy5 - SUB Vx, Vy - Set Vx := Vx - Vy, set VF := NOT borrow
    fn sub(&mut self, x: usize, y: usize) {
        let (val, borrow) = (self.registers.v[x]).overflowing_sub(self.registers.v[y]);
        self.registers.v[x] = val;
        if borrow {
            self.registers.v[0xf] = 0x1;
        } else {
            self.registers.v[0xf] = 0x0;
        }
    }

    /// 8xy6 - SHR Vx - Set Vx := Vx >> 1
    fn shr(&mut self, x: usize) {
        self.registers.v[0xf] = self.registers.v[x] & 0x1;
        self.registers.v[x] >>= 1;
    }

    /// 8xy7 - SUBN Vx, Vy - Set Vx := Vy - Vx, set VF := NOT borrow
    fn subn(&mut self, x: usize, y: usize) {
        let (val, borrow) = (self.registers.v[y]).overflowing_sub(self.registers.v[x]);
        self.registers.v[x] = val;
        if borrow {
            self.registers.v[0xf] = 0x1;
        } else {
            self.registers.v[0xf] = 0x0;
        }
    }

    /// 8xyE - SHL Vx - Set Vx := Vx << 1
    fn shl(&mut self, x: usize) {
        self.registers.v[0xf] = (self.registers.v[x] & 0x80) >> 7;
        self.registers.v[x] <<= 1;
    }

    /// 9xy0 - SNE Vx, Vy - Skip next instruction if Vx != Vy
    fn sne(&mut self, x: usize, y: usize) {
        if self.registers.v[x] != self.registers.v[y] {
            self.registers.pc += 2;
        }
    }

    /// Annn - LD I, addr - Set I := nnn
    fn ldi(&mut self, nnn: u16) {
        self.registers.i = nnn;
    }

    /// Bnnn - JP V0, addr - Jump to location nnn + V0
    fn jp0(&mut self, nnn: u16) {
        self.registers.pc = nnn + self.registers.v[0] as u16;
    }

    /// Cxkk - RND Vx, byte - Set Vx := random byte AND kk
    fn rnd(&mut self, x: usize, kk: u8) {
        let mut rng = rand::thread_rng();
        self.registers.v[x] = rng.gen::<u8>() & kk;
    }

    /// Dxyn - DRW Vx, Vy, nibble - Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
    fn drw(&mut self, x: usize, y: usize, n: u8) {
        self.registers.v[0xF] = 0;
        for i in 0..n as usize {
            let i_offset = (self.registers.v[y] as usize + i) % C8_HEIGHT;
            let sprite = self.memory[self.registers.i as usize + i];
            for j in 0..8 {
                let j_offset = (self.registers.v[x] as usize + j) % C8_WIDTH;
                let pixel = (sprite >> (7 - j)) & 0x1;

                if pixel == 0x1 {
                    if self.display[i_offset][j_offset] {
                        self.registers.v[0xF] = 1;
                    }
                    self.display[i_offset][j_offset] ^= true;
                }
            }
        }
        self.has_disp_update = true;
    }

    /// Ex9E - SKP Vx - Skip next instruction if key with the value of Vx is pressed
    fn skp(&mut self, x: usize) {
        if self.key_state[self.registers.v[x] as usize] {
            self.registers.pc += 2;
        }
    }

    /// ExA1 - SKNP Vx - Skip next instruction if key with value of Vx is not pressed
    fn sknp(&mut self, x: usize) {
        if !self.key_state[self.registers.v[x] as usize] {
            self.registers.pc += 2;
        }
    }

    /// Fx07 - LD Vx, DT - Set Vx := Delay timer value
    fn ldxdt(&mut self, x: usize) {
        self.registers.v[x] = self.registers.delay_timer;
    }

    /// Fx0A - LD Vx, K - Wait for a key press, store the value of the key in Vx
    fn ldxk(&mut self, x: usize) {
        self.waiting = Some(x)
    }

    /// Fx15 - LD DT, Vx - Set delay timer := Vx
    fn lddtx(&mut self, x: usize) {
        self.registers.delay_timer = self.registers.v[x];
    }

    /// Fx18 - LD ST, Vx - Set sound timer := Vx
    fn ldstx(&mut self, x: usize) {
        self.registers.sound_timer = self.registers.v[x];
    }

    /// Fx1E - ADD I, Vx - Set I := I + Vx
    fn addi(&mut self, x: usize) {
        self.registers.i += self.registers.v[x] as u16;
    }

    /// Fx29 - LD F, Vx - Set I := location of sprite for digit Vx
    fn ldf(&mut self, x: usize) {
        self.registers.i = self.registers.v[x] as u16 * 5;
    }

    /// Fx33 - LD B, Vx - Store BCD representation of Vx in memory locations I, I+1, and I+2
    fn ldb(&mut self, x: usize) {
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

    /// Fx55 - LD [I], Vx - Store registers V0 through Vx, in memory starting at location I
    fn ldix(&mut self, x: usize) {
        for i in 0..=x {
            self.memory[self.registers.i as usize + i] = self.registers.v[i];
        }
    }

    /// Fx65 - LD Vx, [I] - Read registers V0 through Vx from memory starting at location I
    fn ldxi(&mut self, x: usize) {
        for i in 0..=x {
            self.registers.v[i] = self.memory[self.registers.i as usize + i];
        }
    }
}
