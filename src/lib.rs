pub mod bus;
pub mod cpu6507;
pub mod debugger;
pub mod riot;
pub mod tia;

use std::{cell::RefCell, error::Error, fs::File, io::Read, rc::Rc};

use image::Rgba;
use log::*;

use crate::{bus::AtariBus, cpu6507::CPU6507, riot::RIOT, tia::TIA};

type SharedCPU = Rc<RefCell<CPU6507>>;
type SharedRIOT = Rc<RefCell<RIOT>>;
type SharedTIA = Rc<RefCell<TIA>>;
// type SharedDebugger = Rc<RefCell<Debugger>>;

const CLOCKS_PER_SCANLINE: usize = 228;

pub struct EmulatorCore {
    pub cpu: SharedCPU,
    pub tia: SharedTIA,
    pub riot: SharedRIOT,
    pub frame_pixels: [[Rgba<u8>; 160]; 192],
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

    fn handle_cpu_clock(&self, c: usize) {
        if !self.tia.borrow().cpu_halt() && c % 3 == 2 {
            self.cpu.borrow_mut().clock();
        }
    }

    fn scanline(&self) {
        for c in 0..CLOCKS_PER_SCANLINE {
            self.handle_riot_clock(c);
            self.tia.borrow_mut().clock();
            // self.debugger.borrow_mut().debug();
            self.handle_cpu_clock(c);
        }
    }
}

pub fn initialize_components<P: AsRef<str>>(
    rom_path: P,
) -> Result<(SharedRIOT, SharedTIA, SharedCPU), Box<dyn Error>> {
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
    // let mut cpu = CPU6507::new(Box::new(bus));
    let cpu = Rc::new(RefCell::new(CPU6507::new(Box::new(bus))));
    cpu.borrow_mut().reset();

    Ok((riot, tia, cpu))
}
