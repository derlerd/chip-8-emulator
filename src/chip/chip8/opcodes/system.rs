use core::convert::TryFrom;
use std::marker::PhantomData;

use crate::chip::chip8::{
    opcodes::{ExecutableOpcode, InstructionParsingError, InstructionWithAddress, Opcode},
    util, Chip8,
};

pub(crate) struct Sys;

pub(crate) type SysInstruction = InstructionWithAddress<Sys>;

implement_try_from_address!(SysInstruction, 0x0);

impl ExecutableOpcode for SysInstruction {
    fn execute(self, mut state: &mut Chip8) {
        match self.address {
            0x0E0 => {
                state.gfx = [false; 64 * 32];
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
