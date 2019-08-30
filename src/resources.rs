extern crate notify;
extern crate png;

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
}

#[derive(Default)]
pub struct Shader {
    path: PathBuf,
    native_shader: Option<u32>,
    native_program: Option<u32>,
    current_version: u64, // for auto-reload
}

impl Shader {
    /// Creates a fragment shader.
    pub fn frag(content: &mut Content, path: &str) -> Shader {
        let mut buf = PathBuf::new();
        buf.push(&content.base_path);
        buf.push(path);
        buf = buf.canonicalize().unwrap();

        &content.resource_versions.insert(buf.clone(), 0);

        println!("Registering: {:?}", &buf);

        Shader {
            path: buf,
            ..Default::default()
        }
    }

    pub fn reset() {
        unsafe {
            gl::UseProgram(0);
        }
    }

    pub fn load(&mut self) -> Result<(), String> {
        println!("Loading: {:?}", &self.path);

        let bytes = std::fs::read(&self.path)
            .map_err(|err| format!("{} when loading {}", err, self.path.to_str().unwrap()))?;

        // Delete old shader.
        self.delete_shader();

        // Load new shader.
        unsafe {
            let shader = gl::CreateShader(gl::FRAGMENT_SHADER);

            let len = bytes.len() as GLint;

            gl::ShaderSource(shader, 1, transmute(&bytes.as_ptr()), transmute(&len));
            gl::CompileShader(shader);

            let program = gl::CreateProgram();
            gl::AttachShader(program, shader);
            gl::LinkProgram(program);

            let linked: GLint = 0;
            gl::GetProgramiv(program, gl::LINK_STATUS, transmute(&linked));
            if linked == 0 {
                Err("Error linking shader.".to_owned())?
            }
            self.native_program = Some(program);
            self.native_shader = Some(shader);
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

        match self.native_program {
            Some(native_program) => unsafe {
                gl::UseProgram(native_program as GLuint);
            },
            _ => (),
        }
    }

    fn delete_shader(&mut self) {
        match self.native_program {
            Some(program) => unsafe {
                gl::DeleteProgram(program);
            },
            _ => (),
        }
        match self.native_shader {
            Some(shader) => unsafe {
                gl::DeleteShader(shader);
            },
            _ => (),
        }
        self.native_program = None;
        self.native_shader = None;
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        self.delete_shader();
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

        // let bytes = fs::read(&self.path)
        //     .map_err(|err| format!("{} when loading {}", err, self.path.to_str().unwrap()))?;

        let decoder = png::Decoder::new(File::open(&self.path).unwrap());
        // .map_err(|err| format!("{} when loading {}", err, self.path.to_sThe glGenTextures function is only available in OpenGL version 1.1 or later.tr().unwrap()))?;

        let (info, mut reader) = decoder.read_info().unwrap();
        let mut buf = vec![0; info.buffer_size()];
        reader.next_frame(&mut buf).unwrap();

        // Delete old texture.
        self.delete_texture();

        // Load new shader.
        unsafe {
            let mut tex: GLuint = 0;
            gl::GenTextures(1, transmute(&tex));
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                info.width as i32,
                info.height as i32,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                transmute(buf.as_mut_ptr()), // try removing as_mut_ptr
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
            gl::Begin(gl::POLYGON);
            gl::Vertex2d(rect.x, rect.y);
            gl::Vertex2d(rect.x + rect.width, rect.y);
            gl::Vertex2d(rect.x + rect.width, rect.y + rect.height);
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
