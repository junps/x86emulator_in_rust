#[derive(Debug)]
pub enum Register {
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
pub const REGISTERS_NAME: [&str; 8] = ["EAX", "ECX", "EDX", "EBX", "ESP", "EBP", "ESI", "EDI"];

#[derive(Debug)]
pub struct Emulator {
    pub registers: [u32; Register::RegistersCount as usize],
    pub eflags: u32,
    pub memory: Vec<u8>,
    pub eip: usize,
}

impl Emulator {
    pub fn new(size: usize, eip: usize, esp: u32) -> Self {
        let mut emu = Self {
            registers: [0; Register::RegistersCount as usize],
            eflags: 0,
            memory: vec![0; size],
            eip,
        };
        emu.registers[Register::ESP as usize] = esp;
        emu
    }

    pub fn dump_registers(&self) {
        for (idx, name) in REGISTERS_NAME.iter().enumerate() {
            println!("{} = {:08x} ({1})", name, self.registers[idx]);
        }
        println!("EIP = {:08x} ({0})", self.eip);
    }

    pub fn get_code8(&self, index: usize) -> u32 {
        self.memory[self.eip + index] as u32
    }

    pub fn get_sign_code8(&self, index: usize) -> i32 {
        self.memory[self.eip + index] as i32
    }

    pub fn get_code32(&self, index: usize) -> u32 {
        let mut ret: u32 = 0;
        for pos in 0..4 {
            ret |= self.get_code8(index + pos) << (pos * 8);
        }
        ret
    }

    pub fn get_sign_code32(&self, index: usize) -> i32 {
        self.get_code32(index) as i32
    }

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

    fn near_jmp(&mut self) {
        let diff = self.get_sign_code32(1);
        self.eip = (diff + 5 + (self.eip as i32)) as usize;
    }

    pub fn call_instruction(&mut self, opcode: u32) -> Result<(), String> {
        match opcode {
            0xB8..=0xC0 => self.mov_r32_imm32(),
            0xE9 => self.near_jmp(),
            0xEB => self.short_jmp(),
            _ => {
                return Err(format!("Not implemented for: {}", opcode));
            }
        }
        Ok(())
    }
}
