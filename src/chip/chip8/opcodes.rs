use crate::chip::chip8::Chip8;

use core::convert::TryFrom;

use crate::chip::chip8::{
    AddInstruction, CallInstruction, DrwInstruction, Executable, JmpInstruction, JmprInstruction,
    LdInstruction, LdrInstruction, LduInstruction, RegInstruction, RndInstruction, SeInstruction,
    SkInstruction, SneInstruction, SreInstruction, SrneInstruction, SysInstruction,
};

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
        // We can safely unwrap the converted instructions below as we know
        // that the instruction class will map the respective instruction.
        match self.instruction_class {
            0x0 => SysInstruction::try_from(self).unwrap().execute(&mut state),
            0x1 => JmpInstruction::try_from(self).unwrap().execute(&mut state),
            0x2 => CallInstruction::try_from(self).unwrap().execute(&mut state),
            0x3 => SeInstruction::try_from(self).unwrap().execute(&mut state),
            0x4 => SneInstruction::try_from(self).unwrap().execute(&mut state),
            0x5 => SreInstruction::try_from(self).unwrap().execute(&mut state),
            0x6 => LdrInstruction::try_from(self).unwrap().execute(&mut state),
            0x7 => AddInstruction::try_from(self).unwrap().execute(&mut state),
            0x8 => RegInstruction::try_from(self).unwrap().execute(&mut state),
            0x9 => SrneInstruction::try_from(self).unwrap().execute(&mut state),
            0xA => LdInstruction::try_from(self).unwrap().execute(&mut state),
            0xB => JmprInstruction::try_from(self).unwrap().execute(&mut state),
            0xC => RndInstruction::try_from(self).unwrap().execute(&mut state),
            0xD => DrwInstruction::try_from(self).unwrap().execute(&mut state),
            0xE => SkInstruction::try_from(self).unwrap().execute(&mut state),
            0xF => LduInstruction::try_from(self).unwrap().execute(&mut state),
            _ => unimplemented!("Unsupported opcode"),
        };
    }
}
