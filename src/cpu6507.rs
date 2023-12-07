use crate::bus::Bus;
use crate::opcode::{AddressingMode, Instruction, Opcode, OPCODES};
use log::{debug, info};
use std::{env, process};

const STACK_INIT: u8 = 0xff;
const LOW_NIBBLE_MASK: u16 = 0x0F;
const HIGH_NIBBLE_MASK: u16 = 0xF0;

lazy_static::lazy_static! {
    static ref CPU6507_DEBUG: bool = match env::var("CPU6507_DEBUG") {
        Ok(val) => !val.is_empty() && val != "0",
        Err(_) => false,
    };
}

fn pages_differ(addr_a: u16, addr_b: u16) -> bool {
    (addr_a & 0xff00) != (addr_b & 0xff00)
}

#[allow(dead_code)]
mod status {
    use modular_bitfield::bitfield;
    #[bitfield(bits = 8)]
    pub(crate) struct StatusRegisterFlags {
        pub c: bool, // Carry flag (1 if last operation resulted in carry, borrow, or extend beyond MSB)
        pub z: bool, // Zero flag (1 if result of last operation was zero)
        pub i: bool, // Interrupt disable flag (1 if interrupts are disabled)
        pub d: bool, // Decimal mode flag (1 if CPU is in BCD mode)
        pub b: bool, // Software interrupt (BRK) flag
        pub u: bool, // Unused flag (ignored)
        pub v: bool, // Overflow flag (1 if signed arithmetic result is too large or too small)
        pub s: bool, // Sign flag (1 if result of last operation was negative)
    }
}
use status::StatusRegisterFlags;

pub(crate) struct CPU6507 {
    bus: Box<dyn Bus>,

    // Main registers
    pub a: u8, // Accumulator
    pub x: u8, // X Index
    pub y: u8, // Y Index

    // Status register flags
    flags: StatusRegisterFlags,

    // Program counter
    pub pc: u16,

    // Stack pointer
    sp: u8,

    // Total number of cycles executed
    cycles: u64,

    current_instruction: Option<Instruction>,
    current_addr: u16,
    current_addr_mode: AddressingMode,
    current_cycles: u64,
}

impl Bus for CPU6507 {
    fn read(&mut self, addr: u16) -> u8 {
        // The 6507 only had 13 address lines connected.
        self.bus.read(addr & 0x1fff)
    }

    fn write(&mut self, addr: u16, val: u8) {
        // The 6507 only had 13 address lines connected.
        self.bus.write(addr & 0x1fff, val);
    }
}

impl CPU6507 {
    pub fn new(bus: Box<dyn Bus>) -> Self {
        Self {
            bus,

            a: 0,
            x: 0,
            y: 0,

            flags: StatusRegisterFlags::new(),

            pc: 0x0000,

            sp: STACK_INIT,

            cycles: 0,

            current_instruction: None,
            current_addr: 0x0000,
            current_addr_mode: AddressingMode::Accumulator,
            current_cycles: 0,
        }
    }

    pub fn reset(&mut self) {
        let lo = self.read(0xFFFC) as u16;
        let hi = self.read(0xFFFD) as u16;
        let addr = (hi << 8) | lo;
        self.pc = addr;
        info!("PC: 0x{:04X}", self.pc);

        self.set_flags(0x24);

        self.sp = STACK_INIT;
        self.a = 0;
        self.x = 0;
        self.y = 0;

        self.cycles = 0;
    }

    fn calculate_absolute_address(&mut self, pc: u16) -> u16 {
        let lo = self.read(pc + 1) as u16;
        let hi = self.read(pc + 2) as u16;
        (hi << 8) | lo
    }

    fn calculate_indirect_address(&mut self, addr: u16) -> u16 {
        let lo = self.read(addr) as u16;
        let hi = if addr & 0xff == 0xff {
            self.read(addr & 0xff00) as u16
        } else {
            self.read(addr + 1) as u16
        };
        (hi << 8) | lo
    }

