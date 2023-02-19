use std::any::Any;

use crate::emulator::Emulator;

#[derive(Debug, Default)]
pub struct ModRM {
    pub modval: u8,
    pub opecode: u8,
    pub reg_index: u8,
    pub rm: u8,
    pub sib: u8,
    pub disp8: i8,
    pub disp32: u32,
}

impl ModRM {
    pub fn parse_modrm(&mut self, emu: &mut Emulator) {
        let code = emu.get_code8(0);
        self.modval = ((code & 0xC0) >> 6).try_into().unwrap();
        self.opecode = ((code & 0x38) >> 3).try_into().unwrap();
        self.reg_index = ((code & 0x38) >> 3).try_into().unwrap();
        self.rm = (code & 0x07).try_into().unwrap();
        emu.eip += 1;
        if self.modval != 3 && self.rm == 4 {
            self.sib = emu.get_code8(0).try_into().unwrap();
            emu.eip += 1;
        }
        if (self.modval == 0 && self.rm == 5) || (self.modval == 2) {
            self.disp32 = emu.get_sign_code32(0).try_into().unwrap();
            self.disp8 = emu.get_sign_code32(0).try_into().unwrap();
            emu.eip += 4;
        } else if self.modval == 1 {
            self.disp8 = emu.get_sign_code32(0).try_into().unwrap();
            emu.eip += 1;
        }
    }
}
