// https://github.com/JetSetIlly/Gopher2600/blob/master/hardware/tia/audio/channels.go

use super::register::Registers;

#[derive(Clone, Debug, Default)]
pub(crate) struct Channel {
    pub(crate) registers: Registers,
    pub(crate) registers_changed: bool,

    clock_enable: bool,
    noise_feedback: bool,
    noise_counter_bit4: bool,
    pulse_counter_hold: bool,

    div_counter: u8,
    pulse_counter: u8,
    noise_counter: u8,

    pub(crate) actual_vol: u8,
}

impl Channel {
    pub fn new() -> Self {
        Channel::default()
    }

    pub fn string(&self) -> String {
        self.registers.to_string()
    }

    // tick should be called at a frequency of 30Khz. when the 10Khz clock is
    // required, the frequency clock is increased by a factor of three.
    pub fn tick(&mut self) {
        self.phase0();
        self.phase1();
    }

    pub fn phase0(&mut self) {
        if self.clock_enable {
            self.noise_counter_bit4 = self.noise_counter & 0x01 != 0x00;

            match self.registers.control & 0x03 {
                0x00 | 0x01 => {
                    self.pulse_counter_hold = false;
                }
                0x02 => {
                    self.pulse_counter_hold = self.noise_counter & 0x1e != 0x02;
                }
                0x03 => {
                    self.pulse_counter_hold = !self.noise_counter_bit4;
                }
                _ => {}
            }

            match self.registers.control & 0x03 {
                0x00 => {
                    self.noise_feedback = (((self.pulse_counter ^ self.noise_counter) & 0x01)
                        != 0x00)
                        || !(self.noise_counter != 0x00 || self.pulse_counter != 0x0a)
                        || (self.registers.control & 0x0c == 0x00);
                }
                _ => {
                    let n = if self.noise_counter & 0x04 != 0x00 {
                        1
                    } else {
                        0
                    };
                    self.noise_feedback =
                        (n ^ (self.noise_counter & 0x01) != 0x00) || self.noise_counter == 0;
                }
            }
        }

        self.clock_enable = self.div_counter == self.registers.freq;

        if self.div_counter == self.registers.freq || self.div_counter == 0x1f {
            self.div_counter = 0;
        } else {
            self.div_counter += 1;
        }
    }

    pub fn phase1(&mut self) {
        if self.clock_enable {
            let pulse_feedback = match self.registers.control >> 2 {
                0x00 => {
                    let n = if self.pulse_counter & 0x02 != 0x00 {
                        1
                    } else {
                        0
                    };
                    (n ^ (self.pulse_counter & 0x01) != 0x00)
                        && (self.pulse_counter != 0x0a)
                        && (self.registers.control & 0x03 != 0x00)
                }
                0x01 => self.pulse_counter & 0x08 == 0x00,
                0x02 => !self.noise_counter_bit4,
                0x03 => {
                    !((self.pulse_counter & 0x02 != 0x00) || (self.pulse_counter & 0x0e == 0x00))
                }
                _ => false,
            };

            self.noise_counter >>= 1;

            if self.noise_feedback {
                self.noise_counter |= 0x10;
            }

            if !self.pulse_counter_hold {
                self.pulse_counter = !(self.pulse_counter >> 1) & 0x07;

                if pulse_feedback {
                    self.pulse_counter |= 0x08;
                }
            }
        }

        self.actual_vol = (self.pulse_counter & 0x01) * self.registers.volume;
    }
}

// changing the value of an AUDx registers causes some side effect.
impl Channel {
    pub fn react_aud_cx(&mut self) {
        self.registers_changed = true;
    }
}