    fn get_data(&mut self, addr_mode: &AddressingMode) -> (u16, bool) {
        let pc = self.pc;
        let next_pc = self.pc + addr_mode.n_bytes() as u16;

        match addr_mode {
            AddressingMode::Immediate => {
                let addr = pc + 1;
                (addr, false)
            }
            AddressingMode::Absolute => {
                let addr = self.calculate_absolute_address(pc);
                (addr, false)
            }
            AddressingMode::Implied => (0, false),
            AddressingMode::Accumulator => (0, false),
            AddressingMode::ZeroPageIndexed => {
                let addr = self.read(pc + 1) as u16;
                (addr, false)
            }
            AddressingMode::Relative => {
                let offset = self.read(pc + 1) as u16;

                // NOTE This has to be based off the program counter, _after_
                // it has been advanced, but before the instruction is
                // being executed. I don't know why though?

                // All of this casting is to handle negative offsets
                (((next_pc as i16) + (offset as i8 as i16)) as u16, false)
            }
            AddressingMode::AbsoluteX => {
                let addr = self.calculate_absolute_address(pc);
                let n_addr = addr.wrapping_add(self.x as u16);
                (n_addr, pages_differ(addr, n_addr))
            }
            AddressingMode::AbsoluteY => {
                let addr = self.calculate_absolute_address(pc);
                let n_addr = addr.wrapping_add(self.y as u16);
                (n_addr, pages_differ(addr, n_addr))
            }
            AddressingMode::Indirect => {
                let addr = self.calculate_absolute_address(pc);
                let addr = self.calculate_indirect_address(addr);

                (addr, false)
            }
            AddressingMode::ZeroPageX => {
                let addr = self.read(pc + 1).wrapping_add(self.x) as u16;
                (addr, false)
            }
            AddressingMode::ZeroPageY => {
                let addr = self.read(pc + 1).wrapping_add(self.y) as u16;
                (addr, false)
            }
            AddressingMode::IndexedIndirect => {
                let lo = self.read(pc + 1);
                let addr = lo.wrapping_add(self.x) as u16;
                let addr = self.calculate_indirect_address(addr);
                (addr, false)
            }
            AddressingMode::IndirectIndexed => {
                let addr = self.read(pc + 1) as u16;
                let addr = self.calculate_indirect_address(addr);
                let n_addr = addr.wrapping_add(self.y as u16);
                (n_addr, pages_differ(addr, n_addr))
            }
            _ => panic!("Bad addressing mode {:?}", addr_mode),
        }
    }

    fn flags(&self) -> u8 {
        (self.flags.c() as u8)
            | ((self.flags.z() as u8) << 1)
            | ((self.flags.i() as u8) << 2)
            | ((self.flags.d() as u8) << 3)
            | ((self.flags.b() as u8) << 4)
            | ((self.flags.u() as u8) << 5)
            | ((self.flags.v() as u8) << 6)
            | ((self.flags.s() as u8) << 7)
    }

    fn set_flags(&mut self, val: u8) {
        self.flags.set_c(val & 0x01 == 1);
        self.flags.set_z((val >> 1 & 0x01) == 1);
        self.flags.set_i((val >> 2 & 0x01) == 1);
        self.flags.set_d((val >> 3 & 0x01) == 1);
        self.flags.set_b((val >> 4 & 0x01) == 1);
        self.flags.set_u((val >> 5 & 0x01) == 1);
        self.flags.set_v((val >> 6 & 0x01) == 1);
        self.flags.set_s((val >> 7 & 0x01) == 1);
    }

    fn stack_push8(&mut self, val: u8) {
        // The stack page exists from 0x0080 to 0x00FF
        let addr = self.sp as u16;
        self.write(addr, val);

        let n = self.sp.wrapping_sub(1);
        self.sp = n;
    }

    fn stack_pop8(&mut self) -> u8 {
        let n = self.sp.wrapping_add(1);
        self.sp = n;

        // The stack page exists from 0x0080 to 0x00FF
        let addr = self.sp as u16;

        self.read(addr)
    }

    fn stack_push16(&mut self, val: u16) {
        let hi = (val >> 8) as u8;
        self.stack_push8(hi);

        let lo = (val & 0x00ff) as u8;
        self.stack_push8(lo);
    }

    fn stack_pop16(&mut self) -> u16 {
        let lo = self.stack_pop8() as u16;
        let hi = self.stack_pop8() as u16;
        (hi << 8) | lo
    }

    fn update_sz(&mut self, val: u8) {
        self.flags.set_s(val & 0x80 != 0);
        self.flags.set_z(val == 0);
    }

