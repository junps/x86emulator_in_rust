use std::fs::File;
use std::io::{Error, ErrorKind, Read};
mod emulator;
use emulator::*;

const MEMORY_SIZE: usize = 1024 * 1024;

fn load_binary(emu: &mut Emulator, file_path: &str) -> std::io::Result<u64> {
    let mut file = File::open(file_path)?;
    let len = file.metadata().unwrap().len();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    emu.memory = vec![0; 0x7c00];
    emu.memory.extend(buffer);
    Ok(len)
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        return Err(Error::new(ErrorKind::Other, "num of args should be 2."));
    }
    let mut emu = Emulator::new(MEMORY_SIZE, 0x7c00, 0x7c00);
    let len = load_binary(&mut emu, &args[1])?;

    while emu.eip < MEMORY_SIZE {
        let opcode = emu.get_code8(0);
        println!("{}", opcode);
        if let Err(s) = emu.call_instruction(opcode) {
            return Err(Error::new(ErrorKind::Other, s));
        }
        if emu.eip >= len as usize || emu.eip == 0x00 {
            break;
        }
    }
    emu.dump_registers();
    Ok(())
}
