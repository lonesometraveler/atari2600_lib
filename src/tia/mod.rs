mod ball;
mod color;
mod counter;
mod graphic;
mod missile;
mod palette;
mod player;
mod playfield;

use crate::memory::{TiaReadAddress, TiaWriteAddress};
use image::Rgba;
use log::debug;
use std::{cell::RefCell, rc::Rc};
use {
    ball::Ball,
    color::Colors,
    counter::Counter,
    graphic::Graphic,
    missile::Missile,
    palette::{DEFAULT_COLOR, NTSC_PALETTE},
    player::Player,
    playfield::Playfield,
};

const LINE_LENGTH: usize = 160;
const H_BLANK_CLOCKS: usize = 68;

pub type SharedColor = Rc<RefCell<Colors>>;

#[derive(Debug)]
pub enum PlayerType {
    Player0,
    Player1,
}

struct Signals;
// https://github.com/jigo2600/jigo2600/blob/master/doc/TIA_Visual_Objects.md
impl Signals {
    // The SHB signal is used to set HB and clear HC.
    const SHB: u8 = 0;
    // Set H-SYNC. The SHS signal is used to set the horizontal sync  HS signal and, together with RHS, it shapes it.
    const SHS: u8 = 4;
    // Reset H-SYNC. The RHS signal resets the horizontal sync HS signal and triggers the color burst CB signal.
    const RHS: u8 = 8;
    // ColourBurst. The RCB signal resets the color burst CB.
    const RCB: u8 = 12;
    // Reset H-BLANK. The RHB signal resets the HBLANK HB signal. It can be ignored for LRHB depending on the HMOVEL latch.
    const RHB: u8 = 16;
    // Late RHB. The LRHB signal resets the HBLANK HB signal later. It can be ignored for RHB depending on the HMOVEL latch.
    const LRHB: u8 = 18;
    // Center. The playfield center signal CNT is starts to draw the second part of the playfield.
    const CNT: u8 = 36;
    // The END signal resets the HC counter.
    const END: u8 = 56;
}

#[allow(clippy::upper_case_acronyms)]
#[repr(u8)]
enum VideoSignal {
    SHB = Signals::SHB,
    SHS = Signals::SHS,
    RHS = Signals::RHS,
    RCB = Signals::RCB,
    RHB = Signals::RHB,
    LRHB = Signals::LRHB,
    CNT = Signals::CNT,
    END = Signals::END,
}

impl TryFrom<u8> for VideoSignal {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            Signals::SHS => Ok(Self::SHS),
            Signals::RHS => Ok(Self::RHS),
            Signals::RCB => Ok(Self::RCB),
            Signals::RHB => Ok(Self::RHB),
            Signals::LRHB => Ok(Self::LRHB),
            Signals::CNT => Ok(Self::CNT),
            Signals::SHB => Ok(Self::SHB),
            Signals::END => Ok(Self::END),
            _ => Err(()),
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
pub struct TIA {
    // HSYNC counter
    ctr: Counter,

    // Vertical sync
    vsync: bool,
    vblank: u8,
    late_reset_hblank: bool,

    // Horizontal sync
    wsync: bool,

    // Input
    // I'm only implementing player 0 joystick controls, so only one input port
    inpt4_port: bool,
    inpt4_latch: bool,

    // Collision registers
    cxm0p: u8,
    cxm1p: u8,
    cxp0fb: u8,
    cxp1fb: u8,
    cxm0fb: u8,
    cxm1fb: u8,
    cxblpf: u8,
    cxppmm: u8,

    colors: SharedColor,

    // Graphics
    pf: Playfield,
    p0: Player,
    p1: Player,
    m0: Missile,
    m1: Missile,
    bl: Ball,