    fn add_branch_cycles(&mut self, pc: u16, addr: u16) {
        self.current_cycles += 1;
        self.cycles += 1;

        // It costs an extra cycle to branch to a different page.
        if (pc & 0xff00) != (addr & 0xff00) {
            self.current_cycles += 1;
            self.cycles += 1;
        }
    }

    fn fetch_and_decode(&mut self) -> u64 {
        // Read opcode from memory
        let opcode = self.read(self.pc);

        // Get opcode information from the lookup table
        let op = &OPCODES[opcode as usize];

        // Destructure Opcode for better readability
        let Opcode(inst, addr_mode, cycles, extra_cycles) = op;

        // Get address and check for page crossing
        let (addr, page_crossed) = self.get_data(addr_mode);

        // Update program counter
        self.pc += addr_mode.n_bytes() as u16;

        // Update CPU state
        self.current_instruction = Some(*inst);
        self.current_addr = addr;
        self.current_addr_mode = *addr_mode;

        // Calculate total cycles, considering page crossing
        cycles + if page_crossed { extra_cycles } else { &0 }
    }

    fn execute(&mut self) {
        if let Some(inst) = self.current_instruction {
            let addr = self.current_addr;
            let addr_mode = self.current_addr_mode;

            match inst {
                Instruction::ADC => self.adc(addr),
                Instruction::ANC => self.anc(addr),
                Instruction::AND => self.and(addr),
                Instruction::ASL => self.asl(addr, addr_mode),
                Instruction::BCC => self.bcc(addr),
                Instruction::BCS => self.bcs(addr),
                Instruction::BEQ => self.beq(addr),
                Instruction::BIT => self.bit(addr),
                Instruction::BMI => self.bmi(addr),
                Instruction::BNE => self.bne(addr),
                Instruction::BPL => self.bpl(addr),
                Instruction::BRK => self.brk(),
                Instruction::BVC => self.bvc(addr),
                Instruction::BVS => self.bvs(addr),
                Instruction::CLC => self.clc(),
                Instruction::CLD => self.cld(),
                Instruction::CLI => self.cli(),
                Instruction::CLV => self.clv(),
                Instruction::CMP => self.cmp(addr),
                Instruction::CPX => self.cpx(addr),
                Instruction::CPY => self.cpy(addr),
                Instruction::DCP => self.dcp(addr),
                Instruction::DEC => self.dec(addr),
                Instruction::DEX => self.dex(),
                Instruction::DEY => self.dey(),
                Instruction::EOR => self.eor(addr),
                Instruction::INC => self.inc(addr),
                Instruction::INX => self.inx(),
                Instruction::INY => self.iny(),
                Instruction::ISB => self.isb(addr),
                Instruction::JAM => self.jam(),
                Instruction::JMP => self.jmp(addr),
                Instruction::JSR => self.jsr(addr),
                Instruction::LAX => self.lax(addr),
                Instruction::LDA => self.lda(addr),
                Instruction::LDX => self.ldx(addr),
                Instruction::LDY => self.ldy(addr),
                Instruction::LSR => self.lsr(addr, addr_mode),
                Instruction::NOP => self.nop(),
                Instruction::ORA => self.ora(addr),
                Instruction::PHA => self.pha(),
                Instruction::PHP => self.php(),
                Instruction::PLA => self.pla(),
                Instruction::PLP => self.plp(),
                Instruction::RLA => self.rla(addr, addr_mode),
                Instruction::ROL => self.rol(addr, addr_mode),
                Instruction::ROR => self.ror(addr, addr_mode),
                Instruction::RRA => self.rra(addr, addr_mode),
                Instruction::RTI => self.rti(),
                Instruction::RTS => self.rts(),
                Instruction::SAX => self.sax(addr),
                Instruction::SBC => self.sbc(addr),
                Instruction::SEC => self.sec(),
                Instruction::SED => self.sed(),
                Instruction::SEI => self.sei(),
                Instruction::SLO => self.slo(addr, addr_mode),
                Instruction::SRE => self.sre(addr, addr_mode),
                Instruction::STA => self.sta(addr),
                Instruction::STX => self.stx(addr),
                Instruction::STY => self.sty(addr),
                Instruction::TAX => self.tax(),
                Instruction::TAY => self.tay(),
                Instruction::TSX => self.tsx(),
                Instruction::TXA => self.txa(),
                Instruction::TXS => self.txs(),
                Instruction::TYA => self.tya(),
                _ => panic!("unsupported instruction {:?}", inst),
            }

            self.current_instruction = None;
        }
    }

