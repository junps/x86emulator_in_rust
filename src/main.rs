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

    fn get_code8(&self, index: usize) -> u32 {
        self.memory[self.eip + index] as u32
    }

    fn get_sign_code8(&self, index: usize) -> i32 {
        (self.memory[self.eip + index] as i8) as i32
    }

    fn get_code32(&self, index: usize) -> u32 {
        let mut ret: u32 = 0;
        for pos in 0..4 {
            ret |= self.get_code8(index + pos) << (pos * 8);
        }
        ret
    }

    fn nop(&mut self) {}

    fn mov_r32_imm32(&mut self) {
        let reg = self.get_code8(0) - 0xB8;
        let value = self.get_code32(1);
        self.registers[reg as usize] = value;
        self.eip += 5;
    }

    fn short_jmp(&mut self) {
        let diff = self.get_sign_code8(1);
        self.eip = (diff + 2 + (self.eip as i32)) as usize;
    }

    fn call_instruction(&mut self, opcode: usize) {
        match opcode {
            0xB8..=0xC0 => self.mov_r32_imm32(),
            0xEB => self.short_jmp(),
            _ => unreachable!(),
        }
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
