use std::{env, fs, mem};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::time::{Duration, Instant};  // Import necessary modules for timer handling
// use std::time::{Duration, Instant};  // Import necessary modules for timer handling

use crossterm::event::{self, Event, KeyCode};

struct Chip8 {
    screen_width: usize,
    screen_height: usize,
    max_memory: usize,
    clock_speed: f64,
    memory: Vec<u8>,
    pc: u16,
    index: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    v: [u8; 16], // Combined V0 to VF registers
    font_offset: usize,
    font: [u8; 80],
    screen: Vec<u8>,
}

impl Chip8 {
    fn new() -> Self {
        let screen_width = 64;
        let screen_height = 32;
        let max_memory = 4 * 1024;
        let clock_speed = 60.0; // in Hz
        let font_offset = 80;
        let font = [
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
            0xF0, 0x80, 0xF0, 0x80, 0x80  // F
        ];
        let screen = vec![0; screen_width * screen_height];

        Chip8 {
            screen_width,
            screen_height,
            max_memory,
            clock_speed,
            memory: vec![0; max_memory],
            pc: 0x200,
            index: 0x0,
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            v: [0; 16], // Single array for all V registers
            font_offset,
            font,
            screen,
        }
    }

    fn read_file_bytes(file_path: &str) -> io::Result<Vec<u8>> {
        // Open the file
        let mut file = File::open(Path::new(file_path))?;
        
        // Create a buffer to hold the file contents
        let mut buffer = Vec::new();
        
        // Read the file contents into the buffer
        file.read_to_end(&mut buffer)?;
        
        Ok(buffer)
    }

    fn fetch(&self) -> u16 {
        // get the 2 bytes from the memory based on the pc offset
        let opcode = (self.memory[self.pc as usize] as u16) << 8 | self.memory[(self.pc + 1) as usize] as u16;
        opcode
    }

    fn decode(&mut self, opcode: u16) {
        // bit masks to get the instruction and operand
        let instruction = (opcode & 0xF000) >> 12;
        let operand = opcode & 0x0FFF;
        println!("Instruction: {:x}, Operand: {:03x}", instruction, operand);
        match instruction {
            0x0 => self.execute(instruction as u8, operand),
            0x1 => self.execute(instruction as u8, operand),
            0x2 => self.execute(instruction as u8, operand),
            0x3 => self.execute(instruction as u8, operand),
            0x4 => self.execute(instruction as u8, operand),
            0x5 => self.execute(instruction as u8, operand),
            0x6 => self.execute(instruction as u8, operand),
            0x7 => self.execute(instruction as u8, operand),
            0x8 => self.execute(instruction as u8, operand),
            0x9 => self.execute(instruction as u8, operand),
            0xA => self.execute(instruction as u8, operand),
            0xB => self.execute(instruction as u8, operand),
            0xC => self.execute(instruction as u8, operand),
            0xD => self.execute(instruction as u8, operand),
            _ => println!("Unknown opcode: {:x}", opcode),
        }
    }

    fn execute(&mut self, instruction: u8, operand: u16) {
        match instruction {
            0x0 => match operand {
                0x00E0 => {
                    for pixel in self.screen.iter_mut() {
                        *pixel = 0;
                    }
                }
                0x00EE => {
                    self.pc = self.stack.pop().unwrap();
                }
                _ => println!("Invalid"),
            },
            0x1 => self.jump_to(operand),
            0x2 => {
                self.stack.push(self.pc);
                self.jump_to(operand);
            },
            0x3 => {
                let x = (operand >> 8) as usize;
                let value = operand as u8;
                if self.v[x] == value {
                    self.pc += 2;
                }
            },
            0x4 => {
                let x = (operand >> 8) as usize;
                let value = operand as u8;
                if self.v[x] != value {
                    self.pc += 2;
                }
            },
            0x5 => {
                let x_reg = (operand >> 8) as usize;
                let y_reg = ((operand & 0x00F0) >> 4) as usize;
                if self.v[x_reg] == self.v[y_reg] {
                    self.pc += 2;
                }
            },
            0x6 => self.set_vx(operand),
            0x7 => self.add_vx(operand),
            //logical and arithmetic operations
            0x8 => self.set_arithmetic(operand),
            0x9 => {
                let x_reg = (operand >> 8) as usize;
                let y_reg = ((operand & 0x00F0) >> 4) as usize;
                if self.v[x_reg] != self.v[y_reg] {
                    self.pc += 2;
                }
            },
            0xA => self.set_index(operand),
            0xB => {
                //jump to NNN + VX
                let x = (operand >> 8) as usize;
                self.jump_to(operand + self.v[x] as u16);
            },
            0xC => {
                let x = (operand >> 8) as usize;
                let value = operand as u8;
                self.v[x] = value & rand::random::<u8>();
            },
            0xD => {
                //x and y are the registers to get the position of the sprite from
                let x_reg = (operand >> 8) as usize; //v[x]
                let y_reg = ((operand & 0x00F0) >> 4) as usize; //v[y]
                let height = (operand & 0x000F) as u8;
                self.draw_sprite(self.v[x_reg], self.v[y_reg], height);
            },
            0xE => self.skip_if_key(operand),
            0xF => {
                match operand {
                    0x1E => self.index += self.v[(operand >> 8) as usize] as u16,
                    0x0A => self.v[(operand >> 8) as usize] = 0, //TODO: get key input
                    0x29 => self.index = self.v[(operand >> 8) as usize] as u16 * 5,
                    0x33 => {
                        let x = (operand >> 8) as usize;
                        let value = self.v[x];
                        self.memory[self.index as usize] = value / 100;
                        self.memory[(self.index + 1) as usize] = (value / 10) % 10;
                        self.memory[(self.index + 2) as usize] = value % 10;
                    },
                    0x55 => {
                        let x = (operand >> 8) as usize;
                        for i in 0..=x {
                            self.memory[(self.index + i as u16) as usize] = self.v[i];
                        }
                    },
                    0x65 => {
                        let x = (operand >> 8) as usize;
                        for i in 0..=x {
                            self.v[i] = self.memory[(self.index + i as u16) as usize];
                        }
                    },
                    _ => self.set_timer(operand),
                }},
            _ => println!("Unknown instruction: {:x}", instruction),
        }
    }

