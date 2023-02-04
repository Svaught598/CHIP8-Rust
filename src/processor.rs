use rand::Rng;
use crate::{HEIGHT, WIDTH};

const RAM_SZ: usize = 4096;
const STACK_SZ: usize = 16;
const KEYS_SZ: usize = 16;
const V_SZ: usize = 16;

#[derive(Clone, Debug)]
pub struct Processor {
    pub pixels: Vec<bool>,
    cycle_buffer: Vec<bool>,

    // memory
    pub ram: [u8; RAM_SZ],
    pub stack: [u16; STACK_SZ],
    pub v: [u8; V_SZ],
    pub i: u16,
    pub pc: u16,
    pub sp: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub keypad: [bool; KEYS_SZ],
}

#[derive(Debug)]
pub enum ProcessorAction {
    NextInstruction,
    SkipInstruction,
    JumpInstruction(u16),
}

#[allow(non_snake_case)]
#[allow(unused)]
impl Processor {
    pub fn new(sz: usize) -> Self {
        Self {
            pixels: vec![false; sz],
            cycle_buffer: vec![false; sz],

            // memory
            ram: [0; RAM_SZ],
            stack: [0; STACK_SZ],
            v: [0; V_SZ],
            i: 0,
            pc: 0x200,
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false; KEYS_SZ],
        }
    }

    pub fn load(&mut self, data: Vec<u8>) {
        for (i, &byte) in data.iter().enumerate() {
            let addr = 0x200 + i;
            if addr < 4096 {
                self.ram[0x200 + i] = byte;
            } else {
                break;
            }
        }
    }

    pub fn get_instruction(&self) -> u16 {
        let high_nibble = (self.ram[self.pc as usize] as u16) << 8;
        let low_nibble = self.ram[(self.pc + 1) as usize] as u16;
        return high_nibble | low_nibble;
    }

    pub fn tick(&mut self) {
        let op = self.get_instruction();
        let nibbles = (
            ((op & 0xF000) >> 12) as u8,
            ((op & 0x0F00) >> 8) as u8,
            ((op & 0x00F0) >> 4) as u8,
            (op & 0x000F) as u8,
        );

        let action = match nibbles {
            (0x0, 0x0, 0xF, 0xF) => self.op_00FF(nibbles),
            (0x0, 0x0, 0xE, 0x0) => self.op_00E0(),
            (0x0, 0x0, 0xE, 0xE) => self.op_00EE(),
            (0x1, _, _, _) => self.op_1nnn(nibbles),
            (0x2, _, _, _) => self.op_2nnn(nibbles),
            (0x3, _, _, _) => self.op_3xkk(nibbles),
            (0x4, _, _, _) => self.op_4xkk(nibbles),
            (0x5, _, _, _) => self.op_5xy0(nibbles),
            (0x6, _, _, _) => self.op_6xkk(nibbles),
            (0x7, _, _, _) => self.op_7xkk(nibbles),
            (0x8, _, _, 0x0) => self.op_8xy0(nibbles),
            (0x8, _, _, 0x1) => self.op_8xy1(nibbles),
            (0x8, _, _, 0x2) => self.op_8xy2(nibbles),
            (0x8, _, _, 0x3) => self.op_8xy3(nibbles),
            (0x8, _, _, 0x4) => self.op_8xy4(nibbles),
            (0x8, _, _, 0x5) => self.op_8xy5(nibbles),
            (0x8, _, _, 0x6) => self.op_8xy6(nibbles),
            (0x8, _, _, 0x7) => self.op_8xy7(nibbles),
            (0x8, _, _, 0xE) => self.op_8xyE(nibbles),
            (0x9, _, _, 0x0) => self.op_9xy0(nibbles),
            (0xA, _, _, _) => self.op_Annn(nibbles),
            (0xB, _, _, _) => self.op_Bnnn(nibbles),
            (0xC, _, _, _) => self.op_Cxkk(nibbles),
            (0xD, _, _, 0x0) => self.op_Dxy0(nibbles),
            (0xD, _, _, _) => self.op_Dxyn(nibbles),
            (0xE, _, 0x9, 0x1) => self.op_Ex9E(nibbles),
            (0xE, _, 0xA, 0x1) => self.op_ExA1(nibbles),
            (0xF, _, 0x0, 0x7) => self.op_Fx07(nibbles),
            (0xF, _, 0x0, 0xA) => self.op_Fx0A(nibbles),
            (0xF, _, 0x1, 0x5) => self.op_Fx15(nibbles),
            (0xF, _, 0x1, 0x8) => self.op_Fx18(nibbles),
            (0xF, _, 0x1, 0xE) => self.op_Fx1E(nibbles),
            (0xF, _, 0x2, 0x9) => self.op_Fx29(nibbles),
            (0xF, _, 0x3, 0x3) => self.op_Fx33(nibbles),
            (0xF, _, 0x5, 0x5) => self.op_Fx55(nibbles),
            (0xF, _, 0x6, 0x5) => self.op_Fx65(nibbles),
            (0x0, 0x0, 0xC, _) => self.op_00Cn(nibbles),
            (0x0, 0x0, 0xF, 0xB) => self.op_00FB(nibbles),
            (0x0, 0x0, 0xF, 0xC) => self.op_00FC(nibbles),
            (0x0, 0x0, 0xF, 0xD) => self.op_00FD(nibbles),
            (0x0, 0x0, 0xF, 0xE) => self.op_00FE(nibbles),
            (0xF, _, 0x3, 0x0) => self.op_Fx30(nibbles),
            (0xF, _, 0x7, 0x5) => self.op_Fx75(nibbles),
            (0xF, _, 0x8, 0x5) => self.op_Fx85(nibbles),
            _ => ProcessorAction::NextInstruction,
        };

        match action {
            ProcessorAction::NextInstruction => self.pc += 2,
            ProcessorAction::SkipInstruction => self.pc += 4,
            ProcessorAction::JumpInstruction(j) => self.pc = j + 2 as u16,
        }
    }

    // Clear the display.
    pub fn op_00E0(&mut self) -> ProcessorAction {
        for pix in self.cycle_buffer.iter_mut() { *pix = false; }
        self.cycle_buffer.clone_into(&mut self.pixels);
        ProcessorAction::NextInstruction
    }

    // The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
    pub fn op_00EE(&mut self) -> ProcessorAction {
        let addr = self.stack[self.sp as usize];
        self.sp -= 1;
        ProcessorAction::JumpInstruction(addr)
    }

    // The interpreter sets the program counter to nnn.
    pub fn op_1nnn(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (nnn, ..) = process_nibbles(nibbles);
        ProcessorAction::JumpInstruction(nnn)
    }

    // The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
    pub fn op_2nnn(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (nnn, ..) = process_nibbles(nibbles);
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc;
        ProcessorAction::JumpInstruction(nnn)
    }

    // The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
    pub fn op_3xkk(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, kk, x, ..) = process_nibbles(nibbles);
        let vx = self.v[x];
        if vx == kk { ProcessorAction::SkipInstruction } 
        else { ProcessorAction::NextInstruction }
    }

    // The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
    pub fn op_4xkk(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, kk, x, ..) = process_nibbles(nibbles);
        let vx = self.v[x];
        skip_if(vx != kk)
    }

    // The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    pub fn op_5xy0(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, _, x, y, ..) = process_nibbles(nibbles);
        let vx = self.v[x];
        let vy = self.v[y];
        skip_if(vx == vy)
    }

    // The interpreter puts the value kk into register Vx.
    pub fn op_6xkk(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, kk, ..) = process_nibbles(nibbles);
        let x = nibbles.1 as usize;
        self.v[x] = kk;
        ProcessorAction::NextInstruction
    }

    // Adds the value kk to the value of register Vx, then stores the result in Vx.
    pub fn op_7xkk(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, kk, x, ..) = process_nibbles(nibbles);
        let vx = self.v[x];
        let result = vx.wrapping_add(kk);
        self.v[x] = result as u8;
        ProcessorAction::NextInstruction
    }

    // Stores the value of register Vy in register Vx.
    pub fn op_8xy0(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, _, x, y, _) = process_nibbles(nibbles);
        let vy = self.v[y];
        self.v[x] = vy;
        ProcessorAction::NextInstruction
    }

    // Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx.
    pub fn op_8xy1(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, _, x, y, _) = process_nibbles(nibbles);
        let vx = self.v[x];
        let vy = self.v[y];
        self.v[x] = vy | vx;
        ProcessorAction::NextInstruction
    }

    // Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx. 
    pub fn op_8xy2(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, _, x, y, _) = process_nibbles(nibbles);
        let vx = self.v[x];
        let vy = self.v[y];
        self.v[x] = vy & vx;
        ProcessorAction::NextInstruction
    }

    // Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx.
    pub fn op_8xy3(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, _, x, y, _) = process_nibbles(nibbles);
        let vx = self.v[x];
        let vy = self.v[y];
        self.v[x] = vy ^ vx;
        ProcessorAction::NextInstruction
    }

    // The values of Vx and Vy are added together. 
    // If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0. 
    // Only the lowest 8 bits of the result are kept, and stored in Vx.
    pub fn op_8xy4(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, _, x, y, _) = process_nibbles(nibbles);
        let vx = self.v[x] as u16;
        let vy = self.v[y] as u16;
        let result = vx + vy;
        let carry = result > 0x100;
        self.v[0xF] = if carry {1} else {0};
        self.v[x] = (result & 0xFF) as u8;
        ProcessorAction::NextInstruction
    }

    // If Vx > Vy, then VF is set to 1, otherwise 0.
    // Then Vy is subtracted from Vx, and the results stored in Vx.
    pub fn op_8xy5(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, _, x, y, _) = process_nibbles(nibbles);
        let vx = self.v[x];
        let vy = self.v[y];
        let vf = vx > vy;
        let result = vx.wrapping_sub(vy);
        self.v[0xF] = if vf {1} else {0};
        self.v[x] = result & 0xFF;
        ProcessorAction::NextInstruction
    }

    // If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. 
    // Then Vx is divided by 2.
    pub fn op_8xy6(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, _, x, _, _) = process_nibbles(nibbles);
        let vx = self.v[x];
        let vf = (vx & 0b1) == 0b1;
        let result = vx >> 2;
        self.v[0xF] = if vf {1} else {0};
        self.v[x] = result & 0xFF;
        ProcessorAction::NextInstruction
    }

    // If Vy > Vx, then VF is set to 1, otherwise 0. 
    // Then Vx is subtracted from Vy, and the results stored in Vx.
    pub fn op_8xy7(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, _, x, y, _) = process_nibbles(nibbles);
        let vx = self.v[x];
        let vy = self.v[y];
        let vf = vy > vx;
        let result = vy - vx;
        self.v[0xF] = if vf {1} else {0};
        self.v[x] = result & 0xFF;
        ProcessorAction::NextInstruction
    }

    // If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. 
    // Then Vx is multiplied by 2.
    pub fn op_8xyE(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, _, x, _, _) = process_nibbles(nibbles);
        let vx = self.v[x];
        let vf = (vx >> 7) & 1 == 1;
        let result = vx.wrapping_mul(2);
        self.v[0xF] = if vf {1} else {0};
        self.v[x] = result;
        ProcessorAction::NextInstruction
    }

    // Skip next instruction if Vx != Vy.
    pub fn op_9xy0(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, _, x, y, _) = process_nibbles(nibbles);
        let vx = self.v[x];
        let vy = self.v[y];
        skip_if(vx != vy)
    }

    // The value of register I is set to nnn.
    pub fn op_Annn(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (nnn, ..) = process_nibbles(nibbles);
        self.i = nnn;
        ProcessorAction::NextInstruction
    }

    // The program counter is set to nnn plus the value of V0.
    pub fn op_Bnnn(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (nnn, ..) = process_nibbles(nibbles);
        self.i = nnn + (self.v[0] as u16);
        ProcessorAction::NextInstruction
    }

    // The interpreter generates a random number from 0 to 255, 
    // which is then ANDed with the value kk. The results are stored in Vx.
    pub fn op_Cxkk(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, kk, x, ..) = process_nibbles(nibbles);
        let rnd = rand::thread_rng().gen_range(0..256) as u8;
        self.v[x] = kk & rnd;
        ProcessorAction::NextInstruction
    }

    // The interpreter reads n bytes from memory, starting at the address stored in I. 
    // These bytes are then displayed as sprites on screen at coordinates (Vx, Vy). 
    // Sprites are XORed onto the existing screen. If this causes any pixels to be erased, 
    // VF is set to 1, otherwise it is set to 0. If the sprite is positioned so part of it 
    // is outside the coordinates of the display, it wraps around to the opposite side of the screen. 
    pub fn op_Dxyn(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, _, x, y, n) = process_nibbles(nibbles);
        let vx = self.v[x] as usize;
        let vy = self.v[y] as usize;
        let I = self.i as usize;

        for byte in 0..n {
            let jj = (byte + vy) % HEIGHT;
            for bit in 0..8 {
                let ii = (bit + vx) % WIDTH;
                let color = (self.ram[I + byte] >> (7 - bit)) & 1;
                self.v[0x0f] |= color & self.cycle_buffer[ii + jj * WIDTH] as u8;
                self.cycle_buffer[ii + jj * WIDTH] ^= color == 1;
            }
        }
        self.cycle_buffer.clone_into(&mut self.pixels);
        ProcessorAction::NextInstruction
    }

    // Skip next instruction if key with the value of Vx is pressed.
    pub fn op_Ex9E(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        // TODO
        ProcessorAction::NextInstruction
    }

    // Skip next instruction if key with the value of Vx is not pressed.
    pub fn op_ExA1(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        // TODO
        ProcessorAction::NextInstruction
    }

    // Set Vx = delay timer value.
    pub fn op_Fx07(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, _, x, ..) = process_nibbles(nibbles);
        self.v[x] = self.delay_timer;
        ProcessorAction::NextInstruction
    }

    // Wait for a key press, store the value of the key in Vx.
    pub fn op_Fx0A(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        // TODO
        ProcessorAction::NextInstruction
    }

    // Set delay timer = Vx.
    pub fn op_Fx15(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, _, x, ..) = process_nibbles(nibbles);
        self.delay_timer = self.v[x];
        ProcessorAction::NextInstruction
    }

    // Set sound timer = Vx.
    pub fn op_Fx18(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, _, x, ..) = process_nibbles(nibbles);
        self.sound_timer = self.v[x];
        ProcessorAction::NextInstruction
    }

    // Set I = I + Vx.
    pub fn op_Fx1E(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, _, x, ..) = process_nibbles(nibbles);
        self.i += self.v[x] as u16;
        ProcessorAction::NextInstruction
    }

    // Set I = location of sprite for digit Vx.
    pub fn op_Fx29(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        // TODO
        ProcessorAction::NextInstruction
    }

    // Store BCD representation of Vx in memory locations I, I+1, and I+2.
    pub fn op_Fx33(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        // TODO
        ProcessorAction::NextInstruction
    }

    // Store registers V0 through Vx in memory starting at location I.
    pub fn op_Fx55(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, _, x, ..) = process_nibbles(nibbles);
        for i in 0..x {
            let vi = self.v[i];
            let index = self.i as usize + i;
            self.ram[index] = vi;
        }
        ProcessorAction::NextInstruction
    }

    // Read registers V0 through Vx from memory starting at location I.
    pub fn op_Fx65(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        let (_, _, x, ..) = process_nibbles(nibbles);
        for i in 0..x {
            let index = i + self.i as usize;
            let vi = self.ram[index];
            self.v[i] = vi;
        }
        ProcessorAction::NextInstruction
    }


    // =====================================================
    // extra instructions we don't need for most chip8 stuff
    // =====================================================

    pub fn op_00Cn(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        ProcessorAction::NextInstruction
    }
    pub fn op_00FB(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        ProcessorAction::NextInstruction
    }
    pub fn op_00FC(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        ProcessorAction::NextInstruction
    }
    pub fn op_00FD(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        ProcessorAction::NextInstruction
    }
    pub fn op_00FE(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        ProcessorAction::NextInstruction
    }
    pub fn op_00FF(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        ProcessorAction::NextInstruction
    }
    pub fn op_Dxy0(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        ProcessorAction::NextInstruction
    }
    pub fn op_Fx30(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        ProcessorAction::NextInstruction
    }
    pub fn op_Fx75(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        ProcessorAction::NextInstruction
    }
    pub fn op_Fx85(&mut self, nibbles: (u8, u8, u8, u8)) -> ProcessorAction {
        ProcessorAction::NextInstruction
    }
}

fn skip_if(v: bool) -> ProcessorAction {
    if v { ProcessorAction::SkipInstruction }
    else { ProcessorAction::NextInstruction }
}

// nnn, kk, x, y, n
fn process_nibbles(nibbles: (u8,u8,u8,u8)) -> (u16, u8, usize, usize, usize) {
    let x = nibbles.1 as usize;
    let y = nibbles.2 as usize;
    let n = nibbles.3 as usize;
    let kk = {
        let nib1 = nibbles.2 << 4;
        let nib2 = nibbles.3;
        nib1 | nib2
    };
    let nnn = {
        let nib1 = (nibbles.1 as u16) << 8;
        let nib2 = (nibbles.2 as u16) << 4;
        let nib3 = nibbles.3 as u16;
        nib1 | nib2 | nib3
    };
    return (nnn, kk, x, y, n)
}


