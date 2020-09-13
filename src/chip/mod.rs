pub mod chip8;

pub trait Chip {
    fn cycle(self) -> Self;
    fn get_gfx(&self) -> [u8; 64 * 32];
    fn set_memory_byte(&mut self, byte: u8, index: usize);
}
