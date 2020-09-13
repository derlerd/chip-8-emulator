mod chip;

use crate::chip::{Chip, chip8::Chip8};

fn main() {
    let mut chip8 = Chip8::new();

    chip8.set_memory_byte(0xC0, 0x200);
    chip8.set_memory_byte(0xF0, 0x201);

    //loop {
    chip8 = chip8.cycle();
    let _gfx = chip8.get_gfx();
    //}
}