    pub fn clock(&mut self) {
        if self.current_cycles == 0 {
            self.current_cycles += self.fetch_and_decode();
        }

        self.current_cycles -= 1;
        if self.current_cycles == 0 {
            self.execute();
        }
    }

    //
    // Legal instructions
    //

    fn adc(&mut self, addr: u16) {
        let val = self.read(addr);

        if self.flags.d() {
            self.adc_bcd(val);
        } else {
            let n = (self.a as u16) + (val as u16) + (self.flags.c() as u16);
            let a = (n & 0x00ff) as u8;

            self.update_sz(a);
            self.flags.set_c(n > 0xff);

            // The first condition checks if the sign of the accumulator and the
            // the sign of value that we're adding are the same.
            //
            // The second condition checks if the result of the addition has a
            // different sign to either of the values we added together.
            self.flags
                .set_v(((self.a ^ val) & 0x80 == 0) && ((self.a ^ a) & 0x80 != 0));

            self.a = a;
        }
    }

    fn adc_bcd(&mut self, val: u8) {
        const BCD_CARRY: u16 = 0x10;
        const BCD_SKIP_VALUES: u16 = 0x60;

        let mut lo = (self.a as u16 & LOW_NIBBLE_MASK)
            + (val as u16 & LOW_NIBBLE_MASK)
            + (self.flags.c() as u16);
        let mut hi = (self.a as u16 & HIGH_NIBBLE_MASK) + (val as u16 & HIGH_NIBBLE_MASK);

        // In BCD, values 0x0A to 0x0F are invalid, so we add 1 to the high nibble for the
        // carry, and the low nibble has to skip 6 values for A-F.
        if lo > 0x09 {
            hi += BCD_CARRY;
            lo += BCD_SKIP_VALUES;
        }

        self.flags.set_s((hi & 0x80) != 0);
        self.flags.set_z(((lo + hi) & 0xFF) != 0);
        self.flags
            .set_v(((self.a ^ val) & 0x80 == 0) && ((self.a ^ hi as u8) & 0x80 != 0));

        // 0xA0 to 0xF0 are invalid for the high nibble, so we need to skip 6 values of the
        // high nibble.
        if hi > 0x90 {
            hi += BCD_SKIP_VALUES;
        }

        self.flags.set_c((hi & 0xFF00) != 0);
        self.a = ((lo & LOW_NIBBLE_MASK) | (hi & HIGH_NIBBLE_MASK)) as u8;
    }

    fn and(&mut self, addr: u16) {
        let val = self.read(addr);
        self.a &= val;
        let a = self.a;
        self.update_sz(a);
    }

    fn asl(&mut self, addr: u16, addr_mode: AddressingMode) {
        let val = match addr_mode {
            AddressingMode::Accumulator => self.a,
            _ => self.read(addr),
        };

        self.flags.set_c(val & 0x80 != 0);
        let n = val << 1;

        match addr_mode {
            AddressingMode::Accumulator => self.a = n,
            _ => self.write(addr, n),
        };

        self.update_sz(n);
    }

    fn branch_if(&mut self, condition: bool, addr: u16) {
        if condition {
            let pc = self.pc;
            self.add_branch_cycles(pc, addr);
            self.pc = addr;
        }
    }

    fn bcc(&mut self, addr: u16) {
        self.branch_if(!self.flags.c(), addr);
    }

    fn bcs(&mut self, addr: u16) {
        self.branch_if(self.flags.c(), addr);
    }

    fn beq(&mut self, addr: u16) {
        self.branch_if(self.flags.z(), addr);
    }

    fn bit(&mut self, addr: u16) {
        let val = self.read(addr);
        self.flags.set_s(val & 0x80 != 0);
        self.flags.set_v((val >> 0x06 & 0x01) == 1);
        let f = self.a & val;
        self.flags.set_z(f == 0);
    }

    fn bmi(&mut self, addr: u16) {
        self.branch_if(self.flags.s(), addr);
    }

    fn bne(&mut self, addr: u16) {
        self.branch_if(!self.flags.z(), addr);
    }

