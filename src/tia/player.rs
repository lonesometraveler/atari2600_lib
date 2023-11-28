use crate::tia::counter::Counter;
use crate::tia::graphic::ScanCounter;
use crate::tia::PlayerType;

use super::graphic::Graphic;
use super::SharedColor;

pub struct Player {
    colors: SharedColor,
    hmove_offset: u8,
    ctr: Counter,
    scan_counter: ScanCounter,
    nusiz: usize,

    // The REFPx register, for rendering backwards
    horizontal_mirror: bool,
    // The 8-bit graphic to draw
    graphic: u8,
    // The VDELPx register
    vdel: bool,
    old_value: u8,

    player: PlayerType,
}

impl Player {
    pub fn new(colors: SharedColor, player: PlayerType) -> Self {
        Self {
            colors,
            player,

            hmove_offset: 0,
            ctr: Counter::default(),

            horizontal_mirror: false,
            nusiz: 0,
            graphic: 0,

            vdel: false,
            old_value: 0,

            scan_counter: ScanCounter::default(),
        }
    }

    pub fn counter(&self) -> &Counter {
        &self.ctr
    }

    pub fn set_graphic(&mut self, graphic: u8) {
        self.graphic = graphic
    }

    pub fn set_horizontal_mirror(&mut self, reflect: bool) {
        self.horizontal_mirror = reflect
    }

    pub fn set_vdel(&mut self, v: bool) {
        self.vdel = v
    }

    pub fn set_vdel_value(&mut self) {
        self.old_value = self.graphic
    }

    pub fn set_hmove_value(&mut self, v: u8) {
        self.hmove_offset = v
    }

    pub fn set_nusiz(&mut self, v: usize) {
        self.nusiz = v & 0x0f
    }

    pub fn hmclr(&mut self) {
        self.hmove_offset = 0
    }

    #[allow(dead_code)]
    pub fn debug(&self) {
        if !self.should_draw_graphic() && !self.should_draw_copy() {
            return;
        }

        println!("p: {:?}, ctr: {}, grp: {:08b}, gv: {:?}, refp: {}, nusiz: {:03b}, vdel: {}, old_value: {:08b}",
                 self.player,
                 self.ctr.value(),
                 self.graphic,
                 self.scan_counter.bit_value,
                 self.horizontal_mirror,
                 self.nusiz,
                 self.vdel,
                 self.old_value,
        );
    }
}

impl Graphic for Player {
    // Player sprites start 1 tick later than other sprites
    const INIT_DELAY: isize = 5;
    // How many bits to a graphic
    const GRAPHIC_SIZE: isize = 8;

    fn size(&self) -> usize {
        match self.nusiz & 0x0f {
            0b0101 => 2,
            0b0111 => 4,
            _ => 1,
        }
    }

    fn pixel_bit(&self) -> bool {
        if let Some(x) = self.scan_counter.bit_idx {
            let graphic = if self.vdel {
                self.old_value
            } else {
                self.graphic
            };

            if self.horizontal_mirror {
                ((graphic >> x) & 1) != 0
            } else {
                ((graphic >> (7 - x)) & 1) != 0
            }
        } else {
            false
        }
    }

    fn should_draw_copy(&self) -> bool {
        let count = self.ctr.value();

        (count == 3 && (self.nusiz == 0b001 || self.nusiz == 0b011))
            || (count == 7 && (self.nusiz == 0b010 || self.nusiz == 0b011 || self.nusiz == 0b110))
            || (count == 15 && (self.nusiz == 0b100 || self.nusiz == 0b110))
    }

    fn get_color(&self) -> Option<u8> {
        self.scan_counter
            .bit_value
            .and_then(|bit_value| match (bit_value, &self.player) {
                (true, PlayerType::Player0) => Some(self.colors.borrow().colup0()),
                (true, PlayerType::Player1) => Some(self.colors.borrow().colup1()),
                (false, _) => None,
            })
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
