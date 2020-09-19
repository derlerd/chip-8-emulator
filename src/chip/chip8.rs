use rand::{thread_rng, Rng};

use crate::chip::Chip;

#[cfg(test)]
mod tests;

const CHIP8_CHARSET_OFFSET: u16 = 0x50; // 80

const CHIP8_CHARSET_LEN: u16 = 0x50; // 80

const CHIP8_CHARSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(Clone)]
pub struct Chip8 {
    opcode: u16,
    memory: [u8; 4096],
    registers: [u8; 16],
    index: u16,
    program_counter: u16,
    gfx: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    stack_pointer: u16,
}

impl Chip for Chip8 {
    fn cycle(self) -> Self {
        let opcode = self.next_instruction();
        let chip = opcode.execute(self);

        chip
    }

    fn get_gfx(&self) -> [u8; 64 * 32] {
        [0; 64 * 32]
    }

    fn set_memory_byte(&mut self, byte: u8, index: usize) {
        assert!(index < 4096);
        self.memory[index] = byte;
    }
}

impl Chip8 {
    pub fn new() -> Self {
        let mut memory = [0; 4096];
        for i in 0..CHIP8_CHARSET_LEN - 1 {
            memory[(i + CHIP8_CHARSET_OFFSET) as usize] = CHIP8_CHARSET[i as usize];
        }

        Chip8 {
            opcode: 0,
            memory,
            registers: [0; 16],
            index: 0,
            program_counter: 0x200,
            gfx: [0; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            stack_pointer: 0,
        }
    }

    pub fn next_instruction(&self) -> Opcode {
        assert!(self.program_counter <= 4094);
        Opcode::new(&[
            self.memory[self.program_counter as usize],
            self.memory[(self.program_counter + 1) as usize],
        ])
    }
}

#[derive(Debug)]
pub struct Opcode {
    bytes: [u8; 4],
}

impl Opcode {
    pub fn new(opcode: &[u8; 2]) -> Opcode {
        Opcode {
            bytes: [
                opcode[0] >> 4,
                opcode[0] & 0xF,
                opcode[1] >> 4,
                opcode[1] & 0xF,
            ],
        }
    }

    pub fn address(&self) -> u16 {
        (self.bytes[1] as u16) << 8 | (self.bytes[2] as u16) << 4 | self.bytes[3] as u16
    }

    pub fn reg_and_value(&self) -> (u8, u8) {
        (self.bytes[1], (self.bytes[2] << 4) | self.bytes[3])
    }

    pub fn regs_and_op(&self) -> (u8, u8, u8) {
        (self.bytes[1], self.bytes[2], self.bytes[3])
    }

