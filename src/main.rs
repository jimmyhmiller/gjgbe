

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
    flags: u8,
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
            flags: 0,
            sp: 0,
            pc: 0,
        }
    }

    fn run(&mut self, instructions: &Vec<u8>) {
        loop {
            let instruction = instructions[self.pc as usize];
            match instruction {
                0x00 => {} // noop
                0xC3 => { // JP nn
                    println!("JP");
                    let value = &instructions[(self.pc as usize) + 1..(self.pc as usize) + 3];
                    let number = ((value[1] as u16) << 8) | value[0] as u16;
                    self.pc = number;
                    continue;
                }
                0xAF => { // xor A A
                    println!("AF");
                    self.a = 0
                    
                }
                x => {
                    println!("Don't know instruction {}", x);
                    break;
                }
            }
            self.pc += 1;
        }  
    }
}





fn main() -> Result<(), String> {
    let mut cpu = CPU::new();
    let tetris_rom = std::fs::read("./roms/tetris.gb").expect("Error loading tetris");
    cpu.run(&tetris_rom);
    
    println!("GameBoy!");
    println!("{:?}", cpu);

    Ok(())
}
