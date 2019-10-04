extern crate freetype;
extern crate gl;
extern crate nalgebra_glm;
extern crate sem_graph_derive;

use std::collections::HashMap;
use std::error::Error;
use nalgebra_glm as glm;
use freetype::library::Library;
use std::ffi::CString;
use gl::types::*;

use crate::render_gl::{Program, Shader, SendUniform, SendUniforms};
use crate::primitives::{Point, DrawCtx};
use sem_graph_derive::SendUniforms;


fn buffer_char_data() -> (GLuint, GLuint) {
    let mut vao: GLuint = 0;
    let mut vbo: GLuint = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, 
            (std::mem::size_of::<f32>() * 4 * 4) as GLsizeiptr,
            std::ptr::null(), gl::DYNAMIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE,
            (4 * std::mem::size_of::<f32>()) as GLint, std::ptr::null());
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }
    (vao, vbo)
}

fn get_char_program() -> Result<Program, String> {
    let vert_shader = Shader::from_vert_source(
        &CString::new(include_str!("shaders/text.vert")).unwrap()
    ).map_err(|e| format!("Error loading vertex shader: {:?}", e))?;
    let frag_shader = Shader::from_frag_source(
        &CString::new(include_str!("shaders/text.frag")).unwrap()
    ).map_err(|e| format!("Error loading text frag shader: {:?}", e))?;
    Program::from_shaders(&[vert_shader, frag_shader])
}

struct Character {
    texture: GLuint,
    size: glm::TVec2<i32>,
    bearing: glm::TVec2<i32>,    // Offset from baseline to left/top of glyph
    advance: GLuint    // Offset to advance to next glyph
}

pub struct RenderText {
    char_map: HashMap<GLchar, Character>,
    vao: GLuint,
    vbo: GLuint,
    prog: Program
}

#[derive(SendUniforms)]
struct TextUniforms {
    text_color: glm::Vec3,
    projection: glm::Mat4
}

impl TextUniforms {
    fn new(text_color: glm::Vec3, vp: &Point) -> Self {
        let projection = glm::ortho(0., vp.x, vp.y, 0., -1., 1.);
        TextUniforms {text_color, projection}
    }
}

impl RenderText {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        unsafe { gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1); }
        let lib = Library::init()?;
        let face = lib.new_face("src/fonts/arial.ttf", 0).map_err(|e| format!("Could not load font face: {:?}", e))?;
        let mut char_map = HashMap::new();
        face.set_char_size(12 * 64, 0, 50, 0).unwrap();
        for c in 0..=127 {
            face.load_char(c as usize, freetype::face::LoadFlag::RENDER).map_err(|e| format!("Could not load char {:?} {:?}", c, e))?;
            face.set_pixel_sizes(0, 48)?;
            let glyph = face.glyph();
            let bitmap = glyph.bitmap();
            let mut texture: GLuint = 0;
            unsafe {
                gl::GenTextures(1, &mut texture) ;
                gl::BindTexture(gl::TEXTURE_2D, texture);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RED as GLint,
                    bitmap.width(),
                    bitmap.rows(),
                    0,
                    gl::RED,
                    gl::UNSIGNED_BYTE,
                    bitmap.buffer().as_ptr() as *const GLvoid
                );
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
                let size = glm::vec2(bitmap.width(), bitmap.rows());
                let bearing = glm::vec2(glyph.bitmap_left(), glyph.bitmap_top());
                let advance = glyph.advance().x as GLuint;
                char_map.insert(c, 
                    Character { texture, size, bearing, advance });
            }
        }
        let (vao, vbo) = buffer_char_data();
        let prog = get_char_program()?;
        Ok(RenderText { char_map, vao, vbo, prog })
    }
    pub fn draw(&self, text: &str, pt: &Point, scale: f32, color: glm::Vec3, draw_ctx: &DrawCtx) {
        self.prog.set_used();
        let trans = TextUniforms::new(color, &draw_ctx.viewport);
        trans.send_uniforms(self.prog.id()).unwrap();
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindVertexArray(self.vao);
        }
        let mut char_pt = *pt;
        for c in text.bytes() {
            let ch = &self.char_map[&(c as GLchar)];
            let offset = Point::new(
                ch.bearing.x as f32 * scale,
                (ch.size.y - ch.bearing.y) as f32 * scale);
            let orig = char_pt + offset;
            let mut size: Point = ch.size.into();
            size *= Point::new(scale, -scale);
            let verts = [
                [orig.x, orig.y + size.y, 0.0, 0.0],
                [orig.x, orig.y, 0.0, 1.0],
                [orig.x + size.x, orig.y, 1.0, 1.0],
                [orig.x + size.x, orig.y + size.y, 1.0, 0.0]
            ];
            unsafe { 
                gl::BindTexture(gl::TEXTURE_2D, ch.texture);
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
                gl::BufferSubData(gl::ARRAY_BUFFER, 0, 
                    (std::mem::size_of::<f32>() * 4 * 4) as GLsizeiptr, verts.as_ptr() as *const GLvoid);
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                gl::DrawArrays(gl::QUADS, 0, 4);
            }
            char_pt.x += (ch.advance >> 6) as f32 * scale;
        }
    }
}
