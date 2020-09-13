use rand::{thread_rng, Rng};

const CHIP8_CHARSET_OFFSET : u16 = 0x50; // 80
const CHIP8_CHARSET_LEN : u16 = 0x50; // 80
const CHIP8_CHARSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(Clone)]
pub struct Chip8 {
    opcode: u16,
    memory: [u8; 4096],
    registers: [u8; 16],
    index: u16,
    program_counter: u16,
    gfx: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    stack_pointer: u16,
}

impl Chip8 {
    pub fn new() -> Self {
    	let mut memory = [0; 4096];
    	for i in 0 .. CHIP8_CHARSET_LEN - 1 {
    		memory[(i + CHIP8_CHARSET_OFFSET) as usize] = CHIP8_CHARSET[i as usize];
    	}

        Chip8 {
            opcode: 0,
            memory,
            registers: [0; 16],
            index: 0,
            program_counter: 0x200,
            gfx: [0; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            stack_pointer: 0,
        }
    }

    pub fn next_instruction(&self) -> Opcode {
    	assert!(self.program_counter <= 4094);
    	Opcode::new(&[self.memory[self.program_counter as usize], self.memory[(self.program_counter + 1) as usize]])
    }
}

#[derive(Debug)]
pub struct Opcode {
  bytes: [u8; 4],
}

impl Opcode {
	pub fn new(opcode : &[u8; 2]) -> Opcode {
		Opcode {
		  bytes : [opcode[0] >> 4, opcode[0] & 0xF, opcode[1] >> 4, opcode[1] & 0xF ]
		}
	}
	
	pub fn address(&self) -> u16 {
		(self.bytes[1] as u16) << 8 | (self.bytes[2] as u16) << 4 | self.bytes[3] as u16
	}

	pub fn reg_and_value(&self) -> (u8, u8) {
		(self.bytes[1], (self.bytes[2] << 4) | self.bytes[3])		
	}

	pub fn regs_and_op(&self) -> (u8, u8, u8) {
		(self.bytes[1], self.bytes[2], self.bytes[3])
	}

	pub fn execute(self, state : Chip8) -> Chip8 {
		let mut next_state = state.clone();
		match self.bytes[0] {
			0x0 => {
				let payload = self.address();
				match payload {
					0x0E0 => {
						next_state.gfx = [0; 64 * 32];
						next_state.program_counter += 2;
					},
					0x0EE => {
						assert!(state.stack_pointer > 0, "Stack underflow");
						next_state.program_counter = state.stack[(state.stack_pointer - 1) as usize];
						next_state.stack_pointer = state.stack_pointer - 1;
						next_state.program_counter += 2;
						
					},
					_ => panic!("Opcode not supported {:x}", payload),
				};
			}
			0x1 => {
				next_state.program_counter = self.address();
   			},
			0x2 => {
				assert!(state.stack_pointer < 16, "Stack overflow");
				next_state.stack[next_state.stack_pointer as usize] = state.program_counter;
				next_state.stack_pointer = state.stack_pointer + 1;
				next_state.program_counter = self.address();
			}
			0xA => {
				next_state.index = self.address();
                next_state.program_counter += 2;
			},
			0xB => {
				next_state.program_counter = self.address() + state.registers[0] as u16;
			}
			0xC => {
				let (reg, value) = self.reg_and_value();
				
				let mut rng = thread_rng();
				let sample = rng.gen_range(0, 255);

				println!("{:b}", sample);
				
				next_state.registers[reg as usize] = sample as u8 & value;

				next_state.program_counter += 2;
			}
			0xD => {
				let (regX, regY, height) = self.regs_and_op();

			}
			_ => unimplemented!(),
		};
		next_state
	}
}

pub trait Chip {
    fn cycle(self) -> Self;
    fn get_gfx(&self) -> [u8; 64 * 32];
    fn set_memory_byte(&mut self, byte : u8, index : usize);
}

impl Chip for Chip8 {
    fn cycle(self) -> Self {
    	println!("{:b}", self.registers[0]);
    	let opcode = self.next_instruction();
    	let chip = opcode.execute(self);
    	println!("{:b}", chip.registers[0]);    	

    	chip
    }

    fn get_gfx(&self) -> [u8; 64 * 32] {
        [0; 64 * 32]
    }

    fn set_memory_byte(&mut self, byte : u8, index : usize) {
    	assert!(index < 4096);
    	self.memory[index] = byte;
    }
}
