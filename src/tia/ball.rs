use crate::tia::counter::Counter;
use crate::tia::graphics::ScanCounter;

use super::graphics::TiaObject;
use super::ColorType;

pub struct Ball {
    init_delay: isize,
    graphic_size: isize,
    colors: ColorType,
    hmove_offset: u8,
    ctr: Counter,
    scan_counter: ScanCounter,
    nusiz: usize,

    enabled: bool,
    // The VDELBL register
    vdel: bool,
    old_value: bool,
}

impl TiaObject for Ball {
    fn set_enabled(&mut self, v: bool) {
        self.enabled = v
    }

    fn set_hmove_value(&mut self, v: u8) {
        self.hmove_offset = v
    }

    fn set_nusiz(&mut self, size: usize) {
        self.nusiz = size
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
        self.nusiz
    }

    fn pixel_bit(&self) -> bool {
        if self.vdel {
            self.old_value
        } else {
            self.enabled
        }
    }

    fn clock(&mut self) {
        self.tick_graphic_circuit();

        if self.ctr.clock() && (self.should_draw_graphic() || self.should_draw_copy()) {
            self.reset_scan_counter();
        }
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
        self.scan_counter
            .bit_value
            .filter(|&bit| bit)
            .map(|_| self.colors.borrow().colupf())
    }

    fn should_draw_copy(&self) -> bool {
        false
    }

    fn reset_scan_counter(&mut self) {
        self.scan_counter.bit_idx = Some(-self.init_delay);
        self.scan_counter.bit_copies_written = 0;
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

impl Ball {
    pub fn new(colors: ColorType, init_delay: isize, graphic_size: isize) -> Self {
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

    pub fn set_vdel(&mut self, v: bool) {
        self.vdel = v
    }

    pub fn set_vdel_value(&mut self) {
        self.old_value = self.enabled
    }
}
