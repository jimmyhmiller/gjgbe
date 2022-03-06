

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
            sp: 0,
            pc: 0x0100,
        }
    }
}

#[derive(Debug)]
struct Memory {
    rom: Vec<u8>,
    working_memory_1: [u8; 0xFFFF],
    working_memory_2: [u8; 0xFFFF],
}

impl Memory {
    fn new() -> Memory {
        Memory {
            rom: vec![],
            working_memory_1: [0; 0xFFFF],
            working_memory_2: [0; 0xFFFF],
        }
    }

    fn load_rom(&mut self, rom: &Vec<u8>) {
        self.rom = rom.clone();
    }

    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xC000..=0xCFFF => self.working_memory_1[address as usize],
            0xD000..=0xDFFF => self.working_memory_2[address as usize],
            _ => panic!("Invalid memory address: 0x{:X}", address),
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0xC000..=0xCFFF => self.working_memory_1[address as usize] = value,
            0xD000..=0xDFFF => self.working_memory_2[address as usize] = value,
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

    fn run(&mut self, rom: &Vec<u8>) {
        self.memory.load_rom(rom);
        loop {
            let instruction = self.memory.rom[self.cpu.pc as usize];
            match instruction {
                0x00 => {
                    println!("NOP");
                } // noop
                0xC3 => { // JP nn
                    let value = &self.memory.rom[(self.cpu.pc as usize) + 1..(self.cpu.pc as usize) + 3];
                    let number = ((value[1] as u16) << 8) | value[0] as u16;
                    self.cpu.pc = number;
                    println!("JP {:#04X}", number);
                    continue;
                }
                0xAF => { // xor A A
                    println!("AF");
                    self.cpu.a = 0
                    
                }
                0x21 => {
                    let value = &self.memory.rom[(self.cpu.pc as usize) + 1..(self.cpu.pc as usize) + 3];
                    let number = ((value[1] as u16) << 8) | value[0] as u16;
                    // TODO: Is the correct for upper and lower?
                    self.cpu.l = value[0];
                    self.cpu.h = value[1];
                    self.cpu.pc += 3;
                    println!("LD HL, {:#04X}", number);
                    continue;
                }
                0x0E => {
                    self.cpu.c = self.memory.rom[self.cpu.pc as usize + 1];
                    self.cpu.pc += 2;
                    println!("LD C, {:#04X}", self.cpu.c);
                    continue;
                }
                0x06 => {
                    self.cpu.b = self.memory.rom[self.cpu.pc as usize + 1];
                    self.cpu.pc += 2;
                    println!("LD B, {:#04X}", self.cpu.b);
                    continue;
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
                0x05 => {
                    self.cpu.b = self.cpu.b.wrapping_sub(1);
                    self.cpu.flags.zero = self.cpu.b == 0;
                    self.cpu.flags.subtraction = true;
                    // TODO: Carry and half-carry
                    println!("DEC b");
                }
                0x20 => {
                    if self.cpu.flags.zero && self.cpu.flags.subtraction {
                        let location = self.memory.rom[self.cpu.pc as usize + 1] as i8;
                        self.cpu.pc = self.cpu.pc.wrapping_add(location as u16);
                        println!("JR NZ {:#04X}", location);
                        continue;
                    }
                    self.cpu.pc += 2;
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
