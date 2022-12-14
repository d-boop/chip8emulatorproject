extern crate rand;

use super::display::Display;
use super::keypad::Keypad;

use std::fs::File;
use std::time::Instant;

use rand::rngs::ThreadRng;
use rand::Rng;
use sdl::event::Key;

const RAMSIZE: usize = 4096;
const STACKSIZE: usize = 16;

pub struct Cpu {
    opcode: u16,
    ram: [u8; RAMSIZE],
    v: [u8; 16],
    i: usize,
    pc: usize,
    stack: [u16; STACKSIZE],
    sp: usize,
    delay_timer: u8,
    sound_timer: u8,
    keypad: Keypad,
    display: Display,
    time_elps: Instant,
    rng: ThreadRng,
}

impl Cpu {
    pub fn new() -> Cpu {
        let mut cpu = Cpu {
            opcode: 0,
            ram: [0; RAMSIZE],
            v: [0; 16],
            i: 0x200,
            pc: 0x200,
            stack: [0; STACKSIZE],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: Keypad::new(),
            display: Display::new(),
            time_elps: Instant::now(),
            rng: rand::thread_rng(),
        };

        for i in 0..80 {
            cpu.ram[i] = FONTSET[i];
        }
        cpu
    }

    pub fn tick(&mut self) {
        while (std::time::Instant::now() - self.time_elps).as_secs_f64() < 1.0 / 60.0 {}
        self.time_elps = std::time::Instant::now();

        self.fetch();
        self.execute();

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {}
            self.sound_timer -= 1;
        }
    }

    pub fn load(&mut self, game: String) {
        use std::path::Path;

        let path = Path::new(&game);
        assert!(path.is_file());
        println!("{}", path.display());
        let mut reader = File::open(path).unwrap();
        self.load_to_memory(&mut reader);
    }

    pub fn press(&mut self, key: Key, state: bool) {
        self.keypad.press(key, state);
    }

    pub fn draw(&mut self) {
        self.display.draw_screen();
    }
}

impl Cpu {
    fn load_to_memory(&mut self, reader: &mut File) {
        use std::io::Read;
        for value in reader.bytes() {
            match value {
                Ok(value) => {
                    self.ram[self.pc] = value;
                    self.pc += 1;
                }
                Err(_) => {}
            }
        }
        self.pc = 0x200;
    }

    fn fetch(&mut self) {
        self.opcode = (self.ram[self.pc] as u16) << 8 | (self.ram[self.pc + 1] as u16);
    }

    fn execute(&mut self) {
        match self.opcode & 0xf000 {
            0x0000 => self.op_0xxx(),
            0x1000 => self.op_1xxx(),
            0x2000 => self.op_2xxx(),
            0x3000 => self.op_3xxx(),
            0x4000 => self.op_4xxx(),
            0x5000 => self.op_5xxx(),
            0x6000 => self.op_6xxx(),
            0x7000 => self.op_7xxx(),
            0x8000 => self.op_8xxx(),
            0x9000 => self.op_9xxx(),
            0xA000 => self.op_axxx(),
            0xB000 => self.op_bxxx(),
            0xC000 => self.op_cxxx(),
            0xD000 => self.op_dxxx(),
            0xE000 => self.op_exxx(),
            0xF000 => self.op_fxxx(),
            _ => not_implemented(self.opcode as usize, self.pc),
        }
    }

    fn op_0xxx(&mut self) {
        match self.opcode & 0x000F {
            0x0000 => self.display.clear(),
            0x000E => {
                self.sp -= 1;
                self.pc = self.stack[self.sp] as usize;
            }
            _ => not_implemented(self.opcode as usize, self.pc),
        }
        self.pc += 2;
    }

    // Jumps to address
    fn op_1xxx(&mut self) {
        self.pc = self.op_nnn() as usize;
    }

    // Calls subroutine
    fn op_2xxx(&mut self) {
        self.stack[self.sp] = self.pc as u16;
        self.sp += 1;
        self.pc = self.op_nnn() as usize;
    }

    // Skips the next instruction if VX equals NN
    fn op_3xxx(&mut self) {
        self.pc += if self.v[self.op_x()] == self.op_nn() {
            4
        } else {
            2
        }
    }

    // Skips the next instruction if VX doesn't equal NN
    fn op_4xxx(&mut self) {
        self.pc += if self.v[self.op_x()] != self.op_nn() {
            4
        } else {
            2
        }
    }

    // Skips the next instruction if VX equals VY
    fn op_5xxx(&mut self) {
        self.pc += if self.v[self.op_x()] == self.v[self.op_y()] {
            4
        } else {
            2
        }
    }

    // Sets VX to NN
    fn op_6xxx(&mut self) {
        self.v[self.op_x()] = self.op_nn();
        self.pc += 2;
    }

    // Adds NN to VX
    fn op_7xxx(&mut self) {
        self.v[self.op_x()] += self.op_nn();
        self.pc += 2;
    }

