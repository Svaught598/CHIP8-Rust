use winit::event::VirtualKeyCode;

use crate::processor::Processor;
use std::{time::{ Duration, Instant }, fs::{self, File}, io::Read};

#[derive(Clone, Debug)]
pub struct CHIPMachine {
    cpu: Processor,
    // Should always be the same size as `cells`. When updating, we read from
    // `cells` and write to `scratch_cells`, then swap. Otherwise it's not in
    // use, and `cells` should be updated directly.
    pub cycle_duration: Duration,
    pub start_time: Instant,
    pub running: bool,
}

impl CHIPMachine {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width != 0 && height != 0);
        let size = width.checked_mul(height).expect("too big");
        Self {
            cpu: Processor::new(size),
            start_time: Instant::now(),
            cycle_duration: Duration::from_micros(200),
            running: false,
        }
    }

    pub fn cycle(&mut self) {
        let op = match self.cpu.get_instruction() {
            Some(op) => op,
            None => 0,
        };
        self.cpu.tick(op);
    }

    pub fn reset_start_time(&mut self) {
        self.start_time = Instant::now();
    }

    pub fn load_rom(&mut self, file_path: String) {
        let mut f = File::open(&file_path).expect("no file found");
        let metadata = fs::metadata(&file_path).expect("unable to read metadata");
        let mut buffer: Vec<u8> = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("buffer overflow");

        self.cpu.load(buffer);
        self.running = true;
    }

    pub fn draw(&self, screen: &mut [u8]) {
        debug_assert_eq!(screen.len(), 4 * self.cpu.pixels.len());
        for (c, pix) in self.cpu.pixels.iter().zip(screen.chunks_exact_mut(4)) {
            let color = if *c {
                //#2a9d8f
                [0x2a, 0x9d, 0x8f, 0xFF]
            } else {
                // #264653
                [0x26, 0x46, 0x53, 0xFF]
            };
            pix.copy_from_slice(&color);
        }
    }

    pub fn process_key(&self, _: VirtualKeyCode) {
    }

}