    // One scanline of pixels to be rendered. It's up to the calling code to call
    // `get_scanline_pixels` at the end of each scanline.
    pixels: [Rgba<u8>; LINE_LENGTH],
}

impl Default for TIA {
    fn default() -> Self {
        let colors = Rc::new(RefCell::new(Colors::new()));
        let hsync_ctr = Counter::new(57, 0);
        let pf = Playfield::new(colors.clone());
        let bl = Ball::new(colors.clone());
        let m0 = Missile::new(colors.clone(), PlayerType::Player0);
        let m1 = Missile::new(colors.clone(), PlayerType::Player1);
        let p0 = Player::new(colors.clone(), PlayerType::Player0);
        let p1 = Player::new(colors.clone(), PlayerType::Player1);

        Self {
            ctr: hsync_ctr,

            vsync: false,
            vblank: 0,
            late_reset_hblank: false,

            wsync: false,

            // These two ports have latches that are both enabled by writing a "1" or disabled by
            // writing a "0" to D6 of VBLANK. When disabled, the microprocessor reads the logic
            // level of the port directly. When enabled, the latch is set for logic one and remains
            // that way until its port goes LOW.
            inpt4_port: false,
            inpt4_latch: true,

            cxm0p: 0,
            cxm1p: 0,
            cxp0fb: 0,
            cxp1fb: 0,
            cxm0fb: 0,
            cxm1fb: 0,
            cxblpf: 0,
            cxppmm: 0,

            colors,

            pf,
            bl,
            m0,
            m1,
            p0,
            p1,

            pixels: [Rgba([0, 0, 0, 0]); LINE_LENGTH],
        }
    }
}

impl TIA {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn in_vblank(&self) -> bool {
        (self.vblank & 0x02) != 0
    }

    pub fn in_vsync(&self) -> bool {
        self.vsync
    }

    pub fn cpu_halt(&self) -> bool {
        self.wsync
    }

    pub fn get_scanline_pixels(&self) -> &[Rgba<u8>; LINE_LENGTH] {
        &self.pixels
    }

    pub fn joystick_fire(&mut self, pressed: bool) {
        self.inpt4_port = !pressed;

        if !self.inpt4_port {
            // When the port goes LOW the latch goes LOW and remains that way (until re-disabled by
            // VBLANK Bit 6) regardless of what the port does
            self.inpt4_latch = false;
        }
    }

    fn reset_latches(&mut self) {
        self.inpt4_latch = true
    }

    // Resolve playfield/player/missile/ball priorities and return the color to
    // be rendered.
    fn get_pixel_color(&self) -> u8 {
        if !self.pf.priority() {
            // When pixels of two or more objects overlap each other, only the
            // pixel of the object with topmost priority is drawn to the screen.
            // The normal priority ordering is:
            //
            //  Priority     Color    Objects
            //  1 (highest)  COLUP0   P0, M0  (and left side of PF in SCORE-mode)
            //  2            COLUP1   P1, M1  (and right side of PF in SCORE-mode)
            //  3            COLUPF   BL, PF  (only BL in SCORE-mode)
            //  4 (lowest)   COLUBK   BK

            self.p0
                .get_color()
                .or(self.m0.get_color())
                .or(self.p1.get_color())
                .or(self.m1.get_color())
                .or(self.bl.get_color())
                .or(self.pf.get_color())
                .unwrap_or(self.colors.borrow().colubk())
        } else {
            // Optionally, the playfield and ball may be assigned to have higher
            // priority (by setting CTRLPF.2). The priority ordering is then:
            //
            //  Priority     Color    Objects
            //  1 (highest)  COLUPF   PF, BL  (always, the SCORE-bit is ignored)
            //  2            COLUP0   P0, M0
            //  3            COLUP1   P1, M1
            //  4 (lowest)   COLUBK   BK

            self.pf
                .get_color()
                .or(self.bl.get_color())
                .or(self.p0.get_color())
                .or(self.m0.get_color())
                .or(self.p1.get_color())
                .or(self.m1.get_color())
                .unwrap_or(self.colors.borrow().colubk())
        }
    }