    fn skip_if_key(&mut self, operand: u16) {
        let x = (operand >> 8) as usize; // Vx register index
        let vx_value = self.v[x]; // Value of Vx register, representing the CHIP-8 key

        // Check if a key is pressed
        if let Ok(true) = event::poll(std::time::Duration::from_millis(10)) {
            if let Event::Key(key_event) = event::read().unwrap() {
                // Map the key event to a CHIP-8 key value
                let chip8_key = match key_event.code {
                    KeyCode::Char('1') => 0x1,
                    KeyCode::Char('2') => 0x2,
                    KeyCode::Char('3') => 0x3,
                    KeyCode::Char('4') => 0xC,
                    KeyCode::Char('q') => 0x4,
                    KeyCode::Char('w') => 0x5,
                    KeyCode::Char('e') => 0x6,
                    KeyCode::Char('r') => 0xD,
                    KeyCode::Char('a') => 0x7,
                    KeyCode::Char('s') => 0x8,
                    KeyCode::Char('d') => 0x9,
                    KeyCode::Char('f') => 0xE,
                    KeyCode::Char('z') => 0xA,
                    KeyCode::Char('x') => 0x0,
                    KeyCode::Char('c') => 0xB,
                    KeyCode::Char('v') => 0xF,
                    _ => 0xFF, // Invalid key
                };

                // Check if the key pressed matches the CHIP-8 key
                if chip8_key == vx_value {
                    // Skip the next instruction if EX9E
                    if operand & 0x00FF == 0x9E {
                        self.pc += 2;
                    }
                    // Do not skip if EXA1
                    else if operand & 0x00FF == 0xA1 {
                        // Do nothing
                    }
                } else {
                    // Skip the next instruction if EXA1
                    if operand & 0x00FF == 0xA1 {
                        self.pc += 2;
                    }
                }
            }
        }
    }

    fn jump_to(&mut self, address: u16) {
        self.pc = address;
    }

    fn set_vx(&mut self, operand: u16) {
        let x = (operand >> 8) as usize;
        let value = operand as u8;
        self.v[x] = value;
    }

    fn add_vx(&mut self, operand: u16) {
        let x = (operand >> 8) as usize;
        let value = operand as u8;
        self.v[x] = self.v[x].wrapping_add(value); // Use wrapping_add to avoid overflow
    }

    fn set_index(&mut self, operand: u16) {
        self.index = operand;
    }

    fn draw_sprite(&mut self, vx: u8, vy: u8, height: u8) {
        // Set the X and Y coordinates to the values in VX and VY, respectively, modulo screen width/height
        let mut x = (vx as usize) % self.screen_width;
        let mut y = (vy as usize) % self.screen_height;
        
        // Set VF to 0 initially; will be set to 1 if any pixel is erased
        self.v[0xF] = 0;
    
        // Loop over each row of the sprite
        for row in 0..height {
            // Stop drawing if the Y coordinate is out of bounds
            if y >= self.screen_height {
                break;
            }
    
            // Get the Nth byte of sprite data from memory starting at the index register (I)
            let sprite_byte = self.memory[(self.index + row as u16) as usize];
    
            // Loop over each of the 8 pixels/bits in this sprite row (from most to least significant bit)
            for bit in 0..8 {
                // Stop drawing if the X coordinate is out of bounds
                if x >= self.screen_width {
                    break;
                }
    
                // Determine if the current bit in the sprite byte is set (1) or not (0)
                let sprite_pixel = (sprite_byte >> (7 - bit)) & 1;
                let screen_index = y * self.screen_width + x;
    
                if sprite_pixel == 1 {
                    // If the current pixel on the screen is also on, turn it off and set VF to 1
                    if self.screen[screen_index] == 1 {
                        self.screen[screen_index] ^= 1; // XOR operation to toggle pixel
                        self.v[0xF] = 1; // Set VF to 1 because there was a collision
                    } else {
                        // Otherwise, draw the pixel on the screen
                        self.screen[screen_index] = 1;
                    }
                }
    
                x += 1; // Increment X
            }
    
            // Reset X coordinate for the next row and increment Y coordinate
            x = (vx as usize) % self.screen_width;
            y += 1;
        }
    }
    

