#[macro_use]
mod macros;
mod arithmetic_and_logic;
mod program_flow;
mod system;

use core::convert::{Into, TryFrom};
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
pub(super) struct Opcode {
    /// The instruction class is the most significant nibble of the opcode.
    /// Note that we use a u8 to represent the instruction class here for
    /// convenience, but ensure that a valid opcode can only be constructed
    /// if the four most significant bits of the u8 are 0.
    instruction_class: u8,
    /// The payload constitutes the remaining nibbles of the opcode.
    payload: OpcodePayload,
}

/// Represents the payload of a Chip 8 opcode. That is the opcode without
/// the most significant nibble.
#[derive(Debug)]
pub(super) struct OpcodePayload {
    /// The nibbles representing the payload. Note that we use the u8 type
    /// here for convenience, but ensure that valid payloads can only be
    /// constructed if the four most significant bits of the u8 are 0.
    bytes: [u8; 3],
}

impl OpcodePayload {
    /// Interprets the opcode payload as an address in the range 0x000 to
    /// 0xFFF (inclusive) and returns an u16 containing this address.
    fn address(&self) -> u16 {
        (self.bytes[0] as u16) << 8 | (self.bytes[1] as u16) << 4 | self.bytes[2] as u16
    }

    /// Interprets the most significant nibble of the opcode as a register
    /// address in range 0x0 - 0xF (inclusive) and the remaining nibbles
    /// as a value in range 0x00 - 0xFF (inclusive) and returns a tuple
    /// representing these values.
    fn reg_and_value(&self) -> (u8, u8) {
        (self.bytes[0], (self.bytes[1] << 4) | self.bytes[2])
    }

    /// Interprets the opcode payload as three operands, each of size
    /// one nibble, i.e., in range 0x0 - 0xF (inclusive) and returns
    /// a triple representing these values.
    fn operands(&self) -> (u8, u8, u8) {
        (self.bytes[0], self.bytes[1], self.bytes[2])
    }
}

impl Opcode {
    /// Constructs a new `Opcode` given its byte representation.
    pub(super) fn new(opcode: &[u8; 2]) -> Opcode {
        Opcode {
            instruction_class: opcode[0] >> 4,
            payload: OpcodePayload {
                bytes: [opcode[0] & 0xF, opcode[1] >> 4, opcode[1] & 0xF],
            },
        }
    }

    pub(super) fn execute(self, mut state: &mut Chip8) {
        let executable_opcode: Box<dyn ExecutableOpcode> = self.into();
        executable_opcode.execute(&mut state);
    }
}

impl From<Opcode> for Box<dyn ExecutableOpcode> {
    fn from(opcode: Opcode) -> Box<dyn ExecutableOpcode> {
        fn into_helper<T>(opcode: Opcode) -> Box<T>
        where
            T: ExecutableOpcode + TryFrom<Opcode>,
            <T as TryFrom<Opcode>>::Error: std::fmt::Debug,
        {
            // We can safely unwrap the converted instructions below as we know
            // that the instruction class will map the respective instruction.
            Box::new(T::try_from(opcode).unwrap())
        }

        match opcode.instruction_class {
            0x0 => into_helper::<SysInstruction>(opcode),
            0x1 => into_helper::<JmpInstruction>(opcode),
            0x2 => into_helper::<CallInstruction>(opcode),
            0x3 => into_helper::<SeInstruction>(opcode),
            0x4 => into_helper::<SneInstruction>(opcode),
            0x5 => into_helper::<SreInstruction>(opcode),
            0x6 => into_helper::<LdrInstruction>(opcode),
            0x7 => into_helper::<AddInstruction>(opcode),
            0x8 => into_helper::<RegInstruction>(opcode),
            0x9 => into_helper::<SrneInstruction>(opcode),
            0xA => into_helper::<LdInstruction>(opcode),
            0xB => into_helper::<JmprInstruction>(opcode),
            0xC => into_helper::<RndInstruction>(opcode),
            0xD => into_helper::<DrwInstruction>(opcode),
            0xE => into_helper::<SkInstruction>(opcode),
            0xF => into_helper::<LduInstruction>(opcode),
            _ => unimplemented!("Unsupported opcode: {}", opcode),
        }
    }
}

/// Captures errors when converting opcodes to their respective instruction object.
#[derive(Debug)]
enum InstructionParsingError {
    /// The given
    InvalidInstructionClass(u8, u8),
}

/// Represents an opcode that can be executed.
trait ExecutableOpcode {
    /// Executes `self` relative to the given state. Note that this
    /// method will in-place modify the given state.
    fn execute(&self, state: &mut Chip8);
}

/// Represents an opcode that expects the payload to be an address.
struct InstructionWithAddress<T> {
    instruction: PhantomData<T>,
    address: u16,
}

/// Represents an opcode that expects the payload to be three operands.
struct InstructionWithOperands<T> {
    instruction: PhantomData<T>,
    op1: u8,
    op2: u8,
    op3: u8,
}

/// Represents an opcode that expects the payload to be a register pointer and a value.
struct InstructionWithRegAndValue<T> {
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
