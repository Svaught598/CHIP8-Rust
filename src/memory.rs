


const RAM_SZ: usize = 4096;
const STACK_SZ: usize = 16;
const KEYS_SZ: usize = 16;
const V_SZ: usize = 16;

#[derive(Clone, Debug)]
pub struct Memory {
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

impl Memory {
    pub fn new() -> Self {
        Self {
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

    // pub fn reset(&mut self) {
    //     self.ram = [0; RAM_SZ];
    //     self.stack = [0; STACK_SZ];
    //     self.v = [0; V_SZ];
    //     self.i = 0;
    //     self.pc = 0;
    //     self.sp = 0;
    //     self.delay_timer = 0;
    //     self.sound_timer = 0;
    //     self.keypad = [false; KEYS_SZ];
    // }

    pub fn get_instruction(&self) -> u16 {
        let high_nibble = (self.ram[self.pc as usize] as u16) << 8;
        let low_nibble = self.ram[(self.pc + 1) as usize] as u16;
        return high_nibble | low_nibble;
    }
}