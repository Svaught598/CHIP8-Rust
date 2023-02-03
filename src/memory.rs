


const RAM_SZ: usize = 4096;
const STACK_SZ: usize = 16;
const KEYS_SZ: usize = 16;
const V_SZ: usize = 16;

#[derive(Clone, Debug)]
pub struct Memory {
    ram: [u8; RAM_SZ],
    stack: [u8; STACK_SZ],
    v: [u8; V_SZ],
    i: u16,
    pc: u16,
    sp: u16,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [bool; KEYS_SZ],
}

impl Memory {
    pub fn make_memory() -> Self {
        Self {
            ram: [0; RAM_SZ],
            stack: [0; STACK_SZ],
            v: [0; V_SZ],
            i: 0,
            pc: 0,
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false; KEYS_SZ],
        }
    }

    pub fn reset(&mut self) {
        self.ram = [0; RAM_SZ];
        self.stack = [0; STACK_SZ];
        self.v = [0; V_SZ];
        self.i = 0;
        self.pc = 0;
        self.sp = 0;
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.keypad = [false; KEYS_SZ];
    }
}