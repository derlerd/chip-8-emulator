/// Defines a struct `$instruction` and a type alias `$name` for
/// `InstructionWithAddress<$name>`. Implements `TryFrom<&Opcode>` for
/// `InstructionWithAddress<$name>`. The implementation of `try_from`
/// will return an error if the instruction class of the given opcode
/// does not match the instruction class given in $instruction_class.
macro_rules! define_instruction_with_address {
    ($instruction:ident, $name:ident, $instruction_class:expr) => {
        pub(super) struct $instruction;
        pub(super) type $name = InstructionWithAddress<$instruction>;
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

/// Defines a struct `$instruction` and a type alias `$name` for
/// `InstructionWithRegAndValue<$name>`. Implements `TryFrom<&Opcode>` for
/// `InstructionWithRegAndValue<$name>`. The implementation of `try_from`
/// will return an error if the instruction class of the given opcode
/// does not match the instruction class given in $instruction_class.
macro_rules! define_instruction_with_reg_and_value {
    ($instruction:ident, $name:ident, $instruction_class:expr) => {
        pub(super) struct $instruction;
        pub(super) type $name = InstructionWithRegAndValue<$instruction>;
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

/// Defines a struct `$instruction` and a type alias `$name` for
/// `InstructionWithOperands<$name>`. Implements `TryFrom<&Opcode>` for
/// `InstructionWithOperands<$name>`. The implementation of `try_from`
/// will return an error if the instruction class of the given opcode
/// does not match the instruction class given in $instruction_class.
macro_rules! define_instruction_with_operands {
    ($instruction:ident, $name:ident, $instruction_class:expr) => {
        pub(super) struct $instruction;
        pub(super) type $name = InstructionWithOperands<$instruction>;
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
