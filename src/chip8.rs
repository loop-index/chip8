pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const MEMORY_SIZE: usize = 4096;
const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;
const BOOT_SECTOR: usize = 512;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
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

pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    registers: [u8; REGISTER_COUNT],
    index: u16,
    pc: u16,
    stack: [u16; STACK_SIZE],
    sp: usize,
    delay_timer: u8,
    sound_timer: u8,
    screen: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
    keypad: [bool; 16],
}

// Public interface
impl Chip8 {
    pub fn new() -> Self {
        let mut new_chip = Self {
            memory: [0; MEMORY_SIZE],
            registers: [0; REGISTER_COUNT],
            index: 0,
            pc: BOOT_SECTOR as u16,
            stack: [0; STACK_SIZE],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            screen: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            keypad: [false; 16],
        };

        // Copy the font set
        new_chip.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        return new_chip;
    }

    pub fn load_rom(&mut self, rom: &[u8]) {
        let start = BOOT_SECTOR;
        let end = start + rom.len();

        self.memory[start..end].copy_from_slice(rom);
    }

    pub fn get_screen_buffer(&self) -> &[u8] {
        return &self.screen;
    }

    pub fn get_keypad(&self) -> &[bool] {
        return &self.keypad;
    }

    pub fn get_sound_timer(&self) -> u8 {
        return self.sound_timer;
    }

    pub fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            // Beep
            self.sound_timer -= 1;
        }
    }

    pub fn clear_keypad(&mut self) {
        self.keypad = [false; 16];
    }

    pub fn set_keypress(&mut self, key: usize) {
        self.keypad[key] = true;
    }

    pub fn cycle(&mut self) {
        let opcode = self.fetch_instruction();

        let mut str_buffer = String::new();
        self.execute_instruction(opcode, &mut str_buffer);

        // println!("{}\r", str_buffer); // Print the instruction for debugging
    }
}

// Private methods
impl Chip8 {
    fn fetch_instruction(&mut self) -> u16 {
        let pc = self.pc as usize;
        let byte1 = self.memory[pc] as u16;
        let byte2 = self.memory[pc + 1] as u16;

        self.pc += 2; // Because one instruction is two bytes
        return (byte1 << 8) | byte2;
    }

