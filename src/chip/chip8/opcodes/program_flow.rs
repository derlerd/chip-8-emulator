use core::convert::TryFrom;
use std::marker::PhantomData;

use crate::chip::chip8::{
    opcodes::{
        Instruction, InstructionParsingError, InstructionWithAddress, InstructionWithOperands,
        InstructionWithRegAndValue, Opcode,
    },
    util, Chip8,
};

define_instruction_with_address!(Jmp, JmpInstruction, 0x1);
impl Instruction for JmpInstruction {
    /// Opcode of the form `0x1XYZ` (JMP). Sets `state.program_counter` to `XYZ`.
    fn execute(&self, state: &mut Chip8) {
        state.program_counter = self.address;
    }
}

define_instruction_with_address!(Call, CallInstruction, 0x2);
impl Instruction for CallInstruction {
    /// Opcode of the form `0x2XYZ` (CALL). Calls the routine at `XYZ`.
    fn execute(&self, state: &mut Chip8) {
        assert!(state.stack_pointer < 16, "Stack overflow");
        state.stack[state.stack_pointer as usize] = state.program_counter;
        state.stack_pointer = state.stack_pointer + 1;
        state.program_counter = self.address;
    }
}

define_instruction_with_reg_and_value!(Se, SeInstruction, 0x3);
impl Instruction for SeInstruction {
    /// Opcode of the form `0x3XYZ` (SE). Skip the next instruction if `state.registers[X] == YZ`.
    fn execute(&self, mut state: &mut Chip8) {
        util::conditional_skip(&self, &mut state, |instruction, state| {
            state.registers[instruction.reg as usize] == instruction.value
        });
        util::increment_program_counter(&mut state);
    }
}

define_instruction_with_reg_and_value!(Sne, SneInstruction, 0x4);
impl Instruction for SneInstruction {
    /// Opcode of the form `0x4XYZ` (SNE). Skip the next instruction if `state.registers[X] != YZ`.
    fn execute(&self, mut state: &mut Chip8) {
        util::conditional_skip(&self, &mut state, |instruction, state| {
            state.registers[instruction.reg as usize] != instruction.value
        });
        util::increment_program_counter(&mut state);
    }
}

define_instruction_with_operands!(Sre, SreInstruction, 0x5);
impl Instruction for SreInstruction {
    /// Opcode of the form `0x5XY0` (SRE). Skip the next instruction if `state.registers[X] == state.registers[y]`.
    fn execute(&self, mut state: &mut Chip8) {
        util::conditional_skip(&self, &mut state, |instruction, state| {
            assert_eq!(instruction.op3, 0, "Unsupported opcode");
            state.registers[instruction.op1 as usize] == state.registers[instruction.op2 as usize]
        });
        util::increment_program_counter(&mut state);
    }
}

define_instruction_with_operands!(Srne, SrneInstruction, 0x9);
impl Instruction for SrneInstruction {
    /// Opcode of the form `0x9XY0` (SRNE). Skip the next instruction if `state.registers[X] != state.registers[Y]`.
    fn execute(&self, mut state: &mut Chip8) {
        util::conditional_skip(&self, &mut state, |instruction, state| {
            assert_eq!(instruction.op3, 0, "Unsupported opcode");
            state.registers[instruction.op1 as usize] != state.registers[instruction.op2 as usize]
        });
        util::increment_program_counter(&mut state);
    }
}

define_instruction_with_address!(Jmpr, JmprInstruction, 0xB);
impl Instruction for JmprInstruction {
    /// Opcode of the form `0xBXYZ` (JMPR). Sets `state.program_counter` to `XYZ + state.registers[0]`
    /// (where the addition wraps around if an overflow occurs).
    fn execute(&self, mut state: &mut Chip8) {
        state.program_counter = self.address.wrapping_add(state.registers[0] as u16);
    }
}

define_instruction_with_reg_and_value!(Sk, SkInstruction, 0xE);
impl Instruction for SkInstruction {
    /// Opcode of the form `0xEXYZ` (SK). Groups skip operation related to keys.
    ///
    /// - If `YZ == 9E`, it skips the next instruction if the key stored in `state.registers[X]`
    ///   is pressed.
    ///
    /// - If `YZ == A1`, it skips the next instruction if the key stored in `state.registers[X]`
    ///   is not pressed.
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
