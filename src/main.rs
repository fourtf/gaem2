extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
// use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
// use sdl2::video::GLProfile;
use std::thread::sleep;
use std::time::{Duration, Instant};

mod gl;
mod map;
mod rect;
mod resources;

use map::Map;
use rect::Rect;
use resources::*;

fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(index as u32);
        }
    }
    None
}

fn main() -> Result<(), String> {
    println!("{}", -0.1 as i64);

    println!(
        "CWD: {}",
        std::env::current_dir().unwrap().to_str().unwrap() // std::path::Path::new(".")
                                                           //     .canonicalize()
                                                           //     .unwrap()
                                                           //     .to_str()
                                                           //     .unwrap()
    );

    // int nrAttributes;
    // glGetIntegerv(GL_MAX_VERTEX_ATTRIBS, &nrAttributes);
    // std::cout << "Maximum nr of vertex attributes supported: " << nrAttributes << std::endl;

    if find_sdl_gl_driver().is_none() {
        Err("Could not initialize opengl")?
    }

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // let window2 = video_subsystem
    //     .window("xD", 600, 400)
    //     .position_centered()
    //     .build()
    //     .map_err(|e| e.to_string())?;

    // sdl2::messagebox::show_simple_message_box(
    //     sdl2::messagebox::MessageBoxFlag::INFORMATION,
    //     "xD",
    //     "uff",
    //     Some(&window2),
    // );

    let window = video_subsystem
        .window("xD", 711, 400)
        .position_centered()
        .opengl()
        .allow_highdpi()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .index(find_sdl_gl_driver().unwrap())
        .target_texture()
        .build()
        .map_err(|e| e.to_string())?;

    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
    canvas.window().gl_set_context_to_current()?;
    init_drawing();

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

    let mut black_texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 64, 64)
        .map_err(|e| e.to_string())?;
    black_texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        for y in 0..64 {
            for x in 0..64 {
                let offset = y * pitch + x * 3;
                buffer[offset] = 0u8;
                buffer[offset + 1] = 0u8;
                buffer[offset + 2] = 0u8;
            }
        }
    })?;

    let mut player_rect = Rect::new(1.0, 0.0, 0.9, 0.7);
    let mut player_on_floor = false;
    let mut player_sliding_on_wall = false;
    let mut player_sliding_on_left_wall = false;
    #[allow(unused_assignments)]
    let mut player_sliding_on_right_wall = false;
    let mut player_dy: f64 = 0.0;
    let mut player_dx: f64 = 0.0;

    let mut is_left_down = false;
    let mut is_right_down = false;
    let mut is_jump_press = false;

    let map = Map::new(vec![
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    ]);

    let mut content = Content::new();
    let mut block_shader = Shader::frag(&mut content, "shaders/block.frag");
    block_shader.load().unwrap();
    let mut bg_shader = Shader::frag(&mut content, "shaders/bg.frag");
    bg_shader.load().unwrap();

    let mut player_texture = Texture::new(&mut content, "textures/pajaW.png");
    player_texture.load().unwrap();

    let mut event_pump = sdl_context.event_pump()?;

    let mut now = Instant::now();
    'main_loop: loop {
        // replace with as_secs_f64 when available
        let time_passed = (((Instant::now() - now).as_micros() as f64) / 1000000.0).min(0.016);
        //println!("time passed: {}", time_passed);

        now = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main_loop,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    // println!("+{}", key);
                    if key == Keycode::A {
                        is_left_down = true;
                    } else if key == Keycode::D {
                        is_right_down = true;
                    } else if key == Keycode::Space {
                        is_jump_press = true;
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    // println!("-{}", key);
                    if key == Keycode::A {
                        is_left_down = false;
                    } else if key == Keycode::D {
                        is_right_down = false;
                    }
                }
                _ => {}
            }
        }

        // debug - update resources
        content.update();

        // physics
        if is_jump_press {
            if player_on_floor {
                player_dy = -25.0;
            } else if player_sliding_on_wall {
                player_dy = -15.0;
                player_dx = if player_sliding_on_left_wall {
                    10.0
                } else {
                    -10.0
                };
            }
        }

        player_dy += 50.0 * time_passed;
        player_dy = player_dy.max(-20.0).min(15.0);

        // left + right input:
        let turn_speed = if player_on_floor { 100.0 } else { 70.0 };
        if is_left_down {
            player_dx -= turn_speed * time_passed;
        }
        if is_right_down {
            player_dx += turn_speed * time_passed;
        }

        if !is_right_down && !is_left_down {
            let slow_down_speed = if player_on_floor { 150.0 } else { 75.0 };
            if player_dx > 0.0 {
                player_dx = (player_dx - slow_down_speed * time_passed).max(0.0);
            } else {
                player_dx = (player_dx + slow_down_speed * time_passed).min(0.0);
            }
        }

        player_dx = player_dx.max(-10.0).min(10.0);

        let player_collision = map.move_item(&mut player_rect, player_dx, player_dy, time_passed);

        // floor collision
        player_on_floor = player_collision.is_on_floor();
        if player_on_floor {
            player_dy = 0.0;
        }

        // ceiling collision
        if player_collision.top {
            player_dy = player_dy.max(0.0);
        }

        // wall collision
        player_sliding_on_left_wall = player_collision.left;
        player_sliding_on_right_wall = player_collision.right;
        player_sliding_on_wall = player_sliding_on_left_wall || player_sliding_on_right_wall;

        if player_sliding_on_wall {
            player_dy = player_dy.min(4.0);
        }

        // cleanup
        is_jump_press = false;

        // render level
        let renderer = Renderer::new();
        renderer.clear(0.5, 0.5, 0.5);

        bg_shader.select(&mut content);
        renderer.rect2(0.0, 0.0, 16.0, 9.0);

        block_shader.select(&mut content);

        let blocks = &map.blocks;
        for i_line in 0..blocks.len() {
            let line = &blocks[i_line];
            for i_col in 0..line.len() {
                let block = line[i_col];
                if block == 1u8 {
                    renderer.rect2(i_col as f64 * 1.0, i_line as f64 * 1.0, 1.0, 1.0);
                }
            }
        }

        // render player
        Shader::reset();
        renderer.rgb(1.0, 0.5, 0.0);
        renderer.rect(&player_rect);

        canvas.present();

        // let frame_time = Duration::from_micros(50000);
        let frame_time = Duration::from_micros(6944);
        // let frame_time = Duration::from_micros(16666);
        // println!(
        //     "{:?}",
        //     frame_time - std::cmp::min(Instant::now() - now, frame_time)
        // );
        // Instant::now() - now);

        sleep(frame_time - std::cmp::min(Instant::now() - now, frame_time));
        // std::thread::sleep(std::time::Duration::from_micros(16666));
    }

    Ok(())
}
