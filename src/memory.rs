use std::error::Error;

#[derive(Debug)]
pub enum Operation {
    Read,
    Write,
}

// https://problemkaputt.de/2k6specs.htm#memorymirrors
#[derive(Debug)]
pub enum MemoryMirrors {
    Cartridge(usize),
    TiaRead(TiaReadAddress),
    TiaWrite(TiaWriteAddress),
    PiaIO(PiaAddress),
    PiaRam(PiaAddress),
}

impl MemoryMirrors {
    pub fn from(address: u16, op: Operation) -> Result<Self, Box<dyn Error>> {
        const A12: u16 = 0b0001_0000_0000_0000; // 0x1000
        const A9: u16 = 0b0000_0010_0000_0000; // 0x0200
        const A7: u16 = 0b0000_0000_1000_0000; // 0x0080

        match address {
            // Cartridge memory is selected by A12=1
            a if a & A12 != 0 => Ok(Self::Cartridge(address as usize & 0xfff)),

            // PIA I/O is selected by A12=0, A9=1, A7=1
            a if a & (A12 | A9 | A7) == A9 | A7 => Ok(Self::PiaIO((address & 0x2ff).try_into()?)),

            // PIA RAM is selected by A12=0, A9=0, A7=1
            a if a & A7 == A7 => Ok(Self::PiaRam((address & 0x7f).try_into()?)),

            // The TIA chip is addressed by A12=0, A7=0
            a if a & A7 == 0 => match op {
                Operation::Read => Ok(Self::TiaRead(((address & 0x0f) | 0x30).try_into()?)),
                Operation::Write => Ok(Self::TiaWrite((address & 0x3f).try_into()?)),
            },

            _ => Err(format!("Invalid address: {:X}", address).into()),
        }
    }
}

#[derive(Debug)]
// Enum representing TIA read addresses
pub enum TiaReadAddress {
    CXM0P,  // 30 - 11...... Read collision M0-P1, M0-P0 (Bit 7, 6)
    CXM1P,  // 31 - 11...... Read collision M1-P0, M1-P1
    CXP0FB, // 32 - 11...... Read collision P0-PF, P0-BL
    CXP1FB, // 33 - 11...... Read collision P1-PF, P1-BL
    CXM0FB, // 34 - 11...... Read collision M0-PF, M0-BL
    CXM1FB, // 35 - 11...... Read collision M1-PF, M1-BL
    CXBLPF, // 36 - 1....... Read collision BL-PF, unused
    CXPPMM, // 37 - 11...... Read collision P0-P1, M0-M1
    INPT0,  // 38 - 1....... Read pot port
    INPT1,  // 39 - 1....... Read pot port
    INPT2,  // 3A - 1....... Read pot port
    INPT3,  // 3B - 1....... Read pot port
    INPT4,  // 3C - 1....... Read input
    INPT5,  // 3D - 1....... Read input
}

impl TryFrom<u16> for TiaReadAddress {
    type Error = Box<dyn Error>;
    fn try_from(address: u16) -> Result<Self, Self::Error> {
        match address {
            // Match each address to the corresponding enum variant
            0x30 => Ok(Self::CXM0P),
            0x31 => Ok(Self::CXM1P),
            0x32 => Ok(Self::CXP0FB),
            0x33 => Ok(Self::CXP1FB),
            0x34 => Ok(Self::CXM0FB),
            0x35 => Ok(Self::CXM1FB),
            0x36 => Ok(Self::CXBLPF),
            0x37 => Ok(Self::CXPPMM),
            0x38 => Ok(Self::INPT0),
            0x39 => Ok(Self::INPT1),
            0x3A => Ok(Self::INPT2),
            0x3B => Ok(Self::INPT3),
            0x3C => Ok(Self::INPT4),
            0x3D => Ok(Self::INPT5),
            _ => Err(format!("Invalid TIA Read address: {:X}", address).into()), // Return an error for invalid addresses
        }
    }
}

