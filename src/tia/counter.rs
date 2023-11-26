/// Represents the result of applying horizontal movement.
pub struct HMoveResult {
    /// Indicates whether movement is required.
    pub moved: bool,
    /// Indicates whether the counter is clocked.
    pub clocked: bool,
}

/// "Visible" counter value ranges from 0-39
const PERIOD: u8 = 40;
/// counter value ranges from 0-39 incrementing every 4 "ticks" from TIA (1/4 of TIA clock)
/// (shift left (<<) are equivalent to multiply by 2^<shift>
/// and shift right (>>) are equivalent to divide by 2^<shift>)
const DIVIDER: u8 = 4;
/// Value set when the TIA RESxx position is strobed
const RESET_VALUE: u8 = 39;
const INTERNAL_PERIOD: u8 = PERIOD * DIVIDER;

/// Internal counters used by all TIA graphics to trigger drawing at appropriate time.
/// Horizontal position is implicitly tracked by the counter value, and movement is
/// implemented by making its cycle higher or lower than the current scanline.
/// See: http://www.atarihq.com/danb/files/TIA_HW_Notes.txt
pub struct Counter {
    period: u8,
    reset_value: u8,
    reset_delay: u8,
    pub internal_value: u8,

    last_value: u8,
    ticks_added: u8,
    movement_required: bool,
}

fn ticks_to_add(v: u8) -> u8 {
    let nibble = v >> 4;

    if nibble < 8 {
        nibble + 8
    } else {
        nibble - 8
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new(PERIOD, RESET_VALUE)
    }
}

impl Counter {
    pub fn new(period: u8, reset_value: u8) -> Self {
        Self {
            period,
            reset_value,
            internal_value: 0,
            reset_delay: 0,

            last_value: 0,
            ticks_added: 0,
            movement_required: false,
        }
    }

    pub fn reset(&mut self) {
        self.internal_value = self.reset_value * DIVIDER;
    }

    pub fn value(&self) -> u8 {
        self.internal_value / DIVIDER
    }

    pub fn reset_to(&mut self, v: u8) {
        self.internal_value = v;
    }

    pub fn reset_to_h1(&mut self) {
        // From TIA_HW_Notes.txt:
        //
        // > RSYNC resets the two-phase clock for the HSync counter to the
        // > H@1 rising edge when strobed.
        self.internal_value = self.value() * DIVIDER;

        // A full H@1-H@2 cycle after RSYNC is strobed, the
        // HSync counter is also reset to 000000 and HBlank is turned on.
        self.reset_delay = 8;
    }

    pub fn clock(&mut self) -> bool {
        if self.reset_delay > 0 {
            self.reset_delay -= 1;

            if self.reset_delay == 0 {
                self.reset();
            }
        }

        self.internal_value = (self.internal_value + 1) % (self.period * DIVIDER);

        let clocked = self.last_value != self.value();
        self.last_value = self.value();

        clocked
    }

    /// Horizontal movement (HMOV) is implemented by extending the horizontal blank
    /// by 8 pixels. That shortens the visible scanline to 152 pixels (producing the
    /// "comb effect" on the left side) and pushes all graphics 8 pixels to the right...
    pub fn start_hmove(&mut self, hm_val: u8) {
        self.ticks_added = 0;
        self.movement_required = ticks_to_add(hm_val) != 0;
    }

    /// ...but then TIA stuffs each counter with an extra cycle, counting those until
    /// it reaches the current value for the HMMxx register for that graphic). Each
    /// extra tick means pushing the graphic 1 pixel to the left, so the final movement
    /// ends up being something betwen 8 pixels to the right (0 extra ticks) and
    /// 7 pixels to the left (15 extra ticks)
    pub fn apply_hmove(&mut self, hm_val: u8) -> HMoveResult {
        if self.movement_required {
            let clocked = self.clock();
            self.ticks_added += 1;
            self.movement_required = self.ticks_added != ticks_to_add(hm_val);

            HMoveResult {
                moved: true,
                clocked,
            }
        } else {
            HMoveResult {
                moved: false,
                clocked: false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_without_reset_delay() {
        let mut counter = Counter::default();
        assert!(!counter.clock());
        assert_eq!(counter.internal_value, 1);
        assert_eq!(counter.value(), 0);

        // Increment internal value to 1
        assert!(!counter.clock());
        assert_eq!(counter.internal_value, 2);
        assert_eq!(counter.value(), 0);

        // Increment internal value to 2
        assert!(!counter.clock());
        assert_eq!(counter.internal_value, 3);
        assert_eq!(counter.value(), 0);

        // Increment internal value to 3
        assert!(counter.clock());
        assert_eq!(counter.internal_value, 4);
        assert_eq!(counter.value(), 1);
    }

    #[test]
    fn clock_with_reset_delay() {
        let mut counter = Counter::default();
        counter.reset_to_h1();

        assert!(!counter.clock()); // reset_delay = 7
        assert_eq!(counter.internal_value, 1);

        assert!(!counter.clock()); // reset_delay = 7
        assert_eq!(counter.internal_value, 2);

        // Increment internal value to 3, reset_delay = 6
        assert!(!counter.clock());
        assert_eq!(counter.internal_value, 3);

        // Increment internal value to 4, reset_delay = 5
        assert!(counter.clock());
        assert_eq!(counter.internal_value, 4);

        // Increment internal value to 5, reset_delay = 4
        assert!(!counter.clock());
        assert_eq!(counter.internal_value, 5);

        // Increment internal value to 6, reset_delay = 3
        assert!(!counter.clock());
        // assert_eq!(counter.value(), 6);
        assert_eq!(counter.internal_value, 6);

        // Increment internal value to 7, reset_delay = 2
        assert!(!counter.clock());
        assert_eq!(counter.internal_value, 7);

        // Increment internal value to 8, reset_delay = 1
        assert!(counter.clock());
        assert_eq!(counter.internal_value, 157);

        // Increment internal value to 9, reset_delay = 0, perform reset
        assert!(!counter.clock());
        assert_eq!(counter.internal_value, 158);
    }

    #[test]
    fn test_clocking() {
        let mut ctr = Counter::new(40, 0);

        assert_eq!(ctr.value(), 0);

        let mut clocked = ctr.clock();
        assert!(!clocked);
        assert_eq!(ctr.value(), 0);

        clocked = ctr.clock();
        assert!(!clocked);
        assert_eq!(ctr.value(), 0);

        clocked = ctr.clock();
        assert!(!clocked);
        assert_eq!(ctr.value(), 0);

        clocked = ctr.clock();
        assert!(clocked);
        assert_eq!(ctr.value(), 1);

        for i in 1..=152 {
            clocked = ctr.clock();

            if i % 4 == 0 {
                assert!(clocked);
            } else {
                assert!(!clocked);
            }
        }

        assert_eq!(ctr.value(), 39);

        ctr.clock();
        assert_eq!(ctr.value(), 39);
        ctr.clock();
        assert_eq!(ctr.value(), 39);
        ctr.clock();
        assert_eq!(ctr.value(), 39);
        let clocked = ctr.clock();

        assert!(clocked);
        assert_eq!(ctr.value(), 0);
    }

    #[test]
    fn test_scanline_counting() {
        // p0, p0, m0, and m1 use a 40 clock counter, so they should reset back to 0 after a full
        // scanline has finished rendering.
        let mut ctr = Counter::new(40, 0);

        assert_eq!(ctr.value(), 0);

        for _ in 0..160 {
            ctr.clock();
        }

        assert_eq!(ctr.value(), 0);
    }
}
