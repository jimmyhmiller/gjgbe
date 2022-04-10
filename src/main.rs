

#[derive(Debug)]
struct Flags {
    zero: bool,
    subtraction: bool,
    half_carry: bool,
    carry: bool,
}


#[allow(dead_code)]
#[derive(Debug)]
struct CPU {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    flags: Flags,
    sp: u16,
    pc: u16,
    interrupt_enabled: bool,
}


impl CPU {

    fn new() -> CPU {
        CPU { 
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            flags: Flags {
                zero: false,
                subtraction: false,
                half_carry: false,
                carry: false,
            },
            sp: 0xFFFE,
            pc: 0,
            interrupt_enabled: true,
        }
    }
}

#[derive(Debug)]
struct Memory {
    cartridge: Vec<u8>,
    boot_rom: Vec<u8>,
    working_memory_1: [u8; 0xFFFF],
    working_memory_2: [u8; 0xFFFF],
    io_registers: [u8; 0x7F],
    interrupt_enable_register: u8,
    boot_rom_enable_register: bool,
    // Is the +1 right?
    high_ram: [u8; 0xFFFE - 0xFF80 + 1],
    video_ram: [u8; 0x9FFF - 0x8000 + 1],
}


impl Memory {
    fn new() -> Memory {
        Memory {
            cartridge: vec![],
            boot_rom: std::fs::read("./roms/boot.gb").expect("Error loading boot rom"),
            // We did the full 16 bit address space
            // But we don't need to because these banks are not that size
            working_memory_1: [0; 0xFFFF],
            working_memory_2: [0; 0xFFFF],
            io_registers: [0; 0x7F],
            interrupt_enable_register: 0,
            boot_rom_enable_register: true, 
            high_ram: [0; 0xFFFE - 0xFF80 + 1],
            video_ram: [0; 0x9FFF - 0x8000 + 1],
        }
    }

    fn load_cartridge(&mut self, cartridge: &Vec<u8>) {
        self.cartridge = cartridge.clone();
    }

    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x00FF => if self.boot_rom_enable_register {
                self.boot_rom[address as usize]
            } else {
                self.cartridge[address as usize]
            },
            0x0000..=0x3FFF => self.cartridge[address as usize],
            0xFF50..=0xFF50 => self.boot_rom_enable_register as u8,
            0xFF00..=0xFF7F => self.io_registers[address as usize - 0xFF00],
            0xFFFF..=0xFFFF => self.interrupt_enable_register,
            0xC000..=0xCFFF => self.working_memory_1[address as usize],
            0xD000..=0xDFFF => self.working_memory_2[address as usize],
            0xFF80..=0xFFFE => self.high_ram[address as usize - 0xFF80],
            0x8000..=0x9FFF => self.video_ram[address as usize - 0x8000],
            _ => panic!("Invalid memory address: 0x{:X}", address),
        }
    }

    fn read_bytes(&self, r: Range<u16>) -> Vec<u8> {
        let mut bytes = vec![];
        for i in r {
            bytes.push(self.read_byte(i))
        }
        return bytes;
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x3FFF => panic!("We cannot write to ROM"),
            0xFF50..=0xFF50 => self.boot_rom_enable_register = value == 0,
            0xFF00..=0xFF7F => self.io_registers[address as usize - 0xFF00] = value,
            0xFFFF..=0xFFFF => self.interrupt_enable_register = value,
            0xC000..=0xCFFF => self.working_memory_1[address as usize] = value,
            0xD000..=0xDFFF => self.working_memory_2[address as usize] = value,
            0xFF80..=0xFFFE => self.high_ram[address as usize - 0xFF80] = value,
            0x8000..=0x9FFF => self.video_ram[address as usize - 0x8000] = value,
            _ => panic!("Invalid memory address: 0x{:X}", address),
        }
    }
}


#[derive(Debug)]
struct Emulator {
    cpu: CPU,
    memory: Memory,
}

impl Emulator {
    fn new() -> Emulator {
        Emulator {
            cpu: CPU::new(),
            memory: Memory::new(),
        }
    }

