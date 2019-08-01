extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

fn main() -> Result<(), String> {
    let context = sdl2::init()?;
    let video = context.video()?;
    let window = video
        .window("xD", 600, 400)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .target_texture()
        .build()
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 256, 256)
        .map_err(|e| e.to_string())?;

    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        for y in 0..256 {
            for x in 0..256 {
                let offset = y * pitch + x * 3;
                buffer[offset] = x as u8;
                buffer[offset + 1] = y as u8;
                buffer[offset + 2] = 0;
            }
        }
    })?;

    let mut event_pump = context.event_pump()?;
    'main_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main_loop,
                _ => {}
            }
        }

        canvas.clear();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.copy(&texture, None, Some(Rect::new(0, 0, 256, 256)))?;
        canvas.set_draw_color(Color::RGB(255, 0, 0));
        canvas.copy(&texture, None, Some(Rect::new(256, 0, 256, 256)))?;
        canvas.set_draw_color(Color::RGB(0, 255, 0));
        canvas.copy(&texture, None, Some(Rect::new(0, 256, 256, 256)))?;
        canvas.set_draw_color(Color::RGB(0, 0, 255));
        canvas.copy(&texture, None, Some(Rect::new(256, 256, 256, 256)))?;

        canvas.present();

        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    Ok(())
}
