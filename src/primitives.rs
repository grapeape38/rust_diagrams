use gl::types::{GLuint, GLint, GLchar, GLfloat, GLenum, GLvoid, GLsizeiptr};
extern crate gl;
extern crate nalgebra_glm;
use nalgebra_glm as glm;
use std::collections::HashMap;
use std::ffi::CString;
use std::f32::{self, consts::PI};
use PrimType::*;

type PrimMap = HashMap<PrimType, GLuint>;

pub fn new_prim_map() -> PrimMap {
    let mut m = HashMap::new();
    for prim in &[Triangle, Circle, Rect, Line] {
        m.insert(*prim, prim.buffer_data());
    }
    m
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum PrimType {
    Triangle,
    Circle,
    Rect,
    Line,
}

const NCIRCLE_VERTS: usize = 30;

impl PrimType {
    fn verts(&self) -> Vec<f32> {
        match self {
            Triangle => { //isosceles
                vec![
                    -0.5, -0.5, 0.0,
                    0.5, -0.5, 0.0,
                    0.0, 0.5, 0.0
                ]
            },
            Circle => {
                let n = NCIRCLE_VERTS as f32;
                let mut v = vec![0.0, 0.0, 0.0];
                v.extend((0..NCIRCLE_VERTS).map(|i| 
                    vec![f32::cos(2.*PI*i as f32 / (n-1.)), f32::sin(2.*PI*i as f32 / (n-1.)), 0.0]
                ).flatten());
                v
            },
            Rect => {
               vec![ 
                    -0.5, -0.5, 0.0,
                    -0.5, 0.5, 0.0,
                    0.5, 0.5, 0.0,
                    -0.5, -0.5, 0.0,
                    0.5, 0.5, 0.0,
                    0.5, -0.5, 0.0
                ]
            },
            Line => {
               vec![ 
                    0.0, 0.0, 0.0,
                    1.0, 0.0, 0.0]
            }
        }
    }
    fn buffer_data(&self) -> GLuint {
        unsafe { buffer_verts(&self.verts()) }
    }
    fn mode(&self) -> GLenum {
        match self {
            Triangle | Rect => gl::TRIANGLES,
            Circle => gl::TRIANGLE_FAN,
            Line => gl::LINES
        }
    }
    fn size(&self) -> usize {
        match self {
            Triangle => 3,
            Rect => 6,
            Circle => 3 * NCIRCLE_VERTS,
            Line => 2
        }
    }
}

unsafe fn buffer_verts(verts: &Vec<f32>) -> GLuint {
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
    gl::EnableVertexAttribArray(0);
    gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE,
        (3 * std::mem::size_of::<f32>()) as GLint,
        std::ptr::null());
    gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    gl::BindVertexArray(0);
    vao
}

#[derive(Debug)]
pub enum TypeParams {
    Triangle {base: u32},
    Rect {width: u32, height: u32},
    Circle {radius: u32},
    Ellipse {rad_x: u32, rad_y: u32},
    Line {p1: Point, p2: Point}
}

impl TypeParams {
    fn ptype(&self) -> PrimType {
        match self {
            TypeParams::Triangle {..} => Triangle,
            TypeParams::Rect {..} => Rect,
            TypeParams::Circle {..} | TypeParams::Ellipse {..} => Circle,
            TypeParams::Line {..} => Line 
        }
    }
}

