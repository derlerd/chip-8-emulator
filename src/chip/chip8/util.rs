use crate::chip::chip8::Chip8;

/// Convenience function to do a conditional skip in case `f(opcode, state)` evaluates to
/// `true`.
pub(crate) fn conditional_skip<T>(opcode: &T, state: &mut Chip8, f: fn(&T, &Chip8) -> bool) {
    if f(opcode, state) {
        increment_program_counter(state);
    }
}

/// Convenience function to increment the program counter.
///
/// # Panics
/// In case incrementing the program counter results in a state with an invalid program
/// counter (i.e., one greater than or equal 4096).
pub(crate) fn increment_program_counter(state: &mut Chip8) {
    state.program_counter = state.program_counter.wrapping_add(2);
    assert!(state.program_counter < 4096);
}
