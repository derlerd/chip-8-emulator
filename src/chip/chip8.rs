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
    gfx: [bool; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    stack_pointer: u16,
    key: [bool; 16],
}

impl Chip for Chip8 {
    type PinAddress = u8;

    fn cycle(self) -> Self {
        let opcode = self.next_instruction();
        let mut chip = opcode.execute(self);

        if chip.delay_timer > 0 {
            chip.delay_timer -= 1;
        }

        if chip.sound_timer > 0 {
            // TODO let it beep
            chip.sound_timer -= 1;
        }

        chip
    }

    fn get_gfx(&self) -> [bool; 64 * 32] {
        self.gfx
    }

    fn set_io_pin(&mut self, pin: u8, value: bool) {
        assert!(pin & 0x0F == pin);
        self.key[pin as usize] = value;
    }

    fn reset_io_pins(&mut self) {
        for i in 0..16 {
            self.key[i] = false;
        }
    }

    fn set_memory_byte(&mut self, byte: u8, index: usize) {
        assert!(index < 4096);
        self.memory[index] = byte;
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

    pub fn operands(&self) -> (u8, u8, u8) {
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

        fn translate_gfx(x: u16, y: u16) -> usize {
            ((x % 64) + ((y % 32) * 64)) as usize
        }

        let mut next_state = state.clone();
        match self.bytes[0] {
            0x0 => {
                let payload = self.address();
                match payload {
                    0x0E0 => {
                        next_state.gfx = [false; 64 * 32];
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
                    let (reg1, reg2, zero) = opcode.operands();
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
                let (reg1, reg2, op) = self.operands();
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
                    let (reg1, reg2, zero) = opcode.operands();
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
            0xD => {
                let (x, y, n) = self.operands();

                let x = next_state.registers[x as usize];
                let y = next_state.registers[y as usize];

                next_state.registers[0xF] = 0;
                for y_pos in 0..n {
                    let pixel_byte =
                        next_state.memory[((next_state.index + y_pos as u16) % 4096) as usize];

                    let mut x_pos = 0;
                    let mut pixel_mask = 0x80;

                    while x_pos < 8 {
                        let pixel_bit = (pixel_byte & pixel_mask) > 0;

                        let pixel_pos = translate_gfx(x as u16 + x_pos, y as u16 + y_pos as u16);
                        if pixel_bit {
                            next_state.gfx[pixel_pos] = true;
                        } else {
                            if next_state.gfx[pixel_pos] {
                                next_state.registers[0xF] = 1;
                            }
                            next_state.gfx[pixel_pos] = false;
                        }

                        x_pos += 1;
                        pixel_mask >>= 1;
                    }
                }
                increment_program_counter(&mut next_state);
            }
            0xE => {
                let (reg, value) = self.reg_and_value();
                let skip = match value {
                    0x9E => next_state.key[next_state.registers[reg as usize] as usize],
                    0xA1 => !next_state.key[next_state.registers[reg as usize] as usize],
                    _ => unimplemented!("Unsupported opcode"),
                };
                if skip {
                    increment_program_counter(&mut next_state);
                }
                increment_program_counter(&mut next_state);
            }
            0xF => {
                let (reg, value) = self.reg_and_value();
                match value {
                    0x07 => {
                        next_state.registers[reg as usize] = next_state.delay_timer;
                    }
                    0x0A => {
                        let mut key_pressed = false;
                        for i in 0x0..=0xF {
                            if next_state.key[i] {
                                next_state.registers[reg as usize] = i as u8;
                                key_pressed = true;
                                break;
                            }
                        }

                        if !key_pressed {
                            // if no key was pressed, we directly return without
                            // incrementing the program counter
                            return next_state;
                        }
                    }
                    0x15 => {
                        next_state.delay_timer = next_state.registers[reg as usize];
                    }
                    0x18 => {
                        next_state.sound_timer = next_state.registers[reg as usize];
                    }
                    0x1E => {
                        next_state.index = next_state
                            .index
                            .wrapping_add(next_state.registers[reg as usize] as u16);
                    }
                    0x29 => {
                        let character: u16 = next_state.registers[reg as usize] as u16;
                        assert!(character <= 0xF);
                        next_state.index = CHIP8_CHARSET_OFFSET + character * 5;
                    }
                    0x33 => {
                        let mut a: u8 = next_state.registers[reg as usize];
                        next_state.memory[(next_state.index + 2) as usize] = (a % 10) as u8;

                        a /= 10;
                        next_state.memory[(next_state.index + 1) as usize] = (a % 10) as u8;

                        a /= 10;
                        next_state.memory[next_state.index as usize] = (a % 10) as u8;
                    }
                    0x55 => {
                        for reg in 0x0..=reg {
                            next_state.memory[((next_state.index + reg as u16) % 4096) as usize] =
                                next_state.registers[reg as usize];
                        }
                    }
                    0x65 => {
                        for reg in 0x0..=reg {
                            next_state.registers[reg as usize] = next_state.memory
                                [((next_state.index + reg as u16) % 4096) as usize];
                        }
                    }
                    _ => unimplemented!("Unsupported opcode"),
                }
                increment_program_counter(&mut next_state);
            }
            _ => unimplemented!("Unsupported opcode"),
        };
        next_state
    }
}
