use std::cell::RefCell;
use std::rc::Rc;

use crate::tia::color::Colors;
use crate::tia::counter::Counter;
use crate::tia::graphics::ScanCounter;

pub struct Ball {
    colors: Rc<RefCell<Colors>>,

    hmove_offset: u8,
    ctr: Counter,

    enabled: bool,
    // The ball sizee from the CTRLPF register
    nusiz: usize,

    // The VDELBL register
    vdel: bool,
    old_value: bool,

    // Graphics Scan Counter
    scan_counter: ScanCounter,

    init_delay: isize,
    graphic_size: isize,
}

impl Ball {
    pub fn new(colors: Rc<RefCell<Colors>>, init_delay: isize, graphic_size: isize) -> Self {
        Self {
            colors,

            hmove_offset: 0,
            ctr: Counter::new(40, 39),

            enabled: false,
            nusiz: 1,

            vdel: false,
            old_value: false,

            scan_counter: ScanCounter::default(),

            init_delay,
            graphic_size,
        }
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.enabled = v
    }

    pub fn set_hmove_value(&mut self, v: u8) {
        self.hmove_offset = v
    }

    pub fn set_vdel(&mut self, v: bool) {
        self.vdel = v
    }

    pub fn set_vdel_value(&mut self) {
        self.old_value = self.enabled
    }

    pub fn set_nusiz(&mut self, size: usize) {
        self.nusiz = size
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

    fn size(&self) -> usize {
        self.nusiz
    }

    fn pixel_bit(&self) -> bool {
        if self.vdel {
            self.old_value
        } else {
            self.enabled
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
        false
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
            return Some(self.colors.borrow().colupf());
        }

        None
    }
}