    pub fn execute(self, state: Chip8) -> Chip8 {
        fn conditional_skip(opcode: &Opcode, state: &mut Chip8, f: fn(&Opcode, &Chip8) -> bool) {
            if f(opcode, state) {
                increment_program_counter(state);
            }
        }

        fn increment_program_counter(state: &mut Chip8) {
            state.program_counter = state.program_counter.wrapping_add(2);
        }

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

        let mut next_state = state.clone();
        match self.bytes[0] {
            0x0 => {
                let payload = self.address();
                match payload {
                    0x0E0 => {
                        next_state.gfx = [0; 64 * 32];
                        next_state.program_counter += 2;
                    }
                    0x0EE => {
                        assert!(state.stack_pointer > 0, "Stack underflow");
                        next_state.program_counter =
                            state.stack[(state.stack_pointer - 1) as usize];
                        next_state.stack_pointer = state.stack_pointer - 1;
                        increment_program_counter(&mut next_state);
                    }
                    _ => panic!("Opcode not supported {:x}", payload),
                };
            }
            0x1 => {
                next_state.program_counter = self.address();
            }
            0x2 => {
                assert!(state.stack_pointer < 16, "Stack overflow");
                increment_program_counter(&mut next_state);
                next_state.stack[next_state.stack_pointer as usize] = next_state.program_counter;
                next_state.stack_pointer = next_state.stack_pointer + 1;
                next_state.program_counter = self.address();
            }
            0x3 => {
                conditional_skip(&self, &mut next_state, |opcode, state| {
                    let (reg, value) = opcode.reg_and_value();
                    state.registers[reg as usize] == value
                });
                increment_program_counter(&mut next_state);
            }
            0x4 => {
                conditional_skip(&self, &mut next_state, |opcode, state| {
                    let (reg, value) = opcode.reg_and_value();
                    state.registers[reg as usize] != value
                });
                increment_program_counter(&mut next_state);
            }
            0x5 => {
                conditional_skip(&self, &mut next_state, |opcode, state| {
                    let (reg1, reg2, zero) = opcode.regs_and_op();
                    assert_eq!(zero, 0, "Unsupported opcode");
                    state.registers[reg1 as usize] == state.registers[reg2 as usize]
                });
                increment_program_counter(&mut next_state);
            }
            0x6 => {
                let (reg, value) = self.reg_and_value();
                next_state.registers[reg as usize] = value;
                increment_program_counter(&mut next_state);
            }
            0x7 => {
                let (reg, value) = self.reg_and_value();
                next_state.registers[reg as usize] =
                    next_state.registers[reg as usize].wrapping_add(value);
                increment_program_counter(&mut next_state);
            }
            0x8 => {
                let (reg1, reg2, op) = self.regs_and_op();
                match op {
                    0x0 => modify_registers(&mut next_state, reg1, reg2, |_, v2| (v2, None)),
                    0x1 => modify_registers(&mut next_state, reg1, reg2, |v1, v2| (v1 | v2, None)),
                    0x2 => modify_registers(&mut next_state, reg1, reg2, |v1, v2| (v1 & v2, None)),
                    0x3 => modify_registers(&mut next_state, reg1, reg2, |v1, v2| (v1 ^ v2, None)),
                    0x4 => modify_registers(&mut next_state, reg1, reg2, |v1, v2| {
                        let (result, overflow) = v1.overflowing_add(v2);
                        (result, Some(overflow))
                    }),
                    0x5 => modify_registers(&mut next_state, reg1, reg2, |v1, v2| {
                        let (result, overflow) = v1.overflowing_sub(v2);
                        (result, Some(!overflow))
                    }),
                    0x6 => modify_registers(&mut next_state, reg1, reg2, |v1, _| {
                        (v1 >> 1, Some(v1 & 1 != 0))
                    }),
                    0x7 => modify_registers(&mut next_state, reg1, reg2, |v1, v2| {
                        let (result, overflow) = v2.overflowing_sub(v1);
                        (result, Some(!overflow))
                    }),
                    0xE => modify_registers(&mut next_state, reg1, reg2, |v1, _| {
                        (v1 << 1, Some(v1 & 0x80 != 0))
                    }),
                    _ => panic!("Unsupported opcode"),
                };
                increment_program_counter(&mut next_state);
            }
            0x9 => {
                conditional_skip(&self, &mut next_state, |opcode, state| {
                    let (reg1, reg2, zero) = opcode.regs_and_op();
                    assert_eq!(zero, 0, "Unsupported opcode");
                    state.registers[reg1 as usize] != state.registers[reg2 as usize]
                });
                increment_program_counter(&mut next_state);
            }
            0xA => {
                next_state.index = self.address();
                increment_program_counter(&mut next_state);
            }
            0xB => {
                next_state.program_counter = self.address().wrapping_add(state.registers[0] as u16);
            }
            0xC => {
                let (reg, value) = self.reg_and_value();

                let mut rng = thread_rng();
                let sample = rng.gen_range(0, 255);

                next_state.registers[reg as usize] = sample as u8 & value;

                increment_program_counter(&mut next_state);
            }
            // TODO implement IO operations
            _ => unimplemented!(),
        };
        next_state
    }
}
