use core::convert::TryFrom;
use std::marker::PhantomData;

use crate::chip::chip8::{
    opcodes::{ExecutableOpcode, InstructionParsingError, InstructionWithAddress, Opcode},
    util, Chip8,
};

define_instruction_with_address!(Sys, SysInstruction, 0x0);
impl ExecutableOpcode for SysInstruction {
    /// Opcode of the form `0x0XYZ` (SYS). Groups various system instructions.
    ///
    /// - If `XYZ == 0x0E0`, it clears the display.
    ///
    /// - If `XYZ == 0x0EE`, it returns from the current subroutine.
    ///
    fn execute(&self, mut state: &mut Chip8) {
        match self.address {
            0x0E0 => {
                state.output_pins = [false; 64 * 32];
                state.program_counter += 2;
            }
            0x0EE => {
                assert!(state.stack_pointer > 0, "Stack underflow");
                state.program_counter = state.stack[(state.stack_pointer - 1) as usize];
                state.stack_pointer = state.stack_pointer - 1;
                util::increment_program_counter(&mut state);
            }
            _ => panic!("Opcode not supported"),
        };
    }
}
