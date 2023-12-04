use crate::memory::{MemoryMirrors, Operation};
use crate::{SharedRIOT, SharedTIA};
use log::error;
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
        match MemoryMirrors::from(address, Operation::Read) {
            Ok(MemoryMirrors::Cartridge(address)) => self.rom[address],
            Ok(MemoryMirrors::PiaIO(address)) => self.riot.borrow_mut().read(address),
            Ok(MemoryMirrors::PiaRam(address)) => self.riot.borrow_mut().read(address),
            Ok(MemoryMirrors::TiaRead(address)) => self.tia.borrow_mut().read(address),
            Err(e) => {
                error!("{}", e);
                0
            }
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: u16, val: u8) {
        match MemoryMirrors::from(address, Operation::Write) {
            Ok(MemoryMirrors::Cartridge(address)) => self.rom[address] = val,
            Ok(MemoryMirrors::PiaIO(address)) => self.riot.borrow_mut().write(address, val),
            Ok(MemoryMirrors::PiaRam(address)) => self.riot.borrow_mut().write(address, val),
            Ok(MemoryMirrors::TiaWrite(address)) => self.tia.borrow_mut().write(address, val),
            Err(e) => error!("{}", e),
            _ => {
                unreachable!();
            }
        }
    }
}
