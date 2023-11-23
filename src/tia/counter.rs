/// Represents the result of applying horizontal movement.
pub struct HMoveResult {
    /// Indicates whether movement is required.
    pub movement_required: bool,
    /// Indicates whether the counter is clocked.
    pub is_clocked: bool,
}

pub struct Counter {
    period: u8,
    reset_value: u8,
    reset_delay: u8,
    pub internal_value: u8,

    last_value: u8,
    ticks_added: u8,
    movement_required: bool,
}

fn hmove_value(v: u8) -> u8 {
    let nibble = v >> 4;

    if nibble < 8 {
        nibble + 8
    } else {
        nibble - 8
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
        self.internal_value = self.reset_value * 4;
    }

    pub fn value(&self) -> u8 {
        self.internal_value / 4
    }

    pub fn reset_to(&mut self, v: u8) {
        self.internal_value = v;
    }

    pub fn reset_to_h1(&mut self) {
        // From TIA_HW_Notes.txt:
        //
        // > RSYNC resets the two-phase clock for the HSync counter to the
        // > H@1 rising edge when strobed.
        self.internal_value = self.value() * 4;

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

        self.internal_value += 1;
        self.internal_value %= self.period * 4;

        if self.last_value != self.value() {
            self.last_value = self.value();
            true
        } else {
            false
        }
    }

    pub fn start_hmove(&mut self, hm_val: u8) {
        self.ticks_added = 0;
        self.movement_required = hmove_value(hm_val) != 0;
    }

    pub fn apply_hmove(&mut self, hm_val: u8) -> HMoveResult {
        if !self.movement_required {
            return HMoveResult {
                movement_required: false,
                is_clocked: false,
            };
        }

        let clocked = self.clock();
        self.ticks_added += 1;
        self.movement_required = self.ticks_added != hmove_value(hm_val);

        HMoveResult {
            movement_required: true,
            is_clocked: clocked,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
