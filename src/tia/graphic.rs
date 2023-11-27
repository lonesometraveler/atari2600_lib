use super::counter::Counter;

// Graphics Scan Counter
#[derive(Default)]
pub struct ScanCounter {
    pub bit_idx: Option<isize>,
    pub bit_copies_written: usize,
    pub bit_value: Option<bool>,
}

/// TIA Object
pub trait Graphic {
    const MAX_COUNTER_VAL: u8 = 39;

    // Initial delay for the graphic scan
    const INIT_DELAY: isize;

    // Size of the graphic (number of pixels to draw)
    const GRAPHIC_SIZE: isize;

    // Setter for enabling/disabling the object
    fn set_enabled(&mut self, v: bool);

    // Setter for horizontal movement value
    fn set_hmove_value(&mut self, v: u8);

    // TODO: verify this
    // Setter for the size of the ???
    fn set_nusiz(&mut self, val: usize);

    fn hmclr(&mut self);

    // Reset method for initializing the object
    fn reset(&mut self) {
        self.get_counter_mut().reset();
        if self.should_draw_graphic() || self.should_draw_copy() {
            self.reset_scan_counter();
        }
    }

    // Method to start horizontal movement
    fn start_hmove(&mut self) {
        let hmove_offset = self.get_hmove_offset();
        self.get_counter_mut().start_hmove(hmove_offset);
        self.tick_graphic_circuit();
    }

    // Method called on each clock cycle
    fn clock(&mut self) {
        self.tick_graphic_circuit();

        if self.get_counter_mut().clock() && (self.should_draw_graphic() || self.should_draw_copy())
        {
            self.reset_scan_counter();
        }
    }

    // Method to apply horizontal movement
    fn apply_hmove(&mut self) {
        let hmove_offset = self.get_hmove_offset();
        let result = self.get_counter_mut().apply_hmove(hmove_offset);

        if result.clocked && (self.should_draw_graphic() || self.should_draw_copy()) {
            self.reset_scan_counter();
        }

        if result.moved {
            self.tick_graphic_circuit();
        }
    }

    // Method to get the color of the pixel
    fn get_color(&self) -> Option<u8>;

    /// Updates the graphic scan circuit based on the current state of the TiaObject.
    /// This method is responsible for advancing the graphic scan, determining
    /// pixel values, and managing the scan counter state.
    /// - If the scan counter has an active bit index, it progresses through the
    ///   graphic pixels, updating the pixel value based on the current state of the
    ///   Ball.
    /// - The method checks if the scan counter has completed copying the graphic,
    ///   and if so, resets the counter and prepares for the next graphic scan.
    /// - If the end of the graphic is reached, the bit index is set to `None`.
    /// - If the scan counter is inactive, the bit value is set to `None`.
    fn tick_graphic_circuit(&mut self) {
        let pixel_bit = self.pixel_bit();
        let size = self.size();
        let graphic_size = self.graphic_size();
        let scan_counter = self.get_scan_counter_mut();

        if let Some(mut idx) = scan_counter.bit_idx {
            if !(0..8).contains(&idx) {
                scan_counter.bit_idx = Some(idx + 1);
                return;
            }

            scan_counter.bit_value = Some(pixel_bit);
            scan_counter.bit_copies_written += 1;

            if scan_counter.bit_copies_written == size {
                scan_counter.bit_copies_written = 0;
                idx += 1;
            }

            scan_counter.bit_idx = if idx == graphic_size { None } else { Some(idx) };
        } else {
            scan_counter.bit_value = None;
        }
    }

    // Method to determine whether a graphic should be drawn
    fn should_draw_graphic(&self) -> bool {
        self.get_counter().value() == Self::MAX_COUNTER_VAL
    }

    // Method to determine whether a copy of the graphic should be drawn
    fn should_draw_copy(&self) -> bool;

    // Method to reset the scan counter
    fn reset_scan_counter(&mut self) {
        self.get_scan_counter_mut().bit_idx = Some(-Self::INIT_DELAY);
        self.get_scan_counter_mut().bit_copies_written = 0;
    }

    // Method to get a mutable reference to the scan counter
    fn get_scan_counter_mut(&mut self) -> &mut ScanCounter;

    // Method to get the pixel value for drawing
    fn pixel_bit(&self) -> bool;

    // TODO: verify this
    // Method to get the size of the ???
    fn size(&self) -> usize;

    // Method to get the size of the graphic
    fn graphic_size(&self) -> isize {
        Self::GRAPHIC_SIZE
    }

    // Method to get the current value of the counter
    // fn counter_value(&self) -> u8;

    // Method to get a immutable reference to the counter
    fn get_counter(&self) -> &Counter;

    // Method to get a mutable reference to the counter
    fn get_counter_mut(&mut self) -> &mut Counter;

    fn get_hmove_offset(&self) -> u8;
}
