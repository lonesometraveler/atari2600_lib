use std::cell::RefCell;
use std::rc::Rc;

use crate::tia::color::Colors;
use crate::tia::counter::Counter;
use crate::tia::graphics::ScanCounter;
use crate::tia::PlayerType;

pub struct Player {
    init_delay: isize,
    graphic_size: isize,
    colors: Rc<RefCell<Colors>>,
    hmove_offset: u8,
    ctr: Counter,
    scan_counter: ScanCounter,
    nusiz: usize,

    // The REFPx register, for rendering backwards
    horizontal_mirror: bool,
    // The 8-bit graphic to draw
    graphic: u8,
    // The VDELPx register
    vdel: bool,
    old_value: u8,

    player: PlayerType,
}

impl Player {
    pub fn new(
        colors: Rc<RefCell<Colors>>,
        player: PlayerType,
        init_delay: isize,
        graphic_size: isize,
    ) -> Self {
        Self {
            colors,
            player,

            hmove_offset: 0,
            ctr: Counter::new(40, 39),

            horizontal_mirror: false,
            nusiz: 0,
            graphic: 0,

            vdel: false,
            old_value: 0,

            scan_counter: ScanCounter::default(),

            init_delay,
            graphic_size,
        }
    }

    pub fn size(&self) -> usize {
        match self.nusiz & 0x0f {
            0b0101 => 2,
            0b0111 => 4,
            _ => 1,
        }
    }

    pub fn counter(&self) -> &Counter {
        &self.ctr
    }

    pub fn set_hmove_value(&mut self, v: u8) {
        self.hmove_offset = v
    }

    pub fn set_graphic(&mut self, graphic: u8) {
        self.graphic = graphic
    }

    pub fn set_horizontal_mirror(&mut self, reflect: bool) {
        self.horizontal_mirror = reflect
    }

    pub fn set_nusiz(&mut self, v: usize) {
        self.nusiz = v & 0x0f
    }

    pub fn set_vdel(&mut self, v: bool) {
        self.vdel = v
    }

    pub fn set_vdel_value(&mut self) {
        self.old_value = self.graphic
    }

    pub fn hmclr(&mut self) {
        self.hmove_offset = 0
    }

    pub fn reset(&mut self) {
        self.ctr.reset();

        if self.should_draw_graphic() || self.should_draw_copy() {
            self.scan_counter.bit_idx = Some(-self.init_delay);
            self.scan_counter.bit_copies_written = 0;
        }
    }

    pub fn start_hmove(&mut self) {
        self.ctr.start_hmove(self.hmove_offset);
        self.tick_graphic_circuit();
    }

    // Based on current state, return whether or not we are rendering this sprite
    fn pixel_bit(&self) -> bool {
        if let Some(x) = self.scan_counter.bit_idx {
            let graphic = if self.vdel {
                self.old_value
            } else {
                self.graphic
            };

            if self.horizontal_mirror {
                (graphic & (1 << x)) != 0
            } else {
                (graphic & (1 << (7 - x))) != 0
            }
        } else {
            false
        }
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

        (count == 3 && (self.nusiz == 0b001 || self.nusiz == 0b011))
            || (count == 7 && (self.nusiz == 0b010 || self.nusiz == 0b011 || self.nusiz == 0b110))
            || (count == 15 && (self.nusiz == 0b100 || self.nusiz == 0b110))
    }

    pub fn clock(&mut self) {
        self.tick_graphic_circuit();

        if self.ctr.clock() && (self.should_draw_graphic() || self.should_draw_copy()) {
            self.scan_counter.bit_idx = Some(-self.init_delay);
            self.scan_counter.bit_copies_written = 0;
        }
    }

    pub fn apply_hmove(&mut self) {
        let (moved, counter_clocked) = self.ctr.apply_hmove(self.hmove_offset);

        if counter_clocked && (self.should_draw_graphic() || self.should_draw_copy()) {
            self.scan_counter.bit_idx = Some(-self.init_delay);
            self.scan_counter.bit_copies_written = 0;
        }

        if moved {
            self.tick_graphic_circuit();
        }
    }

    pub fn get_color(&self) -> Option<u8> {
        if let Some(true) = self.scan_counter.bit_value {
            let color = match self.player {
                PlayerType::Player0 => self.colors.borrow().colup0(),
                PlayerType::Player1 => self.colors.borrow().colup1(),
            };

            return Some(color);
        }

        None
    }

    #[allow(dead_code)]
    pub fn debug(&self) {
        if !self.should_draw_graphic() && !self.should_draw_copy() {
            return;
        }

        println!("p: {:?}, ctr: {}, grp: {:08b}, gv: {:?}, refp: {}, nusiz: {:03b}, vdel: {}, old_value: {:08b}",
                 self.player,
                 self.ctr.value(),
                 self.graphic,
                 self.scan_counter.bit_value,
                 self.horizontal_mirror,
                 self.nusiz,
                 self.vdel,
                 self.old_value,
        );
    }
}
