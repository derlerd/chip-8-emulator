use crate::chip::chip8::Chip8;
use crate::chip::Chip;

fn prepare_state_with_single_instruction(instruction: u16) -> Chip8 {
    let mut chip8 = Chip8::new();
    chip8.memory[0x200] = ((instruction & 0xFF00) >> 8) as u8;
    chip8.memory[0x201] = (instruction & 0xFF) as u8;
    chip8
}

fn do_cycle(instruction: u16, before_cycle: fn(&mut Chip8), after_cycle: fn(&mut Chip8)) {
    let mut before = prepare_state_with_single_instruction(instruction);

    before_cycle(&mut before);
    let mut after = before.cycle();
    after_cycle(&mut after);
}

#[test]
fn test_jump() {
    do_cycle(
        0x1CAF,
        |state| {
            assert_eq!(state.program_counter, 0x200);
        },
        |state| {
            assert_eq!(state.program_counter, 0xCAF);
        },
    )
}

#[test]
fn test_call() {
    do_cycle(
        0x2CAF,
        |state| {
            assert_eq!(state.program_counter, 0x200);
        },
        |state| {
            assert_eq!(state.program_counter, 0xCAF);
            assert_eq!(state.stack[(state.stack_pointer - 1) as usize], 0x202);
        },
    )
}

#[test]
fn test_skip_if_equal() {
    do_cycle(
        0x34AF,
        |state| {
            state.registers[4] = 0xAF;
            assert_eq!(state.program_counter, 0x200);
        },
        |state| {
            assert_eq!(state.program_counter, 0x204);
        },
    );

    do_cycle(
        0x34BF,
        |state| {
            state.registers[4] = 0xAF;
            assert_eq!(state.program_counter, 0x200);
        },
        |state| {
            assert_eq!(state.program_counter, 0x202);
        },
    );
}