    fn draw_screen(&self) {
        // Draw the screen on the terminal
        for (i, &pixel) in self.screen.iter().enumerate() {
            if i % 64 == 0 {
                println!();
            }
            if pixel == 0 {
                print!("░");
            } else {
                print!("▓");
            }
        }
    }
    
    fn set_arithmetic(&mut self, operand: u16) {
        let x = (operand >> 8) as usize; //v[x]
        let y = ((operand & 0x00F0) >> 4) as usize; //v[y]
        let bitwise_op = (operand & 0x000F) as u8;
        match bitwise_op{
            0 => self.v[x] = self.v[y],
            1 => self.v[x] = self.v[x] | self.v[y],
            2 => self.v[x] = self.v[x] & self.v[y],
            3 => self.v[x] = self.v[x] ^ self.v[y],
            4 => {
                let (result, overflow) = self.v[x].overflowing_add(self.v[y]);
                self.v[x] = result;
                self.v[0xF] = overflow as u8;
            },
            5 => {
                let (result, overflow) = self.v[x].overflowing_sub(self.v[y]);
                self.v[x] = result;
                self.v[0xF] = !overflow as u8;
            },
            7 => {
                let (result, overflow) = self.v[y].overflowing_sub(self.v[x]);
                self.v[x] = result;
                self.v[0xF] = !overflow as u8;
            },
            //shift vx right by 1
            6 => {
                self.v[0xF] = self.v[x] & 0x1;
                self.v[x] >>= 1;
            },
            0xE => {
                self.v[0xF] = (self.v[x] & 0x80) >> 7;
                self.v[x] <<= 1;
            },
            
            _ => println!("Invalid")
        
        }
    }
    
    fn set_timer(&mut self, operand: u16) {
        let x = (operand >> 8) as usize;
        match operand & 0x00FF {
            //just implement timers
            0x07 => self.v[x] = self.delay_timer,
            0x15 => self.delay_timer = self.v[x],
            0x18 => self.sound_timer = self.v[x],
            _ => println!("Invalid")
    }   }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut chip8 = Chip8::new();

    let mut debug = false;

    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        return;
    }

    // Load font into memory
    for (i, &byte) in chip8.font.iter().enumerate() {
        chip8.memory[i + chip8.font_offset] = byte;
    }

    // Store file bytes in memory
    let file_path = &args[1];
    let bytes = Chip8::read_file_bytes(file_path).unwrap();
    for (i, &byte) in bytes.iter().enumerate() {
        chip8.memory[i + 0x200] = byte;
    }

    // Timer variables
    let timer_interval = Duration::from_secs_f64(1.0 / 60.0); // 60Hz interval
    let mut last_timer_update = Instant::now();

    loop {
        // Clear console
        print!("\x1B[2J\x1B[1;1H");

        // Draw memory
        draw_debug(&chip8);

        // Fetch, decode, and execute instructions
        let opcode = chip8.fetch();
        chip8.decode(opcode);
        chip8.pc += 2;

        // Draw screen
        chip8.draw_screen();

        // Update timers if 60Hz time has passed
        if last_timer_update.elapsed() >= timer_interval {
            if chip8.delay_timer > 0 {
                chip8.delay_timer -= 1;
            }
            if chip8.sound_timer > 0 {
                chip8.sound_timer -= 1;
                // Play a beep sound here if you want to implement sound
            }
            last_timer_update = Instant::now();  // Reset the timer
        }

        if(debug)
        {
            //pause the program
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();


        }

        std::thread::sleep(std::time::Duration::from_secs_f64(1.0 / chip8.clock_speed));

        if chip8.pc >= 0x200 + bytes.len() as u16 {
            chip8.pc = 0x200;
        }
    }
}

fn draw_debug(chip8: &Chip8) {
    println!("\nMemory:");
    for (i, &byte) in chip8.memory.iter().enumerate() {
        if i % (0x200 / 2) == 0 {
            println!();
            print!("0x{:03x}: ", i);
        }
        if i == chip8.pc as usize {
            print!("\x1B[31m PC >{:02x} \x1B[0m", byte);  // Highlight current PC
        } else {
            print!("{:02x} ", byte);
        }
    }

    println!(); // Newline after memory print

    //draw registers
    println!("\nRegisters:");
    for (i, &reg) in chip8.v.iter().enumerate() {
        if i % 4 == 0 {
            println!();
        }
        print!("V{:x}: {:02x} ", i, reg);
    }

    println!("\n\nIndex: {:03x}", chip8.index);
}
