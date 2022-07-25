use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use std::env::args;
use std::process;
use std::time::Duration;

pub mod chip8;

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().expect("Video error");

    let window = video_subsystem
        .window("CHIP-8", 640, 320)
        .position_centered()
        .build()
        .map_err(|op| op.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|op| op.to_string())?;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let texture_creator = canvas.texture_creator();

    let mut chip_8 = chip8::State::new();

    let mut event_pump = sdl_context.event_pump()?;

    chip_8.initialize();

    let mut args = args();
    args.next();
    chip_8.load_game(args.next().expect("No game provided"))?;
    // chip_8.load_buffer(&[
    //     0x00, 0xE0, 0x70, 0x01, 0x71, 0x01, 0x62, 0x0A, 0xF2, 0x29, 0xD0, 0x15, 0x12, 0x02,
    // ]);

    loop {
        chip_8.emulate_cycle();
        if chip_8.draw_flag {
            let mut texture = texture_creator
                .create_texture_streaming(PixelFormatEnum::RGB24, 64, 32)
                .map_err(|op| op.to_string())?;
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            texture.with_lock(Rect::new(0, 0, 64, 32), |buffer, pitch| {
                for (index, value) in chip_8
                    .get_graphics_buffer()
                    .into_iter()
                    .map(|x| [x, x, x])
                    .flatten()
                    .enumerate()
                {
                    if value == 1 {
                        buffer[index] = 255;
                    }
                }
            })?;
            canvas.copy(&texture, None, None)?;
            canvas.present();
            chip_8.draw_flag = false;
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { timestamp: _ } => {
                    process::exit(0);
                }
                Event::KeyDown {
                    timestamp: _,
                    window_id: _,
                    keycode,
                    scancode: _,
                    keymod: _,
                    repeat: _,
                } => {
                    if let Some(key) = keycode {
                        chip_8.set_key(
                            match key {
                                Keycode::Num1 => 0x1,
                                Keycode::Num2 => 0x2,
                                Keycode::Num3 => 0x3,
                                Keycode::Num4 => 0xC,
                                Keycode::Q => 0x4,
                                Keycode::W => 0x5,
                                Keycode::E => 0x6,
                                Keycode::R => 0xD,
                                Keycode::A => 0x7,
                                Keycode::S => 0x8,
                                Keycode::D => 0x9,
                                Keycode::F => 0xE,
                                Keycode::Z => 0xA,
                                Keycode::X => 0x0,
                                Keycode::C => 0xB,
                                Keycode::V => 0xF,
                                _ => 0xFF,
                            },
                            1,
                        );
                    }
                }
                Event::KeyUp {
                    timestamp: _,
                    window_id: _,
                    keycode,
                    scancode: _,
                    keymod: _,
                    repeat: _,
                } => {
                    if let Some(key) = keycode {
                        chip_8.set_key(
                            match key {
                                Keycode::Num1 => 0x1,
                                Keycode::Num2 => 0x2,
                                Keycode::Num3 => 0x3,
                                Keycode::Num4 => 0xC,
                                Keycode::Q => 0x4,
                                Keycode::W => 0x5,
                                Keycode::E => 0x6,
                                Keycode::R => 0xD,
                                Keycode::A => 0x7,
                                Keycode::S => 0x8,
                                Keycode::D => 0x9,
                                Keycode::F => 0xE,
                                Keycode::Z => 0xA,
                                Keycode::X => 0x0,
                                Keycode::C => 0xB,
                                Keycode::V => 0xF,
                                _ => 0xFF,
                            },
                            0,
                        );
                    }
                }
                _ => {}
            }
        }
        
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
