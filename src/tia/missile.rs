use std::cell::RefCell;
use std::rc::Rc;

use crate::tia::color::Colors;
use crate::tia::counter::Counter;
use crate::tia::graphics::ScanCounter;
use crate::tia::player::Player;
use crate::tia::PlayerType;

const INIT_DELAY: isize = 4;
const GRAPHIC_SIZE: isize = 1;

pub struct Missile {
    colors: Rc<RefCell<Colors>>,
    sibling_player: PlayerType,

    enabled: bool,
    hmove_offset: u8,
    nusiz: u8,
    size: u8,
    copies: u8,
    ctr: Counter,

    // Graphics Scan Counter
    scan_counter: ScanCounter,
}

impl Missile {
    pub fn new(colors: Rc<RefCell<Colors>>, sibling_player: PlayerType) -> Self {
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
        }
    }

    pub fn set_enabled(&mut self, en: bool) {
        self.enabled = en
    }
    pub fn set_hmove_value(&mut self, v: u8) {
        self.hmove_offset = v
    }
    pub fn set_nusiz(&mut self, val: u8) {
        self.nusiz = val;
        self.size = 1 << ((val & 0b0011_0000) >> 4);
        self.copies = val & 0x07;
    }
    pub fn hmclr(&mut self) {
        self.hmove_offset = 0
    }
    pub fn reset(&mut self) {
        self.ctr.reset();

        if self.should_draw_graphic() || self.should_draw_copy() {
            self.scan_counter.bit_idx = Some(-INIT_DELAY);
            self.scan_counter.bit_copies_written = 0;
        }
    }

    pub fn start_hmove(&mut self) {
        self.ctr.start_hmove(self.hmove_offset);
        self.tick_graphic_circuit();
    }

    fn size(&self) -> usize {
        self.size as usize
    }
    fn pixel_bit(&self) -> bool {
        self.enabled
    }

    fn tick_graphic_circuit(&mut self) {
        if let Some(mut idx) = self.scan_counter.bit_idx {
            if (0..8).contains(&idx) {
                self.scan_counter.bit_value = Some(self.pixel_bit());

                self.scan_counter.bit_copies_written += 1;
                if self.scan_counter.bit_copies_written == self.size() {
                    self.scan_counter.bit_copies_written = 0;
                    idx += 1;
                }

                if idx == GRAPHIC_SIZE {
                    self.scan_counter.bit_idx = None;
                } else {
                    self.scan_counter.bit_idx = Some(idx);
                }
            } else {
                self.scan_counter.bit_idx = Some(idx + 1);
            }
        } else {
            self.scan_counter.bit_value = None;
        }
    }

    fn should_draw_graphic(&self) -> bool {
        self.ctr.value() == 39
    }

    fn should_draw_copy(&self) -> bool {
        let count = self.ctr.value();

        (count == 3 && (self.copies == 0b001 || self.copies == 0b011))
            || (count == 7
                && (self.copies == 0b010 || self.copies == 0b011 || self.copies == 0b110))
            || (count == 15 && (self.copies == 0b100 || self.copies == 0b110))
    }

    pub fn clock(&mut self) {
        self.tick_graphic_circuit();

        if self.ctr.clock() && (self.should_draw_graphic() || self.should_draw_copy()) {
            self.scan_counter.bit_idx = Some(-INIT_DELAY);
            self.scan_counter.bit_copies_written = 0;
        }
    }

    pub fn reset_to_player(&mut self, player: &Player) {
        self.ctr.reset_to(player.counter().internal_value);
    }

    pub fn apply_hmove(&mut self) {
        let (moved, counter_clocked) = self.ctr.apply_hmove(self.hmove_offset);

        if counter_clocked && (self.should_draw_graphic() || self.should_draw_copy()) {
            self.scan_counter.bit_idx = Some(-INIT_DELAY);
            self.scan_counter.bit_copies_written = 0;
        }

        if moved {
            self.tick_graphic_circuit();
        }
    }

    pub fn get_color(&self) -> Option<u8> {
        if let Some(true) = self.scan_counter.bit_value {
            let color = match self.sibling_player {
                PlayerType::Player0 => self.colors.borrow().colup0(),
                PlayerType::Player1 => self.colors.borrow().colup1(),
            };

            return Some(color);
        }

        None
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
