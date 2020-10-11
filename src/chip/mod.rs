/// CHIP-8 implementation
pub mod chip8;

use cursive::CbSink;

/// Error type for errors that occur during loading the program
#[derive(Debug)]
pub enum LoadProgramError {
    ProgramTooLarge(usize),
}

/// Represents a chip that supports display output via by sending
/// instructions to callback sink of the cursive terminal UI
/// framework.
pub trait ChipWithCursiveDisplay {
    /// Gives the implementation the possibility to send instructions to
    /// update the UI to `gfx_sink`.
    fn update_ui(&mut self, gfx_sink: &CbSink);
}

/// Represents a chip.
pub trait Chip {
    /// The type used to address input pins
    type PinAddress;
    /// The type used for memory addresses
    type MemoryAddress;

    /// Mutates self in that it loads the program from the given slice and
    /// stores it into the chip's memory.
    fn load_program(&mut self, bytes: &[u8]) -> Result<(), LoadProgramError>;

    /// Preforms an execution cycle. It mutates self so that its state
    /// corresponds to the state after the execution cycle.
    fn cycle(&mut self);

    /// Returns a slice representing the current state of the output
    /// pins.
    fn read_output_pins(&self) -> &[bool];

    /// Mutates self so that the input pin referenced by `pin` is set
    /// to `value` after calling this method.
    fn set_input_pin(&mut self, pin: Self::PinAddress, value: bool);

    /// Mutates self so that all input pins are reset after calling this
    /// method.
    fn reset_input_pins(&mut self);
}

impl std::fmt::Display for LoadProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LoadProgramError::ProgramTooLarge(size) => write!(
                f,
                "Program is too large. Maximum program size is 3584 bytes. Got {} bytes.",
                size
            ),
        }
    }
}
