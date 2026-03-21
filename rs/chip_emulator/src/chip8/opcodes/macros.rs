/// Defines a struct `$instruction` and a type alias `$name` for
/// `Instruction<$name, $payload>`. Implements `TryFrom<&Opcode>` for
/// `Instruction<$name, $payload>`. The implementation of `try_from`
/// will return an error if the instruction class of the given opcode
/// does not match the instruction class given in $instruction_class.
macro_rules! define_instruction {
    ($name:ident, $payload:ident, $instruction_class:expr) => {
        paste::paste! {
            pub(super) struct [<$name Phantom>];
            pub(super) type $name = Instruction<[<$name Phantom>], $payload>;
        }
        impl HasOpcode<Chip8> for $name {
            const INSTRUCTION_CLASS: u8 = $instruction_class;
        }
    };
}
