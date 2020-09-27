/// Chip8 impementation
pub mod chip8;

use cursive::{view::View, CbSink};

/// Error type for errors that occur during loading the program
#[derive(Debug)]
pub enum LoadProgramError {
    CouldNotOpenFile(String),
    CouldNotReadMetadata(String),
    CouldNotReadFile(String),
    ProgramTooLarge(usize),
}

pub trait ChipWithCursiveDisplay {
    type Display: View;

    fn update_ui(&mut self, gfx_sink: &CbSink);
}

pub trait Chip {
    type PinAddress;
    type MemoryAddress;

    fn load_program(&mut self, path: &str) -> Result<usize, LoadProgramError>;
    fn cycle(&mut self);
    fn read_output_pins(&self) -> &[bool];
    fn set_input_pin(&mut self, pin: Self::PinAddress, value: bool);
    fn reset_input_pins(&mut self);
}

impl std::fmt::Display for LoadProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LoadProgramError::CouldNotOpenFile(message) => {
                write!(f, "Could not open file: {:?}", message)
            }
            LoadProgramError::CouldNotReadMetadata(message) => {
                write!(f, "Could not read metadata: {:?}", message)
            }
            LoadProgramError::CouldNotReadFile(message) => {
                write!(f, "Could not read file: {:?}", message)
            }
            LoadProgramError::ProgramTooLarge(size) => write!(
                f,
                "Program is too large. Maximum program size is 3584 bytes. Got {} bytes.",
                size
            ),
        }
    }
}
