use crate::processor::Processor;

#[derive(Clone, Debug)]
pub struct CHIPMachine {
    cpu: Processor,
    cells: Vec<bool>,
    width: usize,
    height: usize,
    // Should always be the same size as `cells`. When updating, we read from
    // `cells` and write to `scratch_cells`, then swap. Otherwise it's not in
    // use, and `cells` should be updated directly.
    scratch_cells: Vec<bool>,
}

impl CHIPMachine {
    pub fn new_chip8(width: usize, height: usize) -> Self {
        assert!(width != 0 && height != 0);
        let size = width.checked_mul(height).expect("too big");
        Self {
            cpu: Processor::make_cpu(),
            cells: vec![false; size],
            scratch_cells: vec![false; size],
            width,
            height,
        }
    }

    pub fn set_few(&mut self) {
        for i in 0..self.cells.len() {
            self.cells[i] = i & 1 != 0;
        }
    }

    pub fn draw(&self, screen: &mut [u8]) {
        debug_assert_eq!(screen.len(), 4 * self.cells.len());
        for (c, pix) in self.cells.iter().zip(screen.chunks_exact_mut(4)) {
            let color = if *c {
                [0x00, 0x00, 0x00, 0xff]
            } else {
                [0, 0xff, 0xff, 0xff]
            };
            pix.copy_from_slice(&color);
        }
    }
}