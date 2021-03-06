use core::convert::TryFrom;
use rand::{thread_rng, Rng};
use std::marker::PhantomData;

use crate::chip::chip8::{
    constants::CHIP8_CHARSET_OFFSET,
    opcodes::{
        Instruction, InstructionParsingError, InstructionWithAddress, InstructionWithOperands,
        InstructionWithRegAndValue, Opcode,
    },
    util, Chip8,
};

define_instruction_with_reg_and_value!(Ldr, LdrInstruction, 0x6);
impl Instruction for LdrInstruction {
    /// Opcode of the form `0x6XYZ` (LDR). Load a value `YZ` into `state.registers[X]`.
    fn execute(&self, mut state: &mut Chip8) {
        state.registers[self.reg as usize] = self.value;
        util::increment_program_counter(&mut state);
    }
}

define_instruction_with_reg_and_value!(Add, AddInstruction, 0x7);
impl Instruction for AddInstruction {
    /// Opcode of the form `0x7XYZ` (ADD). Add value `YZ` into `state.registers[X]`.    
    fn execute(&self, mut state: &mut Chip8) {
        state.registers[self.reg as usize] =
            state.registers[self.reg as usize].wrapping_add(self.value);
        util::increment_program_counter(&mut state);
    }
}

define_instruction_with_operands!(Reg, RegInstruction, 0x8);
impl Instruction for RegInstruction {
    /// Opcode of the form `0x8XYZ` (REG). Applies operation determined by `Z` to `state.registers[X]`
    /// and `state.registers[Y]` and stores the result in `state.registers[X]`.
    ///
    /// - If `Z == 0x0`, it sets `state.registers[X] = state.registers[Y]`
    ///
    /// - If `Z == 0x1`, it sets `state.registers[X] = state.registers[X] | state.registers[Y]`
    ///
    /// - If `Z == 0x2`, it sets `state.registers[X] = state.registers[X] & state.registers[Y]`
    ///
    /// - If `Z == 0x3`, it sets `state.registers[X] = state.registers[X] ^ state.registers[Y]`
    ///
    /// - If `Z == 0x4`, it sets `state.registers[X] = state.registers[X] + state.registers[Y]`;
    ///   `state.registers[0xF]` is set to `1` if there is an overflow, and to `0` otherwise.
    ///
    /// - If `Z == 0x5`, it sets `state.registers[X] = state.registers[X] - state.registers[Y]`;
    ///   `state.registers[0xF]` is set to `0` if there is an underflow, and to `1` otherwise.
    ///
    /// - If `Z == 0x6`, it sets `state.registers[X] = state.registers[X] >> 1`; `state.registers[0xF]`
    ///   is set to `1` if the shifted out bit is set, and to `0` otherwise.
    ///
    /// - If `Z == 0x7`, it sets `state.registers[X] = state.registers[Y] - state.registers[X]`;
    ///   `state.registers[0xF]` is set to `0` if there is an underflow, and to `1` otherwise.
    ///
    /// - If `Z == 0xE`, it sets `state.registers[X] = state.registers[X] << 1`; `state.registers[0xF]`
    ///   is set to `1` if the shifted out bit is set, and to `0` otherwise.
    ///
    fn execute(&self, mut state: &mut Chip8) {
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

define_instruction_with_address!(Ld, LdInstruction, 0xA);
impl Instruction for LdInstruction {
    /// Opcode of the form `0xAXYZ` (LD). Loads `XYZ` into `state.index`.
    fn execute(&self, mut state: &mut Chip8) {
        state.index = self.address;
        util::increment_program_counter(&mut state);
    }
}

define_instruction_with_reg_and_value!(Rnd, RndInstruction, 0xC);
impl Instruction for RndInstruction {
    /// Opcode of the form `0xCXYZ` (RND). Generates a random value `v`, and sets
    /// `state.registers[X] = v & YZ.
    fn execute(&self, mut state: &mut Chip8) {
        let mut rng = thread_rng();
        let sample = rng.gen_range(0, 255);

        state.registers[self.reg as usize] = sample as u8 & self.value;

        util::increment_program_counter(&mut state);
    }
}

define_instruction_with_operands!(Drw, DrwInstruction, 0xD);
impl Instruction for DrwInstruction {
    /// Opcode of the form `0xDXYZ` (DRW). Draws a sprite to the display.
    ///
    /// - Drawing a pixel means flipping the pixel on the display.
    /// - The sprite is drawn at coordinate (`X`, `Y`) on the display.
    /// - A sprite is of width `8` pixels and of height `Z` pixels.
    /// - The bits to be written are taken from the memory starting at the address
    ///   given in the index register.
    /// - `state.registers[0xF]` is set to `1` if any pixel is flipped to `0`, and
    ///   to `0` otherwise.
    fn execute(&self, mut state: &mut Chip8) {
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

                if pixel_bit != state.output_pins[pixel_pos] {
                    state.draw = true;
                }

                if pixel_bit {
                    if state.output_pins[pixel_pos] {
                        state.registers[0xF] = 1;
                    }
                    state.output_pins[pixel_pos] ^= true;
                }

                x_pos += 1;
                pixel_mask >>= 1;
            }
        }
        util::increment_program_counter(&mut state);
    }
}

define_instruction_with_reg_and_value!(Ldu, LduInstruction, 0xF);
impl Instruction for LduInstruction {
    /// Opcode of the form `0xFXYZ` (LDU). Groups various load and store operations.
    /// `YZ` determines which operation is executed relative to `state`.
    ///
    /// - If `YZ == 0x07`, set `state.registers[X] = delay_timer`.
    ///
    /// - If `YZ == 0x0A`, await key press and store in `state.registers[X]`.
    ///
    /// - If `YZ == 0x15`, set `state.delay_timer = state.registers[X]`.
    ///
    /// - If `YZ == 0x18`, set `state.sound_timer = state.registers[X]`.
    ///
    /// - If `YZ == 0x1E`, set `state.index += state.registers[X]` (where the addition will wrap
    ///   around on overflow).
    ///
    /// - If `YZ == 0x29`, load the address of the character in `state.registers[X]` into `index`.
    ///
    /// - If `YZ == 0x33`, store the binary coded decimal representation of `state.registers[X]` in
    ///   `state.memory[index]`-`state.memory[index+2], where the most significant digit goes into
    ///   `state.memory[index]`.
    ///
    /// - If `YZ == 0x55`, store `state.registers[0]` to `state.registers[X]` in memory starting
    ///   at `state.index`.
    ///
    /// - If `YZ == 0x65`, load `state.registers[0]` to `state.registers[X]` from memory starting
    ///   at `state.index`.
    ///
    fn execute(&self, mut state: &mut Chip8) {
        match self.value {
            0x07 => {
                state.registers[self.reg as usize] = state.delay_timer;
            }
            0x0A => {
                let mut input_pin_set = false;
                for i in 0x0..=0xF {
                    if state.input_pins[i] {
                        state.registers[self.reg as usize] = i as u8;
                        input_pin_set = true;
                        break;
                    }
                }

                if !input_pin_set {
                    // if no input pin was set, we directly return without
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