    fn execute_instruction(&mut self, opcode: u16, str_buffer: &mut String) {
        let hex1 = (opcode & 0xF000) >> 12;
        let hex2 = (opcode & 0x0F00) >> 8;
        let hex3 = (opcode & 0x00F0) >> 4;
        let hex4 = opcode & 0x000F;

        match (hex1, hex2, hex3, hex4) {
            // 0000 - Nop
            (0, 0, 0, 0) => {
                str_buffer.push_str("NOP");
            }

            // 00E0 - CLS - Clear screen
            (0, 0, 0xE, 0) => {
                self.screen = [0; SCREEN_WIDTH * SCREEN_HEIGHT];
                str_buffer.push_str("CLS");
            },

            // 00EE - RET - Return from subroutine
            (0, 0, 0xE, 0xE) => {
                self.pc = self.pop_stack();
                str_buffer.push_str("RET");
            },

            // 1nnn - JP addr - Jump to address
            (1, _, _, _) => {
                let jump_addr = opcode & 0x0FFF;
                self.pc = jump_addr;

                str_buffer.push_str(&format!("JP {:X}", jump_addr));
            },

            // 2nnn - CALL addr - Call subroutine
            (2, _, _, _) => {
                let call_addr = opcode & 0x0FFF;
                if self.push_stack(self.pc) {
                    self.pc = call_addr;
                }

                str_buffer.push_str(&format!("CALL {:X}", call_addr));
            },

            // 3xkk - SE Vx, byte - Skip next if Vx == byte
            (3, _, _, _) => {
                let vx = hex2 as usize;
                let byte = (opcode & 0x00FF) as u8;

                if self.registers[vx] == byte {
                    self.pc += 2;
                }

                str_buffer.push_str(&format!("SE V{:X}, {:X}", vx, byte));
            },

            // 4xkk - SNE Vx, byte - Skip next if Vx != byte
            (4, _, _, _) => {
                let vx = hex2 as usize;
                let byte = (opcode & 0x00FF) as u8;

                if self.registers[vx] != byte {
                    self.pc += 2;
                }

                str_buffer.push_str(&format!("SNE V{:X}, {:X}", vx, byte));
            },

            // 5xy0 - SE Vx, Vy - Skip next if Vx == Vy
            (5, _, _, 0) => {
                let vx = hex2 as usize;
                let vy = hex3 as usize;

                if self.registers[vx] == self.registers[vy] {
                    self.pc += 2;
                }

                str_buffer.push_str(&format!("SE V{:X}, V{:X}", vx, vy));
            },

            // 6xkk - LD Vx, byte - Set Vx to byte
            (6, _, _, _) => {
                let vx = hex2 as usize;
                let byte = (opcode & 0x00FF) as u8;

                self.registers[vx] = byte;
                str_buffer.push_str(&format!("LD V{:X}, {:X}", vx, byte));
            },

            // 7xkk - ADD Vx, byte
            (7, _, _, _) => {
                let vx = hex2 as usize;
                let byte = (opcode & 0x00FF) as u8;

                // Handles overflow
                self.registers[vx] = self.registers[vx].wrapping_add(byte);

                str_buffer.push_str(&format!("ADD V{:X}, {:X}", vx, byte));
            },

            // 8xy0 - LD Vx, Vy - Set Vx = Vy
            (8, _, _, 0) => {
                let vx = hex2 as usize;
                let vy = hex3 as usize;

                self.registers[vx] = self.registers[vy];
                str_buffer.push_str(&format!("LD V{:X}, V{:X}", vx, vy));
            },

            // 8xy1 - OR Vx, Vy
            (8, _, _, 1) => {
                let vx = hex2 as usize;
                let vy = hex3 as usize;

                self.registers[vx] |= self.registers[vy];
                str_buffer.push_str(&format!("OR V{:X}, V{:X}", vx, vy));
            },

            // 8xy2 - AND Vx, Vy
            (8, _, _, 2) => {
                let vx = hex2 as usize;
                let vy = hex3 as usize;

                self.registers[vx] &= self.registers[vy];
                str_buffer.push_str(&format!("AND V{:X}, V{:X}", vx, vy));
            },

            // 8xy3 - XOR Vx, Vy
            (8, _, _, 3) => {
                let vx = hex2 as usize;
                let vy = hex3 as usize;

                self.registers[vx] ^= self.registers[vy];
                str_buffer.push_str(&format!("XOR V{:X}, V{:X}", vx, vy));
            },

            // 8xy4 - ADD Vx, Vy
            (8, _, _, 4) => {
                let vx = hex2 as usize;
                let vy = hex3 as usize;

                let (sum, carry) = self.registers[vx].overflowing_add(self.registers[vy]);

                self.registers[vx] = sum;
                self.registers[0xF] = if carry {1} else {0};

                str_buffer.push_str(&format!("ADD V{:X}, V{:X}", vx, vy));
            },

            // 8xy5 - SUB Vx, Vy
            (8, _, _, 5) => {
                let vx = hex2 as usize;
                let vy = hex3 as usize;

                let (sub, carry) = self.registers[vx].overflowing_sub(self.registers[vy]);

                self.registers[vx] = sub;
                self.registers[0xF] = if carry {0} else {1};

                str_buffer.push_str(&format!("SUB V{:X}, V{:X}", vx, vy));
            },

            // 8xy6 - SHR Vx, Vy - Shift right
            (8, _, _, 6) => {
                let vx = hex2 as usize;

                self.registers[0xF] = self.registers[vx] & 1;
                self.registers[vx] >>= 1;

                str_buffer.push_str(&format!("SHR V{:X}", vx));
            },

            // 8xy7 - SUBN Vx, Vy - Vx = Vy SUB Vx
            (8, _, _, 7) => {
                let vx = hex2 as usize;
                let vy = hex3 as usize;

                let (sub, carry) = self.registers[vy].overflowing_sub(self.registers[vx]);

                self.registers[vx] = sub;
                self.registers[0xF] = if carry {0} else {1};

                str_buffer.push_str(&format!("SUBN V{:X}, V{:X}", vx, vy));
            },

            // 8xyE - SHL Vx, Vy - Shift left
            (8, _, _, 0xE) => {
                let vx = hex2 as usize;

                self.registers[0xF] = (self.registers[vx] >> 7) & 1;
                self.registers[vx] <<= 1;

                str_buffer.push_str(&format!("SHL V{:X}", vx));
            },

            // 9xy0 - SNE Vx, Vy - Skip next if Vx != Vy
            (9, _, _, 0) => {
                let vx = hex2 as usize;
                let vy = hex3 as usize;

                if self.registers[vx] != self.registers[vy] {
                    self.pc += 2;
                }

                str_buffer.push_str(&format!("SNE V{:X}, V{:X}", vx, vy));
            },

            // Annn - LD I, addr - Set i to nnn
            (0xA, _, _, _) => {
                let addr = opcode & 0x0FFF;
                self.index = addr;

                str_buffer.push_str(&format!("LD I, {:X}", addr));
            },

            // Bnnn - JP V0, addr - Jump to addr offset by V0
            (0xB, _, _, _) => {
                let addr = opcode & 0x0FFF;
                self.pc = addr + self.registers[0] as u16;

                str_buffer.push_str(&format!("JP V0, {:X}", addr));
            },

            // Cxkk - RND Vx, kk - Set Vx to random byte & kk
            (0xC, _, _, _) => {
                let vx = hex2 as usize;
                let byte = (opcode & 0x00FF) as u8;
                let rand = rand::random::<u8>();

                self.registers[vx] = rand & byte;
                str_buffer.push_str(&format!("RND V{:X}, {:X}", vx, byte));
            },

            // Dxyn - DRW Vx, Vy, n - Draw n lines at Vx, Vy from index location
            (0xD, _, _, _) => {
                let x = self.registers[hex2 as usize] as usize;
                let y = self.registers[hex3 as usize] as usize;
                let n = hex4 as usize;

                self.registers[0xF] = 0;

                for line in 0..n {
                    let row = self.memory[self.index as usize + line];

                    for col in 0..8 {
                        // Check if each bit of the row is set
                        if (row & (0x80 >> col)) != 0 {

                            // Find draw location, wrap if overflow
                            let index = (x + col + ((y + line) * SCREEN_WIDTH)) % (SCREEN_WIDTH * SCREEN_HEIGHT);

                            // If any existing pixels are erased, set VF to 1
                            if self.screen[index] == 1 {
                                self.registers[0xF] = 1;
                            }

                            self.screen[index] ^= 1;
                        }
                    }
                }

                str_buffer.push_str(&format!("DRW V{:X}, V{:X}, {:X}", hex2, hex3, hex4));
            },

            // Ex9E - SKP Vx - Skip next if key Vx is pressed
            (0xE, _, 9, 0xE) => {
                let vx = hex2 as usize;

                if self.keypad[self.registers[vx] as usize] {
                    self.pc += 2;
                }

                str_buffer.push_str(&format!("SKP V{:X}", vx));
            },

            // ExA1 - SKNP Vx - Skip next if key Vx is not pressed
            (0xE, _, 0xA, 1) => {
                let vx = hex2 as usize;

                if !self.keypad[self.registers[vx] as usize] {
                    self.pc += 2;
                }

                str_buffer.push_str(&format!("SKNP V{:X}", vx));
            },

            // Fx07 - LD Vx, DT - Set Vx to delay timer
            (0xF, _, 0, 7) => {
                let vx = hex2 as usize;

                self.registers[vx] = self.delay_timer;
                str_buffer.push_str(&format!("LD V{:X}, DT", vx));
            },

            // Fx0A - LD Vx, K - Wait for key press, store in Vx
            (0xF, _, 0, 0xA) => {
                let vx = hex2 as usize;

                let mut key_pressed = false;
                for i in 0..16 {
                    if self.keypad[i] {
                        self.registers[vx] = i as u8;
                        key_pressed = true;
                    }
                }

                // If no key is pressed, decrement PC to repeat instruction
                if !key_pressed {
                    self.pc -= 2;
                }

                str_buffer.push_str(&format!("LD V{:X}, K", vx));
            },

            // Fx15 - LD DT, Vx - Set delay timer to Vx
            (0xF, _, 1, 5) => {
                let vx = hex2 as usize;
                self.delay_timer = self.registers[vx];

                str_buffer.push_str(&format!("LD DT, V{:X}", vx));
            },

            // Fx18 - LD ST, Vx - Set sound timer to Vx
            (0xF, _, 1, 8) => {
                let vx = hex2 as usize;
                self.sound_timer = self.registers[vx];

                str_buffer.push_str(&format!("LD ST, V{:X}", vx));
            },

            // Fx1E - ADD I, Vx - Set I to I + Vx
            (0xF, _, 1, 0xE) => {
                let vx = hex2 as usize;
                self.index += self.registers[vx] as u16;

                str_buffer.push_str(&format!("ADD I, V{:X}", vx));
            },

            // Fx29 - LD F, Vx - Set I to location of sprite for digit Vx
            (0xF, _, 2, 9) => {
                let vx = hex2 as usize;
                self.index = self.registers[vx] as u16 * 5;

                str_buffer.push_str(&format!("LD F, V{:X}", vx));
            },

            // Fx33 - LD B, Vx - Store BCD representation of Vx in memory locations I, I+1, I+2
            (0xF, _, 3, 3) => {
                let vx = hex2 as usize;
                let value = self.registers[vx];

                self.memory[self.index as usize] = value / 100;
                self.memory[self.index as usize + 1] = (value / 10) % 10;
                self.memory[self.index as usize + 2] = (value % 100) % 10;

                str_buffer.push_str(&format!("LD B, V{:X}", vx));
            },

            // Fx55 - LD [I], Vx - Store registers V0 through Vx in memory starting at I
            (0xF, _, 5, 5) => {
                let vx = hex2 as usize;

                for i in 0..=vx {
                    self.memory[self.index as usize + i] = self.registers[i];
                }

                self.index += vx as u16 + 1;
                str_buffer.push_str(&format!("LD [I], V{:X}", vx));
            },

            // Fx65 - LD Vx, [I] - Fill registers V0 through Vx with memory starting at I
            (0xF, _, 6, 5) => {
                let vx = hex2 as usize;

                for i in 0..=vx {
                    self.registers[i] = self.memory[self.index as usize + i];
                }

                self.index += vx as u16 + 1;
                str_buffer.push_str(&format!("LD V{:X}, [I]", vx));
            },

            (_, _, _, _) => {
                // println!("Instruction not implemented!");
                str_buffer.push_str("???");
            }
        }
    }

