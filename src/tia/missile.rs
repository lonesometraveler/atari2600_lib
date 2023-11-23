use std::cell::RefCell;
use std::rc::Rc;

use crate::tia::color::Colors;
use crate::tia::counter::Counter;
use crate::tia::graphics::ScanCounter;
use crate::tia::player::Player;
use crate::tia::PlayerType;

pub struct Missile {
    init_delay: isize,
    graphic_size: isize,
    colors: Rc<RefCell<Colors>>,
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
    pub fn new(
        colors: Rc<RefCell<Colors>>,
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

    pub fn set_enabled(&mut self, en: bool) {
        self.enabled = en
    }

    pub fn set_hmove_value(&mut self, v: u8) {
        self.hmove_offset = v
    }

    pub fn set_nusiz(&mut self, val: u8) {
        self.nusiz = val as usize;
        self.size = 1 << ((val & 0b0011_0000) >> 4);
        self.copies = val & 0x07;
    }

    pub fn hmclr(&mut self) {
        self.hmove_offset = 0
    }

    pub fn reset(&mut self) {
        self.ctr.reset();

        if self.should_draw_graphic() || self.should_draw_copy() {
            self.reset_scan_counter();
        }
    }

    pub fn start_hmove(&mut self) {
        self.ctr.start_hmove(self.hmove_offset);
        self.tick_graphic_circuit();
    }

    fn size(&self) -> usize {
        self.size
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

                if idx == self.graphic_size {
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
            self.reset_scan_counter();
        }
    }

    fn reset_scan_counter(&mut self) {
        self.scan_counter.bit_idx = Some(-self.init_delay);
        self.scan_counter.bit_copies_written = 0;
    }

    pub fn reset_to_player(&mut self, player: &Player) {
        self.ctr.reset_to(player.counter().internal_value);
    }

    pub fn apply_hmove(&mut self) {
        let result = self.ctr.apply_hmove(self.hmove_offset);

        if result.is_clocked && (self.should_draw_graphic() || self.should_draw_copy()) {
            self.reset_scan_counter();
        }

        if result.movement_required {
            self.tick_graphic_circuit();
        }
    }

    pub fn get_color(&self) -> Option<u8> {
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
