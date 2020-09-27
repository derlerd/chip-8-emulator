use core::convert::TryFrom;
use std::marker::PhantomData;

use crate::chip::chip8::{
    opcodes::{
        ExecutableOpcode, InstructionParsingError, InstructionWithAddress, InstructionWithOperands,
        InstructionWithRegAndValue, Opcode,
    },
    util, Chip8,
};

pub(crate) struct Jmp;

pub(crate) type JmpInstruction = InstructionWithAddress<Jmp>;

implement_try_from_address!(JmpInstruction, 0x1);

impl ExecutableOpcode for JmpInstruction {
    fn execute(self, state: &mut Chip8) {
        state.program_counter = self.address;
    }
}

pub(crate) struct Call;

pub(crate) type CallInstruction = InstructionWithAddress<Call>;

implement_try_from_address!(CallInstruction, 0x2);

impl ExecutableOpcode for CallInstruction {
    fn execute(self, state: &mut Chip8) {
        assert!(state.stack_pointer < 16, "Stack overflow");
        state.stack[state.stack_pointer as usize] = state.program_counter;
        state.stack_pointer = state.stack_pointer + 1;
        state.program_counter = self.address;
    }
}

pub(crate) struct Se;

pub(crate) type SeInstruction = InstructionWithRegAndValue<Se>;

implement_try_from_reg_and_value!(SeInstruction, 0x3);

impl ExecutableOpcode for SeInstruction {
    fn execute(self, mut state: &mut Chip8) {
        util::conditional_skip(&self, &mut state, |instruction, state| {
            state.registers[instruction.reg as usize] == instruction.value
        });
        util::increment_program_counter(&mut state);
    }
}

pub(crate) struct Sne;

pub(crate) type SneInstruction = InstructionWithRegAndValue<Sne>;

implement_try_from_reg_and_value!(SneInstruction, 0x4);

impl ExecutableOpcode for SneInstruction {
    fn execute(self, mut state: &mut Chip8) {
        util::conditional_skip(&self, &mut state, |instruction, state| {
            state.registers[instruction.reg as usize] != instruction.value
        });
        util::increment_program_counter(&mut state);
    }
}

pub(crate) struct Sre;

pub(crate) type SreInstruction = InstructionWithOperands<Sre>;

implement_try_from_operands!(SreInstruction, 0x5);

impl ExecutableOpcode for SreInstruction {
    fn execute(self, mut state: &mut Chip8) {
        util::conditional_skip(&self, &mut state, |instruction, state| {
            assert_eq!(instruction.op3, 0, "Unsupported opcode");
            state.registers[instruction.op1 as usize] == state.registers[instruction.op2 as usize]
        });
        util::increment_program_counter(&mut state);
    }
}

pub(crate) struct Srne;

pub(crate) type SrneInstruction = InstructionWithOperands<Srne>;

implement_try_from_operands!(SrneInstruction, 0x9);

impl ExecutableOpcode for SrneInstruction {
    fn execute(self, mut state: &mut Chip8) {
        util::conditional_skip(&self, &mut state, |instruction, state| {
            assert_eq!(instruction.op3, 0, "Unsupported opcode");
            state.registers[instruction.op1 as usize] != state.registers[instruction.op2 as usize]
        });
        util::increment_program_counter(&mut state);
    }
}

pub(crate) struct Jmpr;

pub(crate) type JmprInstruction = InstructionWithAddress<Jmpr>;

implement_try_from_address!(JmprInstruction, 0xB);

impl ExecutableOpcode for JmprInstruction {
    fn execute(self, mut state: &mut Chip8) {
        state.program_counter = self.address.wrapping_add(state.registers[0] as u16);
    }
}

pub(crate) struct Sk;

pub(crate) type SkInstruction = InstructionWithRegAndValue<Sk>;

implement_try_from_reg_and_value!(SkInstruction, 0xE);

impl ExecutableOpcode for SkInstruction {
    fn execute(self, mut state: &mut Chip8) {
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
