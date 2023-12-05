// https://www.masswerk.at/6502/6502_instruction_set.html
#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Debug)]
pub(crate) enum Instruction {
    None, // No operation

    // Arithmetic Instructions
    ADC, // Add with Carry
    ANC, // AND with Carry
    AND, // Logical AND
    ASL, // Arithmetic Shift Left

    // Branch Instructions
    BCC, // Branch if Carry Clear
    BCS, // Branch if Carry Set
    BEQ, // Branch if Equal (zero set)
    BIT, // Bit Test
    BMI, // Branch if Minus (negative set)
    BNE, // Branch if Not Equal (zero clear)
    BPL, // Branch if Plus (negative clear)
    BRK, // Break / Interrupt
    BVC, // Branch if Overflow Clear
    BVS, // Branch if Overflow Set

    // Status Flag Instructions
    CLC, // Clear Carry Flag
    CLD, // Clear Decimal Mode
    CLI, // Clear Interrupt Disable
    CLV, // Clear Overflow Flag

    // Comparison Instructions
    CMP, // Compare Accumulator
    CPX, // Compare X Register
    CPY, // Compare Y Register

    // Decimal Arithmetic Instructions
    DCP, // Decrement and Compare
    DEC, // Decrement Memory
    DEX, // Decrement X Register
    DEY, // Decrement Y Register

    // Logical Instructions
    EOR, // Exclusive OR

    // Increment Instructions
    INC, // Increment Memory
    INX, // Increment X Register
    INY, // Increment Y Register

    // Arithmetic and Logical Instructions
    ISB, // Increment and Subtract with Borrow

    // Unofficial "JAM" Instruction
    JAM, // Jam (unintentional opcode)

    // Jump Instructions
    JMP, // Jump
    JSR, // Jump to Subroutine

    // Unofficial "LAX" Instruction
    LAX, // Load Accumulator and X Register

    // Load Instructions
    LDA, // Load Accumulator
    LDX, // Load X Register
    LDY, // Load Y Register

    // Shift and Rotate Instructions
    LSR, // Logical Shift Right

    // No Operation
    NOP, // No Operation

    // Logical Instructions
    ORA, // Logical OR

    // Stack Instructions
    PHA, // Push Accumulator
    PHP, // Push Processor Status (SR)
    PLA, // Pull Accumulator
    PLP, // Pull Processor Status (SR)

    // Unofficial "RLA" Instruction
    RLA, // Rotate Left and AND

    // Rotate Instructions
    ROL, // Rotate Left
    ROR, // Rotate Right

    // Unofficial "RRA" Instruction
    RRA, // Rotate Right and Add with Carry

    // Return Instructions
    RTI, // Return from Interrupt
    RTS, // Return from Subroutine

    // Unofficial "SAX" Instruction
    SAX, // Store A AND X

    // Arithmetic Instructions
    SBC, // Subtract with Carry

    // Status Flag Instructions
    SEC, // Set Carry Flag
    SED, // Set Decimal Mode
    SEI, // Set Interrupt Disable

    // Unofficial "SLO" Instruction
    SLO, // Shift Left and OR

    // Unofficial "SRE" Instruction
    SRE, // Shift Right and EOR

    // Store Instructions
    STA, // Store Accumulator
    STX, // Store X Register
    STY, // Store Y Register

    // Transfer Instructions
    TAX, // Transfer Accumulator to X
    TAY, // Transfer Accumulator to Y
    TSX, // Transfer Stack Pointer to X
    TXA, // Transfer X to Accumulator
    TXS, // Transfer X to Stack Pointer
    TYA, // Transfer Y to Accumulator
}

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

#[derive(Copy, Clone, Debug)]
pub(crate) struct Opcode(
    pub(crate) Instruction,
    pub(crate) AddressingMode,
    pub(crate) u64, // number of cycles
    pub(crate) u64,
); // number of extra cycles, if a page boundary is crossed