    fn bpl(&mut self, addr: u16) {
        self.branch_if(!self.flags.s(), addr);
    }

    fn brk(&mut self) {
        let pc = self.pc + 1;
        self.stack_push16(pc);

        self.flags.set_b(true);

        let flags = self.flags() | 0x10;
        self.stack_push8(flags);

        self.flags.set_i(true);

        let lo = self.read(0xFFFE) as u16;
        let hi = self.read(0xFFFF) as u16;
        let pc = (hi << 8) | lo;
        self.pc = pc;
    }

    fn bvc(&mut self, addr: u16) {
        self.branch_if(!self.flags.v(), addr);
    }

    fn bvs(&mut self, addr: u16) {
        self.branch_if(self.flags.v(), addr);
    }

    fn clc(&mut self) {
        self.flags.set_c(false);
    }

    fn cld(&mut self) {
        self.flags.set_d(false);
    }

    fn cli(&mut self) {
        self.flags.set_i(false);
    }

    fn clv(&mut self) {
        self.flags.set_v(false);
    }

    fn cmp(&mut self, addr: u16) {
        let val = self.read(addr);
        let n = self.a.wrapping_sub(val);
        self.flags.set_c(self.a >= val);
        self.update_sz(n);
    }

    fn cpx(&mut self, addr: u16) {
        let val = self.read(addr);
        let n = self.x.wrapping_sub(val);
        self.update_sz(n);
        self.flags.set_c(self.x >= val);
    }

    fn cpy(&mut self, addr: u16) {
        let val = self.read(addr);
        let n = self.y.wrapping_sub(val);
        self.update_sz(n);
        self.flags.set_c(self.y >= val);
    }

    fn dec(&mut self, addr: u16) {
        let val = self.read(addr);
        let n = val.wrapping_sub(1);
        self.update_sz(n);
        self.write(addr, n);
    }

    fn dex(&mut self) {
        self.x = self.x.wrapping_sub(1);
        self.update_sz(self.x);
    }

    fn dey(&mut self) {
        self.y = self.y.wrapping_sub(1);
        self.update_sz(self.y);
    }

    fn eor(&mut self, addr: u16) {
        let val = self.read(addr);
        let val = val ^ self.a;
        self.a = val;
        self.update_sz(val);
    }

    fn inc(&mut self, addr: u16) {
        let val = self.read(addr);
        let n = val.wrapping_add(1);
        self.write(addr, n);
        self.update_sz(n);
    }

    fn inx(&mut self) {
        self.x = self.x.wrapping_add(1);
        self.update_sz(self.x);
    }

    fn iny(&mut self) {
        self.y = self.y.wrapping_add(1);
        self.update_sz(self.y);
    }

    fn jmp(&mut self, addr: u16) {
        self.pc = addr;
    }

    fn jsr(&mut self, addr: u16) {
        let retaddr = self.pc - 1;
        self.stack_push16(retaddr);
        self.pc = addr;
    }

    fn lda(&mut self, addr: u16) {
        self.a = self.read(addr);
        self.update_sz(self.a);
    }

    fn ldx(&mut self, addr: u16) {
        self.x = self.read(addr);
        self.update_sz(self.x);
    }

    fn ldy(&mut self, addr: u16) {
        self.y = self.read(addr);
        self.update_sz(self.y);
    }

    fn lsr(&mut self, addr: u16, addr_mode: AddressingMode) {
        let val = match addr_mode {
            AddressingMode::Accumulator => self.a,
            _ => self.read(addr),
        };

        self.flags.set_c(val & 0x01 == 1);
        let n = val >> 1;
        self.update_sz(n);

        match addr_mode {
            AddressingMode::Accumulator => self.a = n,
            _ => self.write(addr, n),
        };
    }

    fn nop(&self) {}

    fn ora(&mut self, addr: u16) {
        let val = self.read(addr);
        let na = self.a | val;
        self.a = na;
        self.update_sz(na);
    }

    fn pha(&mut self) {
        let a = self.a;
        self.stack_push8(a);
    }

    fn php(&mut self) {
        // https://wiki.nesdev.com/w/index.php/CPU_status_flag_behavior
        // According to the above link, the PHP instruction sets bits 4 and 5 on
        // the value it pushes onto the stack.
        // The PLP call later will ignore these bits.
        let flags = self.flags() | 0x10;
        self.stack_push8(flags);
    }

