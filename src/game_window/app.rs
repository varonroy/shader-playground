use std::path::PathBuf;

use winit::event::{ElementState, KeyEvent, MouseButton};

pub trait App {
    fn quit(&self) -> bool;

    fn on_resize(&mut self, width: u32, height: u32);

    fn handle_mouse_input(&mut self, button: MouseButton, state: ElementState);

    fn handle_mouse_motion_input(&mut self, mouse_position: (f32, f32));

    fn handle_file_drop_input(&mut self, path: PathBuf);

    fn handle_key_input(&mut self, event: KeyEvent);

    fn draw(&mut self);
}
