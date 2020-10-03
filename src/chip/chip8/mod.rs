/// CHIP-8 constants.
mod constants;
/// Cursive display output.
pub mod cursive_display;
/// Decoding of opcodes and their execution.
mod opcodes;
/// Convenience functions for modification of the CHIP-8 state.
mod util;

#[cfg(test)]
mod tests;

use std::fs;
use std::fs::File;
use std::io::Read;

use crate::chip::{
    chip8::constants::{
        CHIP8_CHARSET, CHIP8_CHARSET_LEN, CHIP8_CHARSET_OFFSET, CHIP8_MAX_PROGRAM_SIZE,
        CHIP8_TIMER_RESOLUTION,
    },
    chip8::opcodes::Opcode,
    Chip, LoadProgramError,
};

/// Represents the state of the CHIP-8.
pub struct Chip8 {
    /// 4096 bytes of main memory
    memory: [u8; 4096],

    /// 16 registers where each can store one byte
    registers: [u8; 16],

    /// An index register
    index: u16,

    /// A program counter
    program_counter: u16,

    /// The output pins. Note that those are usually directly wired
    /// up to the pixels of the display. However, given that this implementation
    /// considers a display as optional, we refer to them as output_pins for
    /// the sake of generality.
    output_pins: [bool; 64 * 32],

    /// The delay timer. Note that this timer is decremented every
    /// `CHIP8_TIMER_RESOLUTION` cycles.
    delay_timer: u8,

    /// The sound timer. Note that this timer is decremented every
    /// `CHIP8_TIMER_RESOLUTION` cycles.
    sound_timer: u8,

    /// The input pins. Note that those input pins are usually directly wired
    /// up to the keys. However, we do not prescribe how this is handled and
    /// hence refer to them as input pins rather than as keys.
    input_pins: [bool; 16],

    /// A stack. Note that there are no instructions allowing to modify the
    /// stack and it is only used to store return addresses for the return
    /// opcode.
    stack: [u16; 16],

    /// A pointer, pointing to the current position in the stack.
    stack_pointer: u8,

    /// A helper variable to properly implement the timer resolution.
    cycles_since_timer_dec: u8,

    /// A flag that indicates whether the output pins changed since it
    /// was last set to false.
    draw: bool,
}

impl Chip for Chip8 {
    /// The CHIP-8's pins can actually be addressed by using just half a byte.
    /// However, we use a whole byte here and assert whether it is in the right
    /// range, because it is more convenient to handle.
    type PinAddress = u8;

    /// A CHIP-8 memory address is in the range between 0 and 4096 (exclusive). We
    /// represent it using a u16. This means we have to assert that it is
    /// in the right range whenever we set it.
    type MemoryAddress = u16;

    fn load_program(&mut self, path: &str) -> Result<usize, LoadProgramError> {
        let mut file =
            File::open(path).map_err(|_| LoadProgramError::CouldNotOpenFile(path.to_string()))?;
        let md = fs::metadata(path)
            .map_err(|_| LoadProgramError::CouldNotReadMetadata(path.to_string()))?;
        let mut buffer = vec![0; md.len() as usize];
        file.read(&mut buffer)
            .map_err(|_| LoadProgramError::CouldNotReadFile(path.to_string()))?;

        if buffer.len() > (CHIP8_MAX_PROGRAM_SIZE as usize) {
            return Err(LoadProgramError::ProgramTooLarge(buffer.len()));
        }

        self.load_program_bytes(&buffer);

        Ok(md.len() as usize)
    }

    fn cycle(&mut self) {
        let opcode = self.next_instruction();
        let mut state = self;
        opcode.execute(&mut state);

        state.cycles_since_timer_dec += 1;

        if state.cycles_since_timer_dec % CHIP8_TIMER_RESOLUTION == 0 {
            if state.delay_timer > 0 {
                state.delay_timer -= 1;
            }

            if state.sound_timer > 0 {
                // TODO let it beep
                state.sound_timer -= 1;
            }

            state.cycles_since_timer_dec = 0;
        }
    }

    fn read_output_pins(&self) -> &[bool] {
        &self.output_pins
    }

    fn set_input_pin(&mut self, pin: u8, value: bool) {
        assert!(pin & 0x0F == pin);
        self.input_pins[pin as usize] = value;
    }

    fn reset_input_pins(&mut self) {
        for i in 0..16 {
            self.input_pins[i] = false;
        }
    }
}

impl Chip8 {
    /// Constructs a new CHIP-8 and appropriately initializes all fields so that
    /// it is ready for the first execution cycle. Essentially this means that
    /// the program counter is set to 0x200 and the default CHIP-8 charset is
    /// loaded at memory address `CHIP8_CHARSET_OFFSET`. Note that no program is
    /// loaded upon initialization.
    pub fn new() -> Self {
        let mut memory = [0; 4096];
        for i in 0..CHIP8_CHARSET_LEN {
            memory[(i + CHIP8_CHARSET_OFFSET) as usize] = CHIP8_CHARSET[i as usize];
        }

        Chip8 {
            memory,
            registers: [0; 16],
            index: 0,
            program_counter: 0x200,
            output_pins: [false; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            stack_pointer: 0,
            input_pins: [false; 16],
            draw: false,
            cycles_since_timer_dec: 0,
        }
    }

    /// Fetches the next instruction based on the current state of self.program_counter.
    ///
    /// # Panics
    /// In case `self.program_counter` points to an address which would lead to loading
    /// bytes from invalid memory addresses.
    fn next_instruction(&self) -> Opcode {
        assert!(self.program_counter <= 4094);
        Opcode::new(&[
            self.memory[self.program_counter as usize],
            self.memory[(self.program_counter + 1) as usize],
        ])
    }

    /// Convenience method to load a program from a slice.
    ///
    /// # Panics
    /// In case `program` is too long, i.e., so that loading it would overflow the memory buffer.
    pub fn load_program_bytes(&mut self, program: &[u8]) {
        for i in 0..program.len() {
            self.set_memory_byte(program[i], (0x200 + i) as u16);
        }
    }

    /// Sets a memory byte
    ///
    /// # Panics
    /// In case the supplied memory address would result in a buffer overflow.
    fn set_memory_byte(&mut self, byte: u8, index: u16) {
        assert!(index < 4096);
        self.memory[index as usize] = byte;
    }
}

impl Default for Chip8 {
    fn default() -> Self {
        Chip8::new()
    }
}
