#[macro_use]
mod macros;
mod arithmetic_and_logic;
mod program_flow;
mod system;

use core::convert::TryFrom;
use std::marker::PhantomData;

use crate::chip::chip8::{
    opcodes::{
        arithmetic_and_logic::{
            AddInstruction, DrwInstruction, LdInstruction, LdrInstruction, LduInstruction,
            RegInstruction, RndInstruction,
        },
        program_flow::{
            CallInstruction, JmpInstruction, JmprInstruction, SeInstruction, SkInstruction,
            SneInstruction, SreInstruction, SrneInstruction,
        },
        system::SysInstruction,
    },
    Chip8,
};

/// Represents a Chip 8 opcode. A Chip 8 opcode is two bytes long.  
#[derive(Debug)]
pub struct Opcode {
    /// The instruction class is the most significant nibble of the opcode.
    /// Note that we use a u8 to represent the instruction class here for
    /// convenience, but ensure that a valid opcode can only be constructed
    /// if the four most significant bits of the u8 are 0.
    pub instruction_class: u8,
    /// The payload constitutes the remaining nibbles of the opcode.
    pub payload: OpcodePayload,
}

/// Represents the payload of a Chip 8 opcode. That is the opcode without
/// the most significant nibble.
#[derive(Debug)]
pub struct OpcodePayload {
    /// The nibbles representing the payload. Note that we use the u8 type
    /// here for convenience, but ensure that valid payloads can only be
    /// constructed if the four most significant bits of the u8 are 0.
    bytes: [u8; 3],
}

impl OpcodePayload {
    /// Interprets the opcode payload as an address in the range 0x000 to
    /// 0xFFF (inclusive) and returns an u16 containing this address.
    pub fn address(&self) -> u16 {
        (self.bytes[0] as u16) << 8 | (self.bytes[1] as u16) << 4 | self.bytes[2] as u16
    }

    /// Interprets the most significant nibble of the opcode as a register
    /// address in range 0x0 - 0xF (inclusive) and the remaining nibbles
    /// as a value in range 0x00 - 0xFF (inclusive) and returns a tuple
    /// representing these values.
    pub fn reg_and_value(&self) -> (u8, u8) {
        (self.bytes[0], (self.bytes[1] << 4) | self.bytes[2])
    }

    /// Interprets the opcode payload as three operands, each of size
    /// one nibble, i.e., in range 0x0 - 0xF (inclusive) and returns
    /// a triple representing these values.
    pub fn operands(&self) -> (u8, u8, u8) {
        (self.bytes[0], self.bytes[1], self.bytes[2])
    }
}

impl Opcode {
    /// Constructs a new `Opcode` given its byte representation.
    pub fn new(opcode: &[u8; 2]) -> Opcode {
        Opcode {
            instruction_class: opcode[0] >> 4,
            payload: OpcodePayload {
                bytes: [opcode[0] & 0xF, opcode[1] >> 4, opcode[1] & 0xF],
            },
        }
    }

    /// Executes `self` relative to the given state. Note that this
    /// method will in-place modify the given state.
    pub fn execute(self, mut state: &mut Chip8) {
        fn execute<T>(opcode: Opcode, mut state: &mut Chip8)
        where
            T: ExecutableOpcode + TryFrom<Opcode>,
            <T as TryFrom<Opcode>>::Error: std::fmt::Debug,
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

/// Captures errors when converting opcodes to their respective instruction object.
#[derive(Debug)]
pub(crate) enum InstructionParsingError {
    /// The given
    InvalidInstructionClass(u8, u8),
}

/// Represents an opcode that can be executed.
trait ExecutableOpcode {
    /// Executes `self` relative to the given state. Note that this
    /// method will in-place modify the given state.
    fn execute(self, state: &mut Chip8);
}

/// Represents an opcode that expects the payload to be an address.
pub(crate) struct InstructionWithAddress<T> {
    instruction: PhantomData<T>,
    address: u16,
}

/// Represents an opcode that expects the payload to be three operands.
pub(crate) struct InstructionWithOperands<T> {
    instruction: PhantomData<T>,
    op1: u8,
    op2: u8,
    op3: u8,
}

/// Represents an opcode that expects the payload to be a register pointer and a value.
pub(crate) struct InstructionWithRegAndValue<T> {
    instruction: PhantomData<T>,
    reg: u8,
    value: u8,
}

impl std::fmt::Display for InstructionParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InstructionParsingError::InvalidInstructionClass(got, expected) => write!(
                f,
                "Error while parsing opcode. Instruction class does 
                           not comply with target objects instruction class. 
                           Got {}, expected {}.",
                got, expected
            ),
        }
    }
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Opcode with instruction_class: {} and payload: {}.",
            self.instruction_class, self.payload
        )
    }
}

impl std::fmt::Display for OpcodePayload {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[{}, {}, {}]",
            self.bytes[0], self.bytes[1], self.bytes[2]
        )
    }
}
