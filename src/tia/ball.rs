use crate::tia::counter::Counter;
use crate::tia::graphic::ScanCounter;

use super::graphic::Graphic;
use super::SharedColor;

pub struct Ball {
    // SharedColor is an alias for Rc<RefCell<Colors>> (used for shared ownership and interior mutability)
    colors: SharedColor,
    // Horizontal movement offset
    hmove_offset: u8,
    // Counter for managing horizontal movement and clock cycles
    ctr: Counter,
    // Counter for managing the graphic scan
    scan_counter: ScanCounter,
    // Size of the graphic (number of pixels to draw)
    nusiz: usize,
    // Flag indicating whether the object is enabled for rendering
    enabled: bool,
    // VDELBL register flag for delayed vertical motion
    vdel: bool,
    // Previous value of the pixel for delayed vertical motion
    old_value: bool,
}

impl Graphic for Ball {
    const INIT_DELAY: isize = 4;
    const GRAPHIC_SIZE: isize = 1;

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

    fn get_color(&self) -> Option<u8> {
        self.scan_counter
            .bit_value
            .filter(|&bit| bit)
            .map(|_| self.colors.borrow().colupf())
    }

    fn should_draw_copy(&self) -> bool {
        false
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

impl Ball {
    pub fn new(colors: SharedColor) -> Self {
        Self {
            colors,

            hmove_offset: 0,
            ctr: Counter::default(),

            enabled: false,
            nusiz: 1,

            vdel: false,
            old_value: false,

            scan_counter: ScanCounter::default(),
        }
    }

    pub fn set_vdel(&mut self, v: bool) {
        self.vdel = v
    }

    pub fn set_vdel_value(&mut self) {
        self.old_value = self.enabled
    }
}
