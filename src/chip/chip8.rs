mod constants;
mod opcodes;
mod util;

#[macro_use]
mod macros;

#[cfg(test)]
mod tests;

use rand::{thread_rng, Rng};

use std::fs;
use std::fs::File;
use std::io::Read;

use core::convert::TryFrom;
use std::marker::PhantomData;

use cursive::{
    direction::Direction,
    event::{Event, EventResult},
    theme::{BaseColor, Color, ColorStyle},
    view::View,
    CbSink, Printer, Vec2,
};

use crate::chip::{
    chip8::constants::{
        CHIP8_CHARSET, CHIP8_CHARSET_LEN, CHIP8_CHARSET_OFFSET, CHIP8_TIMER_RESOLUTION,
    },
    chip8::opcodes::Opcode,
    Chip, ChipWithDisplayOutput, LoadProgramError,
};

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
    draw: bool,
    cycles_since_timer_dec: u8,
}

#[derive(Debug)]
enum InstructionParsingError {
    InvalidInstructionClass(u8, u8),
}

trait Executable {
    fn execute(self, state: &mut Chip8);
}

pub(crate) struct InstructionWithAddress<T> {
    instruction: PhantomData<T>,
    address: u16,
}

pub(crate) struct InstructionWithOperands<T> {
    instruction: PhantomData<T>,
    op1: u8,
    op2: u8,
    op3: u8,
}

pub(crate) struct InstructionWithRegAndValue<T> {
    instruction: PhantomData<T>,
    reg: u8,
    value: u8,
}

struct Sys;

type SysInstruction = InstructionWithAddress<Sys>;

implement_try_from_address!(SysInstruction, 0x0);

impl Executable for SysInstruction {
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

struct Jmp;

type JmpInstruction = InstructionWithAddress<Jmp>;

implement_try_from_address!(JmpInstruction, 0x1);

impl Executable for JmpInstruction {
    fn execute(self, state: &mut Chip8) {
        state.program_counter = self.address;
    }
}

struct Call;

type CallInstruction = InstructionWithAddress<Call>;

implement_try_from_address!(CallInstruction, 0x2);

impl Executable for CallInstruction {
    fn execute(self, state: &mut Chip8) {
        assert!(state.stack_pointer < 16, "Stack overflow");
        state.stack[state.stack_pointer as usize] = state.program_counter;
        state.stack_pointer = state.stack_pointer + 1;
        state.program_counter = self.address;
    }
}

struct Se;

type SeInstruction = InstructionWithRegAndValue<Se>;

implement_try_from_reg_and_value!(SeInstruction, 0x3);

impl Executable for SeInstruction {
    fn execute(self, mut state: &mut Chip8) {
        util::conditional_skip(&self, &mut state, |instruction, state| {
            state.registers[instruction.reg as usize] == instruction.value
        });
        util::increment_program_counter(&mut state);
    }
}

struct Sne;

type SneInstruction = InstructionWithRegAndValue<Sne>;

implement_try_from_reg_and_value!(SneInstruction, 0x4);

impl Executable for SneInstruction {
    fn execute(self, mut state: &mut Chip8) {
        util::conditional_skip(&self, &mut state, |instruction, state| {
            state.registers[instruction.reg as usize] != instruction.value
        });
        util::increment_program_counter(&mut state);
    }
}

struct Sre;

type SreInstruction = InstructionWithOperands<Sre>;

implement_try_from_operands!(SreInstruction, 0x5);

impl Executable for SreInstruction {
    fn execute(self, mut state: &mut Chip8) {
        util::conditional_skip(&self, &mut state, |instruction, state| {
            assert_eq!(instruction.op3, 0, "Unsupported opcode");
            state.registers[instruction.op1 as usize] == state.registers[instruction.op2 as usize]
        });
        util::increment_program_counter(&mut state);
    }
}

struct Ldr;

type LdrInstruction = InstructionWithRegAndValue<Ldr>;

implement_try_from_reg_and_value!(LdrInstruction, 0x6);

impl Executable for LdrInstruction {
    fn execute(self, mut state: &mut Chip8) {
        state.registers[self.reg as usize] = self.value;
        util::increment_program_counter(&mut state);
    }
}

struct Add;

type AddInstruction = InstructionWithRegAndValue<Add>;

implement_try_from_reg_and_value!(AddInstruction, 0x7);

impl Executable for AddInstruction {
    fn execute(self, mut state: &mut Chip8) {
        state.registers[self.reg as usize] =
            state.registers[self.reg as usize].wrapping_add(self.value);
        util::increment_program_counter(&mut state);
    }
}

struct Reg;

type RegInstruction = InstructionWithOperands<Reg>;

implement_try_from_operands!(RegInstruction, 0x8);

impl Executable for RegInstruction {
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

struct Srne;

type SrneInstruction = InstructionWithOperands<Srne>;

implement_try_from_operands!(SrneInstruction, 0x9);

impl Executable for SrneInstruction {
    fn execute(self, mut state: &mut Chip8) {
        util::conditional_skip(&self, &mut state, |instruction, state| {
            assert_eq!(instruction.op3, 0, "Unsupported opcode");
            state.registers[instruction.op1 as usize] != state.registers[instruction.op2 as usize]
        });
        util::increment_program_counter(&mut state);
    }
}

struct Ld;

type LdInstruction = InstructionWithAddress<Ld>;

implement_try_from_address!(LdInstruction, 0xA);

impl Executable for LdInstruction {
    fn execute(self, mut state: &mut Chip8) {
        state.index = self.address;
        util::increment_program_counter(&mut state);
    }
}

struct Jmpr;

type JmprInstruction = InstructionWithAddress<Jmpr>;

implement_try_from_address!(JmprInstruction, 0xB);

impl Executable for JmprInstruction {
    fn execute(self, mut state: &mut Chip8) {
        state.program_counter = self.address.wrapping_add(state.registers[0] as u16);
    }
}

struct Rnd;

type RndInstruction = InstructionWithRegAndValue<Rnd>;

implement_try_from_reg_and_value!(RndInstruction, 0xC);

impl Executable for RndInstruction {
    fn execute(self, mut state: &mut Chip8) {
        let mut rng = thread_rng();
        let sample = rng.gen_range(0, 255);

        state.registers[self.reg as usize] = sample as u8 & self.value;

        util::increment_program_counter(&mut state);
    }
}

struct Drw;

type DrwInstruction = InstructionWithOperands<Drw>;

implement_try_from_operands!(DrwInstruction, 0xD);

impl Executable for DrwInstruction {
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

struct Sk;

type SkInstruction = InstructionWithRegAndValue<Sk>;

implement_try_from_reg_and_value!(SkInstruction, 0xE);

impl Executable for SkInstruction {
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

struct Ldu;

type LduInstruction = InstructionWithRegAndValue<Ldu>;

implement_try_from_reg_and_value!(LduInstruction, 0xF);

impl Executable for LduInstruction {
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

impl Chip for Chip8 {
    type PinAddress = u8;
    type MemoryAddress = u16;

