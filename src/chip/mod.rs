pub mod chip8;

#[derive(Debug)]
pub enum LoadProgramError {
    CouldNotOpenFile(String),
    CouldNotReadMetadata(String),
    CouldNotReadFile(String),
} 

pub trait Chip {
    type PinAddress;
    type MemoryAddress;

    fn load_program(&mut self, path : &str) -> Result<usize, LoadProgramError>;
    fn load_program_bytes(&mut self, program: &[u8]);
    fn cycle(self) -> Self;
    fn get_gfx(&self) -> [bool; 64 * 32];
    fn set_io_pin(&mut self, pin: Self::PinAddress, value: bool);
    fn reset_io_pins(&mut self);
    fn set_memory_byte(&mut self, byte: u8, index: Self::MemoryAddress);
}
