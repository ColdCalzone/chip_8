use rand::prelude::*;
use std::{fs, num::Wrapping};

pub struct State {
    // 0x000-0x1FF - Chip 8 interpreter (contains font set in emu)
    // 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
    // 0x200-0xFFF - Program ROM and work RAM
    memory: [Wrapping<u8>; 4096],
    // Drawing is done in XOR mode and if a pixel is turned off as a result of drawing,
    // the VF register is set.
    v: [Wrapping<u8>; 16],
    // index and program counter
    i: u16,
    pc: u16,
    opcode: u16,
    // the screen
    gfx: [Wrapping<u8>; 2048],

    delay_timer: u8,
    // The system’s buzzer sounds whenever the sound timer reaches zero.
    sound_timer: u8,
    // return stack
    stack: Vec<u16>,
    // key values get added/subtracted from this
    keys: [u8; 16],
    fontset: [Wrapping<u8>; 80],
    instructions: [fn(&mut Self) -> (); 16],
    arithmetic_instructions: [fn(&mut Self) -> (); 16],
    rng: ThreadRng,
    pub draw_flag: bool,
}

impl State {
    pub fn new() -> Self {
        Self {
            memory: [Wrapping(0); 4096],
            // Drawing is done in XOR mode and if a pixel is turned off as a result of drawing,
            // the VF register is set.
            v: [Wrapping(0); 16],
            i: 0,
            pc: 0,
            opcode: 0,
            gfx: [Wrapping(0); 2048],
            delay_timer: 0,
            sound_timer: 0,
            stack: Vec::with_capacity(16),
            keys: [0; 16],
            fontset: [
                Wrapping(0xF0),
                Wrapping(0x90),
                Wrapping(0x90),
                Wrapping(0x90),
                Wrapping(0xF0), // 0
                Wrapping(0x20),
                Wrapping(0x60),
                Wrapping(0x20),
                Wrapping(0x20),
                Wrapping(0x70), // 1
                Wrapping(0xF0),
                Wrapping(0x10),
                Wrapping(0xF0),
                Wrapping(0x80),
                Wrapping(0xF0), // 2
                Wrapping(0xF0),
                Wrapping(0x10),
                Wrapping(0xF0),
                Wrapping(0x10),
                Wrapping(0xF0), // 3
                Wrapping(0x90),
                Wrapping(0x90),
                Wrapping(0xF0),
                Wrapping(0x10),
                Wrapping(0x10), // 4
                Wrapping(0xF0),
                Wrapping(0x80),
                Wrapping(0xF0),
                Wrapping(0x10),
                Wrapping(0xF0), // 5
                Wrapping(0xF0),
                Wrapping(0x80),
                Wrapping(0xF0),
                Wrapping(0x90),
                Wrapping(0xF0), // 6
                Wrapping(0xF0),
                Wrapping(0x10),
                Wrapping(0x20),
                Wrapping(0x40),
                Wrapping(0x40), // 7
                Wrapping(0xF0),
                Wrapping(0x90),
                Wrapping(0xF0),
                Wrapping(0x90),
                Wrapping(0xF0), // 8
                Wrapping(0xF0),
                Wrapping(0x90),
                Wrapping(0xF0),
                Wrapping(0x10),
                Wrapping(0xF0), // 9
                Wrapping(0xF0),
                Wrapping(0x90),
                Wrapping(0xF0),
                Wrapping(0x90),
                Wrapping(0x90), // A
                Wrapping(0xE0),
                Wrapping(0x90),
                Wrapping(0xE0),
                Wrapping(0x90),
                Wrapping(0xE0), // B
                Wrapping(0xF0),
                Wrapping(0x80),
                Wrapping(0x80),
                Wrapping(0x80),
                Wrapping(0xF0), // C
                Wrapping(0xE0),
                Wrapping(0x90),
                Wrapping(0x90),
                Wrapping(0x90),
                Wrapping(0xE0), // D
                Wrapping(0xF0),
                Wrapping(0x80),
                Wrapping(0xF0),
                Wrapping(0x80),
                Wrapping(0xF0), // E
                Wrapping(0xF0),
                Wrapping(0x80),
                Wrapping(0xF0),
                Wrapping(0x80),
                Wrapping(0x80), // F
            ],
            instructions: [
                State::zero_opcodes,
                State::jump_to_address,
                State::goto_address,
                State::skip_next_if_eq,
                State::skip_next_if_neq,
                State::skip_next_if_xy_eq,
                State::set_vx,
                State::add_vx,
                State::arithmetic_instruction,
                State::skip_next_if_xy_neq,
                State::set_i_to_address,
                State::jump_to_address_plus_v0,
                State::set_vx_random,
                State::draw,
                State::skip_if_key_pressed,
                State::f_opcodes,
            ],
            arithmetic_instructions: [
                State::set_vx_vy,
                State::vx_or_eq_vy,
                State::vx_and_eq_vy,
                State::vx_xor_eq_vy,
                State::vx_add_vy,
                State::vx_sub_vy,
                State::shift_vx_right,
                State::vy_sub_vx,
                State::invalid_instruction,
                State::invalid_instruction,
                State::invalid_instruction,
                State::invalid_instruction,
                State::invalid_instruction,
                State::invalid_instruction,
                State::vx_shift_left,
                State::invalid_instruction,
            ],
            rng: rand::thread_rng(),
            draw_flag: false,
        }
    }