    fn load_program(&mut self, path: &str) -> Result<usize, LoadProgramError> {
        let mut file =
            File::open(path).map_err(|_| LoadProgramError::CouldNotOpenFile(path.to_string()))?;
        let md = fs::metadata(path)
            .map_err(|_| LoadProgramError::CouldNotReadMetadata(path.to_string()))?;
        let mut buffer = vec![0; md.len() as usize];
        file.read(&mut buffer)
            .map_err(|_| LoadProgramError::CouldNotReadFile(path.to_string()))?;

        self.load_program_bytes(&buffer);

        Ok(md.len() as usize)
    }

    fn load_program_bytes(&mut self, program: &[u8]) {
        for i in 0..program.len() {
            self.set_memory_byte(program[i], (0x200 + i) as u16);
        }
    }

    fn cycle(&mut self) {
        let opcode = self.next_instruction();
        let mut state = self;
        opcode.execute(&mut state);

        state.cycles_since_timer_dec += 1;

        if state.cycles_since_timer_dec % CHIP8_TIMER_RESOLUTION == 0 {
            if state.delay_timer > 0 {
                state.delay_timer -= 1;
            }

            if state.sound_timer > 0 {
                // TODO let it beep
                state.sound_timer -= 1;
            }

            state.cycles_since_timer_dec = 0;
        }
    }

    fn read_output_pins(&self) -> [bool; 64 * 32] {
        self.gfx
    }

    fn set_input_pin(&mut self, pin: u8, value: bool) {
        assert!(pin & 0x0F == pin);
        self.key[pin as usize] = value;
    }

    fn reset_input_pins(&mut self) {
        for i in 0..16 {
            self.key[i] = false;
        }
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
            draw: false,
            cycles_since_timer_dec: 0,
        }
    }

    pub fn next_instruction(&self) -> Opcode {
        assert!(self.program_counter <= 4094);
        Opcode::new(&[
            self.memory[self.program_counter as usize],
            self.memory[(self.program_counter + 1) as usize],
        ])
    }

    fn set_memory_byte(&mut self, byte: u8, index: u16) {
        assert!(index < 4096);
        self.memory[index as usize] = byte;
    }
}

pub struct Display {
    pixels: [bool; 64 * 32],
}

impl Display {
    pub fn new(pixels: [bool; 64 * 32]) -> Self {
        Display { pixels }
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new([false; 64 * 32])
    }
}

impl View for Display {
    fn draw(&self, printer: &Printer) {
        printer.with_color(
            ColorStyle::new(Color::Dark(BaseColor::Black), Color::RgbLowRes(0, 0, 0)),
            |printer| {
                for x in 0..64 {
                    for y in 0..32 {
                        if self.pixels[x + 64 * y] {
                            printer.print((x, y), " ");
                        }
                    }
                }
            },
        );
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        true
    }

    fn on_event(&mut self, _event: Event) -> EventResult {
        EventResult::Ignored
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
        Vec2 { x: 64, y: 32 }
    }
}

impl ChipWithDisplayOutput for Chip8 {
    type Display = Display;

    fn get_display(&self) -> Display {
        Display::new(self.read_output_pins())
    }

    fn update_ui(&mut self, gfx_sink: &CbSink) {
        if !self.draw {
            return;
        }
        let display = self.get_display();
        gfx_sink
            .send(Box::new(Box::new(move |s: &mut cursive::Cursive| {
                s.pop_layer();
                s.add_layer(display);
            })))
            .expect("Sending updated display failed");
        self.draw = false;
    }
}
