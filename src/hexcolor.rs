extern crate gl;
use crate::primitives::{PrimType, DrawCtx, RotateRect};
use crate::render_gl::SendUniforms;
use gl::types::*;

pub struct HexColor(RotateRect);

impl HexColor {
    pub unsafe fn buffer_verts(verts: &[f32]) -> GLuint {
        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER,
            (verts.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
            verts.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW
        );
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);

        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE,
            (5 * std::mem::size_of::<f32>()) as GLint,
            std::ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE,
            (5 * std::mem::size_of::<f32>()) as GLint,
            (3 * std::mem::size_of::<f32>()) as *const std::ffi::c_void);
        gl::EnableVertexAttribArray(1);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
        vao
    }
    pub fn new(r: RotateRect) -> Self {
        HexColor(r)
    }
    pub fn draw(&self, ctx: &DrawCtx) {
        let ptype = &PrimType::HexColor;
        ctx.prog_map[ptype].set_used();
        let trans = self.0.transform(&ctx.viewport);
        let prog_id = ctx.prog_map[ptype].id();
        let vao = ctx.prim_map[ptype];
        unsafe {
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL); 
            trans.send_uniforms(prog_id).unwrap();
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLE_FAN, 0, ptype.size() as i32);
        }
    }
}