#[macro_use]
mod macros;
mod instructions;

use core::convert::TryFrom;
use std::marker::PhantomData;

use crate::chip::chip8::opcodes::instructions::{
    AddInstruction, CallInstruction, DrwInstruction, JmpInstruction, JmprInstruction,
    LdInstruction, LdrInstruction, LduInstruction, RegInstruction, RndInstruction, SeInstruction,
    SkInstruction, SneInstruction, SreInstruction, SrneInstruction, SysInstruction,
};
use crate::chip::chip8::Chip8;

#[derive(Debug)]
pub struct Opcode {
    pub instruction_class: u8,
    pub payload: OpcodePayload,
}

#[derive(Debug)]
pub struct OpcodePayload {
    bytes: [u8; 3],
}

impl OpcodePayload {
    pub fn address(&self) -> u16 {
        (self.bytes[0] as u16) << 8 | (self.bytes[1] as u16) << 4 | self.bytes[2] as u16
    }

    pub fn reg_and_value(&self) -> (u8, u8) {
        (self.bytes[0], (self.bytes[1] << 4) | self.bytes[2])
    }

    pub fn operands(&self) -> (u8, u8, u8) {
        (self.bytes[0], self.bytes[1], self.bytes[2])
    }
}

impl Opcode {
    pub fn new(opcode: &[u8; 2]) -> Opcode {
        Opcode {
            instruction_class: opcode[0] >> 4,
            payload: OpcodePayload {
                bytes: [opcode[0] & 0xF, opcode[1] >> 4, opcode[1] & 0xF],
            },
        }
    }

    pub fn execute(self, mut state: &mut Chip8) {
        fn execute<T>(opcode : Opcode, mut state : &mut Chip8) 
        where 
            T : ExecutableOpcode + TryFrom<Opcode>,
            <T as TryFrom<Opcode>>::Error: std::fmt::Debug
        {
            // We can safely unwrap the converted instructions below as we know
            // that the instruction class will map the respective instruction.
            T::try_from(opcode).unwrap().execute(&mut state);
        }
        
        match self.instruction_class {
            0x0 => execute::<SysInstruction>(self, &mut state),
            0x1 => execute::<JmpInstruction>(self, &mut state),
            0x2 => execute::<CallInstruction>(self, &mut state),
            0x3 => execute::<SeInstruction>(self, &mut state),
            0x4 => execute::<SneInstruction>(self, &mut state),
            0x5 => execute::<SreInstruction>(self, &mut state),
            0x6 => execute::<LdrInstruction>(self, &mut state),
            0x7 => execute::<AddInstruction>(self, &mut state),
            0x8 => execute::<RegInstruction>(self, &mut state),
            0x9 => execute::<SrneInstruction>(self, &mut state),
            0xA => execute::<LdInstruction>(self, &mut state),
            0xB => execute::<JmprInstruction>(self, &mut state),
            0xC => execute::<RndInstruction>(self, &mut state),
            0xD => execute::<DrwInstruction>(self, &mut state),
            0xE => execute::<SkInstruction>(self, &mut state),
            0xF => execute::<LduInstruction>(self, &mut state),
            _ => unimplemented!("Unsupported opcode"),
        };
    }
}

#[derive(Debug)]
pub(crate) enum InstructionParsingError {
    InvalidInstructionClass(u8, u8),
}

trait ExecutableOpcode {
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