    fn pla(&mut self) {
        let rv = self.stack_pop8();
        self.a = rv;
        self.update_sz(rv);
    }

    fn plp(&mut self) {
        let p = self.stack_pop8() & 0xef | 0x20;
        self.set_flags(p);
    }

    fn rotate(&mut self, addr: u16, addr_mode: AddressingMode, shift_left: bool) {
        const BIT_7_MASK: u8 = 0x80;
        const BIT_1_MASK: u8 = 0x01;

        let val = match addr_mode {
            AddressingMode::Accumulator => self.a,
            _ => self.read(addr),
        };

        let n = if shift_left {
            (val << 1) | self.flags.c() as u8
        } else {
            (val >> 1) | (self.flags.c() as u8) << 7
        };

        self.flags
            .set_c((val & (if shift_left { BIT_7_MASK } else { BIT_1_MASK })) != 0);
        self.update_sz(n);

        match addr_mode {
            AddressingMode::Accumulator => self.a = n,
            _ => self.write(addr, n),
        };
    }

    fn rol(&mut self, addr: u16, addr_mode: AddressingMode) {
        self.rotate(addr, addr_mode, true);
    }

    fn ror(&mut self, addr: u16, addr_mode: AddressingMode) {
        self.rotate(addr, addr_mode, false);
    }

    fn rti(&mut self) {
        let flags = self.stack_pop8() & 0xef | 0x20;
        self.set_flags(flags);

        let retaddr = self.stack_pop16();
        self.pc = retaddr;
    }

    fn rts(&mut self) {
        let retaddr = self.stack_pop16();
        self.pc = retaddr + 1;
    }

    fn sbc(&mut self, addr: u16) {
        let val = self.read(addr);

        if self.flags.d() {
            // http://www.6502.org/tutorials/decimal_mode.html
            self.sbc_decimal(val);
        } else {
            let val = !val;
            let n = (self.a as u16) + (val as u16) + (self.flags.c() as u16);
            let a = (n & 0x00ff) as u8;

            self.update_sz(a);
            self.flags.set_c(n > 0xff);

            // The first condition checks if the sign of the accumulator and the
            // the sign of value that we're adding are the same.
            //
            // The second condition checks if the result of the addition has a
            // different sign to either of the values we added together.
            self.flags
                .set_v(((self.a ^ val) & 0x80 == 0) && ((self.a ^ a) & 0x80 != 0));

            self.a = a;
        }
    }

    fn sbc_decimal(&mut self, val: u8) {
        const DECIMAL_CARRY: i16 = 0x60;
        const DECIMAL_SUBTRACT: i16 = 0x06;

        let borrow = !self.flags.c() as i16;
        let temp = (self.a as i16) - (val as i16) - borrow;
        let lo = ((self.a as i16) & LOW_NIBBLE_MASK as i16)
            - ((val as i16) & LOW_NIBBLE_MASK as i16)
            - borrow;

        let temp = if temp < 0 { temp - DECIMAL_CARRY } else { temp };
        let temp = if lo < 0 {
            temp - DECIMAL_SUBTRACT
        } else {
            temp
        };

        debug!(
            "SBC  {:02X} - {:02X} - {:02X} = {:04X}",
            self.a, val, borrow as u8, temp
        );

        let a = (temp & 0xFF) as u8;
        self.update_sz(a);
        self.flags
            .set_v(((self.a ^ val) & 0x80 == 0) && ((self.a ^ a) & 0x80 != 0));
        self.flags.set_c(temp >= 0);
        self.a = a;
    }

    fn sec(&mut self) {
        self.flags.set_c(true);
    }

    fn sed(&mut self) {
        self.flags.set_d(true);
    }

    fn sei(&mut self) {
        self.flags.set_i(true);
    }

    fn sta(&mut self, addr: u16) {
        self.write(addr, self.a);
    }

    fn stx(&mut self, addr: u16) {
        self.write(addr, self.x);
    }

    fn sty(&mut self, addr: u16) {
        self.write(addr, self.y);
    }

    fn tax(&mut self) {
        let n = self.a;
        self.x = n;
        self.update_sz(n);
    }

    fn tay(&mut self) {
        let n = self.a;
        self.y = n;
        self.update_sz(n);
    }

