use crate::chip::chip8::Chip8;
use crate::chip::Chip;

use rand::{thread_rng, Rng};
use std::convert::TryInto;

fn prepare_state_with_single_instruction(instruction: u16) -> Chip8 {
    let mut chip8 = Chip8::new();
    chip8.memory[0x200] = ((instruction & 0xFF00) >> 8) as u8;
    chip8.memory[0x201] = (instruction & 0xFF) as u8;
    chip8
}

fn do_cycle(instruction: u16, before_cycle: impl Fn(&mut Chip8), after_cycle: impl Fn(&mut Chip8)) {
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

enum Condition {
    Equal,
    NotEqual,
}

fn test_skip_if(
    instruction: u16,
    register: usize,
    cmp_value_eq: u8,
    cmp_value_neq: u8,
    condition: Condition,
) {
    assert!(register < 16);
    let check_equal = |state: &mut Chip8| {
        assert_eq!(
            state.program_counter,
            match condition {
                Condition::Equal => 0x204,
                Condition::NotEqual => 0x202,
            }
        );
    };

    do_cycle(
        instruction,
        |state| {
            state.registers[register] = cmp_value_eq;
            assert_eq!(state.program_counter, 0x200);
        },
        check_equal,
    );

    let check_not_equal = |state: &mut Chip8| {
        assert_eq!(
            state.program_counter,
            match condition {
                Condition::NotEqual => 0x204,
                Condition::Equal => 0x202,
            }
        );
    };

    do_cycle(
        instruction,
        |state| {
            state.registers[register] = cmp_value_neq;
            assert_eq!(state.program_counter, 0x200);
        },
        check_not_equal,
    );
}

#[test]
fn test_skip_if_equal() {
    for cmp in 0x0..0xFF {
        for reg in 0..15 {
            let mut instruction = 0x3000;
            instruction |= cmp;
            instruction |= reg << 8;
            test_skip_if(
                instruction,
                reg.try_into().unwrap(),
                cmp.try_into().unwrap(),
                (cmp.wrapping_add(1)).try_into().unwrap(),
                Condition::Equal,
            );
        }
    }
}

#[test]
fn test_skip_if_not_equal() {
    for cmp in 0x0..0xFF {
        for register in 0x0..0xF {
            let mut instruction = 0x4000;
            instruction |= cmp;
            instruction |= register << 8;
            test_skip_if(
                instruction,
                register.try_into().unwrap(),
                cmp.try_into().unwrap(),
                (cmp.wrapping_add(1)).try_into().unwrap(),
                Condition::NotEqual,
            );
        }
    }
}

#[test]
fn test_skip_if_reg_equal() {
    let mut rng = thread_rng();
    let r1: usize = rng.gen_range(0, 15);
    let mut r2: usize = rng.gen_range(0, 15);
    while r1 == r2 {
        r2 = rng.gen_range(0, 15);
    }

    let instruction: u16 = 0x5000 as u16 | (r1 << 8) as u16 | (r2 << 4) as u16;

    do_cycle(
        instruction,
        |state| {
            state.registers[r1] = 0xCA;
            state.registers[r2] = 0xCA;
            assert_eq!(state.program_counter, 0x200);
        },
        |state| {
            assert_eq!(state.program_counter, 0x204);
        },
    );

    do_cycle(
        instruction,
        |state| {
            state.registers[r1] = 0xCA;
            state.registers[r2] = 0xFE;
            assert_eq!(state.program_counter, 0x200);
        },
        |state| {
            assert_eq!(state.program_counter, 0x202);
        },
    );
}

#[test]
fn test_set_register_to_value() {
    for value in 0x0..0xFF {
        for register in 0x0..0xF {
            let instruction: u16 = 0x6000 | (register << 8) as u16 | value as u16;
            do_cycle(
                instruction,
                |state| {
                    assert_eq!(state.registers[register as usize], 0x0);
                },
                |state| {
                    assert_eq!(state.registers[register as usize], value);
                },
            )
        }
    }
}

#[test]
fn test_add_to_register() {
    for value in 0x0..0xFF {
        for register in 0x0..0xF {
            let instruction: u16 = 0x7000 | (register << 8) as u16 | value as u16;
            do_cycle(
                instruction,
                |state| {
                    assert_eq!(state.registers[register as usize], 0x00);
                    state.registers[register as usize] = 0xAB;
                },
                |state| {
                    assert_eq!(
                        state.registers[register as usize],
                        (value as u8).wrapping_add(0xAB)
                    );
                },
            )
        }
    }
}

#[test]
fn test_set_register_to_register() {
	for r1 in 0x0..0xF {
        for r2 in 0x0..0xF {
            let instruction: u16 = 0x8000 | (r1 << 8) as u16 | (r2 << 4) as u16;
            do_cycle(
                instruction,
                |state| {
                    assert_eq!(state.registers[r1 as usize], 0x00);
                    assert_eq!(state.registers[r2 as usize], 0x00);
                    state.registers[r2 as usize] = 0xAB;
                },
                |state| {
                    assert_eq!(state.registers[r1 as usize], 0xAB);
                    assert_eq!(state.registers[r2 as usize], 0xAB);
                },
            )
        }
    }
}
