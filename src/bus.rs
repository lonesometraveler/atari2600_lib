use crate::memory::{MemoryMirrors, Operation};
use crate::{SharedRIOT, SharedTIA};
use heapless::Vec;
use log::error;

pub trait Bus {
    fn read(&mut self, _address: u16) -> u8 {
        0
    }
    fn write(&mut self, _address: u16, _val: u8) {}
}

pub(crate) struct AtariBus {
    rom: Vec<u8, 4096>,
    tia: SharedTIA,
    riot: SharedRIOT,
}

impl AtariBus {
    pub fn new(tia: SharedTIA, riot: SharedRIOT, rom: Vec<u8, 4096>) -> Self {
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