    fn push_stack(&mut self, value: u16) -> bool {
        if self.sp < STACK_SIZE {
            self.stack[self.sp] = value;
            self.sp += 1;
            return true;
        } else {
            // println!("Stack Overflow!");
            return false;
        }
    }

    fn pop_stack(&mut self) -> u16 {
        if self.sp > 0 {
            self.sp -= 1;
            return self.stack[self.sp];
        } else {
            // println!("Stack empty!");
            return 0;
        }
    }
}

/// Disassembles a Chip-8 program into a human-readable format
/// 
/// ## Arguments
/// 
/// * `program` - The Chip-8 program to disassemble, as a byte array
pub fn disassemble(program: &[u8]) -> String {
    let count = program.len() / 2;
    let mut str_buffer = String::new();
    for i in 0..count {
        let hex1: u8 = program[i * 2] >> 4;
        let hex2 = program[i * 2] & 0x0F;
        let hex3 = program[i * 2 + 1] >> 4;
        let hex4 = program[i * 2 + 1] & 0x0F;
        let opcode = (program[i * 2] as u16) << 8 | program[i * 2 + 1] as u16;
    
        // Currently, the translated instruction is written without commas because it messes with the digit parsing
        match (hex1, hex2, hex3, hex4) {
            // 0000 - Nop
            (0, 0, 0, 0) => {
                str_buffer.push_str("NOP");
            }
    
            // 00E0 - CLS - Clear screen
            (0, 0, 0xE, 0) => {
                str_buffer.push_str("CLS");
            },
    
            // 00EE - RET - Return from subroutine
            (0, 0, 0xE, 0xE) => {
                str_buffer.push_str("RET");
            },
    
            // 1nnn - JP addr - Jump to address
            (1, _, _, _) => {
                let jump_addr = opcode & 0x0FFF;
                str_buffer.push_str(&format!("JP {:X}", jump_addr));
            },
    
            // 2nnn - CALL addr - Call subroutine
            (2, _, _, _) => {
                let call_addr = opcode & 0x0FFF;
                str_buffer.push_str(&format!("CALL {:X}", call_addr));
            },
    
            // 3xkk - SE Vx, byte - Skip next if Vx == byte
            (3, _, _, _) => {
                let vx = hex2 as usize;
                let byte = (opcode & 0x00FF) as u8;
                str_buffer.push_str(&format!("SE V{:X} {:X}", vx, byte));
            },
    
            // 4xkk - SNE Vx, byte - Skip next if Vx != byte
            (4, _, _, _) => {
                let vx = hex2 as usize;
                let byte = (opcode & 0x00FF) as u8;
                str_buffer.push_str(&format!("SNE V{:X} {:X}", vx, byte));
            },
    
            // 5xy0 - SE Vx, Vy - Skip next if Vx == Vy
            (5, _, _, 0) => {
                let vx = hex2 as usize;
                let vy = hex3 as usize;
                str_buffer.push_str(&format!("SE V{:X} V{:X}", vx, vy));
            },
    
            // 6xkk - LD Vx, byte - Set Vx to byte
            (6, _, _, _) => {
                let vx = hex2 as usize;
                let byte = (opcode & 0x00FF) as u8;
                str_buffer.push_str(&format!("LD V{:X} {:X}", vx, byte));
            },
    
            // 7xkk - ADD Vx, byte
            (7, _, _, _) => {
                let vx = hex2 as usize;
                let byte = (opcode & 0x00FF) as u8;
                str_buffer.push_str(&format!("ADD V{:X} {:X}", vx, byte));
            },
    
            // 8xy0 - LD Vx, Vy - Set Vx = Vy
            (8, _, _, 0) => {
                let vx = hex2 as usize;
                let vy = hex3 as usize;
                str_buffer.push_str(&format!("LD V{:X} V{:X}", vx, vy));
            },
    
            // 8xy1 - OR Vx, Vy
            (8, _, _, 1) => {
                let vx = hex2 as usize;
                let vy = hex3 as usize;
                str_buffer.push_str(&format!("OR V{:X} V{:X}", vx, vy));
            },
    
            // 8xy2 - AND Vx, Vy
            (8, _, _, 2) => {
                let vx = hex2 as usize;
                let vy = hex3 as usize;
                str_buffer.push_str(&format!("AND V{:X} V{:X}", vx, vy));
            },
    
            // 8xy3 - XOR Vx, Vy
            (8, _, _, 3) => {
                let vx = hex2 as usize;
                let vy = hex3 as usize;
                str_buffer.push_str(&format!("XOR V{:X} V{:X}", vx, vy));
            },
    
            // 8xy4 - ADD Vx, Vy
            (8, _, _, 4) => {
                let vx = hex2 as usize;
                let vy = hex3 as usize;
                str_buffer.push_str(&format!("ADD V{:X} V{:X}", vx, vy));
            },
    
            // 8xy5 - SUB Vx, Vy
            (8, _, _, 5) => {
                let vx = hex2 as usize;
                let vy = hex3 as usize;
                str_buffer.push_str(&format!("SUB V{:X} V{:X}", vx, vy));
            },
    
            // 8xy6 - SHR Vx, Vy - Shift right
            (8, _, _, 6) => {
                let vx = hex2 as usize;
                str_buffer.push_str(&format!("SHR V{:X}", vx));
            },
    
            // 8xy7 - SUBN Vx, Vy - Vx = Vy SUB Vx
            (8, _, _, 7) => {
                let vx = hex2 as usize;
                let vy = hex3 as usize;
                str_buffer.push_str(&format!("SUBN V{:X} V{:X}", vx, vy));
            },
    
            // 8xyE - SHL Vx, Vy - Shift left
            (8, _, _, 0xE) => {
                let vx = hex2 as usize;
                str_buffer.push_str(&format!("SHL V{:X}", vx));
            },
    
            // 9xy0 - SNE Vx, Vy - Skip next if Vx != Vy
            (9, _, _, 0) => {
                let vx = hex2 as usize;
                let vy = hex3 as usize;
                str_buffer.push_str(&format!("SNE V{:X} V{:X}", vx, vy));
            },
    
            // Annn - LD I, addr - Set i to nnn
            (0xA, _, _, _) => {
                let addr = opcode & 0x0FFF;
                str_buffer.push_str(&format!("LD I {:X}", addr));
            },
    
            // Bnnn - JP V0, addr - Jump to addr offset by V0
            (0xB, _, _, _) => {
                let addr = opcode & 0x0FFF;
                str_buffer.push_str(&format!("JP V0 {:X}", addr));
            },
    
            // Cxkk - RND Vx, kk - Set Vx to random byte & kk
            (0xC, _, _, _) => {
                let vx = hex2 as usize;
                let byte = (opcode & 0x00FF) as u8;
                str_buffer.push_str(&format!("RND V{:X} {:X}", vx, byte));
            },
    
            // Dxyn - DRW Vx, Vy, n - Draw n lines at Vx, Vy from index location
            (0xD, _, _, _) => {
                str_buffer.push_str(&format!("DRW V{:X} V{:X} {:X}", hex2, hex3, hex4));
            },
    
            // Ex9E - SKP Vx - Skip next if key Vx is pressed
            (0xE, _, 9, 0xE) => {
                let vx = hex2 as usize;
                str_buffer.push_str(&format!("SKP V{:X}", vx));
            },
    
            // ExA1 - SKNP Vx - Skip next if key Vx is not pressed
            (0xE, _, 0xA, 1) => {
                let vx = hex2 as usize;
                str_buffer.push_str(&format!("SKNP V{:X}", vx));
            },
    
            // Fx07 - LD Vx, DT - Set Vx to delay timer
            (0xF, _, 0, 7) => {
                let vx = hex2 as usize;
                str_buffer.push_str(&format!("LD V{:X} DT", vx));
            },
    
            // Fx0A - LD Vx, K - Wait for key press, store in Vx
            (0xF, _, 0, 0xA) => {
                let vx = hex2 as usize;
                str_buffer.push_str(&format!("LD V{:X} K", vx));
            },
    
            // Fx15 - LD DT, Vx - Set delay timer to Vx
            (0xF, _, 1, 5) => {
                let vx = hex2 as usize;
                str_buffer.push_str(&format!("LD DT V{:X}", vx));
            },
    
            // Fx18 - LD ST, Vx - Set sound timer to Vx
            (0xF, _, 1, 8) => {
                let vx = hex2 as usize;
                str_buffer.push_str(&format!("LD ST V{:X}", vx));
            },
    
            // Fx1E - ADD I, Vx - Set I to I + Vx
            (0xF, _, 1, 0xE) => {
                let vx = hex2 as usize;
                str_buffer.push_str(&format!("ADD I V{:X}", vx));
            },
    
            // Fx29 - LD F, Vx - Set I to location of sprite for digit Vx
            (0xF, _, 2, 9) => {
                let vx = hex2 as usize;
                str_buffer.push_str(&format!("LD F V{:X}", vx));
            },
    
            // Fx33 - LD B, Vx - Store BCD representation of Vx in memory locations I, I+1, I+2
            (0xF, _, 3, 3) => {
                let vx = hex2 as usize;
                str_buffer.push_str(&format!("LD B V{:X}", vx));
            },
    
            // Fx55 - LD [I], Vx - Store registers V0 through Vx in memory starting at I
            (0xF, _, 5, 5) => {
                let vx = hex2 as usize;
                str_buffer.push_str(&format!("LD [I] V{:X}", vx));
            },
    
            // Fx65 - LD Vx, [I] - Fill registers V0 through Vx with memory starting at I
            (0xF, _, 6, 5) => {
                let vx = hex2 as usize;
                str_buffer.push_str(&format!("LD V{:X} [I]", vx));
            },
    
            (_, _, _, _) => {
                str_buffer.push_str("???");
            }
        }
        str_buffer.push_str("\n");
    }

    return str_buffer;
}

