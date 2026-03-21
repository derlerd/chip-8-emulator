/// Defines a struct `$name||Phantom` and a type alias `$name` for
/// `Instruction<$name||Phantom, $payload>`. Implements `HasOpcode<Chip8>`
/// for `$name`.
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
