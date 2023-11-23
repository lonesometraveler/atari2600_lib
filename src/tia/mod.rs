mod ball;
mod color;
mod counter;
mod graphics;
mod missile;
mod palette;
mod player;
mod playfield;

use std::cell::RefCell;
use std::rc::Rc;

use image::Rgba;

use crate::bus::Bus;
use crate::tia::ball::Ball;
use crate::tia::color::Colors;
use crate::tia::counter::Counter;
use crate::tia::missile::Missile;
use crate::tia::palette::DEFAULT_COLOR;
use crate::tia::player::Player;
use crate::tia::playfield::Playfield;

use log::debug;

use self::palette::NTSC_PALETTE;

const BALL_INIT_DELAY: isize = 4;
const BALL_GRAPHIC_SIZE: isize = 1;
const MISSILE_INIT_DELAY: isize = 4;
const MISSILE_GRAPHIC_SIZE: isize = 1;
// Player sprites start 1 tick later than other sprites
const PLAYER_INIT_DELAY: isize = 5;
// How many bits to a graphic
const PLAYER_GRAPHIC_SIZE: isize = 8;

#[derive(Debug)]
pub enum PlayerType {
    Player0,
    Player1,
}

// Set H-SYNC
const SHS: u8 = 4;

// Reset H-SYNC
const RHS: u8 = 8;

// ColourBurst
const RCB: u8 = 12;

// Reset H-BLANK
const RHB: u8 = 16;

// Late RHB
const LRHB: u8 = 18;

// Center
const CNT: u8 = 36;

// RESET, H-BLANK
const SHB: u8 = 56;

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

    colors: Rc<RefCell<Colors>>,

    // Graphics
    pf: Playfield,
    p0: Player,
    p1: Player,
    m0: Missile,
    m1: Missile,
    bl: Ball,

    // One scanline of pixels to be rendered. It's up to the calling code to call
    // `get_scanline_pixels` at the end of each scanline.
    pixels: Vec<Rgba<u8>>,
}

