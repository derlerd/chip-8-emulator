use core::convert::TryFrom;
use rand::{thread_rng, Rng};
use std::marker::PhantomData;

use crate::chip::chip8::{
    constants::CHIP8_CHARSET_OFFSET,
    opcodes::{
        ExecutableOpcode, InstructionParsingError, InstructionWithAddress, InstructionWithOperands,
        InstructionWithRegAndValue, Opcode,
    },
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

pub(crate) struct Ldr;

pub(crate) type LdrInstruction = InstructionWithRegAndValue<Ldr>;

implement_try_from_reg_and_value!(LdrInstruction, 0x6);

impl ExecutableOpcode for LdrInstruction {
    fn execute(self, mut state: &mut Chip8) {
        state.registers[self.reg as usize] = self.value;
        util::increment_program_counter(&mut state);
    }
}

pub(crate) struct Add;

pub(crate) type AddInstruction = InstructionWithRegAndValue<Add>;

implement_try_from_reg_and_value!(AddInstruction, 0x7);

impl ExecutableOpcode for AddInstruction {
    fn execute(self, mut state: &mut Chip8) {
        state.registers[self.reg as usize] =
            state.registers[self.reg as usize].wrapping_add(self.value);
        util::increment_program_counter(&mut state);
    }
}

pub(crate) struct Reg;

pub(crate) type RegInstruction = InstructionWithOperands<Reg>;

implement_try_from_operands!(RegInstruction, 0x8);

impl ExecutableOpcode for RegInstruction {
    fn execute(self, mut state: &mut Chip8) {
        fn modify_registers(
            state: &mut Chip8,
            r1: u8,
            r2: u8,
            f: fn(u8, u8) -> (u8, Option<bool>),
        ) {
            let (val, carry) = f(state.registers[r1 as usize], state.registers[r2 as usize]);
            state.registers[r1 as usize] = val;
            match carry {
                Some(true) => state.registers[0xF] = 1,
                Some(false) => state.registers[0xF] = 0,
                _ => {}
            }
        }

        match self.op3 {
            0x0 => modify_registers(&mut state, self.op1, self.op2, |_, v2| (v2, None)),
            0x1 => modify_registers(&mut state, self.op1, self.op2, |v1, v2| (v1 | v2, None)),
            0x2 => modify_registers(&mut state, self.op1, self.op2, |v1, v2| (v1 & v2, None)),
            0x3 => modify_registers(&mut state, self.op1, self.op2, |v1, v2| (v1 ^ v2, None)),
            0x4 => modify_registers(&mut state, self.op1, self.op2, |v1, v2| {
                let (result, overflow) = v1.overflowing_add(v2);
                (result, Some(overflow))
            }),
            0x5 => modify_registers(&mut state, self.op1, self.op2, |v1, v2| {
                let (result, overflow) = v1.overflowing_sub(v2);
                (result, Some(!overflow))
            }),
            0x6 => modify_registers(&mut state, self.op1, self.op2, |v1, _| {
                (v1 >> 1, Some(v1 & 1 != 0))
            }),
            0x7 => modify_registers(&mut state, self.op1, self.op2, |v1, v2| {
                let (result, overflow) = v2.overflowing_sub(v1);
                (result, Some(!overflow))
            }),
            0xE => modify_registers(&mut state, self.op1, self.op2, |v1, _| {
                (v1 << 1, Some(v1 & 0x80 != 0))
            }),
            _ => panic!("Unsupported opcode"),
        };
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

pub(crate) struct Ld;

pub(crate) type LdInstruction = InstructionWithAddress<Ld>;

implement_try_from_address!(LdInstruction, 0xA);

impl ExecutableOpcode for LdInstruction {
    fn execute(self, mut state: &mut Chip8) {
        state.index = self.address;
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

pub(crate) struct Rnd;

pub(crate) type RndInstruction = InstructionWithRegAndValue<Rnd>;

implement_try_from_reg_and_value!(RndInstruction, 0xC);

impl ExecutableOpcode for RndInstruction {
    fn execute(self, mut state: &mut Chip8) {
        let mut rng = thread_rng();
        let sample = rng.gen_range(0, 255);

        state.registers[self.reg as usize] = sample as u8 & self.value;

        util::increment_program_counter(&mut state);
    }
}

pub(crate) struct Drw;

pub(crate) type DrwInstruction = InstructionWithOperands<Drw>;

implement_try_from_operands!(DrwInstruction, 0xD);

impl ExecutableOpcode for DrwInstruction {
    fn execute(self, mut state: &mut Chip8) {
        fn translate_gfx(x: u16, y: u16) -> usize {
            ((x % 64) + ((y % 32) * 64)) as usize
        }

        let x = state.registers[self.op1 as usize];
        let y = state.registers[self.op2 as usize];
        let n = self.op3;

        state.registers[0xF] = 0;
        for y_pos in 0..n {
            let pixel_byte = state.memory[((state.index + y_pos as u16) % 4096) as usize];

            let mut x_pos = 0;
            let mut pixel_mask = 0x80;

            while x_pos < 8 {
                let pixel_bit = (pixel_byte & pixel_mask) > 0;

                let pixel_pos = translate_gfx(x as u16 + x_pos, y as u16 + y_pos as u16);

                if pixel_bit != state.gfx[pixel_pos] {
                    state.draw = true;
                }

                if pixel_bit {
                    if state.gfx[pixel_pos] {
                        state.registers[0xF] = 1;
                    }
                    state.gfx[pixel_pos] ^= true;
                }

                x_pos += 1;
                pixel_mask >>= 1;
            }
        }
        util::increment_program_counter(&mut state);
    }
}

pub(crate) struct Sk;

pub(crate) type SkInstruction = InstructionWithRegAndValue<Sk>;

implement_try_from_reg_and_value!(SkInstruction, 0xE);

impl ExecutableOpcode for SkInstruction {
    fn execute(self, mut state: &mut Chip8) {
        let skip = match self.value {
            0x9E => state.key[state.registers[self.reg as usize] as usize],
            0xA1 => !state.key[state.registers[self.reg as usize] as usize],
            _ => unimplemented!("Unsupported opcode"),
        };
        if skip {
            util::increment_program_counter(&mut state);
        }
        util::increment_program_counter(&mut state);
    }
}

pub(crate) struct Ldu;

pub(crate) type LduInstruction = InstructionWithRegAndValue<Ldu>;

implement_try_from_reg_and_value!(LduInstruction, 0xF);

impl ExecutableOpcode for LduInstruction {
    fn execute(self, mut state: &mut Chip8) {
        match self.value {
            0x07 => {
                state.registers[self.reg as usize] = state.delay_timer;
            }
            0x0A => {
                let mut key_pressed = false;
                for i in 0x0..=0xF {
                    if state.key[i] {
                        state.registers[self.reg as usize] = i as u8;
                        key_pressed = true;
                        break;
                    }
                }

                if !key_pressed {
                    // if no key was pressed, we directly return without
                    // incrementing the program counter
                    return;
                }
            }
            0x15 => {
                state.delay_timer = state.registers[self.reg as usize];
            }
            0x18 => {
                state.sound_timer = state.registers[self.reg as usize];
            }
            0x1E => {
                state.index = state
                    .index
                    .wrapping_add(state.registers[self.reg as usize] as u16);
            }
            0x29 => {
                let character: u16 = state.registers[self.reg as usize] as u16;
                assert!(character <= 0xF);
                state.index = CHIP8_CHARSET_OFFSET + character * 5;
            }
            0x33 => {
                let mut a: u8 = state.registers[self.reg as usize];
                state.memory[(state.index + 2) as usize] = (a % 10) as u8;

                a /= 10;
                state.memory[(state.index + 1) as usize] = (a % 10) as u8;

                a /= 10;
                state.memory[state.index as usize] = (a % 10) as u8;
            }
            0x55 => {
                for reg in 0x0..=self.reg {
                    state.memory[((state.index + reg as u16) % 4096) as usize] =
                        state.registers[reg as usize];
                }
            }
            0x65 => {
                for reg in 0x0..=self.reg {
                    state.registers[reg as usize] =
                        state.memory[((state.index + reg as u16) % 4096) as usize];
                }
            }
            _ => unimplemented!("Unsupported opcode"),
        }
        util::increment_program_counter(&mut state);
    }
}
