extern crate colored;
extern crate notify;
extern crate png;

use self::colored::Colorize;
use self::notify::{RecommendedWatcher, RecursiveMode, Watcher};
use gl;
use gl::types::*;
use rect::Rect;
use std::collections::HashMap;
use std::fs::File;
use std::mem::transmute;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

pub fn init_drawing() {
    unsafe {
        gl::Enable(gl::TEXTURE_2D);
    }
}

pub struct Content {
    base_path: PathBuf,
    resource_versions: HashMap<PathBuf, u64>,
    #[allow(dead_code)]
    watcher: Option<RecommendedWatcher>,
    receiver: Receiver<notify::DebouncedEvent>,
}

impl Content {
    pub fn new() -> Content {
        // #[cfg(debug_assertions)]
        // let s = std::fs::read_to_string("./contentpath").unwrap();
        // #[cfg(not(debug_assertions))]
        let s = String::from("./content");

        println!("Content path: {}", s);

        let mut buf = PathBuf::new();
        buf.push(s);

        let (sender, receiver) = channel();

        let mut w = RecommendedWatcher::new(sender, Duration::from_millis(200)).ok();
        if w.is_some() {
            w.as_mut()
                .unwrap()
                .watch("./content", RecursiveMode::Recursive)
                .expect("xD");
        }

        Content {
            base_path: buf,
            resource_versions: HashMap::default(),
            watcher: w,
            receiver: receiver,
        }
    }

    pub fn update(&mut self) {
        // for (k, v) in &self.shader_versions {
        //     println!("{:?} , {}", k, v)
        // }

        for event in self.receiver.try_iter() {
            // println!("changed: {:?}", event.kind);
            match event {
                notify::DebouncedEvent::Write(path) => {
                    let new_path = path.canonicalize().unwrap();
                    match self.resource_versions.get_mut(&new_path) {
                        Some(val) => *val = *val + 1,
                        _ => println!("Didn't find: {:?}", &new_path),
                    }
                }
                _ => (),
            }
        }
    }

