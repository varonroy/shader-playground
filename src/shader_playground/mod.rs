pub mod file_watcher;
pub mod reloaded_shader;

use std::path::{Path, PathBuf};

use anyhow::Context;
use log::{debug, error, info};
use winit::event::{ElementState, MouseButton};

use crate::{game_window::app::App, gl, plane_buffer::PlaneBuffer, shader::ShaderProgram};

use self::{file_watcher::FileWatcher, reloaded_shader::ReloadedShader};

#[derive(Debug, Clone, Copy)]
struct Uniforms {
    window_resolution: i32,
    mouse_position: i32,
    time: i32,
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            window_resolution: -1,
            mouse_position: -1,
            time: -1,
        }
    }
}

impl Uniforms {
    fn init(shader: &ShaderProgram) -> Self {
        Self {
            window_resolution: shader.uniform_location("uResolution"),
            mouse_position: shader.uniform_location("uMouse"),
            time: shader.uniform_location("uTime"),
        }
    }

    fn bind(self, window_resolution: (f32, f32), mouse_position: (f32, f32), time: f32) {
        unsafe {
            gl::Uniform2f(
                self.window_resolution,
                window_resolution.0,
                window_resolution.1,
            );

            gl::Uniform2f(
                self.mouse_position,
                mouse_position.0,
                window_resolution.1 - mouse_position.1,
            );

            gl::Uniform1f(self.time, time);
        }
    }
}

#[derive(Debug)]
pub struct ShaderPlaygroundArgs {
    pub file: Option<PathBuf>,
    pub debouncer_ms: u32,
}

pub struct ShaderPlayground {
    quit: bool,
    shader: ReloadedShader,
    plane: PlaneBuffer,
    uniforms: Uniforms,

    watcher: FileWatcher,

    time_root: std::time::Instant,

    window_resolution: (f32, f32),
    mouse_position: (f32, f32),
    time: f32,
}

impl ShaderPlayground {
    pub fn new(args: ShaderPlaygroundArgs) -> anyhow::Result<Self> {
        let plane = PlaneBuffer::new().with_context(|| "creating plane buffer")?;

        let watcher =
            FileWatcher::new(args.debouncer_ms).with_context(|| "creating a file watcher")?;

        let mut this = Self {
            quit: false,
            shader: Default::default(),
            plane,
            uniforms: Default::default(),

            watcher,

            time_root: std::time::Instant::now(),

            window_resolution: (0.0, 0.0),
            mouse_position: (0.0, 0.0),
            time: 0.0,
        };

        if let Some(path) = &args.file {
            this.watch_file(&path);
            this.load_shader(&path);
        } else {
            info!("No file has been provided. Please re-run the program with a file, or drag and drop one onto the window.")
        }

        Ok(this)
    }

    fn watch_file(&mut self, path: &Path) {
        let _ = self.watcher.unwatch_all();

        match self.watcher.watch(path) {
            Ok(_) => {
                debug!("successfully watching `{}`", path.display())
            }
            Err(err) => {
                error!("could not watch file `{}`. Error: {}", path.display(), err);
            }
        }
    }

    fn load_shader(&mut self, path: &Path) {
        self.time_root = std::time::Instant::now();

        self.shader = ReloadedShader::from_file(path);
        match &self.shader {
            ReloadedShader::NotProvided => error!("unexpected state: `NotProvided`."),
            ReloadedShader::Shader(_) => info!("shader successfully loadded"),
            ReloadedShader::FileReadingError(err) => error!("could not load file. Error: {}", err),
            ReloadedShader::ShaderError(err) => error!("shader compilation error: {}", err),
        };

        self.uniforms = self
            .shader
            .as_shader()
            .map(|shader| Uniforms::init(shader))
            .unwrap_or_default();
    }
}

impl App for ShaderPlayground {
    fn quit(&self) -> bool {
        self.quit
    }

    fn on_resize(&mut self, width: u32, height: u32) {
        debug!("on_resize event: ({}, {})", width, height);
        self.window_resolution = (width as _, height as _);
        unsafe {
            crate::gl::Viewport(0, 0, width as i32, height as i32);
        }
    }

    fn handle_mouse_input(&mut self, _button: MouseButton, _state: ElementState) {}

    fn handle_mouse_motion_input(&mut self, mouse_position: (f32, f32)) {
        self.mouse_position = mouse_position;
    }

    fn handle_file_drop_input(&mut self, path: PathBuf) {
        info!("loading and watching `{}`", path.display());
        self.load_shader(&path);
        self.watch_file(&path);
    }

    fn handle_key_input(&mut self, event: winit::event::KeyEvent) {
        match event.physical_key {
            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) => {
                debug!("escape pressed - quitting");
                self.quit = true
            }
            _ => {}
        }
    }

    fn draw(&mut self) {
        if let Some(path) = self.watcher.file_changed() {
            self.load_shader(&path);
        }

        self.time = self.time_root.elapsed().as_secs_f32();

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);

            match self.shader.as_shader() {
                Some(shader) => {
                    shader.use_program();
                    self.uniforms
                        .bind(self.window_resolution, self.mouse_position, self.time);

                    gl::BindVertexArray(self.plane.vao);
                    gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);
                }
                _ => {}
            }
        }
    }
}
