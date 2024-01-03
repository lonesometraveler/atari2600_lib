// https://github.com/JetSetIlly/Gopher2600/blob/master/hardware/tia/audio/audio.go

mod channel;
mod register;

use channel::Channel;

// SampleFreq represents the number of samples generated per second. This is
// the 30Khz reference frequency desribed in the Stella Programmer's Guide.
const SAMPLE_FREQ: i32 = 31400;

// Audio is the implementation of the TIA audio sub-system, using Ron Fries'
// method. Reference source code here:
//
// https://raw.githubusercontent.com/alekmaul/stella/master/emucore/TIASound.c
#[derive(Clone, Debug, Default)]
pub struct Audio {
    // the reference frequency for all sound produced by the TIA is 30Khz.
    // this is the 3.58Mhz clock, which the TIA operates at, divided by
    // 114. that's one half of a scanline so we count to 228 and update
    // twice in that time
    clock_228: i32,

    // From the "Stella Programmer's Guide":
    //
    // "There are two audio circuits for generating sound. They are identical but
    // completely independent and can be operated simultaneously [...]"
    pub channel0: Channel,
    pub channel1: Channel,

    // the volume output for each channel
    vol0: u8,
    vol1: u8,

    registers_changed: bool,
}

// Plumb audio into emulation
impl Audio {
    pub fn new() -> Audio {
        Audio::default()
    }

    pub fn reset(&mut self) {
        self.clock_228 = 0;
        self.channel0 = Channel::default();
        self.channel1 = Channel::default();
        self.vol0 = 0;
        self.vol1 = 0;
    }

    // Snapshot creates a copy of the TIA Audio sub-system in its current state.
    pub fn snapshot(&self) -> Audio {
        self.clone()
    }

    // Step the audio on one TIA clock. The step will be filtered to produce a
    // 30Khz clock.
    pub fn step(&mut self) -> bool {
        self.registers_changed = false;

        self.clock_228 += 1;
        if self.clock_228 >= 228 {
            self.clock_228 = 0;
            return false;
        }

        match self.clock_228 {
            10 => {
                self.channel0.phase0();
                self.channel1.phase0();
                return false;
            }
            82 => {
                self.channel0.phase0();
                self.channel1.phase0();
                return false;
            }
            38 => {
                self.channel0.phase1();
                self.channel1.phase1();
            }
            150 => {
                self.channel0.phase1();
                self.channel1.phase1();
            }
            _ => return false,
        }

        self.vol0 = self.channel0.actual_vol;
        self.vol1 = self.channel1.actual_vol;

        true
    }

    // HasTicked returns whether the audio channels were ticked on the previous
    // video cycle. The return values indicate the ticking for phase 0 & phase 1;
    // and whether an audio register has changed. Can never return three true values
    //
    // The function is only useful for emulator reflection.
    pub fn has_ticked(&self) -> (bool, bool, bool) {
        match self.clock_228 {
            10 => (true, false, self.registers_changed),
            82 => (true, false, self.registers_changed),
            38 => (false, true, self.registers_changed),
            150 => (false, true, self.registers_changed),
            _ => (false, false, self.registers_changed),
        }
    }
}