impl Default for TIA {
    fn default() -> Self {
        let colors = Rc::new(RefCell::new(Colors::new()));
        let hsync_ctr = Counter::new(57, 0);
        let pf = Playfield::new(colors.clone());
        let bl = Ball::new(colors.clone(), BALL_INIT_DELAY, BALL_GRAPHIC_SIZE);
        let m0 = Missile::new(
            colors.clone(),
            PlayerType::Player0,
            MISSILE_INIT_DELAY,
            MISSILE_GRAPHIC_SIZE,
        );
        let m1 = Missile::new(
            colors.clone(),
            PlayerType::Player1,
            MISSILE_INIT_DELAY,
            MISSILE_GRAPHIC_SIZE,
        );
        let p0 = Player::new(
            colors.clone(),
            PlayerType::Player0,
            PLAYER_INIT_DELAY,
            PLAYER_GRAPHIC_SIZE,
        );
        let p1 = Player::new(
            colors.clone(),
            PlayerType::Player1,
            PLAYER_INIT_DELAY,
            PLAYER_GRAPHIC_SIZE,
        );

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

            pixels: vec![Rgba([0, 0, 0, 0]); 160],
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

    pub fn get_scanline_pixels(&self) -> &Vec<Rgba<u8>> {
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
        if self.m0.get_color().is_some() && self.p0.get_color().is_some() {
            self.cxm0p |= 0x40
        }
        if self.m0.get_color().is_some() && self.p1.get_color().is_some() {
            self.cxm0p |= 0x80
        }

        if self.m1.get_color().is_some() && self.p0.get_color().is_some() {
            self.cxm1p |= 0x40
        }
        if self.m1.get_color().is_some() && self.p1.get_color().is_some() {
            self.cxm1p |= 0x80
        }

        if self.p0.get_color().is_some() && self.bl.get_color().is_some() {
            self.cxp0fb |= 0x40
        }
        if self.p0.get_color().is_some() && self.pf.get_color().is_some() {
            self.cxp0fb |= 0x80
        }

        if self.p1.get_color().is_some() && self.bl.get_color().is_some() {
            self.cxp1fb |= 0x40
        }
        if self.p1.get_color().is_some() && self.pf.get_color().is_some() {
            self.cxp1fb |= 0x80
        }

        if self.m0.get_color().is_some() && self.bl.get_color().is_some() {
            self.cxm0fb |= 0x40
        }
        if self.m0.get_color().is_some() && self.pf.get_color().is_some() {
            self.cxm0fb |= 0x80
        }

        if self.m1.get_color().is_some() && self.bl.get_color().is_some() {
            self.cxm0fb |= 0x40
        }
        if self.m1.get_color().is_some() && self.pf.get_color().is_some() {
            self.cxm0fb |= 0x80
        }

        if self.bl.get_color().is_some() && self.pf.get_color().is_some() {
            self.cxblpf |= 0x80
        }

        if self.m0.get_color().is_some() && self.m1.get_color().is_some() {
            self.cxppmm |= 0x40
        }
        if self.p0.get_color().is_some() && self.p1.get_color().is_some() {
            self.cxppmm |= 0x80
        }
    }

    fn visible_cycle(&self) -> bool {
        self.ctr.value() > RHB && self.ctr.value() <= SHB
    }

    fn in_late_reset(&self) -> bool {
        self.late_reset_hblank && self.ctr.value() > RHB && self.ctr.value() <= LRHB
    }

    pub fn clock(&mut self) {
        // Clock the horizontal sync counter
        let clocked = self.ctr.clock();

        if self.visible_cycle() {
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

            let x = self.ctr.internal_value as usize - 68;
            self.pixels[x] = NTSC_PALETTE[color];
        } else {
            // During HBLANK we apply extra HMOVE clocks
            self.apply_hmove_all()
        }

        if clocked {
            match self.ctr.value() {
                // If we've reset the counter back to 0, we've finished the scanline and started
                // a new scanline, in HBlank.
                0 => {
                    // Simply writing to the WSYNC causes the microprocessor to halt until the
                    // electron beam reaches the right edge of the screen.
                    self.wsync = false;
                    self.late_reset_hblank = false;
                }
                RHB => { /* Reset HBlank */ }
                LRHB => { /* Late Reset HBlank */ }
                _ => {}
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
}

impl Bus for TIA {
    // https://problemkaputt.de/2k6specs.htm#memoryandiomap

    fn read(&mut self, address: u16) -> u8 {
        match address {
            // CXM0P   11......  read collision M0-P1, M0-P0 (Bit 7,6)
            0x0030 => self.cxm0p,

            // CXM1P   11......  read collision M1-P0, M1-P1
            0x0031 => self.cxm1p,

            // CXP0FB  11......  read collision P0-PF, P0-BL
            0x0032 => self.cxp0fb,

            // CXP1FB  11......  read collision P1-PF, P1-BL
            0x0033 => self.cxp1fb,

            // CXM0FB  11......  read collision M0-PF, M0-BL
            0x0034 => self.cxm0fb,

            // CXM1FB  11......  read collision M1-PF, M1-BL
            0x0035 => self.cxm1fb,

            // CXBLPF  1.......  read collision BL-PF, unused
            0x0036 => self.cxblpf,

            // CXPPMM  11......  read collision P0-P1, M0-M1
            0x0037 => self.cxppmm,

            // INPT4   1.......  read input
            0x003C => {
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

    fn write(&mut self, address: u16, val: u8) {
        match address {
            //
            // Frame timing and synchronisation
            //

            // VSYNC   ......1.  vertical sync set-clear
            0x0000 => self.vsync = (val & 0x02) != 0,

            // VBLANK  11....1.  vertical blank set-clear
            0x0001 => {
                self.vblank = val;

                if (val & 0x80) != 0 {
                    // INPT4-5 latches are reset when D6 of VBLANK is 1
                    self.reset_latches();
                }
            }

            // WSYNC   <strobe>  wait for leading edge of horizontal blank
            0x0002 => self.wsync = true,

            // RSYNC   <strobe>  reset horizontal sync counter
            0x0003 => self.ctr.reset_to_h1(),

            //
            // Colors
            //

            // COLUP0  1111111.  color-lum player 0 and missile 0
            0x0006 => self.colors.borrow_mut().set_colup0(val & 0xfe),

            // COLUP1  1111111.  color-lum player 1 and missile 1
            0x0007 => self.colors.borrow_mut().set_colup1(val & 0xfe),

            // COLUPF  1111111.  color-lum playfield and ball
            0x0008 => self.colors.borrow_mut().set_colupf(val & 0xfe),

            // COLUBK  1111111.  color-lum background
            0x0009 => self.colors.borrow_mut().set_colubk(val & 0xfe),

            // CTRLPF  ..11.111  control playfield ball size & collisions
            0x000a => {
                self.pf.set_control(val);
                self.bl.set_nusiz(1 << ((val & 0b0011_0000) >> 4));
            }

            //
            // Playfield
            //

            // PF0     1111....  playfield register byte 0
            0x000d => self.pf.set_pf0(val),

            // PF1     11111111  playfield register byte 1
            0x000e => self.pf.set_pf1(val),

            // PF2     11111111  playfield register byte 2
            0x000f => self.pf.set_pf2(val),

            //
            // Sprites
            //

            // NUSIZ0  ..111111  number-size player-missile 0
            0x0004 => {
                let player_copies = val & 0b0000_0111;

                self.m0.set_nusiz(val);
                self.p0.set_nusiz(player_copies);
            }

            // NUSIZ1  ..111111  number-size player-missile 1
            0x0005 => {
                let player_copies = val & 0b0000_0111;

                self.m1.set_nusiz(val);
                self.p1.set_nusiz(player_copies);
            }

            // REFP0   ....1...  reflect player 0
            0x000b => self.p0.set_horizontal_mirror((val & 0b0000_1000) != 0),

            // REFP1   ....1...  reflect player 1
            0x000c => self.p1.set_horizontal_mirror((val & 0b0000_1000) != 0),

            // RESP0   <strobe>  reset player 0
            0x0010 => {
                // If the write takes place anywhere within horizontal blanking
                // then the position is set to the left edge of the screen (plus
                // a few pixels towards right: 3 pixels for P0/P1, and only 2
                // pixels for M0/M1/BL).
                self.p0.reset();
            }

            // RESP1   <strobe>  reset player 1
            0x0011 => {
                self.p1.reset();
            }

            // RESM0   <strobe>  reset missile 0
            0x0012 => self.m0.reset(),

            // RESM1   <strobe>  reset missile 1
            0x0013 => self.m1.reset(),

            // RESBL   <strobe>  reset ball
            0x0014 => self.bl.reset(),

            // AUDV0
            0x0015 => {
                debug!("AUDV0: {}", val)
            }

            // AUDV1
            0x0016 => {
                debug!("AUDV1: {}", val)
            }

            // AUDF0
            0x0017 => {
                debug!("AUDF0: {}", val)
            }

            // AUDF1
            0x0018 => {
                debug!("AUDF1: {}", val)
            }

            // AUDC0
            0x0019 => {
                debug!("AUDC0: {}", val)
            }

            // AUDC1
            0x001a => {
                debug!("AUDC1: {}", val)
            }

            // GRP0    11111111  graphics player 0
            0x001b => {
                self.p0.set_graphic(val);
                self.p1.set_vdel_value();
            }

            // GRP1    11111111  graphics player 1
            0x001c => {
                self.p1.set_graphic(val);
                self.p0.set_vdel_value();
                self.bl.set_vdel_value();
            }

            // ENAM0   ......1.  graphics (enable) missile 0
            0x001d => self.m0.set_enabled((val & 0x02) != 0),

            // ENAM1   ......1.  graphics (enable) missile 1
            0x001e => self.m1.set_enabled((val & 0x02) != 0),

            // ENABL   ......1.  graphics (enable) ball
            0x001f => self.bl.set_enabled((val & 0x02) != 0),

            //
            // Horizontal motion
            //

            // HMP0    1111....  horizontal motion player 0
            0x0020 => self.p0.set_hmove_value(val),

            // HMP1    1111....  horizontal motion player 1
            0x0021 => self.p1.set_hmove_value(val),

            // HMM0    1111....  horizontal motion missile 0
            0x0022 => self.m0.set_hmove_value(val),

            // HMM1    1111....  horizontal motion missile 1
            0x0023 => self.m1.set_hmove_value(val),

            // HMBL    1111....  horizontal motion ball
            0x0024 => self.bl.set_hmove_value(val),

            // VDELP0  .......1  vertical delay player 0
            0x0025 => self.p0.set_vdel((val & 0x01) != 0),

            // VDELP1  .......1  vertical delay player 1
            0x0026 => self.p1.set_vdel((val & 0x01) != 0),

            // VDELBL  .......1  vertical delay ball
            0x0027 => self.bl.set_vdel((val & 0x01) != 0),

            // RESMP0  ......1.  reset missile 0 to player 0
            0x0028 => {
                if (val & 0x02) != 0 {
                    self.m0.reset_to_player(&self.p0);
                }
            }

            // RESMP1  ......1.  reset missile 1 to player 1
            0x0029 => {
                if (val & 0x02) != 0 {
                    self.m1.reset_to_player(&self.p1);
                }
            }

            // HMOVE   <strobe>  apply horizontal motion
            0x002a => {
                self.bl.start_hmove();
                self.m0.start_hmove();
                self.m1.start_hmove();
                self.p0.start_hmove();
                self.p1.start_hmove();

                self.late_reset_hblank = true;
            }

            // HMCLR   <strobe>  clear horizontal motion registers
            0x002b => {
                self.bl.hmclr();
                self.m0.hmclr();
                self.m1.hmclr();
                self.p0.hmclr();
                self.p1.hmclr();
            }

            // CXCLR   <strobe>  clear collision latches
            0x002C => {
                self.cxm0p = 0;
                self.cxm1p = 0;
                self.cxp0fb = 0;
                self.cxp1fb = 0;
                self.cxm0fb = 0;
                self.cxm1fb = 0;
                self.cxblpf = 0;
                self.cxppmm = 0;
            }

            _ => {}
        }
    }
}
