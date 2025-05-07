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
