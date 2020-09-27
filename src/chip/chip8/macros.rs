/// Implements TryFrom<Opcode> for InstructionWithAddress<$name>. The implementation
/// of try_from will return an error if the instruction class of the given opcode
/// does not match the instruction class given in $instruction_class.
macro_rules! implement_try_from_address {
    ($name:ty, $instruction_class:expr) => {
        impl TryFrom<Opcode> for $name {
            type Error = InstructionParsingError;

            fn try_from(opcode: Opcode) -> Result<Self, Self::Error> {
                if opcode.instruction_class != $instruction_class {
                    return Err(InstructionParsingError::InvalidInstructionClass(
                        opcode.instruction_class,
                        $instruction_class,
                    ));
                }
                Ok(Self {
                    instruction: PhantomData,
                    address: opcode.payload.address(),
                })
            }
        }
    };
}

/// Implements TryFrom<Opcode> for InstructionWithRegAndValue<$name>. The implementation
/// of try_from will return an error if the instruction class of the given opcode does
/// not match the instruction class given in $instruction_class.
macro_rules! implement_try_from_reg_and_value {
    ($name:ty, $instruction_class:expr) => {
        impl TryFrom<Opcode> for $name {
            type Error = InstructionParsingError;

            fn try_from(opcode: Opcode) -> Result<Self, Self::Error> {
                if opcode.instruction_class != $instruction_class {
                    return Err(InstructionParsingError::InvalidInstructionClass(
                        opcode.instruction_class,
                        $instruction_class,
                    ));
                }
                let (reg, value) = opcode.payload.reg_and_value();
                Ok(Self {
                    instruction: PhantomData,
                    reg,
                    value,
                })
            }
        }
    };
}

/// Implements TryFrom<Opcode> for InstructionWithOperands<$name>. The implementation
/// of try_from will return an error if the instruction class of the given opcode
/// does not match the instruction class given in $instruction_class.
macro_rules! implement_try_from_operands {
    ($name:ty, $instruction_class:expr) => {
        impl TryFrom<Opcode> for $name {
            type Error = InstructionParsingError;

            fn try_from(opcode: Opcode) -> Result<Self, Self::Error> {
                if opcode.instruction_class != $instruction_class {
                    return Err(InstructionParsingError::InvalidInstructionClass(
                        opcode.instruction_class,
                        $instruction_class,
                    ));
                }
                let (op1, op2, op3) = opcode.payload.operands();
                Ok(Self {
                    instruction: PhantomData,
                    op1,
                    op2,
                    op3,
                })
            }
        }
    };
}
