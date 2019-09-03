// extern crate gl;
extern crate sdl2;
mod gl;

use gl::types::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
// use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
// use sdl2::video::GLProfile;
use std::mem::transmute;

fn find_sdl_gl_driver() -> Option<u32> {
    for (index, item) in sdl2::render::drivers().enumerate() {
        if item.name == "opengl" {
            return Some(index as u32);
        }
    }
    None
}

fn main() -> Result<(), String> {
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
        .window("xD", 600, 400)
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

    let mut x = 1.0;
    let y = 0.0;

    let mut is_left_down = false;
    let mut is_right_down = false;

    let blocks: Vec<Vec<u8>> = vec![
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 1, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 1],
        vec![1, 1, 1, 1, 1, 1, 1],
    ];

    let program: GLuint;

    unsafe {
        let shader = gl::CreateShader(gl::FRAGMENT_SHADER);

        let shader_src = "
void main()
{
	gl_FragColor = vec4(0.4,0.4,0.4,1.0);
}
            ";

        let len = shader_src.len() as GLint;

        //as *const *const u8 as *const *const i8
        //as *const GLint
        gl::ShaderSource(shader, 1, transmute(&shader_src.as_ptr()), transmute(&len));
        gl::CompileShader(shader);

        program = gl::CreateProgram();
        gl::AttachShader(program, shader);
        gl::LinkProgram(program);

        let linked: GLint = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, transmute(&linked));
        // if linked != 0 {
        //     Err("Error linking shader.")?;
        // }
    }

    let mut event_pump = sdl_context.event_pump()?;
    'main_loop: loop {
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

        if is_left_down {
            x -= 0.05;
        }
        if is_right_down {
            x += 0.05;
        }

        unsafe {
            gl::ClearColor(0.5, 0.5, 0.5, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        unsafe {
            gl::PushMatrix();
            gl::Scaled(40.0, 40.0, 40.0);
        }

        unsafe {
            gl::Color3d(0.05, 0.05, 0.05);
            gl::UseProgram(program);
        }

        for i_line in 0..blocks.len() {
            let line = &blocks[i_line];
            for i_col in 0..line.len() {
                let block = line[i_col];
                if block == 1u8 {
                    unsafe {
                        gl::Begin(gl::POLYGON);
                        gl::Vertex2d(i_col as f64 * 1.0, i_line as f64 * 1.0);
                        gl::Vertex2d(i_col as f64 * 1.0 + 1.0, i_line as f64 * 1.0);
                        gl::Vertex2d(i_col as f64 * 1.0 + 1.0, i_line as f64 * 1.0 + 1.0);
                        gl::Vertex2d(i_col as f64 * 1.0, i_line as f64 * 1.0 + 1.0);
                        gl::End();
                    }
                    // gl::Matrix::P(1.0, 0.5, 0.5);

                    // canvas.draw_rect(Rect::new(
                    //     (i_col * 1000) as i32,
                    //     (i_line * 1000) as i32,
                    //     1000,
                    //     1000,
                    // ))?;

                    // canvas.copy(
                    //     &black_texture,
                    //     None,
                    //     Some(Rect::new(
                    //         (i_col * 1000) as i32,
                    //         (i_line * 1000) as i32,
                    //         1000,
                    //         1000,
                    //     )),
                    // )?;
                }
            }
        }

        unsafe {
            gl::Color3d(0.2, 0.8, 0.2);
            gl::UseProgram(0);
        }
        unsafe {
            gl::Begin(gl::POLYGON);
            gl::Vertex2d(x as f64 * 1.0, y as f64 * 1.0);
            gl::Vertex2d(x as f64 * 1.0 + 1.0, y as f64 * 1.0);
            gl::Vertex2d(x as f64 * 1.0 + 1.0, y as f64 * 1.0 + 1.0);
            gl::Vertex2d(x as f64 * 1.0, y as f64 * 1.0 + 1.0);
            gl::End();
        }
        // canvas.copy(
        //     &texture,
        //     None,
        //     Some(Rect::new(
        //         (x * 1000.0) as i32,
        //         (y * 1000.0) as i32,
        //         1000,
        //         1000,
        //     )),
        // )?;

        unsafe {
            gl::PopMatrix();
        }

        canvas.present();

        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    Ok(())
}
