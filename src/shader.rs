use std::ffi::CString;

use crate::gl;
use gl::types::*;

pub struct ShaderProgram(pub u32);

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Could not compile shader.\n{0}")]
    ShaderCompilationError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

fn get_program_info_log(program: u32) -> String {
    let mut info_log = Vec::with_capacity(512);
    unsafe {
        info_log.set_len(512); // subtract 1 to skip the trailing null character
    }

    unsafe {
        gl::GetProgramInfoLog(
            program,
            512,
            std::ptr::null_mut(),
            info_log.as_mut_ptr() as *mut GLchar,
        );
    }

    std::str::from_utf8(&info_log).unwrap().to_string()
}

fn get_shader_info_log(shader: u32) -> String {
    let mut info_log = Vec::with_capacity(512);
    let mut log_len = 0;
    unsafe {
        gl::GetShaderInfoLog(
            shader,
            512,
            &mut log_len,
            info_log.as_mut_ptr() as *mut GLchar,
        );
        info_log.set_len(log_len as _);
    }
    std::str::from_utf8(&info_log).unwrap().to_string()
}

impl ShaderProgram {
    pub fn new(vertex_shader_source: &str, fragment_shader_source: &str) -> Result<Self> {
        unsafe {
            // vertex shader
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let c_str_vert = CString::new(vertex_shader_source.as_bytes()).unwrap();
            gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), std::ptr::null());
            gl::CompileShader(vertex_shader);

            let mut success = gl::FALSE as GLint;
            gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let info_log = get_shader_info_log(vertex_shader);
                return Err(Error::ShaderCompilationError(info_log));
            }

            // fragment shader
            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let c_str_frag = CString::new(fragment_shader_source.as_bytes()).unwrap();
            gl::ShaderSource(fragment_shader, 1, &c_str_frag.as_ptr(), std::ptr::null());
            gl::CompileShader(fragment_shader);

            gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let info_log = get_shader_info_log(fragment_shader);
                return Err(Error::ShaderCompilationError(info_log));
            }

            // link shaders
            let shader_program = gl::CreateProgram();
            gl::AttachShader(shader_program, vertex_shader);
            gl::AttachShader(shader_program, fragment_shader);
            gl::LinkProgram(shader_program);

            gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
            if success != gl::TRUE as GLint {
                let info_log = get_program_info_log(shader_program);
                return Err(Error::ShaderCompilationError(info_log));
            }

            // cleanup
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            Ok(Self(shader_program))
        }
    }

    pub fn uniform_location(&self, name: &str) -> i32 {
        let name = CString::new(name).expect("Failed to create CString");
        let name = name.as_bytes_with_nul();
        let p: *const std::ffi::c_char = name.as_ptr().cast();
        unsafe { gl::GetUniformLocation(self.0, p) }
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.0);
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.0);
        }
    }
}
