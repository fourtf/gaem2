extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
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

    if find_sdl_gl_driver().is_none() {
        Err("Could not initialize opengl")?
    }

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Compatibility);
    gl_attr.set_context_version(3, 3);

    let window = video_subsystem
        .window("xD", 711, 400)
        .position_centered()
        .opengl()
        // .allow_highdpi()
        .build()
        .map_err(|e| e.to_string())?;

    /*let ctx =*/
    window.gl_create_context().unwrap();

    gl::load_with(|name| video_subsystem.gl_get_proc_address(name) as *const _);
    init_drawing();

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

    let mut player_texture = Texture::new(&mut content, "textures/pajaW128.png");
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

        //// render player
        Shader::reset();

        unsafe {
            gl::Enable(gl::TEXTURE_2D);
        }

        player_texture.select(&mut content);

        unsafe {
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::BLEND);
        }

        renderer.rect(&player_rect);

        window.gl_swap_window();

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
