#[macro_use]
mod macros;
mod arithmetic_and_logic;
mod program_flow;
mod system;

use core::convert::{Into, TryFrom};
use std::marker::PhantomData;

use crate::{
    Executable, HasOpcode,
    chip8::{
        Chip8,
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
    },
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
        let executable_opcode: Box<dyn Executable<Chip8>> = self.into();
        executable_opcode.execute(&mut state);
    }
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

/// Interprets the opcode payload as an address in the range 0x000 to
/// 0xFFF (inclusive) and returns an u16 containing this address.
type Address = u16;

impl From<OpcodePayload> for Address {
    fn from(opcode_payload: OpcodePayload) -> Address {
        (opcode_payload.bytes[0] as u16) << 8
            | (opcode_payload.bytes[1] as u16) << 4
            | opcode_payload.bytes[2] as u16
    }
}

type RegAndValue = (u8, u8);

/// Interprets the most significant nibble of the opcode as a register
/// address in range 0x0 - 0xF (inclusive) and the remaining nibbles
/// as a value in range 0x00 - 0xFF (inclusive) and returns a tuple
/// representing these values.
impl From<OpcodePayload> for RegAndValue {
    fn from(opcode_payload: OpcodePayload) -> RegAndValue {
        (
            opcode_payload.bytes[0],
            (opcode_payload.bytes[1] << 4) | opcode_payload.bytes[2],
        )
    }
}

type Operands = (u8, u8, u8);

/// Interprets the opcode payload as three operands, each of size
/// one nibble, i.e., in range 0x0 - 0xF (inclusive) and returns
/// a triple representing these values.
impl From<OpcodePayload> for Operands {
    fn from(opcode_payload: OpcodePayload) -> Operands {
        (
            opcode_payload.bytes[0],
            opcode_payload.bytes[1],
            opcode_payload.bytes[2],
        )
    }
}

/// Represents an interpreted, type safe version of an opcode
struct Instruction<T, P> {
    instruction: PhantomData<T>,
    payload: P,
}

impl<T, P> TryFrom<Opcode> for Instruction<T, P>
where
    Self: HasOpcode<Chip8>,
    P: From<OpcodePayload>,
{
    type Error = InstructionParsingError;
    fn try_from(opcode: Opcode) -> Result<Self, Self::Error> {
        if Self::INSTRUCTION_CLASS != opcode.instruction_class {
            return Err(InstructionParsingError::InvalidInstructionClass(
                opcode.instruction_class,
                Self::INSTRUCTION_CLASS,
            ));
        }
        Ok(Self {
            instruction: PhantomData,
            payload: opcode.payload.into(),
        })
    }
}

/// Represents an opcode that expects the payload to be an address.
type InstructionWithAddress<T> = Instruction<T, Address>;

impl<T> InstructionWithAddress<T> {
    fn address(&self) -> u16 {
        self.payload
    }
}

/// Represents an opcode that expects the payload to be three operands.
type InstructionWithOperands<T> = Instruction<T, Operands>;

impl<T> InstructionWithOperands<T> {
    fn op1(&self) -> u8 {
        self.payload.0
    }

    fn op2(&self) -> u8 {
        self.payload.1
    }

    fn op3(&self) -> u8 {
        self.payload.2
    }
}

/// Represents an opcode that expects the payload to be a register pointer and a value.
type InstructionWithRegAndValue<T> = Instruction<T, RegAndValue>;

impl<T> InstructionWithRegAndValue<T> {
    fn reg(&self) -> u8 {
        self.payload.0
    }

    fn value(&self) -> u8 {
        self.payload.1
    }
}

impl From<Opcode> for Box<dyn Executable<Chip8>> {
    fn from(opcode: Opcode) -> Box<dyn Executable<Chip8>> {
        fn into_helper<T>(opcode: Opcode) -> Box<T>
        where
            T: Executable<Chip8> + TryFrom<Opcode>,
            <T as TryFrom<Opcode>>::Error: std::fmt::Debug,
        {
            // We can safely unwrap the converted instructions below as we know
            // that the instruction class will map the respective instruction.
            Box::new(T::try_from(opcode).unwrap())
        }

        match opcode.instruction_class {
            SysInstruction::INSTRUCTION_CLASS => into_helper::<SysInstruction>(opcode),
            JmpInstruction::INSTRUCTION_CLASS => into_helper::<JmpInstruction>(opcode),
            CallInstruction::INSTRUCTION_CLASS => into_helper::<CallInstruction>(opcode),
            SeInstruction::INSTRUCTION_CLASS => into_helper::<SeInstruction>(opcode),
            SneInstruction::INSTRUCTION_CLASS => into_helper::<SneInstruction>(opcode),
            SreInstruction::INSTRUCTION_CLASS => into_helper::<SreInstruction>(opcode),
            LdrInstruction::INSTRUCTION_CLASS => into_helper::<LdrInstruction>(opcode),
            AddInstruction::INSTRUCTION_CLASS => into_helper::<AddInstruction>(opcode),
            RegInstruction::INSTRUCTION_CLASS => into_helper::<RegInstruction>(opcode),
            SrneInstruction::INSTRUCTION_CLASS => into_helper::<SrneInstruction>(opcode),
            LdInstruction::INSTRUCTION_CLASS => into_helper::<LdInstruction>(opcode),
            JmprInstruction::INSTRUCTION_CLASS => into_helper::<JmprInstruction>(opcode),
            RndInstruction::INSTRUCTION_CLASS => into_helper::<RndInstruction>(opcode),
            DrwInstruction::INSTRUCTION_CLASS => into_helper::<DrwInstruction>(opcode),
            SkInstruction::INSTRUCTION_CLASS => into_helper::<SkInstruction>(opcode),
            LduInstruction::INSTRUCTION_CLASS => into_helper::<LduInstruction>(opcode),
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