    fn run(&mut self, cartridge: &Vec<u8>) {
        
        self.memory.load_cartridge(cartridge);
        
        loop {
            let instruction = self.memory.read_byte(self.cpu.pc);
            match instruction {
                0x00 => {
                    println!("NOP");
                } // noop
                0xC3 => { // JP nn
                    let value = &self.memory.read_bytes(self.cpu.pc + 1..self.cpu.pc + 3);
                    let number = ((value[1] as u16) << 8) | value[0] as u16;
                    self.cpu.pc = number;
                    println!("JP {:#04X}", number);
                    continue;
                }
                0xAF => { // xor A A
                    println!("AF");
                    self.cpu.a = 0
                }
                // load into hl register
                0x21 => {
                    let value = &self.memory.read_bytes(self.cpu.pc + 1..self.cpu.pc + 3);
                    let number = ((value[1] as u16) << 8) | value[0] as u16;
                    self.cpu.l = value[0];
                    self.cpu.h = value[1];
                    self.cpu.pc += 3;
                    println!("LD HL, {:#04X}", number);
                    continue;
                }
                0x11 => {
                    let value = &self.memory.read_bytes(self.cpu.pc + 1..self.cpu.pc  + 3);
                    let number = ((value[1] as u16) << 8) | value[0] as u16;
                    self.cpu.e = value[0];
                    self.cpu.d = value[1];
                    self.cpu.pc += 3;
                    println!("LD DE, {:#04X}", number);
                    continue;
                }
                0x1A => {
                    let memory_address = ((self.cpu.d as u16) << 8) | self.cpu.e as u16;
                    let value = self.memory.read_byte(memory_address);
                    self.cpu.a = value;
                    println!("LD a, [de], {:#04X}, {:#04X}", value, memory_address);
                }
                0x3E => {
                    self.cpu.a = self.memory.read_byte(self.cpu.pc + 1);
                    self.cpu.pc += 2;
                    println!("LD A, {:#04X}", self.cpu.a);
                    continue;
                }
                0xE0 => {
                    let value = self.memory.read_byte(self.cpu.pc + 1);
                    self.memory.write_byte(0xFF00 + (value as u16), self.cpu.a);
                    self.cpu.pc += 2;
                    println!("LDH  {:#04X}, {:#04X}", 0xFF00 + (value as u16), self.cpu.a);
                    continue;
                }
                0xE2 => {
                    let address = 0xFF00 + self.cpu.c as u16;
                    self.memory.write_byte(address, self.cpu.a);
                    println!("LDH {:#04X}, {:#04X}", address, self.cpu.a);
                }
                0xF0 => {
                    let value = self.memory.read_byte(self.cpu.pc + 1);
                    self.cpu.a = self.memory.read_byte(0xFF00 + (value as u16));
                    self.cpu.pc += 2;
                    println!("LDH A, {:#04X}", 0xFF00 + (value as u16));
                    continue;
                }
                0x0C => {
                    self.cpu.c += 1;
                }
                0x06 => {
                    self.cpu.b = self.memory.read_byte(self.cpu.pc + 1);
                    self.cpu.pc += 2;
                    println!("LD B, {:#04X}", self.cpu.b);
                    continue;
                }
                0x0E => {
                    self.cpu.c = self.memory.read_byte(self.cpu.pc + 1);
                    self.cpu.pc += 2;
                    println!("LD C, {:#04X}", self.cpu.c);
                    continue;
                }
                0x77 => {
                    let memory_address = ((self.cpu.h as u16) << 8) | self.cpu.l as u16;
                    let value = self.cpu.a;
                    self.memory.write_byte(memory_address, value);
                    println!("LD [hl],a, {:#04X}", memory_address);
                }
                0x32 => {
                    let memory_address = ((self.cpu.h as u16) << 8) | self.cpu.l as u16;
                    let value = self.cpu.a;
                    self.memory.write_byte(memory_address, value);
                    let new_memory_address = memory_address.wrapping_sub(1);
                    self.cpu.h = (new_memory_address >> 8) as u8;
                    self.cpu.l = new_memory_address as u8;
                    println!("LD [hl],a; hl--, {:#04X}", memory_address);
                }
                0xFE => {
                    let value = self.memory.read_byte(self.cpu.pc + 1);
                    self.cpu.flags.zero = self.cpu.a == value;
                    self.cpu.flags.subtraction = true;
                    // TODO: Carry and half-carry
                    println!("CP A({:#04X}) {:#04X}", self.cpu.a, value);
                    self.cpu.pc += 2;
                    continue;
                }
                0x0D => {
                    self.cpu.c = self.cpu.c.wrapping_sub(1);
                    self.cpu.flags.zero = self.cpu.c == 0;
                    self.cpu.flags.subtraction = true;
                    // TODO: Carry and half-carry
                    println!("DEC c");
                }
                0x05 => {
                    self.cpu.b = self.cpu.b.wrapping_sub(1);
                    self.cpu.flags.zero = self.cpu.b == 0;
                    self.cpu.flags.subtraction = true;
                    // TODO: Carry and half-carry
                    println!("DEC b");
                }
                0x20 => {
                    if !self.cpu.flags.zero {
                        let location = self.memory.read_byte(self.cpu.pc + 1) as i8;
                        self.cpu.pc += 2;
                        self.cpu.pc = self.cpu.pc.wrapping_add(location as u16);
                        println!("JR NZ {:#04X}", location);
                        continue;
                    }
                    self.cpu.pc += 2;
                    continue;
                }
                0x31 => {
                    let value = &self.memory.read_bytes(self.cpu.pc + 1..self.cpu.pc + 3);
                    let number = ((value[1] as u16) << 8) | value[0] as u16;
                    self.cpu.sp = number;
                    self.cpu.pc += 3;
                    println!("LD SP, {:#04X}", number);
                    continue;
                }
                0x30 => {
                    if !self.cpu.flags.carry {
                        let location = self.memory.read_byte(self.cpu.pc + 1) as i8;
                        self.cpu.pc += 2;
                        self.cpu.pc = self.cpu.pc.wrapping_add(location as u16);
                        println!("JR NC {:#04X}", location);
                        continue;
                    }
                    self.cpu.pc += 2;
                    continue;
                }

                0xF3 => {
                    self.cpu.interrupt_enabled = false;
                    println!("DI");
                }

                0xCD => {
                    let value = &self.memory.read_bytes(self.cpu.pc + 1..self.cpu.pc + 3);
                    let fn_address = ((value[1] as u16) << 8) | value[0] as u16;
                    self.cpu.pc += 3;
                    let [lower, upper] = self.cpu.pc.to_le_bytes();
                    self.cpu.sp = self.cpu.sp.wrapping_sub(1);
                    self.memory.write_byte(self.cpu.sp, upper);
                    self.cpu.sp = self.cpu.sp.wrapping_sub(1);
                    self.memory.write_byte(self.cpu.sp, lower);
                    self.cpu.pc = fn_address;
                    println!("CALL {:#04X}", fn_address);
                    continue;
                }

                0xCB => {
                    let opcode = self.memory.read_byte(self.cpu.pc + 1);
                    self.cpu.pc += 2;
                    match opcode {
                        0x7C => {
                            self.cpu.flags.zero = (self.cpu.h & 0b1000000) != 0;
                            self.cpu.flags.subtraction = false;
                            println!("BIT 7 H");
                        }
                        x => {
                            println!("Don't know instruction cb {:#04x}", x);
                            break;
                        }
                    }
                    continue;
                }

                x => {
                    println!("Don't know instruction {:#04x}", x);
                    break;
                }
            }
            self.cpu.pc += 1;
        }  
    }
}




fn main() -> Result<(), String> {
    let mut emulator = Emulator::new();
    let tetris_rom = std::fs::read("./roms/tetris.gb").expect("Error loading tetris");
    emulator.run(&tetris_rom);
    
    println!("GameBoy!");
    println!("{:?}", emulator.cpu);

    Ok(())
}

// LDH (0xFF00 + 0x0F), A ???????
// LDH (0xFF00 + 0xFF), A // Enable interrupt register

// LDH (0xFF00 + 0x42), A
// LDH (0xFF00 + 0x43), A
// scrolling
// (x, y) = (1, 1)

// LDH (0xFF00 + 0xA4), A
// High RAM 

// LDH (0xFF00 + 0x41), A
// Set LCD stat mode flag to vblank


// LDH (0xFF00 + 0x01), A
// Serial data

// LDH  0xFF02, A
// Set Serial to Internal Clock



// LDH  0xFF40, A
// LCD and PPU enable


// LDH A, 0xFF44
// COPY LY INTO A

// CP A(0x00) 0x94
// Check LY == 0x94 (148)

use std::ops::Range;