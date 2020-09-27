use crate::chip::chip8::Chip8;

pub fn conditional_skip<T>(opcode: &T, state: &mut Chip8, f: fn(&T, &Chip8) -> bool) {
    if f(opcode, state) {
        increment_program_counter(state);
    }
}

pub fn increment_program_counter(state: &mut Chip8) {
    state.program_counter = state.program_counter.wrapping_add(2);
}
