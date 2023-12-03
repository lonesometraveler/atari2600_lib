use crate::{bus::Bus, memory::PiaAddress};

#[allow(clippy::upper_case_acronyms)]
// The RIOT (RAM/IO/Timer) chip. Also known as the PIA. It's a MOS 6532 chip.
pub(crate) struct RIOT {
    ram: [u8; 128],

    // Registers
    swcha: u8,
    swacnt: u8,
    swchb: u8,
    swbcnt: u8,
    intim: u8,
    instat: u8,

    // Internal things
    port_a: u8,
    port_b: u8,

    resolution: usize,
    cycle_count: usize,
}

impl Default for RIOT {
    fn default() -> Self {
        // Initialise port B with the P0 and P1 difficulty bits set to 1. Should probably make this
        // switchable in the interface. We also set the color switch to color, just because that's a
        // nicer default in 2023.
        let port_b = 0b1100_1000;

        Self {
            ram: [0; 128],

            swcha: 0,
            swacnt: 0,
            swchb: 0,
            swbcnt: 0,
            intim: 0,
            instat: 0,

            port_a: 0,
            port_b,
            resolution: 0,
            cycle_count: 0,
        }
    }
}

impl RIOT {
    pub fn new() -> Self {
        Self::default()
    }

    //
    // Console switches
    //
    pub fn color(&mut self) {
        if (self.port_b & 0b0000_1000) != 0 {
            self.port_b &= 0b1111_0111;
        } else {
            self.port_b |= 0b0000_1000
        }
    }

    pub fn reset(&mut self, pressed: bool) {
        if pressed {
            self.port_b &= 0b1111_1110;
        } else {
            self.port_b |= 0b0000_0001;
        }
    }

    pub fn select(&mut self, pressed: bool) {
        if pressed {
            self.port_b &= 0b1111_1101;
        } else {
            self.port_b |= 0b0000_0010;
        }
    }

    //
    // Player 0 joystick controls
    //
    pub fn up(&mut self, pressed: bool) {
        if pressed {
            self.port_a &= 0b1110_1111
        } else {
            self.port_a |= 0b0001_0000
        }
    }

    pub fn down(&mut self, pressed: bool) {
        if pressed {
            self.port_a &= 0b1101_1111
        } else {
            self.port_a |= 0b0010_0000
        }
    }

    pub fn left(&mut self, pressed: bool) {
        if pressed {
            self.port_a &= 0b1011_1111
        } else {
            self.port_a |= 0b0100_0000
        }
    }

    pub fn right(&mut self, pressed: bool) {
        if pressed {
            self.port_a &= 0b0111_1111
        } else {
            self.port_a |= 0b1000_0000
        }
    }

    pub fn clock(&mut self) {
        self.cycle_count -= 1;

        if self.cycle_count == 0 {
            self.decrement();
        }
    }

    // Initialises the timer at a certain resolution. The resolution determines how many clocks of
    // the RIOT are required to decrement the timer value denoted by the INTIM register.
    fn init_timer(&mut self, val: u8, resolution: usize) {
        self.intim = val;
        self.resolution = resolution;
        self.decrement();
    }

    fn decrement(&mut self) {
        let (new_intim, underflowed) = self.intim.overflowing_sub(1);
        self.intim = new_intim;

        // If we've successfully decremented the timer down to zero, set a flag in the INSTAT
        // register to record this fact.
        if underflowed {
            self.instat = 0b1100_0000;

            // Once when the timer does underflow, it restarts at FFh, and is then decremented once
            // per clock cycle, regardless of the selected interval.
            self.resolution = 1;
        }

        self.cycle_count = self.resolution;
    }
}

impl Bus for RIOT {
    fn read(&mut self, address: u16) -> u8 {
        let pia_address = PiaAddress::from_address(address).unwrap();
        use PiaAddress::*;
        match pia_address {
            RAM => self.ram[address as usize],
            SWCHA => {
                // The bits of SWACNT set the data direction for the corresponding bits of SWCHA, 0
                // being for input, and 1 for output.
                // So all this faffing about is to enforce this.
                // This is also the case for SWCHB/SWBCNT.
                (self.swcha & self.swacnt) | (self.port_a & (self.swacnt ^ 0xff))
            }
            SWCHB => (self.swchb & self.swbcnt) | (self.port_b & (self.swbcnt ^ 0xff)),
            INTIM => self.intim,
            INSTAT => {
                let rv = self.instat;
                self.instat &= 0b1011_1111;
                rv
            }
            _ => 0,
        }
    }

    fn write(&mut self, address: u16, val: u8) {
        let pia_address = PiaAddress::from_address(address).unwrap();
        use PiaAddress::*;
        match pia_address {
            RAM => self.ram[address as usize] = val,
            SWACNT => self.swacnt = val,
            SWBCNT => self.swbcnt = val,
            TIM1T => self.init_timer(val, 1),
            TIM8T => self.init_timer(val, 8),
            TIM64T => self.init_timer(val, 64),
            T1024T => self.init_timer(val, 1024),
            _ => {}
        }
    }
}