    fn update_collisions(&mut self) {
        const BIT_6: u8 = 0x40;
        const BIT_7: u8 = 0x80;

        macro_rules! check_collision {
            ($register: ident, $a: expr, $b: expr, $c: expr) => {
                if $a.get_color().is_some() && $b.get_color().is_some() {
                    self.$register |= BIT_6;
                }
                if $a.get_color().is_some() && $c.get_color().is_some() {
                    self.$register |= BIT_7;
                }
            };
        }

        check_collision!(cxm0p, self.m0, self.p0, self.p1);
        check_collision!(cxm1p, self.m1, self.p1, self.p0);
        check_collision!(cxp0fb, self.p0, self.bl, self.pf);
        check_collision!(cxp1fb, self.p1, self.bl, self.pf);
        check_collision!(cxm0fb, self.m0, self.bl, self.pf);
        check_collision!(cxm1fb, self.m1, self.bl, self.pf);

        // bit 6 of CXLBPF is unused
        if self.bl.get_color().is_some() && self.pf.get_color().is_some() {
            self.cxblpf |= BIT_7
        }

        if self.m0.get_color().is_some() && self.m1.get_color().is_some() {
            self.cxppmm |= BIT_6
        }

        if self.p0.get_color().is_some() && self.p1.get_color().is_some() {
            self.cxppmm |= BIT_7
        }
    }

    fn visible_cycle(&self) -> bool {
        self.ctr.value() > Signals::RHB && self.ctr.value() <= Signals::END
    }

    fn in_late_reset(&self) -> bool {
        self.late_reset_hblank
            && self.ctr.value() > Signals::RHB
            && self.ctr.value() <= Signals::LRHB
    }

    pub fn clock(&mut self) {
        // Clock the horizontal sync counter
        let clocked = self.ctr.clock();

        if self.visible_cycle() {
            self.set_pixel();
        } else {
            // During HBLANK we apply extra HMOVE clocks
            self.apply_hmove_all();
        }

        if clocked {
            if let Ok(signal) = self.ctr.value().try_into() {
                self.handle_video_signal(signal);
            }
        }
    }

    fn set_pixel(&mut self) {
        // Playfield is clocked on every visible cycle
        self.pf.clock();

        // Update the collision registers
        self.update_collisions();

        let color = if self.in_late_reset() {
            // During LRHB we apply extra HMOVE clocks
            self.apply_hmove_all();
            DEFAULT_COLOR
        } else {
            // Player, missile, and ball counters only get clocked on visible cycles
            self.clock_visible_components();
            self.get_pixel_color() as usize
        };

        let x = self.ctr.internal_value as usize - H_BLANK_CLOCKS;
        self.pixels[x] = NTSC_PALETTE[color];
    }

    fn handle_video_signal(&mut self, signal: VideoSignal) {
        match signal {
            // If we've reset the counter back to 0, we've finished the scanline and started
            // a new scanline, in HBlank.
            VideoSignal::SHB => {
                // The SHB signal is used to set HB and clear HC.
                // Simply writing to the WSYNC causes the microprocessor to halt until the
                // electron beam reaches the right edge of the screen.
                self.wsync = false;
                self.late_reset_hblank = false;
            }
            VideoSignal::SHS => {
                // The SHS signal is used to set the horizontal sync HS signal and, together with RHS, it shapes it.
            }
            VideoSignal::RHS => {
                // The RHS signal resets the horizontal sync HS signal and triggers the color burst CB signal.
            }
            VideoSignal::RCB => {
                // The RCB signal resets the color burst CB.
            }
            VideoSignal::RHB => {
                // The RHB signal resets the HBLANK HB signal. It can be ignored for LRHB depending on the HMOVEL latch.
            }
            VideoSignal::LRHB => {
                // The LRHB signal resets the HBLANK HB signal later. It can be ignored for RHB depending on the HMOVEL latch.
            }
            VideoSignal::CNT => {
                // The playfield center signal CNT is starts to draw the second part of the playfield.
            }
            VideoSignal::END => {
                // The END signal resets the HC counter.
            }
        }
    }