    fn op_8xxx(&mut self) {
        match self.opcode & 0x000F {
            0 => {
                self.v[self.op_x()] = self.v[self.op_y()];
            }
            1 => {
                self.v[self.op_x()] |= self.v[self.op_y()];
            }
            2 => {
                self.v[self.op_x()] &= self.v[self.op_y()];
            }
            3 => {
                self.v[self.op_x()] ^= self.v[self.op_y()];
            }
            4 => {
                self.v[self.op_x()] += self.v[self.op_y()];
                self.v[15] = if self.v[self.op_x()] < self.v[self.op_y()] {
                    1
                } else {
                    0
                };
            }
            5 => {
                self.v[15] = if self.v[self.op_y()] > self.v[self.op_x()] {
                    0
                } else {
                    1
                };
                self.v[self.op_x()] -= self.v[self.op_y()];
            }
            6 => {
                self.v[15] = self.v[self.op_x()] & 0x1;
                self.v[self.op_x()] >>= 1;
            }
            7 => {
                self.v[15] = if self.v[self.op_x()] > self.v[self.op_y()] {
                    0
                } else {
                    1
                };
                self.v[self.op_x()] = self.v[self.op_y()] - self.v[self.op_x()];
            }
            0xE => {
                self.v[15] = self.v[self.op_x()] >> 7;
                self.v[self.op_x()] <<= 1;
            }
            _ => not_implemented(self.opcode as usize, self.pc),
        }
        self.pc += 2;
    }

    fn op_9xxx(&mut self) {
        self.pc += if self.v[self.op_x()] != self.v[self.op_y()] {
            4
        } else {
            2
        }
    }

    fn op_axxx(&mut self) {
        self.i = self.op_nnn() as usize;
        self.pc += 2;
    }

    fn op_bxxx(&mut self) {
        self.pc = (self.op_nnn() + (self.v[0] as u16)) as usize;
    }

    fn op_cxxx(&mut self) {
        self.v[self.op_x()] = self.op_nn() & self.rng.gen::<u8>();
        self.pc += 2;
    }

    fn op_dxxx(&mut self) {
        let from = self.i;
        let to = from + (self.op_n() as usize);
        let x = self.v[self.op_x()];
        let y = self.v[self.op_y()];
        self.v[15] = self
            .display
            .draw(x as usize, y as usize, &self.ram[from..to]);
        self.pc += 2;
    }

    fn op_exxx(&mut self) {
        let v = self.v[self.op_x()] as usize;
        self.pc += match self.opcode & 0x00FF {
            0x9E => {
                if self.keypad.pressed(v) {
                    4
                } else {
                    2
                }
            }
            0xA1 => {
                if !self.keypad.pressed(v) {
                    4
                } else {
                    2
                }
            }
            _ => 2,
        }
    }

    fn op_fxxx(&mut self) {
        match self.opcode & 0x00FF {
            0x07 => {
                self.v[self.op_x()] = self.delay_timer;
            }
            0x0A => {
                self.wait_keypress();
            }
            0x15 => {
                self.delay_timer = self.v[self.op_x()];
            }
            0x18 => {
                self.sound_timer = self.v[self.op_x()];
            }
            0x1E => {
                self.i += self.v[self.op_x()] as usize;
            }
            0x29 => {
                self.i = (self.v[self.op_x()] as usize) * 5;
            }
            0x33 => {
                self.ram[self.i] = self.v[self.op_x()] / 100;
                self.ram[self.i + 1] = (self.v[self.op_x()] / 10) % 10;
                self.ram[self.i + 2] = (self.v[self.op_x()] % 100) % 10;
            }
            0x55 => {
                for i in 0..=self.op_x() {
                    self.ram[self.i + i] = self.v[i]
                }
                self.i += self.op_x() + 1;
            }
            0x65 => {
                for i in 0..=self.op_x() {
                    self.v[i] = self.ram[self.i + i]
                }
                self.i += self.op_x() + 1;
            }
            _ => not_implemented(self.opcode as usize, self.pc),
        }
        self.pc += 2;
    }

    fn op_x(&self) -> usize {
        ((self.opcode & 0x0F00) >> 8) as usize
    }
    fn op_y(&self) -> usize {
        ((self.opcode & 0x00F0) >> 4) as usize
    }
    fn op_n(&self) -> u8 {
        (self.opcode & 0x000F) as u8
    }
    fn op_nn(&self) -> u8 {
        (self.opcode & 0x00FF) as u8
    }
    fn op_nnn(&self) -> u16 {
        self.opcode & 0x0FFF
    }

    fn wait_keypress(&mut self) {
        for i in 0u8..16 {
            if self.keypad.pressed(i as usize) {
                self.v[self.op_x()] = i;
                break;
            }
        }
        self.pc -= 2;
    }
}

fn not_implemented(op: usize, pc: usize) {
    println!("Not implemented:: op: {:x}, pc: {:x}", op, pc)
}

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];
