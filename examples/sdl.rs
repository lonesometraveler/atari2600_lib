use atari2600_audio::{
    sdl_audio::{AudioPlayer, AudioSubsystemSendWrapper},
    Aud, SAMPLE_FREQ,
};
use atari2600_lib::{EmulatorCore, KeyEvent};
use image::Rgba;
use log::{error, info};
use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, Texture, TextureCreator, WindowCanvas};
use sdl2::video::{Window, WindowContext};
use sdl2::{EventPump, VideoSubsystem};
use std::env;
use std::error::Error;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};

const ATARI_FPS: f64 = 60.0;
const FRAME_DURATION: Duration = Duration::from_millis(((1.0 / ATARI_FPS) * 1000.0) as u64);
const HORIZONTAL_SCALING_FACTOR: usize = 4;
const VERTICAL_SCALING_FACTOR: usize = 2;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let rom_path = env::args().nth(1).expect("missing argument: rom file");

    let mut emulator_core = atari2600_lib::init_emulator(rom_path)?;

    info!("Graphics: init");
    let width = 160 * HORIZONTAL_SCALING_FACTOR as u32;
    let height = 192 * VERTICAL_SCALING_FACTOR as u32;

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let audio_subsystem = sdl_context.audio()?;

    // Specify the desired audio format
    let desired_spec = AudioSpecDesired {
        freq: Some(SAMPLE_FREQ as i32),
        channels: Some(1), // Mono
        samples: None,     // Default sample size
    };

    // Messaging channels for tone
    let (tx, rx) = channel::<Vec<f32>>();

    let audio_player = AudioPlayer::new(
        AudioSubsystemSendWrapper(audio_subsystem.clone()),
        desired_spec,
        rx,
    );
    thread::spawn(move || audio_player.run());

    let (mut canvas, texture_creator) =
        create_sdl_window_and_canvas(video_subsystem, width, height)?;

    let mut texture = initialize_texture(width, height, &texture_creator)?;

    canvas.clear();
    canvas.copy(&texture, None, None)?;
    canvas.present();

    let mut event_pump = sdl_context.event_pump()?;
    let mut fps_start = Instant::now();

    loop {
        emulator_core.run();

        render_frame(&mut canvas, &mut texture, emulator_core.frame_pixels())?;

        handle_events(&mut emulator_core, &mut event_pump);

        // Audio
        if emulator_core.should_play_audio() {
            let tone = emulator_core.get_tone();
            if let Err(e) = tx.send(tone) {
                error!("Error sending tone: {}", e);
            };
        }

        if let Some(delay) = FRAME_DURATION.checked_sub(fps_start.elapsed()) {
            thread::sleep(delay);
        }

        fps_start = Instant::now();
    }
}

fn handle_events(emu: &mut EmulatorCore, event_pump: &mut EventPump) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. } => std::process::exit(0),
            Event::KeyDown {
                keycode: Some(key), ..
            } => {
                match key {
                    // Joystick controls
                    Keycode::W => emu.up(true),
                    Keycode::A => emu.left(true),
                    Keycode::S => emu.down(true),
                    Keycode::D => emu.right(true),
                    Keycode::N => emu.joystick_fire(true),

                    // Console switches
                    Keycode::F1 => emu.select(true),
                    Keycode::F2 => emu.reset(true),
                    Keycode::F3 => emu.color(),

                    _ => {}
                }
            }
            Event::KeyUp {
                keycode: Some(key), ..
            } => match key {
                Keycode::W => emu.up(false),
                Keycode::A => emu.left(false),
                Keycode::S => emu.down(false),
                Keycode::D => emu.right(false),
                Keycode::N => emu.joystick_fire(false),

                Keycode::F1 => emu.select(false),
                Keycode::F2 => emu.reset(false),

                _ => {}
            },
            _ => {}
        }
    }
}

fn render_frame(
    canvas: &mut WindowCanvas,
    texture: &mut Texture,
    frame_pixels: &[[Rgba<u8>; 160]; 192],
) -> Result<(), Box<dyn Error>> {
    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        for (y, row) in frame_pixels.iter().enumerate() {
            for (x, color) in row.iter().enumerate() {
                for row_offset in (0..VERTICAL_SCALING_FACTOR).map(|i| i * pitch) {
                    for col_offset in (0..HORIZONTAL_SCALING_FACTOR).map(|i| i * 3) {
                        let offset = VERTICAL_SCALING_FACTOR * (y * pitch)
                            + HORIZONTAL_SCALING_FACTOR * (x * 3)
                            + row_offset
                            + col_offset;
                        buffer[offset..offset + 3].copy_from_slice(&color.0[0..3]);
                    }
                }
            }
        }
    })?;

    canvas.clear();
    canvas.copy(texture, None, None).unwrap();
    canvas.present();

    Ok(())
}

fn initialize_texture(
    width: u32,
    height: u32,
    texture_creator: &TextureCreator<WindowContext>,
) -> Result<Texture, Box<dyn Error>> {
    let mut texture =
        texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, width, height)?;

    texture.with_lock(None, |buffer: &mut [u8], _pitch: usize| {
        // Initialise a black canvas
        for y in 0..height {
            for x in 0..width {
                let offset = (y * width) + x;
                buffer[offset as usize] = 0;
            }
        }
    })?;

    Ok(texture)
}

fn create_sdl_window_and_canvas(
    video_subsystem: VideoSubsystem,
    width: u32,
    height: u32,
) -> Result<(Canvas<Window>, TextureCreator<WindowContext>), Box<dyn Error>> {
    info!("  video driver: {}", video_subsystem.current_video_driver());

    let window = video_subsystem
        .window("atari2600", width, height)
        .position_centered()
        .build()?;

    let canvas = window.into_canvas().target_texture().build()?;

    info!("  canvas driver: {}", canvas.info().name);

    let texture_creator = canvas.texture_creator();

    Ok((canvas, texture_creator))
}