    // Helper method to apply extra HMOVE clocks to all components
    fn apply_hmove_all(&mut self) {
        self.p0.apply_hmove();
        self.p1.apply_hmove();
        self.m0.apply_hmove();
        self.m1.apply_hmove();
        self.bl.apply_hmove();
    }

    // Helper method to clock player, missile, and ball counters on visible cycles
    fn clock_visible_components(&mut self) {
        self.p0.clock();
        self.p1.clock();
        self.m0.clock();
        self.m1.clock();
        self.bl.clock();
    }

    pub fn debug(&self) {
        //self.p0.debug();
        //self.p1.debug();
        //self.m0.debug();
        self.m1.debug();
    }

    // TODO: https://github.com/stella-emu/stella/blob/8fe2adf28affc0477ee91689edef3b90168cd3ce/src/emucore/tia/TIA.cxx#L1519
    // fn apply_rsync(&mut self) {
    //     const H_BLANK_CLOCKS: u8 = 68;
    //     const H_CLOCKS: u8 = 228;
    //     const H_PIXEL: u8 = 160;
    //     let x = if self.ctr.value() > H_BLANK_CLOCKS {
    //         self.ctr.value() - H_BLANK_CLOCKS
    //     } else {
    //         0
    //     };

    //     self.myHctrDelta = H_CLOCKS - 3 - self.ctr.value();

    //     if self.myFrameManager.is_rendering() {
    //         let start_index = (self.myFrameManager.get_y() * H_PIXEL + x) as usize;
    //         let end_index = start_index + (H_PIXEL - x) as usize;

    //         self.myBackBuffer[start_index..end_index].fill(0);
    //     }

    //     self.ctr.reset_to(H_CLOCKS - 3);
    // }
}

impl TIA {
    pub fn read(&mut self, address: TiaReadAddress) -> u8 {
        use TiaReadAddress::*;
        match address {
            CXM0P => self.cxm0p,
            CXM1P => self.cxm1p,
            CXP0FB => self.cxp0fb,
            CXP1FB => self.cxp1fb,
            CXM0FB => self.cxm0fb,
            CXM1FB => self.cxm1fb,
            CXBLPF => self.cxblpf,
            CXPPMM => self.cxppmm,
            INPT4 => {
                // Check the logic level of the port
                let mut level = self.inpt4_port;

                // When the latch is enabled in D6 of VBLANK, check the latch value aswell
                if (self.vblank & 0x40) != 0 {
                    level = level && self.inpt4_latch;
                }

                if level {
                    0x80
                } else {
                    0x00
                }
            }
            _ => 0,
        }
    }

