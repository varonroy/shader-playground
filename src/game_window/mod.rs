pub mod app;
mod utils;

use std::{ffi::CString, num::NonZeroU32};

use anyhow::{anyhow, Context};
use glutin::{
    config::{ConfigTemplateBuilder, GlConfig},
    context::{ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext},
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, SwapInterval, WindowSurface},
};
use glutin_winit::{DisplayBuilder, GlWindow};
use log::debug;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::EventLoop,
    raw_window_handle::HasWindowHandle,
    window::{Window, WindowAttributes},
};

use self::app::App;

fn common_window_attributes(title: impl ToString) -> WindowAttributes {
    Window::default_attributes()
        .with_transparent(true)
        .with_title(title.to_string())
}

struct GlState {
    gl_context: PossiblyCurrentContext,
    gl_surface: glutin::surface::Surface<WindowSurface>,
    window: Window,
}

pub struct GameWindow<A, ARG> {
    template: ConfigTemplateBuilder,
    display_builder: DisplayBuilder,
    exit_state: anyhow::Result<()>,
    title: String,
    vsync: bool,
    gl_state: Option<GlState>,

    app: Option<A>,
    app_constructor: fn(ARG) -> anyhow::Result<A>,
    app_arg: Option<ARG>,
}

impl<A: App, ARG> GameWindow<A, ARG> {
    fn create_window_attributes(&self) -> WindowAttributes {
        common_window_attributes(&self.title)
    }

    pub fn new(
        title: impl ToString,
        vsync: bool,
        app_constructor: fn(ARG) -> anyhow::Result<A>,
        app_arg: ARG,
    ) -> Self {
        let title = title.to_string();

        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_transparency(cfg!(cgl_backend));

        let display_builder =
            DisplayBuilder::new().with_window_attributes(Some(common_window_attributes(&title)));

        GameWindow {
            template,
            display_builder,
            exit_state: Ok(()),
            title,
            vsync,
            gl_state: None,
            app: None,
            app_constructor,
            app_arg: Some(app_arg),
        }
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let event_loop = EventLoop::new().with_context(|| "Creating event_loop")?;

        event_loop
            .run_app(&mut self)
            .with_context(|| "`event_loop.run_app`")?;

        self.exit_state
    }

    fn handle_resumed(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
    ) -> anyhow::Result<()> {
        let (window, gl_config) = self
            .display_builder
            .clone()
            .build(event_loop, self.template.clone(), utils::gl_config_picker)
            .map_err(|err| anyhow!(err.to_string()))
            .with_context(|| "`display_builder.build`")?;

        debug!("gl samples: {}", gl_config.num_samples());

        let raw_window_handle = window
            .as_ref()
            .and_then(|window| window.window_handle().ok())
            .map(|handle| handle.as_raw());

        let gl_display = gl_config.display();

        let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);

        let gl_context = unsafe { gl_display.create_context(&gl_config, &context_attributes) }
            .with_context(|| "creating gl context")?;

        let window = match window {
            Some(window) => window,
            None => glutin_winit::finalize_window(
                event_loop,
                self.create_window_attributes(),
                &gl_config,
            )?,
        };

        let attrs = window
            .build_surface_attributes(Default::default())
            .with_context(|| "failed to build surface attributes")?;

        let gl_surface = unsafe { gl_display.create_window_surface(&gl_config, &attrs) }
            .with_context(|| "creating window gl surface")?;

        let gl_context = gl_context
            .make_current(&gl_surface)
            .with_context(|| "making gl context current")?;

        crate::gl::load_with(|symbol| {
            let symbol = CString::new(symbol).unwrap(); // no way to void this unwrap
            gl_display.get_proc_address(symbol.as_c_str()).cast()
        });

        if self.app.is_none() {
            let app = (self.app_constructor)(self.app_arg.take().unwrap())
                .with_context(|| "creating app")?;
            self.app = Some(app);

            debug!("created app");
        }

        let swap_interval = if self.vsync {
            SwapInterval::Wait(NonZeroU32::new(1).unwrap())
        } else {
            SwapInterval::DontWait
        };
        gl_surface
            .set_swap_interval(&gl_context, swap_interval)
            .with_context(|| "setting `vsync`")?;

        self.gl_state = Some(GlState {
            gl_context,
            gl_surface,
            window,
        });

        Ok(())
    }
}

impl<A: App, ARG> ApplicationHandler for GameWindow<A, ARG> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        match self.handle_resumed(event_loop) {
            Ok(_) => {}
            Err(err) => {
                self.exit_state = Err(err);
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(PhysicalSize::<u32> { width, height })
                if width != 0 && height != 0 =>
            {
                if let Some(state) = &self.gl_state {
                    state.gl_surface.resize(
                        &state.gl_context,
                        NonZeroU32::new(width).unwrap(),
                        NonZeroU32::new(height).unwrap(),
                    );
                    if let Some(app) = &mut self.app {
                        app.on_resize(width, height);
                    }
                }
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(app) = &mut self.app {
                    app.handle_key_input(event);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(app) = &mut self.app {
                    app.handle_mouse_input(button, state);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(app) = &mut self.app {
                    app.handle_mouse_motion_input((position.x as _, position.y as _));
                }
            }
            WindowEvent::DroppedFile(file) => {
                if let Some(app) = &mut self.app {
                    app.handle_file_drop_input(file);
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(state) = &self.gl_state {
            if let Some(app) = &mut self.app {
                app.draw();

                if app.quit() {
                    event_loop.exit();
                }
            }

            state.window.request_redraw();

            state.gl_surface.swap_buffers(&state.gl_context).unwrap();
        }
    }
}