    fn should_update_resource(&mut self, path: &PathBuf, current_version: &mut u64) -> bool {
        match self.resource_versions.get(path) {
            Some(new_version) => {
                if *new_version != *current_version {
                    *current_version = *new_version;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

#[derive(Default)]
pub struct Shader {
    frag_path: PathBuf,
    vert_path: PathBuf,
    native_frag: Option<u32>,
    native_vert: Option<u32>,
    native_program: Option<u32>,
    frag_version: u64, // for auto-reload
    vert_version: u64, // for auto-reload
}

fn register_file(content: &mut Content, path: &str) -> PathBuf {
    let mut buf = PathBuf::new();
    buf.push(&content.base_path);
    buf.push(path);
    buf = buf.canonicalize().unwrap();

    &content.resource_versions.insert(buf.clone(), 0);

    buf
}

impl Shader {
    /// Creates a fragment shader.
    pub fn frag(content: &mut Content, path: &str) -> Shader {
        // println!("Registering: {:?}", path);

        Shader {
            frag_path: register_file(content, path),
            vert_path: register_file(content, "shaders/2d.vert"),
            ..Default::default()
        }
    }

    pub fn reset() {
        unsafe {
            gl::UseProgram(0);
        }
    }

    unsafe fn load_file(path: &PathBuf, type_: GLenum) -> Result<u32, String> {
        let bytes = std::fs::read(path)
            .map_err(|err| format!("{} when loading {}", err, path.to_str().unwrap()))?;

        let shader = gl::CreateShader(type_);

        let len = bytes.len() as GLint;
        gl::ShaderSource(shader, 1, transmute(&bytes.as_ptr()), transmute(&len));
        gl::CompileShader(shader);

        Ok(shader)
    }

    unsafe fn check_shader_status(program: u32) -> Result<(), String> {
        let buf_len: GLint = 0;
        gl::GetShaderiv(program, gl::INFO_LOG_LENGTH, transmute(&buf_len));

        if buf_len >= 1 {
            let mut buf: Vec<u8> = vec![0; buf_len as usize];
            gl::GetShaderInfoLog(
                program,
                buf_len,
                transmute(0u64),
                transmute(buf.as_mut_ptr()),
            );

            match String::from_utf8(buf) {
                Err(err) => Err(format!("Error parsing utf8: {}", err)),
                Ok(val) => Err(format!("Error loading shader: {}", val)),
            }?
        }
        Ok(())
    }

    unsafe fn check_program_status(program: u32) -> Result<(), String> {
        // Check & get native error
        let mut linked: GLint = 1;
        gl::GetProgramiv(program, gl::LINK_STATUS, transmute(&mut linked));
        if linked == 0 {
            Err("Error linking shader program")?
        }

        Ok(())
    }

    pub fn load(&mut self) -> Result<(), String> {
        // println!("Loading: {:?}", &self.frag_path);

        // Delete old shader.
        self.unload();

        // Load new shader.
        unsafe {
            let vert = Shader::load_file(&self.vert_path, gl::VERTEX_SHADER)?;
            let frag = Shader::load_file(&self.frag_path, gl::FRAGMENT_SHADER)?;

            let program = gl::CreateProgram();
            // gl::AttachShader(program, vert);
            gl::AttachShader(program, frag);
            gl::LinkProgram(program);
            Shader::check_shader_status(frag)?;
            Shader::check_shader_status(vert)?;
            Shader::check_program_status(program)?;

            self.native_program = Some(program);
            self.native_vert = Some(vert);
            self.native_frag = Some(frag);
        }

        Ok(())
    }

    pub fn try_load(&mut self) {
        match self.load() {
            Err(err) => println!(
                "Error loading shader: {} ({})",
                self.frag_path.to_str().unwrap(),
                err.red()
            ),
            _ => (),
        }
    }

    pub fn select(&mut self, content: &mut Content) {
        if content.should_update_resource(&self.frag_path, &mut self.frag_version) {
            self.unload();
            self.try_load();
        }

        if content.should_update_resource(&self.vert_path, &mut self.vert_version) {
            self.unload();
            self.try_load();
        }

        match self.native_program {
            Some(native_program) => unsafe {
                gl::UseProgram(native_program as GLuint);
            },
            _ => (),
        }
    }

    fn unload(&mut self) {
        match self.native_program {
            Some(program) => unsafe {
                gl::DeleteProgram(program);
            },
            _ => (),
        }
        match self.native_frag {
            Some(shader) => unsafe {
                gl::DeleteShader(shader);
            },
            _ => (),
        }
        match self.native_vert {
            Some(shader) => unsafe {
                gl::DeleteShader(shader);
            },
            _ => (),
        }

        self.native_program = None;
        self.native_vert = None;
        self.native_frag = None;
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        self.unload();
    }
}

#[derive(Default)]
pub struct Texture {
    path: PathBuf,
    native: Option<u32>,
    current_version: u64, // for auto-reload
}

impl Texture {
    /// Creates a fragment shader.
    pub fn new(content: &mut Content, path: &str) -> Texture {
        let mut buf = PathBuf::new();
        buf.push(&content.base_path);
        buf.push(path);
        buf = buf.canonicalize().unwrap();

        &content.resource_versions.insert(buf.clone(), 0);

        println!("Registering: {:?}", &buf);

        Texture {
            path: buf,
            ..Default::default()
        }
    }

    pub fn reset() {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn load(&mut self) -> Result<(), String> {
        println!("Loading: {:?}", &self.path);

        let decoder = png::Decoder::new(File::open(&self.path).unwrap());
        // .map_err(|err| format!("{} when loading {}", err, self.path.to_sThe glGenTextures function is only available in OpenGL version 1.1 or later.tr().unwrap()))?;

        let (info, mut reader) = decoder.read_info().unwrap();
        let mut buf = vec![0; info.buffer_size()];
        reader.next_frame(&mut buf).unwrap();

        // Delete old texture.
        self.delete_texture();

        println!("{} {}", info.width, info.height);

        // Load new shader.
        unsafe {
            let tex: GLuint = 0;
            gl::GenTextures(1, transmute(&tex));
            gl::BindTexture(gl::TEXTURE_2D, tex);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, 0x2601);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, 0x2601);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                info.width as i32,
                info.height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                transmute(buf.as_mut_ptr()),
            );

            self.native = Some(tex);
        }

        Ok(())
    }

    pub fn select(&mut self, content: &mut Content) {
        // println!("Selecting: {:?}", &self.path);
        match content.resource_versions.get(&self.path) {
            Some(new_version) => {
                if *new_version != self.current_version {
                    self.load().unwrap();
                    self.current_version = *new_version;
                }
            }
            _ => (),
        }

        match self.native {
            Some(native) => unsafe {
                gl::BindTexture(gl::TEXTURE_2D, native);
            },
            _ => (),
        }
    }

    fn delete_texture(&mut self) {
        match self.native {
            Some(program) => unsafe {
                gl::DeleteTextures(1, &program);
            },
            _ => (),
        }
        self.native = None;
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        self.delete_texture();
    }
}

pub struct Renderer {}

impl Renderer {
    pub fn new() -> Renderer {
        unsafe {
            gl::PushMatrix();
            gl::LoadIdentity();
            gl::Scaled(2.0 / 16.0, -2.0 / 9.0, 1.0);
            gl::Translated(-8.0, -4.5, 1.0);
        }

        Renderer {}
    }

    pub fn clear(&self, r: f32, g: f32, b: f32) {
        unsafe {
            gl::ClearColor(r, g, b, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    pub fn rgb(&self, r: f32, g: f32, b: f32) {
        unsafe {
            gl::Color3f(r, g, b);
        }
    }

    pub fn rect(&self, rect: &Rect) {
        unsafe {
            gl::Begin(gl::QUADS);
            gl::TexCoord2d(0.0, 0.0);
            gl::Vertex2d(rect.x, rect.y);
            gl::TexCoord2d(1.0, 0.0);
            gl::Vertex2d(rect.x + rect.width, rect.y);
            gl::TexCoord2d(1.0, 1.0);
            gl::Vertex2d(rect.x + rect.width, rect.y + rect.height);
            gl::TexCoord2d(0.0, 1.0);
            gl::Vertex2d(rect.x, rect.y + rect.height);
            gl::End();
        }
    }

    pub fn rect2(&self, x: f64, y: f64, w: f64, h: f64) {
        unsafe {
            gl::Begin(gl::POLYGON);
            gl::Vertex2d(x, y);
            gl::Vertex2d(x + w, y);
            gl::Vertex2d(x + w, y + h);
            gl::Vertex2d(x, y + h);
            gl::End();
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::PopMatrix();
        }
    }
}
