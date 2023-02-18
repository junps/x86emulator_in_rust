use std::fs::File;
use std::io::{Error, ErrorKind, Read};

const MEMORY_SIZE: usize = 1024 * 1024;
#[derive(Debug)]
enum Register {
    EAX,
    ECX,
    EDX,
    EBX,
    ESP,
    EBP,
    ESI,
    EDI,
    RegistersCount,
}

#[derive(Debug)]
struct Emulator {
    registers: [u32; Register::RegistersCount as usize],
    eflags: u32,
    memory: Vec<u8>,
    eip: usize,
}

impl Emulator {
    fn new(size: usize, eip: usize, esp: u32) -> Self {
        let mut emu = Self {
            registers: [0; Register::RegistersCount as usize],
            eflags: 0,
            memory: vec![0; size],
            eip,
        };
        emu.registers[Register::ESP as usize] = esp;
        emu
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        return Err(Error::new(ErrorKind::Other, "num of args should be 2."));
    }
    let mut file = File::open(&args[1])?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    println!("{:?}", buffer);
    let mut emu = Emulator::new(MEMORY_SIZE, 0x0000, 0x7c00);
    emu.memory = buffer;
    println!("{:?}", emu);
    Ok(())
}