pub(crate) const OPCODES: [Opcode; 256] = [
    // 0x00
    Opcode(Instruction::BRK, AddressingMode::Implied, 7, 0),
    Opcode(Instruction::ORA, AddressingMode::IndexedIndirect, 6, 0),
    Opcode(Instruction::JAM, AddressingMode::Implied, 0, 0),
    Opcode(Instruction::SLO, AddressingMode::IndexedIndirect, 8, 0),
    Opcode(Instruction::NOP, AddressingMode::ZeroPageIndexed, 3, 0),
    Opcode(Instruction::ORA, AddressingMode::ZeroPageIndexed, 3, 0),
    Opcode(Instruction::ASL, AddressingMode::ZeroPageIndexed, 5, 0),
    Opcode(Instruction::SLO, AddressingMode::ZeroPageIndexed, 5, 0),
    Opcode(Instruction::PHP, AddressingMode::Implied, 3, 0),
    Opcode(Instruction::ORA, AddressingMode::Immediate, 2, 0),
    Opcode(Instruction::ASL, AddressingMode::Accumulator, 2, 0),
    Opcode(Instruction::ANC, AddressingMode::Immediate, 2, 0),
    Opcode(Instruction::NOP, AddressingMode::Absolute, 4, 0),
    Opcode(Instruction::ORA, AddressingMode::Absolute, 4, 0),
    Opcode(Instruction::ASL, AddressingMode::Absolute, 6, 0),
    Opcode(Instruction::SLO, AddressingMode::Absolute, 6, 0),
    // 0x10
    Opcode(Instruction::BPL, AddressingMode::Relative, 2, 1),
    Opcode(Instruction::ORA, AddressingMode::IndirectIndexed, 5, 1),
    Opcode(Instruction::JAM, AddressingMode::Implied, 0, 0),
    Opcode(Instruction::SLO, AddressingMode::IndirectIndexed, 8, 0),
    Opcode(Instruction::NOP, AddressingMode::ZeroPageX, 4, 0),
    Opcode(Instruction::ORA, AddressingMode::ZeroPageX, 4, 0),
    Opcode(Instruction::ASL, AddressingMode::ZeroPageX, 6, 0),
    Opcode(Instruction::SLO, AddressingMode::ZeroPageX, 6, 0),
    Opcode(Instruction::CLC, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::ORA, AddressingMode::AbsoluteY, 4, 1),
    Opcode(Instruction::NOP, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::SLO, AddressingMode::AbsoluteY, 7, 0),
    Opcode(Instruction::NOP, AddressingMode::AbsoluteX, 4, 1),
    Opcode(Instruction::ORA, AddressingMode::AbsoluteX, 4, 1),
    Opcode(Instruction::ASL, AddressingMode::AbsoluteX, 7, 0),
    Opcode(Instruction::SLO, AddressingMode::AbsoluteX, 7, 0),
    // 0x20
    Opcode(Instruction::JSR, AddressingMode::Absolute, 6, 0),
    Opcode(Instruction::AND, AddressingMode::IndexedIndirect, 6, 0),
    Opcode(Instruction::JAM, AddressingMode::Implied, 0, 0),
    Opcode(Instruction::RLA, AddressingMode::IndexedIndirect, 8, 0),
    Opcode(Instruction::BIT, AddressingMode::ZeroPageIndexed, 3, 0),
    Opcode(Instruction::AND, AddressingMode::ZeroPageIndexed, 3, 0),
    Opcode(Instruction::ROL, AddressingMode::ZeroPageIndexed, 5, 0),
    Opcode(Instruction::RLA, AddressingMode::ZeroPageIndexed, 5, 0),
    Opcode(Instruction::PLP, AddressingMode::Implied, 4, 0),
    Opcode(Instruction::AND, AddressingMode::Immediate, 2, 0),
    Opcode(Instruction::ROL, AddressingMode::Accumulator, 2, 0),
    Opcode(Instruction::ANC, AddressingMode::Immediate, 2, 0),
    Opcode(Instruction::BIT, AddressingMode::Absolute, 4, 0),
    Opcode(Instruction::AND, AddressingMode::Absolute, 4, 0),
    Opcode(Instruction::ROL, AddressingMode::Absolute, 6, 0),
    Opcode(Instruction::RLA, AddressingMode::Absolute, 6, 0),
    // 0x30
    Opcode(Instruction::BMI, AddressingMode::Relative, 2, 1),
    Opcode(Instruction::AND, AddressingMode::IndirectIndexed, 5, 1),
    Opcode(Instruction::JAM, AddressingMode::Implied, 0, 0),
    Opcode(Instruction::RLA, AddressingMode::IndirectIndexed, 8, 0),
    Opcode(Instruction::NOP, AddressingMode::ZeroPageX, 4, 0),
    Opcode(Instruction::AND, AddressingMode::ZeroPageX, 4, 0),
    Opcode(Instruction::ROL, AddressingMode::ZeroPageX, 6, 0),
    Opcode(Instruction::RLA, AddressingMode::ZeroPageX, 6, 0),
    Opcode(Instruction::SEC, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::AND, AddressingMode::AbsoluteY, 4, 1),
    Opcode(Instruction::NOP, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::RLA, AddressingMode::AbsoluteY, 7, 0),
    Opcode(Instruction::NOP, AddressingMode::AbsoluteX, 4, 1),
    Opcode(Instruction::AND, AddressingMode::AbsoluteX, 4, 1),
    Opcode(Instruction::ROL, AddressingMode::AbsoluteX, 7, 0),
    Opcode(Instruction::RLA, AddressingMode::AbsoluteX, 7, 0),
    // 0x40
    Opcode(Instruction::RTI, AddressingMode::Implied, 6, 0),
    Opcode(Instruction::EOR, AddressingMode::IndexedIndirect, 6, 0),
    Opcode(Instruction::JAM, AddressingMode::Implied, 0, 0),
    Opcode(Instruction::SRE, AddressingMode::IndexedIndirect, 8, 0),
    Opcode(Instruction::NOP, AddressingMode::ZeroPageIndexed, 3, 0),
    Opcode(Instruction::EOR, AddressingMode::ZeroPageIndexed, 3, 0),
    Opcode(Instruction::LSR, AddressingMode::ZeroPageIndexed, 5, 0),
    Opcode(Instruction::SRE, AddressingMode::ZeroPageIndexed, 5, 0),
    Opcode(Instruction::PHA, AddressingMode::Implied, 3, 0),
    Opcode(Instruction::EOR, AddressingMode::Immediate, 2, 0),
    Opcode(Instruction::LSR, AddressingMode::Accumulator, 2, 0),
    Opcode(Instruction::None, AddressingMode::None, 0, 0),
    Opcode(Instruction::JMP, AddressingMode::Absolute, 3, 0),
    Opcode(Instruction::EOR, AddressingMode::Absolute, 4, 0),
    Opcode(Instruction::LSR, AddressingMode::Absolute, 6, 0),
    Opcode(Instruction::SRE, AddressingMode::Absolute, 6, 0),
    // 0x50
    Opcode(Instruction::BVC, AddressingMode::Relative, 2, 1),
    Opcode(Instruction::EOR, AddressingMode::IndirectIndexed, 5, 1),
    Opcode(Instruction::JAM, AddressingMode::Implied, 0, 0),
    Opcode(Instruction::SRE, AddressingMode::IndirectIndexed, 8, 0),
    Opcode(Instruction::NOP, AddressingMode::ZeroPageX, 4, 0),
    Opcode(Instruction::EOR, AddressingMode::ZeroPageX, 4, 0),
    Opcode(Instruction::LSR, AddressingMode::ZeroPageX, 6, 0),
    Opcode(Instruction::SRE, AddressingMode::ZeroPageX, 6, 0),
    Opcode(Instruction::CLI, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::EOR, AddressingMode::AbsoluteY, 4, 1),
    Opcode(Instruction::NOP, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::SRE, AddressingMode::AbsoluteY, 7, 0),
    Opcode(Instruction::NOP, AddressingMode::AbsoluteX, 4, 1),
    Opcode(Instruction::EOR, AddressingMode::AbsoluteX, 4, 1),
    Opcode(Instruction::LSR, AddressingMode::AbsoluteX, 7, 0),
    Opcode(Instruction::SRE, AddressingMode::AbsoluteX, 7, 0),
    // 0x60
    Opcode(Instruction::RTS, AddressingMode::Implied, 6, 0),
    Opcode(Instruction::ADC, AddressingMode::IndexedIndirect, 6, 0),
    Opcode(Instruction::JAM, AddressingMode::Implied, 0, 0),
    Opcode(Instruction::RRA, AddressingMode::IndexedIndirect, 8, 0),
    Opcode(Instruction::NOP, AddressingMode::ZeroPageIndexed, 3, 0),
    Opcode(Instruction::ADC, AddressingMode::ZeroPageIndexed, 3, 0),
    Opcode(Instruction::ROR, AddressingMode::ZeroPageIndexed, 5, 0),
    Opcode(Instruction::RRA, AddressingMode::ZeroPageIndexed, 5, 0),
    Opcode(Instruction::PLA, AddressingMode::Implied, 4, 0),
    Opcode(Instruction::ADC, AddressingMode::Immediate, 2, 0),
    Opcode(Instruction::ROR, AddressingMode::Accumulator, 2, 0),
    Opcode(Instruction::None, AddressingMode::None, 0, 0),
    Opcode(Instruction::JMP, AddressingMode::Indirect, 5, 0),
    Opcode(Instruction::ADC, AddressingMode::Absolute, 4, 0),
    Opcode(Instruction::ROR, AddressingMode::Absolute, 6, 0),
    Opcode(Instruction::RRA, AddressingMode::Absolute, 6, 0),
    // 0x70
    Opcode(Instruction::BVS, AddressingMode::Relative, 2, 1),
    Opcode(Instruction::ADC, AddressingMode::IndirectIndexed, 5, 1),
    Opcode(Instruction::JAM, AddressingMode::Implied, 0, 0),
    Opcode(Instruction::RRA, AddressingMode::IndirectIndexed, 8, 0),
    Opcode(Instruction::NOP, AddressingMode::ZeroPageX, 4, 0),
    Opcode(Instruction::ADC, AddressingMode::ZeroPageX, 4, 0),
    Opcode(Instruction::ROR, AddressingMode::ZeroPageX, 6, 0),
    Opcode(Instruction::RRA, AddressingMode::ZeroPageX, 6, 0),
    Opcode(Instruction::SEI, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::ADC, AddressingMode::AbsoluteY, 4, 1),
    Opcode(Instruction::NOP, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::RRA, AddressingMode::AbsoluteY, 7, 0),
    Opcode(Instruction::NOP, AddressingMode::AbsoluteX, 4, 1),
    Opcode(Instruction::ADC, AddressingMode::AbsoluteX, 4, 1),
    Opcode(Instruction::ROR, AddressingMode::AbsoluteX, 7, 0),
    Opcode(Instruction::RRA, AddressingMode::AbsoluteX, 7, 0),
    // 0x80
    Opcode(Instruction::NOP, AddressingMode::Immediate, 2, 0),
    Opcode(Instruction::STA, AddressingMode::IndexedIndirect, 6, 0),
    Opcode(Instruction::NOP, AddressingMode::Immediate, 2, 0),
    Opcode(Instruction::SAX, AddressingMode::IndexedIndirect, 6, 0),
    Opcode(Instruction::STY, AddressingMode::ZeroPageIndexed, 3, 0),
    Opcode(Instruction::STA, AddressingMode::ZeroPageIndexed, 3, 0),
    Opcode(Instruction::STX, AddressingMode::ZeroPageIndexed, 3, 0),
    Opcode(Instruction::SAX, AddressingMode::ZeroPageIndexed, 3, 0),
    Opcode(Instruction::DEY, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::NOP, AddressingMode::Immediate, 2, 0),
    Opcode(Instruction::TXA, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::None, AddressingMode::None, 0, 0),
    Opcode(Instruction::STY, AddressingMode::Absolute, 4, 0),
    Opcode(Instruction::STA, AddressingMode::Absolute, 4, 0),
    Opcode(Instruction::STX, AddressingMode::Absolute, 4, 0),
    Opcode(Instruction::SAX, AddressingMode::Absolute, 4, 0),
    // 0x90
    Opcode(Instruction::BCC, AddressingMode::Relative, 2, 1),
    Opcode(Instruction::STA, AddressingMode::IndirectIndexed, 6, 0),
    Opcode(Instruction::JAM, AddressingMode::Implied, 0, 0),
    Opcode(Instruction::None, AddressingMode::None, 0, 0),
    Opcode(Instruction::STY, AddressingMode::ZeroPageX, 4, 0),
    Opcode(Instruction::STA, AddressingMode::ZeroPageX, 4, 0),
    Opcode(Instruction::STX, AddressingMode::ZeroPageY, 4, 0),
    Opcode(Instruction::SAX, AddressingMode::ZeroPageY, 4, 0),
    Opcode(Instruction::TYA, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::STA, AddressingMode::AbsoluteY, 5, 0),
    Opcode(Instruction::TXS, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::None, AddressingMode::None, 0, 0),
    Opcode(Instruction::None, AddressingMode::None, 0, 0),
    Opcode(Instruction::STA, AddressingMode::AbsoluteX, 5, 0),
    Opcode(Instruction::None, AddressingMode::None, 0, 0),
    Opcode(Instruction::None, AddressingMode::None, 0, 0),
    // 0xA0
    Opcode(Instruction::LDY, AddressingMode::Immediate, 2, 0),
    Opcode(Instruction::LDA, AddressingMode::IndexedIndirect, 6, 0),
    Opcode(Instruction::LDX, AddressingMode::Immediate, 2, 0),
    Opcode(Instruction::LAX, AddressingMode::IndexedIndirect, 6, 0),
    Opcode(Instruction::LDY, AddressingMode::ZeroPageIndexed, 3, 0),
    Opcode(Instruction::LDA, AddressingMode::ZeroPageIndexed, 3, 0),
    Opcode(Instruction::LDX, AddressingMode::ZeroPageIndexed, 3, 0),
    Opcode(Instruction::LAX, AddressingMode::ZeroPageIndexed, 3, 0),
    Opcode(Instruction::TAY, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::LDA, AddressingMode::Immediate, 2, 0),
    Opcode(Instruction::TAX, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::None, AddressingMode::None, 0, 0),
    Opcode(Instruction::LDY, AddressingMode::Absolute, 4, 0),
    Opcode(Instruction::LDA, AddressingMode::Absolute, 4, 0),
    Opcode(Instruction::LDX, AddressingMode::Absolute, 4, 0),
    Opcode(Instruction::LAX, AddressingMode::Absolute, 4, 0),
    // 0xB0
    Opcode(Instruction::BCS, AddressingMode::Relative, 2, 1),
    Opcode(Instruction::LDA, AddressingMode::IndirectIndexed, 5, 1),
    Opcode(Instruction::JAM, AddressingMode::Implied, 0, 0),
    Opcode(Instruction::LAX, AddressingMode::IndirectIndexed, 5, 1),
    Opcode(Instruction::LDY, AddressingMode::ZeroPageX, 4, 0),
    Opcode(Instruction::LDA, AddressingMode::ZeroPageX, 4, 0),
    Opcode(Instruction::LDX, AddressingMode::ZeroPageY, 4, 0),
    Opcode(Instruction::LAX, AddressingMode::ZeroPageY, 4, 0),
    Opcode(Instruction::CLV, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::LDA, AddressingMode::AbsoluteY, 4, 1),
    Opcode(Instruction::TSX, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::None, AddressingMode::None, 0, 0),
    Opcode(Instruction::LDY, AddressingMode::AbsoluteX, 4, 1),
    Opcode(Instruction::LDA, AddressingMode::AbsoluteX, 4, 1),
    Opcode(Instruction::LDX, AddressingMode::AbsoluteY, 4, 1),
    Opcode(Instruction::LAX, AddressingMode::AbsoluteY, 4, 1),
    // 0xC0
    Opcode(Instruction::CPY, AddressingMode::Immediate, 2, 0),
    Opcode(Instruction::CMP, AddressingMode::IndexedIndirect, 6, 0),
    Opcode(Instruction::NOP, AddressingMode::Immediate, 2, 0),
    Opcode(Instruction::DCP, AddressingMode::IndexedIndirect, 8, 0),
    Opcode(Instruction::CPY, AddressingMode::ZeroPageIndexed, 3, 0),
    Opcode(Instruction::CMP, AddressingMode::ZeroPageIndexed, 3, 0),
    Opcode(Instruction::DEC, AddressingMode::ZeroPageIndexed, 5, 0),
    Opcode(Instruction::DCP, AddressingMode::ZeroPageIndexed, 5, 0),
    Opcode(Instruction::INY, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::CMP, AddressingMode::Immediate, 2, 0),
    Opcode(Instruction::DEX, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::None, AddressingMode::None, 0, 0),
    Opcode(Instruction::CPY, AddressingMode::Absolute, 4, 0),
    Opcode(Instruction::CMP, AddressingMode::Absolute, 4, 0),
    Opcode(Instruction::DEC, AddressingMode::Absolute, 6, 0),
    Opcode(Instruction::DCP, AddressingMode::Absolute, 6, 0),
    // 0xD0
    Opcode(Instruction::BNE, AddressingMode::Relative, 2, 1),
    Opcode(Instruction::CMP, AddressingMode::IndirectIndexed, 5, 1),
    Opcode(Instruction::JAM, AddressingMode::Implied, 0, 0),
    Opcode(Instruction::DCP, AddressingMode::IndirectIndexed, 8, 0),
    Opcode(Instruction::NOP, AddressingMode::ZeroPageX, 4, 0),
    Opcode(Instruction::CMP, AddressingMode::ZeroPageX, 4, 0),
    Opcode(Instruction::DEC, AddressingMode::ZeroPageX, 6, 0),
    Opcode(Instruction::DCP, AddressingMode::ZeroPageX, 6, 0),
    Opcode(Instruction::CLD, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::CMP, AddressingMode::AbsoluteY, 4, 1),
    Opcode(Instruction::NOP, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::DCP, AddressingMode::AbsoluteY, 7, 0),
    Opcode(Instruction::NOP, AddressingMode::AbsoluteX, 4, 1),
    Opcode(Instruction::CMP, AddressingMode::AbsoluteX, 4, 1),
    Opcode(Instruction::DEC, AddressingMode::AbsoluteX, 7, 0),
    Opcode(Instruction::DCP, AddressingMode::AbsoluteX, 7, 0),
    // 0xE0
    Opcode(Instruction::CPX, AddressingMode::Immediate, 2, 0),
    Opcode(Instruction::SBC, AddressingMode::IndexedIndirect, 6, 0),
    Opcode(Instruction::NOP, AddressingMode::Immediate, 2, 0),
    Opcode(Instruction::ISB, AddressingMode::IndexedIndirect, 8, 0),
    Opcode(Instruction::CPX, AddressingMode::ZeroPageIndexed, 3, 0),
    Opcode(Instruction::SBC, AddressingMode::ZeroPageIndexed, 3, 0),
    Opcode(Instruction::INC, AddressingMode::ZeroPageIndexed, 5, 0),
    Opcode(Instruction::ISB, AddressingMode::ZeroPageIndexed, 5, 0),
    Opcode(Instruction::INX, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::SBC, AddressingMode::Immediate, 2, 0),
    Opcode(Instruction::NOP, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::SBC, AddressingMode::Immediate, 2, 0),
    Opcode(Instruction::CPX, AddressingMode::Absolute, 4, 0),
    Opcode(Instruction::SBC, AddressingMode::Absolute, 4, 0),
    Opcode(Instruction::INC, AddressingMode::Absolute, 6, 0),
    Opcode(Instruction::ISB, AddressingMode::Absolute, 6, 0),
    // 0xF0
    Opcode(Instruction::BEQ, AddressingMode::Relative, 2, 1),
    Opcode(Instruction::SBC, AddressingMode::IndirectIndexed, 5, 1),
    Opcode(Instruction::JAM, AddressingMode::Implied, 0, 0),
    Opcode(Instruction::ISB, AddressingMode::IndirectIndexed, 8, 0),
    Opcode(Instruction::NOP, AddressingMode::ZeroPageX, 4, 0),
    Opcode(Instruction::SBC, AddressingMode::ZeroPageX, 4, 0),
    Opcode(Instruction::INC, AddressingMode::ZeroPageX, 6, 0),
    Opcode(Instruction::ISB, AddressingMode::ZeroPageX, 6, 0),
    Opcode(Instruction::SED, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::SBC, AddressingMode::AbsoluteY, 4, 1),
    Opcode(Instruction::NOP, AddressingMode::Implied, 2, 0),
    Opcode(Instruction::ISB, AddressingMode::AbsoluteY, 7, 0),
    Opcode(Instruction::NOP, AddressingMode::AbsoluteX, 4, 1),
    Opcode(Instruction::SBC, AddressingMode::AbsoluteX, 4, 1),
    Opcode(Instruction::INC, AddressingMode::AbsoluteX, 7, 0),
    Opcode(Instruction::ISB, AddressingMode::AbsoluteX, 7, 0),
];