pub fn new_line(x1: i32, y1: i32, x2: i32, y2: i32) -> TypeParams {
    TypeParams::Line {
        p1: Point {x: x1, y: y1},
        p2: Point {x: x2, y: y2},
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32
}

impl Point {
    fn neg_y(self) -> Self {
        Point {x: self.x, y: -self.y}
    }
}

impl std::ops::Add for Point {
    type Output = Point;
    fn add(self, other: Point) -> Self::Output {
        Point {x:self.x + other.x, y: self.y + other.y}
    }
}

impl std::ops::Neg for Point {
    type Output = Point;
    fn neg(self) -> Self::Output {
        Point {x: -self.x, y: -self.y}
    }
}

impl std::ops::Sub for Point {
    type Output = Point;
    fn sub(self, other: Point) -> Self::Output {
        self + (-other)
    }
}

pub struct Shape {
    pub params: TypeParams,
    pub offset: Point, //pixels from top corner
    pub rot: f32, //degrees
    pub line_width: f32,
    pub color: (u8, u8, u8), //rgb
}

pub struct ShapeBuilder {
    s: Shape
}

impl ShapeBuilder {
    pub fn new() -> ShapeBuilder {
        ShapeBuilder { s: Shape::new() }
    }
    pub fn params(mut self, params: TypeParams) -> ShapeBuilder {
        self.s.params = params;
        self
    }
    pub fn offset(mut self, x: i32, y: i32) -> ShapeBuilder {
        self.s.offset = Point {x,y};
        self
    }
    pub fn rot(mut self, rot: f32) -> ShapeBuilder {
        self.s.rot = rot;
        self
    }
    pub fn color(mut self, r: u8, g: u8, b: u8) -> ShapeBuilder {
        self.s.color = (r,g,b);
        self
    }
    pub fn line_width(mut self, width: f32) -> ShapeBuilder {
        self.s.line_width = width;
        self
    }
}

impl From<ShapeBuilder> for Shape {
    fn from(sb: ShapeBuilder) -> Self {
        sb.s
    }
}

impl Shape {
    pub fn new() -> Shape {
        Shape {
            params: TypeParams::Triangle{base: 5},
            offset: Point{x:0,y:0},
            rot: 0.,
            line_width: 3.,
            color: (0, 0, 0)
        }
    }
    fn scale(&self, viewport: &Point) -> glm::TVec3<f32> {
        let x_scale;
        let mut y_scale = 1.0;
        match self.params {
            TypeParams::Triangle { base } => {
                x_scale = 2. * base as f32 / viewport.x as f32;
                y_scale = 2. * base as f32 / viewport.y as f32;
            }
            TypeParams::Rect { width, height } => {
                x_scale = 2. * width as f32 / viewport.x as f32;
                y_scale = 2. * height as f32 / viewport.y as f32;
            }
            TypeParams::Circle { radius } => {
                x_scale = 2. * radius as f32 / viewport.x as f32;
                y_scale = 2. * radius as f32 / viewport.y as f32;
            }
            TypeParams::Ellipse { rad_x, rad_y } => {
                x_scale = 2. * rad_x as f32 / viewport.x as f32;
                y_scale = 2. * rad_y as f32 / viewport.y as f32;
            }
            TypeParams::Line { p1, p2 } => {
                let d = p2 - p1;
                //let mag1 = f32::sqrt((d.x * d.x + d.y * d.y) as f32);
                //let tvec = pixels_to_trans_vec(&p1, viewport);
                //let tvec2 = pixels_to_trans_vec(&p2, viewport);
                //let d2 = tvec2 - tvec;
                //let mag = f32::sqrt((d.x * d.x + d.y * d.y) as f32); 
                x_scale = 2. * d.x as f32 / viewport.x as f32;
                y_scale = 2. * d.y as f32 / viewport.y as f32; 
                //y_scale = 2. * mag as f32 / viewport.y as f32;
                //println!("scale: {:?}", x_scale);
            }
        }
        glm::vec3(x_scale, y_scale, 1.0)
    }
    fn trans(&self, viewport: &Point) -> glm::TMat4<f32> {
        let mut trans: glm::TMat4<f32> = glm::identity();
        match self.params {
            TypeParams::Line {p1, p2} => {
                //trans = glm::scale(&trans, &self.scale(viewport));
                let tvec = pixels_to_trans_vec(&p1, viewport);
                let tvec2 = pixels_to_trans_vec(&p2, viewport);
                trans = glm::translate(&trans, &tvec);
                let d = tvec2 - tvec; 
                let rad = f32::atan(d.y / d.x);
                trans = glm::rotate(&trans, rad, &glm::vec3(0.0, 0.0, 1.0));
                println!("{:?}", trans * glm::vec4(1.0, 0.0, 0.0, 1.0));
                trans
            }
            _ => {
                trans = glm::translate(&trans, &pixels_to_trans_vec(&self.offset, viewport));
                trans = glm::rotate(&trans, 180. * self.rot / PI, &glm::vec3(0.0, 0.0, 1.0));
                glm::scale(&trans, &self.scale(viewport))
            }
        }
    }
    fn mode(&self) -> GLenum { self.params.ptype().mode() }
    fn size(&self) -> GLint { self.params.ptype().size() as GLint }
}

pub struct ShapeCtx {
    prim_map: PrimMap,
    viewport: Point,
    program: GLuint 
}

impl ShapeCtx {
    pub fn new(program: GLuint, viewport: Point) -> ShapeCtx {
        ShapeCtx { prim_map: new_prim_map(), viewport, program}
    }
    pub fn draw_shape(&self, s: Shape) {
        let trans = s.trans(&self.viewport);
        //let test_point = glm::vec4(-0.5, 0.0, 0.0, 1.0);
        //println!("{:?}, translated point: {:?}", s.params, trans * test_point);
        let color = rgb_to_f32(&s.color);
        unsafe {
            gl::BindVertexArray(self.prim_map[&s.params.ptype()]);
            let trans_loc = gl::GetUniformLocation(self.program, GChar::new("transform").ptr());
            gl::UniformMatrix4fv(trans_loc, 1, gl::FALSE, trans.as_ptr());
            let color_loc = gl::GetUniformLocation(self.program, GChar::new("color").ptr());
            gl::Uniform4f(color_loc, color.0, color.1, color.2, 1.0);
            gl::LineWidth(s.line_width as GLfloat);
            gl::DrawArrays(s.mode(), 0, s.size());
        }
    }
}

fn rgb_to_f32(rgb: &(u8, u8, u8)) -> (f32, f32, f32) {
    (rgb.0 as f32 / 255., rgb.1 as f32 / 255., rgb.2 as f32 / 255.)
}

fn pixels_to_trans_vec(pixels: &Point, vp: &Point) -> glm::TVec3<f32> {
    let coords = pixels_to_coords(pixels, vp);
    glm::vec3(coords.0, coords.1, 0.0)
}

fn pixels_to_coords(pixels: &Point, vp: &Point) -> (f32, f32) {
    (-1. + 2. * pixels.x as f32 / vp.x as f32, 1. - 2. * pixels.y as f32 / vp.y as f32)
}

struct GChar {
    cstr: CString
}

impl GChar {
    fn new(s: &str) -> GChar {
        GChar { cstr:  CString::new(s.as_bytes()).unwrap() }
    }
    fn ptr(&self) -> *const GLchar {
        self.cstr.as_ptr() as *const GLchar
    }
}

