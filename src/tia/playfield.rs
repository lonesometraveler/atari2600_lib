use super::SharedColor;
use crate::tia::counter::Counter;

#[allow(dead_code)]
mod pf_data {
    use modular_bitfield::prelude::*;
    use std::array;
    // 20-bit playfield
    // .... | .... .... | .... ....
    // PF0  |    PF1    |    PF2
    #[derive(Clone, Copy)]
    #[bitfield(bits = 20)]
    pub(crate) struct PlayfieldData {
        pub pf0: B4,
        pub pf1: B8,
        pub pf2: B8,
    }

    impl PlayfieldData {
        // returns pf0, pf1, pf2 as [bool; 20]
        pub fn bits(&self) -> [bool; 20] {
            let val = (self.pf0() as u32) << 16 | (self.pf1() as u32) << 8 | self.pf2() as u32;
            array::from_fn(|i| val & (1 << (19 - i)) != 0)
        }
    }
}
use pf_data::PlayfieldData;

const PF_LENGTH: usize = 20;

pub(crate) struct Playfield {
    colors: SharedColor,
    ctr: Counter,

    pf_data: PlayfieldData,
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

            pf_data: PlayfieldData::from_bytes([0, 0, 0]),

            horizontal_mirror: false,
            score_mode: false,
            priority: false,

            graphic_bit_value: None,
        }
    }

    pub fn set_pf0(&mut self, val: u8) {
        // PF0 is the first 4 bits, in big-endian order
        let val = reverse_bit_order(val);
        self.pf_data.set_pf0(val & 0x0f);
    }

    pub fn set_pf1(&mut self, val: u8) {
        // PF1 is the next 8 bits, in little-endian order
        self.pf_data.set_pf1(val);
    }

    pub fn set_pf2(&mut self, val: u8) {
        // PF2 is the last 8 bits, in big-endian order
        let val = reverse_bit_order(val);
        self.pf_data.set_pf2(val);
    }

    pub fn set_control(&mut self, val: u8) {
        self.horizontal_mirror = (val & 0x01) != 0;
        self.priority = (val & 0x04) != 0;
        self.score_mode = (val & 0x02) != 0 && !self.priority;
    }

    fn tick_graphic_circuit(&mut self) {
        let ctr = self.ctr.value() as usize;
        let pf_x = ctr % 20;
        let data_bits = self.pf_data.bits();
        let colors = self.colors.borrow();

        if ctr < 20 {
            self.graphic_bit_value = match (data_bits[pf_x], self.score_mode) {
                (true, true) => Some(colors.colup0()),
                (true, false) => Some(colors.colupf()),
                (false, _) => None,
            };
        } else {
            // The playfield also makes up the right-most side of the
            // screen, optionally mirrored horizontally as denoted by the
            // CTRLPF register.
            let idx = if self.horizontal_mirror {
                PF_LENGTH - 1 - pf_x
            } else {
                pf_x
            };

            self.graphic_bit_value = match (data_bits[idx], self.score_mode) {
                (true, true) => Some(colors.colup1()),
                (true, false) => Some(colors.colupf()),
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

fn reverse_bit_order(value: u8) -> u8 {
    let mut value = value;
    let mut result = 0;

    for _ in 0..8 {
        result = (result << 1) | (value & 1);
        value >>= 1;
    }

    result
}
