pub mod chip8;

pub trait Chip {
    type PinAddress;
    fn cycle(self) -> Self;
    fn get_gfx(&self) -> [bool; 64 * 32];
    fn set_io_pin(&mut self, pin: Self::PinAddress, value: bool);
    fn reset_io_pins(&mut self);
    fn set_memory_byte(&mut self, byte: u8, index: usize);
}