    fn tsx(&mut self) {
        let s = self.sp;
        self.update_sz(s);
        self.x = s;
    }

    fn txa(&mut self) {
        let n = self.x;
        self.a = n;
        self.update_sz(n);
    }

    fn txs(&mut self) {
        self.sp = self.x;
    }

    fn tya(&mut self) {
        let n = self.y;
        self.a = n;
        self.update_sz(n);
    }

    //
    // Illegal instructions
    //

    fn anc(&mut self, addr: u16) {
        let val = self.read(addr);
        let a = self.a & val;
        self.a = a;
        self.update_sz(a);
        self.flags.set_c((a as i8) < 0);
    }

    fn lax(&mut self, addr: u16) {
        let val = self.read(addr);
        self.a = val;
        self.x = val;
        self.update_sz(val);
    }

    fn sax(&mut self, addr: u16) {
        let val = self.x & self.a;
        self.write(addr, val);
    }

    fn dcp(&mut self, addr: u16) {
        // Copied from dec
        let val = self.read(addr);
        let n = val.wrapping_sub(1);
        self.update_sz(n);
        self.write(addr, n);

        // Copied from cmp
        let n = self.a.wrapping_sub(n);
        self.flags.set_c(self.a >= n);
        self.update_sz(n);
    }

    fn isb(&mut self, addr: u16) {
        // Copied from inc
        let val = self.read(addr);
        let n = val.wrapping_add(1);
        self.write(addr, n);
        self.update_sz(n);

        // Copied from sbc
        let val = n;
        let n: i16 = (self.a as i16)
            .wrapping_sub(val as i16)
            .wrapping_sub(1 - self.flags.c() as i16);

        let a = n as u8;
        self.update_sz(a);
        self.flags
            .set_v(((self.a ^ val) & 0x80 > 0) && ((self.a ^ n as u8) & 0x80 > 0));
        self.a = a;
        self.flags.set_c(n >= 0);
    }

    fn slo(&mut self, addr: u16, addr_mode: AddressingMode) {
        // Copied from asl
        let val = self.read(addr);
        self.flags.set_c(val & 0x80 != 0);
        let n = val << 1;

        match addr_mode {
            AddressingMode::Accumulator => self.a = n,
            _ => self.write(addr, n),
        };

        self.update_sz(n);

        // Copied from ora
        let val = n;
        let na = self.a | val;
        self.a = na;
        self.update_sz(na);
    }

    fn rla(&mut self, addr: u16, addr_mode: AddressingMode) {
        // Copied from rol
        let val = self.read(addr);
        let c = self.flags.c();
        self.flags.set_c(val & 0x80 != 0);
        let n = (val << 1) | (c as u8);
        self.update_sz(n);

        match addr_mode {
            AddressingMode::Accumulator => self.a = n,
            _ => self.write(addr, n),
        };

        // Copied from and
        let val = n;
        self.a &= val;
        let a = self.a;
        self.update_sz(a);
    }

    fn sre(&mut self, addr: u16, addr_mode: AddressingMode) {
        // Copied from lsr
        let val = self.read(addr);
        self.flags.set_c(val & 0x01 == 1);
        let n = val >> 1;
        self.update_sz(n);

        match addr_mode {
            AddressingMode::Accumulator => self.a = n,
            _ => self.write(addr, n),
        };

        // Copied from eor
        let val = n;
        let val = val ^ self.a;
        self.a = val;
        self.update_sz(val);
    }

    fn rra(&mut self, addr: u16, addr_mode: AddressingMode) {
        // Copied from ror
        let val = self.read(addr);
        let c = self.flags.c();
        self.flags.set_c(val & 0x01 == 1);
        let n = (val >> 1) | ((c as u8) << 7);
        self.update_sz(n);

        match addr_mode {
            AddressingMode::Accumulator => self.a = n,
            _ => self.write(addr, n),
        };

        // Copied from adc
        let val = n;
        let n = (val as u16) + (self.a as u16) + (self.flags.c() as u16);
        let a = (n & 0xff) as u8;
        self.update_sz(a);
        self.flags.set_c(n > 0xff);
        self.flags
            .set_v(((self.a ^ val) & 0x80 == 0) && ((self.a ^ n as u8) & 0x80 > 0));
        self.a = a;
    }

    fn jam(&mut self) {
        process::exit(0);
    }
}
