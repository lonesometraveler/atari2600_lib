use super::SharedColor;
use crate::tia::counter::Counter;

pub(crate) struct Playfield {
    colors: SharedColor,
    ctr: Counter,

    // 20-bit playfield
    // .... | .... .... | .... ....
    // PF0  |    PF1    |    PF2
    pf0: u8,
    pf1: u8,
    pf2: u8,
    pf: [bool; 20],

    horizontal_mirror: bool,
    score_mode: bool,
    priority: bool,

    graphic_bit_value: Option<u8>,
}

impl Playfield {
    pub fn new(colors: SharedColor) -> Self {
        Self {
            colors,
            ctr: Counter::default(),

            pf0: 0,
            pf1: 0,
            pf2: 0,
            pf: [false; 20],

            horizontal_mirror: false,
            score_mode: false,
            priority: false,

            graphic_bit_value: None,
        }
    }

    pub fn set_pf0(&mut self, val: u8) {
        self.pf0 = val;

        // PF0 is the first 4 bits, in big-endian order
        for x in 0..4 {
            self.pf[x] = (self.pf0 >> (x + 4)) & 0x01 != 0;
        }
    }

    pub fn set_pf1(&mut self, val: u8) {
        self.pf1 = val;

        // PF1 is the next 8 bits, in little-endian order
        for x in 0..8 {
            self.pf[x + 4] = (self.pf1 >> (7 - x)) & 0x01 != 0;
        }
    }

    pub fn set_pf2(&mut self, val: u8) {
        self.pf2 = val;

        // PF2 is the last 8 bits, in big-endian order
        for x in 0..8 {
            self.pf[x + 12] = (self.pf2 >> x) & 0x01 != 0;
        }
    }

    pub fn set_control(&mut self, val: u8) {
        self.horizontal_mirror = (val & 0x01) != 0;
        self.priority = (val & 0x04) != 0;
        self.score_mode = (val & 0x02) != 0 && !self.priority;
    }

    fn tick_graphic_circuit(&mut self) {
        let ctr = self.ctr.value() as usize;
        let pf_x = ctr % 20;

        if ctr < 20 {
            self.graphic_bit_value = match (self.pf[pf_x], self.score_mode) {
                (true, true) => Some(self.colors.borrow().colup0()),
                (true, false) => Some(self.colors.borrow().colupf()),
                (false, _) => None,
            };
        } else {
            // The playfield also makes up the right-most side of the
            // screen, optionally mirrored horizontally as denoted by the
            // CTRLPF register.
            let idx = if self.horizontal_mirror {
                self.pf.len() - 1 - pf_x
            } else {
                pf_x
            };

            self.graphic_bit_value = match (self.pf[idx], self.score_mode) {
                (true, true) => Some(self.colors.borrow().colup1()),
                (true, false) => Some(self.colors.borrow().colupf()),
                (false, _) => None,
            };
        }
    }

    pub fn clock(&mut self) {
        self.tick_graphic_circuit();
        self.ctr.clock();
    }

    pub fn priority(&self) -> bool {
        self.priority
    }

    pub fn get_color(&self) -> Option<u8> {
        self.graphic_bit_value
    }
}