#[derive(Debug)]
// Enum representing TIA write addresses
pub enum TiaWriteAddress {
    VSYNC,  // 00 - ......1. Vertical sync set-clear
    VBLANK, // 01 - 11....1. Vertical blank set-clear
    WSYNC,  // 02 - <strobe> Wait for leading edge of horizontal blank
    RSYNC,  // 03 - <strobe> Reset horizontal sync counter
    NUSIZ0, // 04 - ..11.111 Number-size player-missile 0
    NUSIZ1, // 05 - ..11.111 Number-size player-missile 1
    COLUP0, // 06 - 1111111. Color-lum player 0 and missile 0
    COLUP1, // 07 - 1111111. Color-lum player 1 and missile 1
    COLUPF, // 08 - 1111111. Color-lum playfield and ball
    COLUBK, // 09 - 1111111. Color-lum background
    CTRLPF, // 0A - ..11.111 Control playfield ball size & collisions
    REFP0,  // 0B - ....1... Reflect player 0
    REFP1,  // 0C - ....1... Reflect player 1
    PF0,    // 0D - 1111.... Playfield register byte 0
    PF1,    // 0E - 11111111 Playfield register byte 1
    PF2,    // 0F - 11111111 Playfield register byte 2
    RESP0,  // 10 - <strobe> Reset player 0
    RESP1,  // 11 - <strobe> Reset player 1
    RESM0,  // 12 - <strobe> Reset missile 0
    RESM1,  // 13 - <strobe> Reset missile 1
    RESBL,  // 14 - <strobe> Reset ball
    AUDC0,  // 15 - ....1111 Audio control 0
    AUDC1,  // 16 - ....1111 Audio control 1
    AUDF0,  // 17 - ...11111 Audio frequency 0
    AUDF1,  // 18 - ...11111 Audio frequency 1
    AUDV0,  // 19 - ....1111 Audio volume 0
    AUDV1,  // 1A - ....1111 Audio volume 1
    GRP0,   // 1B - 11111111 Graphics player 0
    GRP1,   // 1C - 11111111 Graphics player 1
    ENAM0,  // 1D - ......1. Graphics (enable) missile 0
    ENAM1,  // 1E - ......1. Graphics (enable) missile 1
    ENABL,  // 1F - ......1. Graphics (enable) ball
    HMP0,   // 20 - 1111.... Horizontal motion player 0
    HMP1,   // 21 - 1111.... Horizontal motion player 1
    HMM0,   // 22 - 1111.... Horizontal motion missile 0
    HMM1,   // 23 - 1111.... Horizontal motion missile 1
    HMBL,   // 24 - 1111.... Horizontal motion ball
    VDELP0, // 25 - .......1 Vertical delay player 0
    VDELP1, // 26 - .......1 Vertical delay player 1
    VDELBL, // 27 - .......1 Vertical delay ball
    RESMP0, // 28 - ......1. Reset missile 0 to player 0
    RESMP1, // 29 - ......1. Reset missile 1 to player 1
    HMOVE,  // 2A - <strobe> Apply horizontal motion
    HMCLR,  // 2B - <strobe> Clear horizontal motion registers
    CXCLR,  // 2C - <strobe> Clear collision latches
}