    pub fn initialize(&mut self) {
        // reset everything
        self.pc = 0x200;
        self.i = 0;
        self.gfx.fill(Wrapping(0));
        self.memory.fill(Wrapping(0));
        self.keys.fill(0);
        self.stack = Vec::with_capacity(16);
        self.v.fill(Wrapping(0));

        // load fonts
        for i in 0..80 {
            self.memory[i] = self.fontset[i];
        }
        self.delay_timer = 0;
        self.sound_timer = 0;
    }

    pub fn load_game(&mut self, path: String) -> Result<(), String> {
        // println!("{}", path);
        for (index, value) in fs::read(path)
            .map_err(|op| op.to_string())?
            .iter()
            .enumerate()
        {
            self.memory[index + 0x200] = Wrapping(*value);
        }
        Ok(())
    }

    pub fn load_buffer(&mut self, buffer: &[u8]) {
        for (index, value) in buffer.iter().enumerate() {
            if index + 0x200 >= 4096 {
                eprintln!("Invalid buffer length");
                break;
            }
            self.memory[index + 0x200] = Wrapping(*value);
        }
    }

    pub fn emulate_cycle(&mut self) {
        self.opcode = ((self.memory[self.pc as usize].0 as u16) << 8u8)
            | self.memory[(self.pc + 1) as usize].0 as u16;
        //println!("{:#02X}: {:#02X}", self.pc, self.opcode);
        self.instructions[((self.opcode & 0xF000) >> 12) as usize](self);
        self.pc += 2;

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("BEEP!");
            }
            self.sound_timer -= 1;
        }
    }

    // Return, clear screen, HCF
    // 0x0NNN
    fn zero_opcodes(&mut self) {
        match self.opcode & 0x0FFF {
            0x00EE => {
                self.pc = self
                    .stack
                    .pop()
                    .expect("Error: No subtroutine to return from.");
            }
            0x00E0 => {
                self.gfx.fill(Wrapping(0));
            }
            _ => self.invalid_instruction(),
        }
    }

    // 0x1NNN
    fn jump_to_address(&mut self) {
        self.pc = (self.opcode & 0xFFF) - 2;
    }

    // 0x2NNN
    fn goto_address(&mut self) {
        if self.stack.len() < self.stack.capacity() {
            self.stack.push(self.pc);
        } else {
            panic!("Literal stack overflow!");
        }
        self.pc = self.opcode & 0xFFF;
    }

    // 0x3XNN
    fn skip_next_if_eq(&mut self) {
        if self.v[((self.opcode & 0xF00) >> 8) as usize].0 == (self.opcode & 0xFF) as u8 {
            self.pc += 2;
        }
    }

    // 0x4XNN
    fn skip_next_if_neq(&mut self) {
        if self.v[((self.opcode & 0xF00) >> 8) as usize].0 != (self.opcode & 0xFF) as u8 {
            self.pc += 2;
        }
    }

    // 0x5XY0
    fn skip_next_if_xy_eq(&mut self) {
        if self.v[((self.opcode & 0xF00) >> 8) as usize]
            == self.v[((self.opcode & 0xF0) >> 4) as usize]
        {
            self.pc += 2;
        }
    }

    // 0x6XNN
    fn set_vx(&mut self) {
        self.v[((self.opcode & 0x0F00) >> 8) as usize].0 = (self.opcode & 0x00FF) as u8;
    }

    // 0x7XNN
    fn add_vx(&mut self) {
        self.v[((self.opcode & 0x0F00) >> 8) as usize] += (self.opcode & 0x00FF) as u8;
    }

    // 0x8XY0
    fn set_vx_vy(&mut self) {
        self.v[((self.opcode & 0xF00) >> 8) as usize] =
            self.v[((self.opcode & 0x00F0) >> 4) as usize];
    }

    // 0x8XY1
    fn vx_or_eq_vy(&mut self) {
        self.v[((self.opcode & 0x0F00) >> 8) as usize] |=
            self.v[((self.opcode & 0x00F0) >> 4) as usize];
    }

    // 0x8XY2
    fn vx_and_eq_vy(&mut self) {
        self.v[((self.opcode & 0xF00) >> 8) as usize] &=
            self.v[((self.opcode & 0xF0) >> 4) as usize];
    }

    // 0x8XY3
    fn vx_xor_eq_vy(&mut self) {
        self.v[((self.opcode & 0xF00) >> 8) as usize] ^=
            self.v[((self.opcode & 0xF0) >> 4) as usize];
    }

    // 0x8XY4
    fn vx_add_vy(&mut self) {
        let x = self.v[((self.opcode & 0xF00) >> 8) as usize];
        let y = self.v[((self.opcode & 0xF0) >> 4) as usize];
        if y > Wrapping(0xFF - x.0) {
            self.v[0xF] = Wrapping(1);
        } else {
            self.v[0xF] = Wrapping(0);
        }
        self.v[((self.opcode & 0xF00) >> 8) as usize] += y;
    }

    // 0x8XY5
    fn vx_sub_vy(&mut self) {
        let x = self.v[((self.opcode & 0xF00) >> 8) as usize];
        let y = self.v[((self.opcode & 0xF0) >> 4) as usize];
        if y >= x {
            self.v[0xF] = Wrapping(0);
        } else {
            self.v[0xF] = Wrapping(1);
        }
        self.v[((self.opcode & 0xF00) >> 8) as usize] -= y;
    }

    // 0x8XY6
    fn shift_vx_right(&mut self) {
        self.v[0xF] = Wrapping(self.v[((self.opcode & 0xF00) >> 8) as usize].0 & 0x1);
        self.v[((self.opcode & 0xF00) >> 8) as usize] >>= 1;
    }

    // 0x8XY7
    fn vy_sub_vx(&mut self) {
        let x = self.v[((self.opcode & 0xF00) >> 8) as usize];
        let y = self.v[((self.opcode & 0xF0) >> 4) as usize];
        if x >= y {
            self.v[0xF] = Wrapping(0);
        } else {
            self.v[0xF] = Wrapping(1);
        }
        self.v[((self.opcode & 0xF00) >> 8) as usize] = y - x;
    }

    // 0x8XYE
    fn vx_shift_left(&mut self) {
        self.v[0xF] = Wrapping(self.v[((self.opcode & 0xF00) >> 8) as usize].0 & 0x80);
        self.v[((self.opcode & 0xF00) >> 8) as usize] <<= 1;
    }

    // 0x8NNN
    fn arithmetic_instruction(&mut self) {
        self.arithmetic_instructions[(self.opcode & 0xF) as usize](self);
    }

    // 0x9XY0
    fn skip_next_if_xy_neq(&mut self) {
        if self.v[((self.opcode & 0x0F00) >> 8) as usize]
            != self.v[((self.opcode & 0x00F0) >> 4) as usize]
        {
            self.pc += 2;
        }
    }

    // 0xANNN
    fn set_i_to_address(&mut self) {
        self.i = self.opcode & 0x0FFF;
    }

    // 0xBNNN
    fn jump_to_address_plus_v0(&mut self) {
        self.pc = (self.opcode & 0x0FFF) + self.v[0].0 as u16 - 2;
    }

    // 0xCXNN
    fn set_vx_random(&mut self) {
        self.v[(((self.opcode & 0x0F00) >> 8) as usize)] =
            Wrapping(self.rng.gen::<u8>() & ((self.opcode & 0x00FF) as u8))
    }

    // 0xDXYN
    fn draw(&mut self) {
        // stolen directly from the tutorial
        let x = self.v[((self.opcode & 0x0F00) >> 8) as usize].0 as u16;
        let y = self.v[((self.opcode & 0x00F0) >> 4) as usize].0 as u16;
        let height = (self.opcode & 0x000F) as u16;

        self.v[0xF] = Wrapping(0);
        for yline in 0..height {
            let pixel = self.memory[(self.i + yline as u16) as usize].0;
            for xline in 0..8 {
                if (pixel & (0x80 >> xline)) != 0 {
                    if self.gfx[(x + xline + ((y + yline) * 64)) as usize].0 == 1 {
                        self.v[0xF] = Wrapping(1);
                    }
                    self.gfx[(x + xline + ((y + yline) * 64)) as usize].0 ^= 1;
                }
            }
        }
        self.draw_flag = true;
    }

    // 0xEXxx
    fn skip_if_key_pressed(&mut self) {
        // 0xEX9E
        if self.opcode & 0xFF == 0x9E {
            if self.keys[self.v[(((self.opcode & 0xF00) >> 8) as usize)].0 as usize] != 0 {
                self.pc += 2;
            }
        }
        // 0xEXA1
        else if self.opcode & 0xFF == 0xA1 {
            if self.keys[self.v[(((self.opcode & 0xF00) >> 8) as usize)].0 as usize] == 0 {
                self.pc += 2;
            }
        } else {
            self.invalid_instruction()
        }
    }

    // 0xFXxx
    fn f_opcodes(&mut self) {
        let register = ((self.opcode & 0xF00) >> 8) as usize;
        match self.opcode & 0xFF {
            0x07 => {
                self.v[register] = Wrapping(self.delay_timer);
            }
            0x0A => {
                if let Some(pressed) = self.keys.iter().filter(|x| **x == 1u8).next() {
                    self.v[register] = Wrapping(*pressed);
                } else {
                    self.pc -= 2;
                }
            }
            0x15 => {
                self.delay_timer = self.v[register].0;
            }
            0x18 => {
                self.sound_timer = self.v[register].0;
            }
            0x1E => {
                self.i += self.v[register].0 as u16;
            }
            0x29 => {
                self.i = (self.v[register].0 * 0x5) as u16;
            }
            0x33 => {
                println!("0xFX33 called");
                self.memory[(self.i as usize)] = Wrapping((self.v[register].0 / 100));
                self.memory[(self.i as usize) + 1] = Wrapping((self.v[register].0 % 100) / 10);
                self.memory[(self.i as usize) + 2] = Wrapping(self.v[register].0 % 10);
                println!("first register: {}", register);
                println!("{}: {}, {}, {}", self.v[register].0, self.memory[(self.i as usize)], self.memory[(self.i as usize + 1)], self.memory[(self.i as usize + 2)]);
                println!("v registers 0-3: {}, {}, {}", self.v[0].0, self.v[1], self.v[2]);
            }
            0x55 => {
                for i in 0..(register + 1) {
                    self.memory[self.i as usize + i] = self.v[i];
                    println!("{}", self.memory[self.i as usize + i]);
                }
            }
            0x65 => {
                for (v, i) in ((self.i as usize)..(register + 1)).enumerate() {
                    self.v[v] = self.memory[i];
                }
            }   
            _ => self.invalid_instruction(),
        }
    }

    fn invalid_instruction(&mut self) {
        //println!("Invalid opcode called: {:#02X}", self.opcode);
    }

    pub fn set_key(&mut self, key: usize, value: u8) {
        if key <= 0xF {
            self.keys[key] = value
        }
    }

    pub fn get_graphics_buffer(&mut self) -> Vec<u8> {
        self.gfx.iter_mut().map(|x| x.0).collect()
    }
}
