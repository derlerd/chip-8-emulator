use core::convert::TryFrom;
use std::marker::PhantomData;

use crate::chip::chip8::{
    opcodes::{
        ExecutableOpcode, InstructionParsingError, InstructionWithAddress, InstructionWithOperands,
        InstructionWithRegAndValue, Opcode,
    },
    util, Chip8,
};

define_instruction_with_address!(Jmp, JmpInstruction, 0x1);
impl ExecutableOpcode for JmpInstruction {
    fn execute(&self, state: &mut Chip8) {
        state.program_counter = self.address;
    }
}

define_instruction_with_address!(Call, CallInstruction, 0x2);
impl ExecutableOpcode for CallInstruction {
    fn execute(&self, state: &mut Chip8) {
        assert!(state.stack_pointer < 16, "Stack overflow");
        state.stack[state.stack_pointer as usize] = state.program_counter;
        state.stack_pointer = state.stack_pointer + 1;
        state.program_counter = self.address;
    }
}

define_instruction_with_reg_and_value!(Se, SeInstruction, 0x3);
impl ExecutableOpcode for SeInstruction {
    fn execute(&self, mut state: &mut Chip8) {
        util::conditional_skip(&self, &mut state, |instruction, state| {
            state.registers[instruction.reg as usize] == instruction.value
        });
        util::increment_program_counter(&mut state);
    }
}

define_instruction_with_reg_and_value!(Sne, SneInstruction, 0x4);
impl ExecutableOpcode for SneInstruction {
    fn execute(&self, mut state: &mut Chip8) {
        util::conditional_skip(&self, &mut state, |instruction, state| {
            state.registers[instruction.reg as usize] != instruction.value
        });
        util::increment_program_counter(&mut state);
    }
}

define_instruction_with_operands!(Sre, SreInstruction, 0x5);
impl ExecutableOpcode for SreInstruction {
    fn execute(&self, mut state: &mut Chip8) {
        util::conditional_skip(&self, &mut state, |instruction, state| {
            assert_eq!(instruction.op3, 0, "Unsupported opcode");
            state.registers[instruction.op1 as usize] == state.registers[instruction.op2 as usize]
        });
        util::increment_program_counter(&mut state);
    }
}

define_instruction_with_operands!(Srne, SrneInstruction, 0x9);
impl ExecutableOpcode for SrneInstruction {
    fn execute(&self, mut state: &mut Chip8) {
        util::conditional_skip(&self, &mut state, |instruction, state| {
            assert_eq!(instruction.op3, 0, "Unsupported opcode");
            state.registers[instruction.op1 as usize] != state.registers[instruction.op2 as usize]
        });
        util::increment_program_counter(&mut state);
    }
}

define_instruction_with_address!(Jmpr, JmprInstruction, 0xB);
impl ExecutableOpcode for JmprInstruction {
    fn execute(&self, mut state: &mut Chip8) {
        state.program_counter = self.address.wrapping_add(state.registers[0] as u16);
    }
}

define_instruction_with_reg_and_value!(Sk, SkInstruction, 0xE);
impl ExecutableOpcode for SkInstruction {
    fn execute(&self, mut state: &mut Chip8) {
        let skip = match self.value {
            0x9E => state.input_pins[state.registers[self.reg as usize] as usize],
            0xA1 => !state.input_pins[state.registers[self.reg as usize] as usize],
            _ => unimplemented!("Unsupported opcode"),
        };
        if skip {
            util::increment_program_counter(&mut state);
        }
        util::increment_program_counter(&mut state);
    }
}
