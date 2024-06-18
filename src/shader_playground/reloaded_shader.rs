use std::{ops::Deref, path::Path};

use crate::shader::ShaderProgram;

const VERTEX_SHADER: &'static str = include_str!("../res/basic_shaders/vert.glsl");
const FRAG_SHADER_SIMPLE: &'static str =
    include_str!("../res/basic_shaders/frag_simple_2d_gradient.glsl");

#[derive(Default)]
pub enum ReloadedShader {
    #[default]
    NotProvided,
    Shader(ShaderProgram),
    FileReadingError(std::io::Error),
    ShaderError(crate::shader::Error),
}

impl ReloadedShader {
    pub fn as_shader(&self) -> Option<&ShaderProgram> {
        if let Self::Shader(program) = &self {
            Some(program)
        } else {
            None
        }
    }

    pub fn example_shader() -> Self {
        Self::from_str(FRAG_SHADER_SIMPLE)
    }

    pub fn from_str(fragment_shader: &str) -> Self {
        match ShaderProgram::new(VERTEX_SHADER, fragment_shader) {
            Err(err) => Self::ShaderError(err),
            Ok(shader) => Self::Shader(shader),
        }
    }

    pub fn from_file(fragment_shader_path: &Path) -> Self {
        match std::fs::read_to_string(fragment_shader_path) {
            Err(err) => return Self::FileReadingError(err),
            Ok(x) => Self::from_str(&x),
        }
    }
}
