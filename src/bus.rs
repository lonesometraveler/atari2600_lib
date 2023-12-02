use crate::{SharedRIOT, SharedTIA};
use std::fs::File;
use std::io;

pub trait Bus {
    fn read(&mut self, _address: u16) -> u8 {
        0
    }
    fn write(&mut self, _address: u16, _val: u8) {}
    fn save(&self, _output: &mut File) -> io::Result<()> {
        Ok(())
    }
    fn load(&mut self, _input: &mut File) -> io::Result<()> {
        Ok(())
    }
}

pub(crate) struct AtariBus {
    rom: Vec<u8>,
    tia: SharedTIA,
    riot: SharedRIOT,
}

impl AtariBus {
    pub fn new(tia: SharedTIA, riot: SharedRIOT, rom: Vec<u8>) -> Self {
        Self { rom, tia, riot }
    }
}

impl Bus for AtariBus {
    fn read(&mut self, address: u16) -> u8 {
        let memory_mirror = MemoryMirrors::from(address);
        match memory_mirror {
            MemoryMirrors::Cartridge => self.rom[address as usize & 0xfff],
            MemoryMirrors::PiaIO => self.riot.borrow_mut().read(address & 0x2ff),
            MemoryMirrors::PiaRam => self.riot.borrow_mut().read(address & 0x7f),
            MemoryMirrors::Tia => self.tia.borrow_mut().read((address & 0x0f) | 0x30),
        }
    }

    fn write(&mut self, address: u16, val: u8) {
        let memory_mirror = MemoryMirrors::from(address);
        match memory_mirror {
            MemoryMirrors::Cartridge => self.rom[address as usize & 0xfff] = val,
            MemoryMirrors::PiaIO => self.riot.borrow_mut().write(address & 0x2ff, val),
            MemoryMirrors::PiaRam => self.riot.borrow_mut().write(address & 0x7f, val),
            MemoryMirrors::Tia => self.tia.borrow_mut().write(address & 0x3f, val),
        }
    }
}

// https://problemkaputt.de/2k6specs.htm#memorymirrors
enum MemoryMirrors {
    Cartridge,
    Tia,
    PiaIO,
    PiaRam,
}

impl From<u16> for MemoryMirrors {
    fn from(address: u16) -> Self {
        const A12: u16 = 0b0001_0000_0000_0000; // 0x1000
        const A9: u16 = 0b0000_0010_0000_0000; // 0x0200
        const A7: u16 = 0b0000_0000_1000_0000; // 0x0080

        match address {
            // Cartridge memory is selected by A12=1
            a if a & A12 != 0 => Self::Cartridge,
            // PIA I/O is selected by A12=0, A9=1, A7=1
            a if a & (A12 | A9 | A7) == A9 | A7 => Self::PiaIO,
            // PIA RAM is selected by A12=0, A9=0, A7=1
            a if a & A7 == A7 => Self::PiaRam,
            // The TIA chip is addressed by A12=0, A7=0
            a if a & A7 == 0 => Self::Tia,
            _ => unreachable!(),
        }
    }
}
