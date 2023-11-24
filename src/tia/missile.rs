use crate::tia::counter::Counter;
use crate::tia::graphics::ScanCounter;
use crate::tia::player::Player;
use crate::tia::PlayerType;

use super::graphics::TiaObject;
use super::ColorType;

pub struct Missile {
    init_delay: isize,
    graphic_size: isize,
    colors: ColorType,
    hmove_offset: u8,
    ctr: Counter,
    scan_counter: ScanCounter,
    nusiz: usize,

    enabled: bool,
    size: usize,
    copies: u8,
    sibling_player: PlayerType,
}

impl TiaObject for Missile {
    fn set_enabled(&mut self, en: bool) {
        self.enabled = en
    }

    fn set_hmove_value(&mut self, v: u8) {
        self.hmove_offset = v
    }

    fn set_nusiz(&mut self, val: usize) {
        self.nusiz = val;
        self.size = 1 << ((val & 0b0011_0000) >> 4);
        self.copies = val as u8 & 0x07;
    }

    fn hmclr(&mut self) {
        self.hmove_offset = 0
    }

    fn reset(&mut self) {
        self.ctr.reset();

        if self.should_draw_graphic() || self.should_draw_copy() {
            self.reset_scan_counter();
        }
    }

    fn start_hmove(&mut self) {
        self.ctr.start_hmove(self.hmove_offset);
        self.tick_graphic_circuit();
    }

    fn size(&self) -> usize {
        self.size
    }

    fn pixel_bit(&self) -> bool {
        self.enabled
    }

    fn should_draw_copy(&self) -> bool {
        let count = self.ctr.value();

        (count == 3 && (self.copies == 0b001 || self.copies == 0b011))
            || (count == 7
                && (self.copies == 0b010 || self.copies == 0b011 || self.copies == 0b110))
            || (count == 15 && (self.copies == 0b100 || self.copies == 0b110))
    }

    fn clock(&mut self) {
        self.tick_graphic_circuit();

        if self.ctr.clock() && (self.should_draw_graphic() || self.should_draw_copy()) {
            self.reset_scan_counter();
        }
    }

    fn reset_scan_counter(&mut self) {
        self.scan_counter.bit_idx = Some(-self.init_delay);
        self.scan_counter.bit_copies_written = 0;
    }

    fn apply_hmove(&mut self) {
        let result = self.ctr.apply_hmove(self.hmove_offset);

        if result.is_clocked && (self.should_draw_graphic() || self.should_draw_copy()) {
            self.reset_scan_counter();
        }

        if result.movement_required {
            self.tick_graphic_circuit();
        }
    }

    fn get_color(&self) -> Option<u8> {
        self.scan_counter.bit_value.and_then(|bit_value| {
            if bit_value {
                match self.sibling_player {
                    PlayerType::Player0 => Some(self.colors.borrow().colup0()),
                    PlayerType::Player1 => Some(self.colors.borrow().colup1()),
                }
            } else {
                None
            }
        })
    }

    fn scan_counter(&mut self) -> &mut ScanCounter {
        &mut self.scan_counter
    }

    fn graphic_size(&self) -> isize {
        self.graphic_size
    }

    fn counter_value(&self) -> u8 {
        self.ctr.value()
    }
}

impl Missile {
    pub fn new(
        colors: ColorType,
        sibling_player: PlayerType,
        init_delay: isize,
        graphic_size: isize,
    ) -> Self {
        Self {
            colors,
            sibling_player,

            enabled: false,
            hmove_offset: 0,
            nusiz: 0,
            size: 0,
            copies: 0,
            ctr: Counter::new(40, 39),

            scan_counter: ScanCounter::default(),

            init_delay,
            graphic_size,
        }
    }

    pub fn reset_to_player(&mut self, player: &Player) {
        self.ctr.reset_to(player.counter().internal_value);
    }

    pub fn debug(&self) {
        if !self.should_draw_graphic() && !self.should_draw_copy() {
            return;
        }

        println!(
            "ctr: {}, nusiz: {:03b}, gv: {:?}",
            self.ctr.value(),
            self.nusiz,
            self.scan_counter.bit_value,
        );
    }
}
