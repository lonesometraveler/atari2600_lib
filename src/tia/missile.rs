use super::graphic::Graphic;
use super::SharedColor;
use crate::tia::{counter::Counter, graphic::ScanCounter, player::Player, PlayerType};

pub(crate) struct Missile {
    colors: SharedColor,
    hmove_offset: u8,
    ctr: Counter,
    scan_counter: ScanCounter,
    nusiz: usize,

    enabled: bool,
    size: usize,
    copies: u8,
    sibling_player: PlayerType,
}

impl Missile {
    pub fn new(colors: SharedColor, sibling_player: PlayerType) -> Self {
        Self {
            colors,
            sibling_player,

            enabled: false,
            hmove_offset: 0,
            nusiz: 0,
            size: 0,
            copies: 0,
            ctr: Counter::default(),

            scan_counter: ScanCounter::default(),
        }
    }

    pub fn set_enabled(&mut self, en: bool) {
        self.enabled = en
    }

    pub fn set_hmove_value(&mut self, v: u8) {
        self.hmove_offset = v
    }

    pub fn set_nusiz(&mut self, val: usize) {
        self.nusiz = val;
        self.size = 1 << ((val & 0b0011_0000) >> 4);
        self.copies = val as u8 & 0x07;
    }

    pub fn hmclr(&mut self) {
        self.hmove_offset = 0
    }

    pub fn reset_to_player(&mut self, player: &Player) {
        self.ctr.reset_to(player.counter().internal_value);
    }
}

impl Graphic for Missile {
    const INIT_DELAY: isize = 4;
    const GRAPHIC_SIZE: isize = 1;

    fn size(&self) -> usize {
        self.size
    }

    fn pixel_bit(&self) -> bool {
        self.enabled
    }

    fn should_draw_copy(&self) -> bool {
        let count = self.ctr.value();
        let copies = self.copies;

        (count == 3 && (copies == 0b001 || copies == 0b011))
            || (count == 7 && (copies == 0b010 || copies == 0b011 || copies == 0b110))
            || (count == 15 && (copies == 0b100 || copies == 0b110))
    }

    fn get_color(&self) -> Option<u8> {
        self.scan_counter
            .bit_value
            .and_then(|bit_value| match (bit_value, &self.sibling_player) {
                (true, PlayerType::Player0) => Some(self.colors.borrow().colup0()),
                (true, PlayerType::Player1) => Some(self.colors.borrow().colup1()),
                (false, _) => None,
            })
    }

    fn get_scan_counter_mut(&mut self) -> &mut ScanCounter {
        &mut self.scan_counter
    }

    fn get_counter(&self) -> &Counter {
        &self.ctr
    }

    fn get_counter_mut(&mut self) -> &mut Counter {
        &mut self.ctr
    }

    fn get_hmove_offset(&self) -> u8 {
        self.hmove_offset
    }
}
