use std::fmt;

#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct Registers {
    pub(crate) control: u8, // 4 bit
    pub(crate) freq: u8,    // 5 bit
    pub(crate) volume: u8,  // 4 bit
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:04b} @ {:05b} ^ {:04b}",
            self.control, self.freq, self.volume
        )
    }
}

// CmpRegisters returns true if the two registers contain the same values
pub(crate) fn cmp_registers(a: Registers, b: Registers) -> bool {
    (a.control & 0x0f) == (b.control & 0x0f)
        && (a.freq & 0x1f) == (b.freq & 0x1f)
        && (a.volume & 0x0f) == (b.volume & 0x0f)
}
