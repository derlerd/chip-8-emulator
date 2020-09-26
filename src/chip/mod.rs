pub mod chip8;

use cursive::{view::View, CbSink};

#[derive(Debug)]
pub enum LoadProgramError {
    CouldNotOpenFile(String),
    CouldNotReadMetadata(String),
    CouldNotReadFile(String),
}

pub trait ChipWithDisplayOutput {
    type Display: View;

    fn get_display(&self) -> Self::Display;
    fn update_ui(&mut self, gfx_sink: &CbSink);
}

pub trait Chip {
    type PinAddress;
    type MemoryAddress;

    fn load_program(&mut self, path: &str) -> Result<usize, LoadProgramError>;
    fn load_program_bytes(&mut self, program: &[u8]);
    fn cycle(&mut self);
    fn read_output_pins(&self) -> [bool; 64 * 32];
    fn set_input_pin(&mut self, pin: Self::PinAddress, value: bool);
    fn reset_input_pins(&mut self);
}
