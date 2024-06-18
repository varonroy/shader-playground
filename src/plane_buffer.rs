use crate::gl;
use crate::gl::types::*;

const VERTICES: [GLfloat; 2 * 4] = [
    // bottom left
    -1.0, -1.0, //
    // bottom right
    1.0, -1.0, //
    // top right
    1.0, 1.0, //
    // top left
    -1.0, 1.0,
];

pub struct PlaneBuffer {
    pub vao: u32,
    pub vbo: u32,
}

impl PlaneBuffer {
    pub fn new() -> anyhow::Result<Self> {
        unsafe {
            // create vao
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);

            // create vbo
            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);

            // bind vao
            gl::BindVertexArray(vao);

            // bind vbo
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            // data
            gl::BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of::<[f32; 8]>() as GLsizeiptr,
                &VERTICES[0] as *const f32 as *const std::ffi::c_void,
                gl::STATIC_DRAW,
            );

            // set attributes
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                2 * std::mem::size_of::<GLfloat>() as GLsizei,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(0);

            // cleanup
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);

            Ok(Self { vao, vbo })
        }
    }
}

impl Drop for PlaneBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &[self.vao][0]);
            gl::DeleteBuffers(1, &[self.vbo][0]);
        }
    }
}