/// Assembles a Chip-8 program into machine code
/// 
/// ## Arguments
/// 
/// * `program` - The Chip-8 program to assemble, as a string read from a file
pub fn assemble(program: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut lines = program.lines();

    while let Some(line) = lines.next() {

        // Currently can only parses instructions without commas, so remove them
        // Stray commas can cause ParseIntError, which is then defaulted to 0xF (because it's a reserved register, so it's more likely to stick out)
        let mut tokens = line.split_whitespace();
        let opcode = tokens.next().unwrap();
        match opcode {
            // 0000 - Nop
            "NOP" => {
                bytes.push(0x00);
                bytes.push(0x00);
            },

            // 00E0 - CLS - Clear screen
            "CLS" => {
                bytes.push(0x00);
                bytes.push(0xE0);
            },

            // 00EE - RET - Return from subroutine
            "RET" => {
                bytes.push(0x00);
                bytes.push(0xEE);
            },

            // Can either be 1nnn - JP addr or Bnnn - JP V0, addr
            "JP" => {
                let next = tokens.next().unwrap();
                if next.starts_with("V") {
                    let addr = tokens.next().unwrap();
                    let addr = u16::from_str_radix(addr, 16).unwrap_or(0xF);
                    bytes.push(0xB0 | ((addr & 0xF00) >> 8) as u8);
                    bytes.push((addr & 0x0FF) as u8);
                } else {
                    let addr = u16::from_str_radix(next, 16).unwrap_or(0xF);
                    bytes.push(0x10 | ((addr & 0xF00) >> 8) as u8);
                    bytes.push((addr & 0x0FF) as u8);
                }
            },

            // 2nnn - CALL addr - Call subroutine
            "CALL" => {
                let addr = tokens.next().unwrap();
                let addr = u16::from_str_radix(addr, 16).unwrap_or(0xF);
                bytes.push(0x20 | ((addr & 0xF00) >> 8) as u8);
                bytes.push((addr & 0x0FF) as u8);
            },

            // Can either be 3xkk - SE Vx, byte or 5xy0 - SE Vx, Vy
            "SE" => {
                let vx = tokens.next().unwrap();
                let vx = u8::from_str_radix(&vx[1..], 16).unwrap_or(0xF);

                let next = tokens.next().unwrap();
                if next.starts_with("V") {
                    let vy = u8::from_str_radix(&next[1..], 16).unwrap_or(0xF);
                    bytes.push(0x50 | vx);
                    bytes.push(vy << 4);
                } else {
                    let byte = u8::from_str_radix(next, 16).unwrap_or(0xF);
                    bytes.push(0x30 | vx);
                    bytes.push(byte);
                }
            },

            // Can either be 4xkk - SNE Vx, byte or 9xy0 - SNE Vx, Vy
            "SNE" => {
                let vx = tokens.next().unwrap();
                let vx = u8::from_str_radix(&vx[1..], 16).unwrap_or(0xF);

                let next = tokens.next().unwrap();
                if next.starts_with("V") {
                    let vy = u8::from_str_radix(&next[1..], 16).unwrap_or(0xF);
                    bytes.push(0x90 | vx);
                    bytes.push(vy << 4);
                } else {
                    let byte = u8::from_str_radix(next, 16).unwrap_or(0xF);
                    bytes.push(0x40 | vx);
                    bytes.push(byte);
                }
            },

            // Bunch of cases:
            // LD Vx, byte - 6xkk
            // LD Vx, Vy - 8xy0
            // LD I, addr - Annn
            // LD Vx, DT - Fx07
            // LD Vx, K - Fx0A
            // LD DT, Vx - Fx15
            // LD ST, Vx - Fx18
            // LD F, Vx - Fx29
            // LD B, Vx - Fx33
            // LD [I], Vx - Fx55
            // LD Vx, [I] - Fx65
            "LD" => {
                let arg1 = tokens.next().unwrap();
                let arg2 = tokens.next().unwrap();

                // LD Vx, [something]
                if arg1.starts_with("V") {
                    let vx = u8::from_str_radix(&arg1[1..], 16).unwrap_or(0xF);

                    // LD Vx, Vy - 8xy0
                    if arg2.starts_with("V") {
                        let vy = u8::from_str_radix(&arg2[1..], 16).unwrap_or(0xF);
                        bytes.push(0x80 | vx);
                        bytes.push(vy << 4);
                    } 
                    // LD Vx, DT - Fx07
                    else if arg2.starts_with("DT") {
                        bytes.push(0xF0 | vx);
                        bytes.push(0x07);
                    } 
                    // LD Vx, K - Fx0A
                    else if arg2.starts_with("K") {
                        bytes.push(0xF0 | vx);
                        bytes.push(0x0A);
                    } 
                    // LD Vx, [I] - Fx65
                    else if arg2.starts_with("[I]") {
                        bytes.push(0xF0 | vx);
                        bytes.push(0x65);
                    } 
                    // LD Vx, byte - 6xkk
                    else {
                        let byte = u8::from_str_radix(arg2, 16).unwrap_or(0xF);
                        bytes.push(0x60 | vx);
                        bytes.push(byte);
                    }
                } 
                // LD I, addr - Annn
                else if arg1.starts_with("I") {
                    let addr = u16::from_str_radix(arg2, 16).unwrap_or(0xF);
                    bytes.push(0xA0 | ((addr & 0xF00) >> 8) as u8);
                    bytes.push((addr & 0x0FF) as u8);
                } 
                // LD DT, Vx - Fx15
                else if arg1.starts_with("DT") {
                    let vx = u8::from_str_radix(&arg2[1..], 16).unwrap_or(0xF);
                    bytes.push(0xF0 | vx);
                    bytes.push(0x15);
                } 
                // LD ST, Vx - Fx18
                else if arg1.starts_with("ST") {
                    let vx = u8::from_str_radix(&arg2[1..], 16).unwrap_or(0xF);
                    bytes.push(0xF0 | vx);
                    bytes.push(0x18);
                } 
                // LD F, Vx - Fx29
                else if arg1.starts_with("F") {
                    let vx = u8::from_str_radix(&arg2[1..], 16).unwrap_or(0xF);
                    bytes.push(0xF0 | vx);
                    bytes.push(0x29);
                } 
                // LD B, Vx - Fx33
                else if arg1.starts_with("B") {
                    let vx = u8::from_str_radix(&arg2[1..], 16).unwrap_or(0xF);
                    bytes.push(0xF0 | vx);
                    bytes.push(0x33);
                } 
                // LD [I], Vx - Fx55
                else if arg1.starts_with("[I]") {
                    let vx = u8::from_str_radix(&arg2[1..], 16).unwrap_or(0xF);
                    bytes.push(0xF0 | vx);
                    bytes.push(0x55);
                }
            },

            // Either ADD Vx, byte - 7xkk or ADD Vx, Vy - 8xy4 or ADD I, Vx - Fx1E
            "ADD" => {
                let arg1 = tokens.next().unwrap();
                let arg2 = tokens.next().unwrap();

                if arg1.starts_with("V") {
                    let vx = u8::from_str_radix(&arg1[1..], 16).unwrap_or(0xF);

                    // ADD Vx, Vy - 8xy4
                    if arg2.starts_with("V") {
                        let vy = u8::from_str_radix(&arg2[1..], 16).unwrap_or(0xF);
                        bytes.push(0x80 | vx);
                        bytes.push(vy << 4 | 0x04);
                    } 
                    // ADD Vx, byte - 7xkk
                    else {
                        let byte = u8::from_str_radix(arg2, 16).unwrap_or(0xF);
                        bytes.push(0x70 | vx);
                        bytes.push(byte);
                    }
                } 
                // ADD I, Vx - Fx1E
                else if arg1.starts_with("I") {
                    let vx = u8::from_str_radix(&arg2[1..], 16).unwrap_or(0xF);
                    bytes.push(0xF0 | vx);
                    bytes.push(0x1E);
                }
            },

            // OR Vx, Vy - 8xy1
            "OR" => {
                let vx = tokens.next().unwrap();
                let vx = u8::from_str_radix(&vx[1..], 16).unwrap_or(0xF);
                let vy = tokens.next().unwrap();
                let vy = u8::from_str_radix(&vy[1..], 16).unwrap_or(0xF);
                bytes.push(0x80 | vx);
                bytes.push(vy << 4 | 0x01);
            },

            // AND Vx, Vy - 8xy2
            "AND" => {
                let vx = tokens.next().unwrap();
                let vx = u8::from_str_radix(&vx[1..], 16).unwrap_or(0xF);
                let vy = tokens.next().unwrap();
                let vy = u8::from_str_radix(&vy[1..], 16).unwrap_or(0xF);
                bytes.push(0x80 | vx);
                bytes.push(vy << 4 | 0x02);
            },

            // XOR Vx, Vy - 8xy3
            "XOR" => {
                let vx = tokens.next().unwrap();
                let vx = u8::from_str_radix(&vx[1..], 16).unwrap_or(0xF);
                let vy = tokens.next().unwrap();
                let vy = u8::from_str_radix(&vy[1..], 16).unwrap_or(0xF);
                bytes.push(0x80 | vx);
                bytes.push(vy << 4 | 0x03);
            },

            // SUB Vx, Vy - 8xy5
            "SUB" => {
                let vx = tokens.next().unwrap();
                let vx = u8::from_str_radix(&vx[1..], 16).unwrap_or(0xF);
                let vy = tokens.next().unwrap();
                let vy = u8::from_str_radix(&vy[1..], 16).unwrap_or(0xF);
                bytes.push(0x80 | vx);
                bytes.push(vy << 4 | 0x05);
            },

            // SHR Vx, Vy - 8xy6
            "SHR" => {
                let vx = tokens.next().unwrap();
                let vx = u8::from_str_radix(&vx[1..], 16).unwrap_or(0xF);

                // The instruction hex takes a Vy but it's not used, so just use V0

                bytes.push(0x80 | vx);
                bytes.push(0x06);
            },

            // SUBN Vx, Vy - 8xy7
            "SUBN" => {
                let vx = tokens.next().unwrap();
                let vx = u8::from_str_radix(&vx[1..], 16).unwrap_or(0xF);
                let vy = tokens.next().unwrap();
                let vy = u8::from_str_radix(&vy[1..], 16).unwrap_or(0xF);
                bytes.push(0x80 | vx);
                bytes.push(vy << 4 | 0x07);
            },

            // SHL Vx, Vy - 8xyE
            "SHL" => {
                let vx = tokens.next().unwrap();
                let vx = u8::from_str_radix(&vx[1..], 16).unwrap_or(0xF);

                // The instruction hex takes a Vy but it's not used, so just use V0

                bytes.push(0x80 | vx);
                bytes.push(0x0E);
            },

            // RND Vx, byte - Cxkk
            "RND" => {
                let vx = tokens.next().unwrap();
                let vx = u8::from_str_radix(&vx[1..], 16).unwrap_or(0xF);
                let byte = u8::from_str_radix(tokens.next().unwrap(), 16).unwrap_or(0xF);
                bytes.push(0xC0 | vx);
                bytes.push(byte);
            },

            // DRW Vx, Vy, n - Dxyn
            "DRW" => {
                let vx = tokens.next().unwrap();
                let vx = u8::from_str_radix(&vx[1..], 16).unwrap_or(0xF);
                let vy = tokens.next().unwrap();
                let vy = u8::from_str_radix(&vy[1..], 16).unwrap_or(0xF);
                let n = u8::from_str_radix(tokens.next().unwrap(), 16).unwrap_or(0xF);
                bytes.push(0xD0 | vx);
                bytes.push(vy << 4 | n);
            },

            // SKP Vx - Ex9E
            "SKP" => {
                let vx = tokens.next().unwrap();
                let vx = u8::from_str_radix(&vx[1..], 16).unwrap_or(0xF);
                bytes.push(0xE0 | vx);
                bytes.push(0x9E);
            },

            // SKNP Vx - ExA1
            "SKNP" => {
                let vx = tokens.next().unwrap();
                let vx = u8::from_str_radix(&vx[1..], 16).unwrap_or(0xF);
                bytes.push(0xE0 | vx);
                bytes.push(0xA1);
            },

            _ => {
                // Do nothing
            },
        }
    }

    return bytes;
}