    pub fn write(&mut self, address: TiaWriteAddress, val: u8) {
        use TiaWriteAddress::*;
        match address {
            //
            // Frame timing and synchronisation
            //
            VSYNC => self.vsync = (val & 0x02) != 0,
            VBLANK => {
                self.vblank = val;

                if (val & 0x80) != 0 {
                    // INPT4-5 latches are reset when D6 of VBLANK is 1
                    self.reset_latches();
                }
            }
            WSYNC => self.wsync = true,
            // TODO: Commenting this out fixes the frame shifted bown by 1 pixel
            // RSYNC   <strobe>  reset horizontal sync counter
            // from TIA_HW_Notes.txt:
            //
            // "RSYNC resets the two-phase clock for the HSync counter to the H@1
            // rising edge when strobed."
            // RSYNC => self.ctr.reset_to_h1(),
            RSYNC => (),

            //
            // Colors
            //
            COLUP0 => self.colors.borrow_mut().set_colup0(val & 0xfe),
            COLUP1 => self.colors.borrow_mut().set_colup1(val & 0xfe),
            COLUPF => self.colors.borrow_mut().set_colupf(val & 0xfe),
            COLUBK => self.colors.borrow_mut().set_colubk(val & 0xfe),
            CTRLPF => {
                self.pf.set_control(val);
                self.bl.set_nusiz(1 << ((val & 0b0011_0000) >> 4));
            }

            //
            // Playfield
            //
            PF0 => self.pf.set_pf0(val),
            PF1 => self.pf.set_pf1(val),
            PF2 => self.pf.set_pf2(val),

            //
            // Sprites
            //
            NUSIZ0 => {
                let player_copies = val & 0b0000_0111;

                self.m0.set_nusiz(val as usize);
                self.p0.set_nusiz(player_copies as usize);
            }
            NUSIZ1 => {
                let player_copies = val & 0b0000_0111;

                self.m1.set_nusiz(val as usize);
                self.p1.set_nusiz(player_copies as usize);
            }
            REFP0 => self.p0.set_horizontal_mirror((val & 0b0000_1000) != 0),
            REFP1 => self.p1.set_horizontal_mirror((val & 0b0000_1000) != 0),
            RESP0 => {
                // If the write takes place anywhere within horizontal blanking
                // then the position is set to the left edge of the screen (plus
                // a few pixels towards right: 3 pixels for P0/P1, and only 2
                // pixels for M0/M1/BL).
                self.p0.reset();
            }
            RESP1 => {
                self.p1.reset();
            }
            RESM0 => self.m0.reset(),
            RESM1 => self.m1.reset(),
            RESBL => self.bl.reset(),
            AUDC0 => {
                debug!("AUDC0: {}", val)
            }
            AUDC1 => {
                debug!("AUDC1: {}", val)
            }
            AUDF0 => {
                debug!("AUDF0: {}", val)
            }
            AUDF1 => {
                debug!("AUDF1: {}", val)
            }
            AUDV0 => {
                debug!("AUDV0: {}", val)
            }
            AUDV1 => {
                debug!("AUDV1: {}", val)
            }
            GRP0 => {
                self.p0.set_graphic(val);
                self.p1.set_vdel_value();
            }
            GRP1 => {
                self.p1.set_graphic(val);
                self.p0.set_vdel_value();
                self.bl.set_vdel_value();
            }
            ENAM0 => self.m0.set_enabled((val & 0x02) != 0),
            ENAM1 => self.m1.set_enabled((val & 0x02) != 0),
            ENABL => self.bl.set_enabled((val & 0x02) != 0),

            //
            // Horizontal motion
            //
            HMP0 => self.p0.set_hmove_value(val),
            HMP1 => self.p1.set_hmove_value(val),
            HMM0 => self.m0.set_hmove_value(val),
            HMM1 => self.m1.set_hmove_value(val),
            HMBL => self.bl.set_hmove_value(val),
            VDELP0 => self.p0.set_vdel((val & 0x01) != 0),
            VDELP1 => self.p1.set_vdel((val & 0x01) != 0),
            VDELBL => self.bl.set_vdel((val & 0x01) != 0),
            RESMP0 => {
                if (val & 0x02) != 0 {
                    self.m0.reset_to_player(&self.p0);
                }
            }
            RESMP1 => {
                if (val & 0x02) != 0 {
                    self.m1.reset_to_player(&self.p1);
                }
            }
            HMOVE => {
                self.bl.start_hmove();
                self.m0.start_hmove();
                self.m1.start_hmove();
                self.p0.start_hmove();
                self.p1.start_hmove();

                self.late_reset_hblank = true;
            }
            HMCLR => {
                self.bl.hmclr();
                self.m0.hmclr();
                self.m1.hmclr();
                self.p0.hmclr();
                self.p1.hmclr();
            }
            CXCLR => {
                self.cxm0p = 0;
                self.cxm1p = 0;
                self.cxp0fb = 0;
                self.cxp1fb = 0;
                self.cxm0fb = 0;
                self.cxm1fb = 0;
                self.cxblpf = 0;
                self.cxppmm = 0;
            }
        }
    }
}
