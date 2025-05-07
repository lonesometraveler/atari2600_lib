#[derive(Copy, Clone, Debug)]
pub enum AddressingMode {
    None,
    Immediate,
    Absolute,
    Implied,
    Accumulator,
    AbsoluteX,
    AbsoluteY,
    ZeroPageIndexed,
    ZeroPageX,
    ZeroPageY,
    Indirect,
    IndexedIndirect,
    IndirectIndexed,
    Relative,
}

impl AddressingMode {
    pub fn n_bytes(&self) -> usize {
        match *self {
            AddressingMode::Implied | AddressingMode::Accumulator => 1,

            AddressingMode::Immediate
            | AddressingMode::ZeroPageIndexed
            | AddressingMode::Relative
            | AddressingMode::ZeroPageX
            | AddressingMode::ZeroPageY
            | AddressingMode::IndexedIndirect
            | AddressingMode::IndirectIndexed => 2,

            AddressingMode::Absolute
            | AddressingMode::AbsoluteX
            | AddressingMode::AbsoluteY
            | AddressingMode::Indirect => 3,

            _ => panic!("Bad addressing mode {:?}", *self),
        }
    }
}
