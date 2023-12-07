mod bus;
mod cpu6507;
#[allow(clippy::upper_case_acronyms)]
pub(crate) mod memory;
mod opcode;
mod riot;
mod tia;

use crate::{bus::AtariBus, cpu6507::CPU6507, riot::RIOT, tia::TIA};
use image::Rgba;
use log::info;
use std::{cell::RefCell, error::Error, fs::File, io::Read, rc::Rc};

type SharedRIOT = Rc<RefCell<RIOT>>;
type SharedTIA = Rc<RefCell<TIA>>;
// type SharedDebugger = Rc<RefCell<Debugger>>;

const CLOCKS_PER_SCANLINE: usize = 228;

pub struct EmulatorCore {
    cpu: CPU6507,
    tia: SharedTIA,
    riot: SharedRIOT,
    frame_pixels: [[Rgba<u8>; 160]; 192],
}

pub fn init_emulator<P: AsRef<str>>(rom_path: P) -> Result<EmulatorCore, Box<dyn Error>> {
    let (riot, tia, cpu) = initialize_components(rom_path)?;
    let frame_pixels = [[Rgba::<u8>([0, 0, 0, 0xff]); 160]; 192];
    Ok(EmulatorCore {
        cpu,
        tia,
        riot,
        frame_pixels,
    })
}

impl EmulatorCore {
    pub fn frame_pixels(&self) -> &[[Rgba<u8>; 160]; 192] {
        &self.frame_pixels
    }

    pub fn run(&mut self) {
        // VSync
        while self.tia.borrow().in_vsync() {
            self.scanline();
        }

        // VBlank
        while self.tia.borrow().in_vblank() {
            self.scanline();
        }

        for i in 0..192 {
            if self.tia.borrow().in_vblank() {
                break;
            }
            self.scanline();

            let borrowed_tia = self.tia.borrow();
            let array: &[Rgba<u8>] = borrowed_tia.get_scanline_pixels();
            self.frame_pixels[i] = array.try_into().expect("Conversion failed");
        }

        // Overscan
        while !self.tia.borrow().in_vsync() {
            self.scanline();
        }
    }

    fn handle_riot_clock(&self, c: usize) {
        if c % 3 == 0 {
            self.riot.borrow_mut().clock();
        }
    }

    fn handle_cpu_clock(&mut self, c: usize) {
        if !self.tia.borrow().cpu_halt() && c % 3 == 2 {
            self.cpu.clock();
        }
    }

    fn scanline(&mut self) {
        for c in 0..CLOCKS_PER_SCANLINE {
            self.handle_riot_clock(c);
            self.tia.borrow_mut().clock();
            self.handle_cpu_clock(c);
        }
    }
}

pub trait KeyEvent {
    fn up(&mut self, pressed: bool);
    fn down(&mut self, pressed: bool);
    fn left(&mut self, pressed: bool);
    fn right(&mut self, pressed: bool);
    fn select(&mut self, pressed: bool);
    fn reset(&mut self, pressed: bool);
    fn joystick_fire(&mut self, pressed: bool);
    fn color(&mut self);
    // TODO: Debugger
    // fn toggle(&mut self);
    // fn step_frame(&mut self);
}

impl KeyEvent for EmulatorCore {
    fn up(&mut self, pressed: bool) {
        self.riot.borrow_mut().up(pressed);
    }

    fn down(&mut self, pressed: bool) {
        self.riot.borrow_mut().down(pressed);
    }

    fn left(&mut self, pressed: bool) {
        self.riot.borrow_mut().left(pressed);
    }

    fn right(&mut self, pressed: bool) {
        self.riot.borrow_mut().right(pressed);
    }

    fn reset(&mut self, pressed: bool) {
        self.riot.borrow_mut().reset(pressed);
    }

    fn select(&mut self, pressed: bool) {
        self.riot.borrow_mut().select(pressed);
    }

    fn joystick_fire(&mut self, pressed: bool) {
        self.tia.borrow_mut().joystick_fire(pressed);
    }

    fn color(&mut self) {
        self.riot.borrow_mut().color();
    }
}

fn initialize_components<P: AsRef<str>>(
    rom_path: P,
) -> Result<(SharedRIOT, SharedTIA, CPU6507), Box<dyn Error>> {
    let mut fh = File::open(rom_path.as_ref()).expect("unable to open rom");

    let mut rom = vec![];
    let bytes = fh.read_to_end(&mut rom).expect("unable to read rom data");
    info!("ROM: {} ({} bytes)", rom_path.as_ref(), bytes);

    info!("RIOT: init");
    let riot = Rc::new(RefCell::new(RIOT::new()));
    riot.borrow_mut().up(false);
    riot.borrow_mut().down(false);
    riot.borrow_mut().left(false);
    riot.borrow_mut().right(false);
    riot.borrow_mut().select(false);
    riot.borrow_mut().reset(false);

    info!("TIA: init");
    let tia = Rc::new(RefCell::new(TIA::new()));
    tia.borrow_mut().joystick_fire(false);

    let bus = AtariBus::new(tia.clone(), riot.clone(), rom);

    info!("CPU: init");
    let mut cpu = CPU6507::new(Box::new(bus));
    cpu.reset();

    Ok((riot, tia, cpu))
}
