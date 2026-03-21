use crate::chip8::{
    Chip8,
    opcodes::{Address, Executable, HasOpcode, Instruction, Operands, RegAndValue},
    util,
};

define_instruction!(JmpInstruction, Address, 0x1);
impl Executable<Chip8> for JmpInstruction {
    /// Opcode of the form `0x1XYZ` (JMP). Sets `state.program_counter` to `XYZ`.
    fn execute(&self, state: &mut Chip8) {
        state.program_counter = self.address();
    }
}

define_instruction!(CallInstruction, Address, 0x2);
impl Executable<Chip8> for CallInstruction {
    /// Opcode of the form `0x2XYZ` (CALL). Calls the routine at `XYZ`.
    fn execute(&self, state: &mut Chip8) {
        assert!(state.stack_pointer < 16, "Stack overflow");
        state.stack[state.stack_pointer as usize] = state.program_counter;
        state.stack_pointer = state.stack_pointer + 1;
        state.program_counter = self.address();
    }
}

define_instruction!(SeInstruction, RegAndValue, 0x3);
impl Executable<Chip8> for SeInstruction {
    /// Opcode of the form `0x3XYZ` (SE). Skip the next instruction if `state.registers[X] == YZ`.
    fn execute(&self, mut state: &mut Chip8) {
        util::conditional_skip(&self, &mut state, |instruction, state| {
            state.registers[instruction.reg() as usize] == instruction.value()
        });
        util::increment_program_counter(&mut state);
    }
}

define_instruction!(SneInstruction, RegAndValue, 0x4);
impl Executable<Chip8> for SneInstruction {
    /// Opcode of the form `0x4XYZ` (SNE). Skip the next instruction if `state.registers[X] != YZ`.
    fn execute(&self, mut state: &mut Chip8) {
        util::conditional_skip(&self, &mut state, |instruction, state| {
            state.registers[instruction.reg() as usize] != instruction.value()
        });
        util::increment_program_counter(&mut state);
    }
}

define_instruction!(SreInstruction, Operands, 0x5);
impl Executable<Chip8> for SreInstruction {
    /// Opcode of the form `0x5XY0` (SRE). Skip the next instruction if `state.registers[X] == state.registers[y]`.
    fn execute(&self, mut state: &mut Chip8) {
        util::conditional_skip(&self, &mut state, |instruction, state| {
            assert_eq!(instruction.op3(), 0, "Unsupported opcode");
            state.registers[instruction.op1() as usize]
                == state.registers[instruction.op2() as usize]
        });
        util::increment_program_counter(&mut state);
    }
}

define_instruction!(SrneInstruction, Operands, 0x9);
impl Executable<Chip8> for SrneInstruction {
    /// Opcode of the form `0x9XY0` (SRNE). Skip the next instruction if `state.registers[X] != state.registers[Y]`.
    fn execute(&self, mut state: &mut Chip8) {
        util::conditional_skip(&self, &mut state, |instruction, state| {
            assert_eq!(instruction.op3(), 0, "Unsupported opcode");
            state.registers[instruction.op1() as usize]
                != state.registers[instruction.op2() as usize]
        });
        util::increment_program_counter(&mut state);
    }
}

define_instruction!(JmprInstruction, Address, 0xB);
impl Executable<Chip8> for JmprInstruction {
    /// Opcode of the form `0xBXYZ` (JMPR). Sets `state.program_counter` to `XYZ + state.registers[0]`
    /// (where the addition wraps around if an overflow occurs).
    fn execute(&self, state: &mut Chip8) {
        state.program_counter = self.address().wrapping_add(state.registers[0] as u16);
    }
}

define_instruction!(SkInstruction, RegAndValue, 0xE);
impl Executable<Chip8> for SkInstruction {
    /// Opcode of the form `0xEXYZ` (SK). Groups skip operation related to keys.
    ///
    /// - If `YZ == 9E`, it skips the next instruction if the key stored in `state.registers[X]`
    ///   is pressed.
    ///
    /// - If `YZ == A1`, it skips the next instruction if the key stored in `state.registers[X]`
    ///   is not pressed.
    fn execute(&self, mut state: &mut Chip8) {
        let skip = match self.value() {
            0x9E => state.input_pins[state.registers[self.reg() as usize] as usize],
            0xA1 => !state.input_pins[state.registers[self.reg() as usize] as usize],
            _ => unimplemented!("Unsupported opcode"),
        };
        if skip {
            util::increment_program_counter(&mut state);
        }
        util::increment_program_counter(&mut state);
    }
}
