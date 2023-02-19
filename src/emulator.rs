use crate::modrm::ModRM;
use std::process;

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

    fn get_register32(&self, index: usize) -> u32 {
        self.registers[index]
    }

    fn get_rm32(&mut self, modrm: &ModRM) -> u32 {
        if modrm.modval == 3 {
            self.get_register32(modrm.rm.try_into().unwrap())
        } else {
            let address = self.calc_memory_address(modrm);
            self.get_memory32(address.try_into().unwrap())
        }
    }

    fn get_r32(&self, modrm: &ModRM) -> u32 {
        self.get_register32(modrm.reg_index.try_into().unwrap())
    }

    fn set_register32(&mut self, index: usize, value: u32) {
        self.registers[index] = value;
    }

    fn set_r32(&mut self, modrm: &ModRM, value: u32) {
        self.set_register32(modrm.reg_index.try_into().unwrap(), value);
    }

    fn get_memory8(&self, address: usize) -> u32 {
        self.memory[address].try_into().unwrap()
    }

    fn get_memory32(&self, address: usize) -> u32 {
        let mut ret = 0;
        for idx in 0..4 {
            ret |= self.get_memory8(address + idx) << (8 * idx);
        }
        ret
    }

    fn set_memory8(&mut self, address: usize, value: u32) {
        self.memory[address] = (value & 0xFF).try_into().unwrap();
    }

    fn set_memory32(&mut self, address: usize, value: u32) {
        for idx in 0..4 {
            self.set_memory8(address + idx, (value >> (idx * 8)).try_into().unwrap());
        }
    }

    fn calc_memory_address(&mut self, modrm: &ModRM) -> u32 {
        if modrm.modval == 0 {
            if modrm.rm == 4 {
                println!("not implemented ModRM mod = 0, rm = 4");
                process::exit(1);
            } else if modrm.rm == 5 {
                return modrm.disp32;
            }
            return self.get_register32(modrm.rm.try_into().unwrap());
        } else if modrm.modval == 1 {
            if modrm.rm == 4 {
                println!("not implemented ModRM mod = 1, rm = 4");
                process::exit(1);
            }
            return (self.get_register32(modrm.rm.try_into().unwrap()) as i32 + modrm.disp8 as i32)
                as u32;
        } else if modrm.modval == 2 {
            if modrm.rm == 4 {
                println!("not implemented ModRM mod = 2, rm = 4");
                process::exit(1);
            }
            return self.get_register32(modrm.rm.try_into().unwrap()) + modrm.disp32;
        }
        println!("not implemented ModRM mod = 3");
        process::exit(1);
    }

    fn set_rm32(&mut self, modrm: &ModRM, value: u32) {
        if modrm.modval == 3 {
            self.set_register32(modrm.rm.try_into().unwrap(), value);
        } else {
            let address = self.calc_memory_address(modrm).try_into().unwrap();
            self.set_memory32(address, value);
        }
    }

    fn mov_r32_imm32(&mut self) {
        let reg = self.get_code8(0) - 0xB8;
        let value = self.get_code32(1);
        self.registers[reg as usize] = value;
        self.eip += 5;
    }

    fn mov_rm32_imm32(&mut self) {
        self.eip += 1;
        let mut modrm = ModRM::default();
        modrm.parse_modrm(self);
        let value = self.get_code32(0);
        self.eip += 4;
        self.set_rm32(&modrm, value);
    }

    fn mov_rm32_r32(&mut self) {
        self.eip += 1;
        let mut modrm = ModRM::default();
        modrm.parse_modrm(self);
        let r32 = self.get_r32(&modrm);
        self.set_rm32(&modrm, r32);
    }

    fn mov_r32_rm32(&mut self) {
        self.eip += 1;
        let mut modrm = ModRM::default();
        modrm.parse_modrm(self);
        let rm32 = self.get_rm32(&modrm);
        self.set_r32(&modrm, rm32);
    }

    fn short_jmp(&mut self) {
        let diff = self.get_sign_code8(1);
        self.eip = (diff + 2 + (self.eip as i32)) as usize;
    }

    fn near_jmp(&mut self) {
        let diff = self.get_sign_code32(1);
        self.eip = (diff + 5 + (self.eip as i32)) as usize;
    }

    fn add_rm32_r32(&mut self) {
        self.eip += 1;
        let mut modrm = ModRM::default();
        modrm.parse_modrm(self);
        let r32 = self.get_r32(&modrm);
        let rm32 = self.get_rm32(&modrm);
        self.set_rm32(&modrm, r32 + rm32);
    }

    fn sub_rm32_imm8(&mut self, modrm: &ModRM) {
        let rm32 = self.get_rm32(modrm);
        let imm8: u32 = self.get_sign_code8(0).try_into().unwrap();
        self.eip += 1;
        self.set_rm32(modrm, rm32 - imm8);
    }

    fn code_83(&mut self) {
        self.eip += 1;
        let mut modrm = ModRM::default();
        modrm.parse_modrm(self);
        match modrm.opecode {
            5 => self.sub_rm32_imm8(&modrm),
            _ => {
                println!("not implemented: 83 {}", modrm.opecode);
                process::exit(1);
            }
        }
    }

    fn inc_rm32(&mut self, modrm: &ModRM) {
        let value = self.get_rm32(modrm);
        self.set_rm32(modrm, value + 1);
    }

    fn code_ff(&mut self) {
        self.eip += 1;
        let mut modrm = ModRM::default();
        modrm.parse_modrm(self);
        match modrm.opecode {
            0 => self.inc_rm32(&modrm),
            _ => {
                println!("not implemented: FF {}", modrm.opecode);
                process::exit(1);
            }
        }
    }

    pub fn call_instruction(&mut self, opcode: u32) -> Result<(), String> {
        match opcode {
            0x01 => self.add_rm32_r32(),
            0x83 => self.code_83(),
            0x89 => self.mov_rm32_r32(),
            0x8B => self.mov_r32_rm32(),
            0xB8..=0xC0 => self.mov_r32_imm32(),
            0xC7 => self.mov_rm32_imm32(),
            0xE9 => self.near_jmp(),
            0xEB => self.short_jmp(),
            0xFF => self.code_ff(),
            _ => {
                return Err(format!("Not implemented for: {}", opcode));
            }
        }
        Ok(())
    }
}
