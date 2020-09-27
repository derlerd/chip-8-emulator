mod constants;
mod opcodes;
mod util;

#[cfg(test)]
mod tests;

use std::fs;
use std::fs::File;
use std::io::Read;

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
