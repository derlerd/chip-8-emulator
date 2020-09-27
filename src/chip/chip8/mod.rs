mod constants;
pub mod cursive_display;
mod opcodes;
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

#[derive(Clone)]
pub struct Chip8 {
    opcode: u16,
    memory: [u8; 4096],
    registers: [u8; 16],
    index: u16,
    program_counter: u16,
    gfx: [bool; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    stack_pointer: u16,
    key: [bool; 16],
    draw: bool,
    cycles_since_timer_dec: u8,
}

impl Chip for Chip8 {
    type PinAddress = u8;
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
        &self.gfx
    }

    fn set_input_pin(&mut self, pin: u8, value: bool) {
        assert!(pin & 0x0F == pin);
        self.key[pin as usize] = value;
    }

    fn reset_input_pins(&mut self) {
        for i in 0..16 {
            self.key[i] = false;
        }
    }
}

impl Chip8 {
    pub fn new() -> Self {
        let mut memory = [0; 4096];
        for i in 0..CHIP8_CHARSET_LEN {
            memory[(i + CHIP8_CHARSET_OFFSET) as usize] = CHIP8_CHARSET[i as usize];
        }

        Chip8 {
            opcode: 0,
            memory,
            registers: [0; 16],
            index: 0,
            program_counter: 0x200,
            gfx: [false; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            stack_pointer: 0,
            key: [false; 16],
            draw: false,
            cycles_since_timer_dec: 0,
        }
    }

    fn next_instruction(&self) -> Opcode {
        assert!(self.program_counter <= 4094);
        Opcode::new(&[
            self.memory[self.program_counter as usize],
            self.memory[(self.program_counter + 1) as usize],
        ])
    }

    pub fn load_program_bytes(&mut self, program: &[u8]) {
        for i in 0..program.len() {
            self.set_memory_byte(program[i], (0x200 + i) as u16);
        }
    }

    fn set_memory_byte(&mut self, byte: u8, index: u16) {
        assert!(index < 4096);
        self.memory[index as usize] = byte;
    }
}