impl TryFrom<u16> for TiaWriteAddress {
    type Error = Box<dyn Error>;
    fn try_from(address: u16) -> Result<Self, Self::Error> {
        match address {
            0x00 => Ok(Self::VSYNC),
            0x01 => Ok(Self::VBLANK),
            0x02 => Ok(Self::WSYNC),
            0x03 => Ok(Self::RSYNC),
            0x04 => Ok(Self::NUSIZ0),
            0x05 => Ok(Self::NUSIZ1),
            0x06 => Ok(Self::COLUP0),
            0x07 => Ok(Self::COLUP1),
            0x08 => Ok(Self::COLUPF),
            0x09 => Ok(Self::COLUBK),
            0x0A => Ok(Self::CTRLPF),
            0x0B => Ok(Self::REFP0),
            0x0C => Ok(Self::REFP1),
            0x0D => Ok(Self::PF0),
            0x0E => Ok(Self::PF1),
            0x0F => Ok(Self::PF2),
            0x10 => Ok(Self::RESP0),
            0x11 => Ok(Self::RESP1),
            0x12 => Ok(Self::RESM0),
            0x13 => Ok(Self::RESM1),
            0x14 => Ok(Self::RESBL),
            0x15 => Ok(Self::AUDC0),
            0x16 => Ok(Self::AUDC1),
            0x17 => Ok(Self::AUDF0),
            0x18 => Ok(Self::AUDF1),
            0x19 => Ok(Self::AUDV0),
            0x1A => Ok(Self::AUDV1),
            0x1B => Ok(Self::GRP0),
            0x1C => Ok(Self::GRP1),
            0x1D => Ok(Self::ENAM0),
            0x1E => Ok(Self::ENAM1),
            0x1F => Ok(Self::ENABL),
            0x20 => Ok(Self::HMP0),
            0x21 => Ok(Self::HMP1),
            0x22 => Ok(Self::HMM0),
            0x23 => Ok(Self::HMM1),
            0x24 => Ok(Self::HMBL),
            0x25 => Ok(Self::VDELP0),
            0x26 => Ok(Self::VDELP1),
            0x27 => Ok(Self::VDELBL),
            0x28 => Ok(Self::RESMP0),
            0x29 => Ok(Self::RESMP1),
            0x2A => Ok(Self::HMOVE),
            0x2B => Ok(Self::HMCLR),
            0x2C => Ok(Self::CXCLR),
            _ => Err(format!("Invalid TIA Write address: {:X}", address).into()), // Return an error for invalid addresses
        }
    }
}

#[derive(Debug)]
// Enum representing PIA 6532 addresses for read and write operations
pub enum PiaAddress {
    RAM(usize), // 00..=7F - 128 bytes RAM (in PIA chip) for variables and stack
    SWCHA,      // 0280 - Port A; input or output (read or write)
    SWACNT,     // 0281 - Port A DDR, 0=input, 1=output (read or write)
    SWCHB,      // 0282 - Port B; console switches (read only)
    SWBCNT,     // 0283 - Port B DDR (hardwired as input) (read only)
    INTIM,      // 0284 - Timer output (read only)
    INSTAT,     // 0285 - Timer Status (read only, undocumented)
    TIM1T,      // 0294 - Set 1 clock interval (838 nsec/interval) (read or write)
    TIM8T,      // 0295 - Set 8 clock interval (6.7 usec/interval) (read or write)
    TIM64T,     // 0296 - Set 64 clock interval (53.6 usec/interval) (read or write)
    T1024T,     // 0297 - Set 1024 clock interval (858.2 usec/interval) (read or write)
}

impl TryFrom<u16> for PiaAddress {
    type Error = Box<dyn Error>;
    fn try_from(address: u16) -> Result<Self, Self::Error> {
        match address {
            0x0000..=0x007F => Ok(Self::RAM(address as usize)),
            0x0280 => Ok(Self::SWCHA), // Initialize with a dummy value
            0x0281 => Ok(Self::SWACNT),
            0x0282 => Ok(Self::SWCHB),
            0x0283 => Ok(Self::SWBCNT),
            0x0284 => Ok(Self::INTIM),
            0x0285 => Ok(Self::INSTAT),
            0x0294 => Ok(Self::TIM1T),
            0x0295 => Ok(Self::TIM8T),
            0x0296 => Ok(Self::TIM64T),
            0x0297 => Ok(Self::T1024T),
            _ => Err(format!("Invalid PIA address: {:X}", address).into()), // Return an error for invalid addresses
        }
    }
}
