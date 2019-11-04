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
use crate::primitives::{Point, rgb_to_f32, DrawCtx, Rect, RotateRect} ;
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
    advance: GLint 
}

#[derive(SendUniforms, PartialEq, Clone)]
pub struct TextUniforms {
    text_color: glm::Vec3,
    model: glm::Mat4,
    projection: glm::Mat4
}

impl TextUniforms {
    pub fn new(text_color: &glm::Vec4, r: &RotateRect, off: &Point, vp: &Point) -> Self {
        let pct = Point::new(off.x / r.size.x, off.y / r.size.y);
        let r2 = Rect::new(pct, Point::new(1.,1.));
        let mut r = r.clone();
        r.resize(&r2, vp);

        let projection = glm::ortho(0., vp.x, vp.y, 0., -1., 1.);
        let mut model = glm::translate(&glm::identity(), &r.offset.to_vec3());

        model = glm::translate(&model, &(r.size / 2.).to_vec3());
        model = glm::rotate(&model, r.rot.0, &glm::vec3(0., 0., 1.));
        model = glm::translate(&model, &(-r.size / 2.).to_vec3());

        let text_color = glm::vec4_to_vec3(text_color);
        TextUniforms {text_color, model, projection}
    }
}

pub struct TextParams<'a> {
    pub text: &'a str,
    pub color: glm::Vec3,
    pub scale: f32,
    pub trans: &'a TextUniforms,
}

#[allow(dead_code)]
impl<'a> TextParams<'a> {
    pub fn new(text: &'a str, trans: &'a TextUniforms) -> Self {
        TextParams {
            text,
            color: glm::vec3(0.,0.,0.),
            scale: 1.0,
            trans
        }
    }
    pub fn color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.color = glm::vec4_to_vec3(&rgb_to_f32(r, g, b));
        self
    }
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
}

pub struct RenderText {
    char_map: HashMap<GLchar, Character>,
    vao: GLuint,
    vbo: GLuint,
    prog: Program,
}

impl RenderText {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        unsafe { gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1); }
        let lib = Library::init()?;
        let face = lib.new_face("fonts/arial.ttf", 0).map_err(|e| format!("Could not load font face: {:?}", e))?;
        let mut char_map = HashMap::new();
        face.set_char_size(12 * 64, 0, 50, 0).unwrap();
        for c in 0..=127 {
            face.load_char(c as usize, freetype::face::LoadFlag::RENDER).map_err(|e| format!("Could not load char {:?} {:?}", c, e))?;
            face.set_pixel_sizes(0, 24)?;
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
                let advance = glyph.advance().x as GLint; 
                char_map.insert(c, 
                    Character { texture, size, bearing, advance });
            }
        }
        unsafe { gl::BindTexture(gl::TEXTURE_2D, 0); }
        let (vao, vbo) = buffer_char_data();
        let prog = get_char_program()?;
        Ok(RenderText { char_map, vao, vbo, prog })
    }
    pub fn draw(&self, params: &TextParams, _: &DrawCtx) {
        self.prog.set_used();
        let (text, scale, trans) = (params.text, params.scale, params.trans);
        trans.send_uniforms(self.prog.id()).unwrap();
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindVertexArray(self.vao);
        }
        let mut char_pt = Point::origin();
        for c in text.bytes() {
            if c == '\n' as u8 {
                char_pt.x = 0.;
                char_pt.y += self.line_height(scale);
                continue;
            }
            let ch = &self.char_map[&(c as GLchar)];
            let offset = Point::new(
                ch.bearing.x as f32 * scale,
                (ch.size.y - ch.bearing.y) as f32 * scale);
            let size = Point::new(scale, scale) * ch.size.into();
            let orig = char_pt + offset;
            let verts = [
                [orig.x, orig.y - size.y, 0.0, 0.0],
                [orig.x, orig.y, 0.0, 1.0],
                [orig.x + size.x, orig.y, 1.0, 1.0],
                [orig.x + size.x, orig.y - size.y, 1.0, 0.0]
            ];
            unsafe { 
                gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
                gl::BindTexture(gl::TEXTURE_2D, ch.texture);
                gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
                gl::BufferSubData(gl::ARRAY_BUFFER, 0, 
                    (std::mem::size_of::<f32>() * 4 * 4) as GLsizeiptr, verts.as_ptr() as *const GLvoid);
                gl::BindBuffer(gl::ARRAY_BUFFER, 0);
                gl::DrawArrays(gl::QUADS, 0, 4);
            }
            char_pt.x += (ch.advance >> 6) as f32 * scale;
        }
        unsafe {
            gl::BindVertexArray(0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }
    pub fn has_char(&self, ch: char) -> bool {
        self.char_map.contains_key(&(ch as GLchar))
    }
    pub fn line_height(&self, scale: f32) -> f32 {
        self.char_map[&('a' as i8)].size.y as f32 * scale * 1.4
    }
    pub fn char_size(&self, ch: char, scale: f32) -> Point {
        self.char_map.get(&(ch as GLchar)).map(|ch| Point::new(scale * ch.size.x as f32, scale * ch.size.y as f32))
            .unwrap_or(Point::origin())
    }
    pub fn char_size_w_advance(&self, ch: char, scale: f32) -> Point {
        self.char_map.get(&(ch as GLchar)).map(|ch| Point::new(scale * (ch.advance >> 6) as f32, scale * ch.size.y as f32))
            .unwrap_or(Point::origin())
    }
    pub fn measure(&self, text: &str, scale: f32) -> Point {
        if text.is_empty() {
            return Point::origin();
        } 
        let lh = self.line_height(scale);
        text.bytes().fold(Point::new(0., lh), |size, c| {
            let ch = &self.char_map[&(c as GLchar)];
            size + match c as char {
                '\n' => Point::new(0., lh),
                _ => Point::new(scale * (ch.advance >> 6) as f32, 0.)
            } 
        }) 
    } 
}

