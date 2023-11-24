// Graphics Scan Counter
#[derive(Default)]
pub struct ScanCounter {
    pub bit_idx: Option<isize>,
    pub bit_copies_written: usize,
    pub bit_value: Option<bool>,
}

/// TIA Object
pub trait TiaObject {
    const MAX_COUNTER_VAL: u8 = 39;

    fn set_enabled(&mut self, v: bool);
    fn set_hmove_value(&mut self, v: u8);
    fn set_nusiz(&mut self, val: usize);
    fn hmclr(&mut self);
    fn reset(&mut self);
    fn start_hmove(&mut self);
    fn clock(&mut self);
    fn apply_hmove(&mut self);
    fn get_color(&self) -> Option<u8>;

    fn tick_graphic_circuit(&mut self) {
        if let Some(mut idx) = self.scan_counter().bit_idx {
            if (0..8).contains(&idx) {
                self.scan_counter().bit_value = Some(self.pixel_bit());

                self.scan_counter().bit_copies_written += 1;
                if self.scan_counter().bit_copies_written == self.size() {
                    self.scan_counter().bit_copies_written = 0;
                    idx += 1;
                }

                if idx == self.graphic_size() {
                    self.scan_counter().bit_idx = None;
                } else {
                    self.scan_counter().bit_idx = Some(idx);
                }
            } else {
                self.scan_counter().bit_idx = Some(idx + 1);
            }
        } else {
            self.scan_counter().bit_value = None;
        }
    }

    fn should_draw_graphic(&self) -> bool {
        self.counter_value() == Self::MAX_COUNTER_VAL
    }

    fn should_draw_copy(&self) -> bool;

    fn reset_scan_counter(&mut self);

    // Add these abstract methods to the trait
    fn scan_counter(&mut self) -> &mut ScanCounter;
    fn pixel_bit(&self) -> bool;
    fn size(&self) -> usize;
    fn graphic_size(&self) -> isize;
    fn counter_value(&self) -> u8;
}
