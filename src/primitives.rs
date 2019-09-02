use gl::types::{GLuint, GLint, GLchar, GLfloat, GLenum, GLvoid, GLsizeiptr};
extern crate gl;
extern crate nalgebra_glm;
use nalgebra_glm as glm;
use std::collections::HashMap;
use std::ffi::CString;
use std::f32::{self, consts::PI};
use PrimType::*;
use crate::render_gl::Program;

type PrimMap = HashMap<PrimType, GLuint>;

pub fn prim_map() -> PrimMap {
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
               vec![0.0, 0.0, 0.0]
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
            Line => 1
        }
    }
    fn in_bounds(&self, p: &(f32, f32)) -> bool {
        match self {
            Triangle => {
                p.0 >= -0.5 && p.0 <= 0.5 && p.1 <= (0.5 - f32::abs(p.0))
            }
            Circle => {
                (p.0 * p.0 + p.1 * p.1) <= 1.
            }
            Rect => {
                p.0 >= -0.5 && p.0 <= 0.5 && p.1 >= -0.5 && p.1 <= 0.5 
            }
            Line => {
                false
            }
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
    Ellipse {rad_x: u32, rad_y: u32}
}

impl TypeParams {
    fn ptype(&self) -> PrimType {
        match self {
            TypeParams::Triangle {..} => Triangle,
            TypeParams::Rect {..} => Rect,
            TypeParams::Ellipse {..} => Circle
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: i32,
    pub y: i32
}

impl Point {
    fn mag(&self) -> f32 {
        f32::sqrt((self.x * self.x + self.y * self.y) as f32)
    }
}

impl std::ops::Add for Point {
    type Output = Point;
    fn add(self, other: Point) -> Self::Output {
        Point {x:self.x + other.x, y: self.y + other.y}
    }
}

impl std::ops::AddAssign for Point {
    fn add_assign(&mut self, other: Point) {
        self.x += other.x;
        self.y += other.y;
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

pub struct Line {
    p1: Point,
    p2: Point,
    color: (u8, u8, u8),
    line_width: f32
}

impl Line {
    pub fn new() -> Line {
        Line {
            p1: Point {x: 0, y: 0},
            p2: Point {x: 0, y: 0},
            color: (0, 0, 0),
            line_width: 3.
        }
    }
}

pub struct LineBuilder {
    l: Line
}

impl LineBuilder {
    pub fn new() -> LineBuilder {
        LineBuilder { l: Line::new() }
    }
    pub fn points(mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> LineBuilder {
        self.l.p1 = Point {x: x1, y: y1};
        self.l.p2 = Point {x: x2, y: y2};
        self
    }
    pub fn color(mut self, r: u8, g: u8, b: u8) -> LineBuilder {
        self.l.color = (r,g,b);
        self
    }
    pub fn line_width(mut self, width: f32) -> LineBuilder {
        self.l.line_width = width;
        self
    }
    pub fn get(self) -> Line { self.l }
}

impl From<LineBuilder> for Line {
    fn from(lb: LineBuilder) -> Self {
        lb.l
    }
}

pub struct DrawList<'a> {
    m: HashMap<u32, Box<Drawable + 'a>>,
    next_id: u32
}

impl<'a> DrawList<'a> {
    pub fn new() -> DrawList<'a> {
        DrawList {m: HashMap::new(), next_id: 0}
    }
    pub fn add<D: Drawable + 'a>(&mut self, s: D) {
        self.m.insert(self.next_id, Box::new(s));
        self.next_id+=1;
    }
    pub fn get_mut(&mut self, id: &u32) -> Option<&mut Box<Drawable + 'a>> {
        self.m.get_mut(id)
    }
    pub fn intersect(&self, p: &Point, vp: &Point) -> Option<u32> {
        self.m.iter().find(|(_,v)| v.in_bounds(p, vp)).map(|(k,_)| *k)
    }
    pub fn draw_all(&self, ctx: &DrawCtx) {
        ctx.program.set_used();
        self.m.values().for_each(|s| s.draw(ctx));
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

#[allow(dead_code)]
impl ShapeBuilder {
    pub fn new() -> ShapeBuilder {
        ShapeBuilder { s: Shape::new() }
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

    pub fn circle(mut self, rad: u32) -> ShapeBuilder {
        self.s.params = TypeParams::Ellipse{
            rad_x: rad,
            rad_y: rad
        };
        self
    }

    pub fn ellipse(mut self, rad_x: u32, rad_y: u32) -> ShapeBuilder {
        self.s.params = TypeParams::Ellipse{
            rad_x,
            rad_y
        };
        self
    }

    pub fn rect(mut self, width: u32, height: u32) -> ShapeBuilder {
        self.s.params = TypeParams::Rect {
            width,
            height
        };
        self
    }

    pub fn square(mut self, side: u32) -> ShapeBuilder {
        self.s.params = TypeParams::Rect {
            width: side,
            height: side
        };
        self
    }

    pub fn tri(mut self, base: u32) -> ShapeBuilder {
        self.s.params = TypeParams::Triangle {
            base
        };
        self
    }
    pub fn get(self) -> Shape { self.s }
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
        let y_scale; 
        match self.params {
            TypeParams::Triangle { base } => {
                x_scale = 2. * base as f32 / viewport.x as f32;
                y_scale = 2. * base as f32 / viewport.y as f32;
            }
            TypeParams::Rect { width, height } => {
                x_scale = 2. * width as f32 / viewport.x as f32;
                y_scale = 2. * height as f32 / viewport.y as f32;
            }
            TypeParams::Ellipse { rad_x, rad_y } => {
                x_scale = 2. * rad_x as f32 / viewport.x as f32;
                y_scale = 2. * rad_y as f32 / viewport.y as f32;
            }
        }
        glm::vec3(x_scale, y_scale, 1.0)
    }
    fn trans(&self, viewport: &Point) -> glm::TMat4<f32> {
        let mut trans: glm::TMat4<f32> = glm::identity();
        trans = glm::translate(&trans, &pixels_to_trans_vec(&self.offset, viewport));
        trans = glm::scale(&trans, &self.scale(viewport));
        glm::rotate(&trans, 180. * self.rot / PI, &glm::vec3(0.0, 0.0, 1.0))
    }
    fn mode(&self) -> GLenum { self.params.ptype().mode() }
    fn size(&self) -> GLint { self.params.ptype().size() as GLint }
}

pub trait Drawable {
    fn draw(&self, ctx: &DrawCtx);
    fn drag(&mut self, dir: Point);
    fn in_bounds(&self, p: &Point, vp: &Point) -> bool;
}

impl Drawable for Shape {
    fn draw(&self, ctx: &DrawCtx) {
        let trans = self.trans(&ctx.viewport);
        let color = rgb_to_f32(&self.color);
        unsafe {
            gl::BindVertexArray(ctx.prim_map[&self.params.ptype()]);
            let trans_loc = gl::GetUniformLocation(ctx.program.id(), GChar::new("transform").ptr());
            gl::UniformMatrix4fv(trans_loc, 1, gl::FALSE, trans.as_ptr());
            let color_loc = gl::GetUniformLocation(ctx.program.id(), GChar::new("color").ptr());
            gl::Uniform4f(color_loc, color.0, color.1, color.2, 1.0);
            gl::LineWidth(self.line_width as GLfloat);
            gl::DrawArrays(self.mode(), 0, self.size());
        }
    }
    fn drag(&mut self, dir: Point) {
        self.offset += dir;
    }
    fn in_bounds(&self, p: &Point, vp: &Point) -> bool {
        let trans_inv = glm::inverse(&self.trans(vp));
        let pc = pixels_to_trans_vec(p, vp);
        let normpt = trans_inv * glm::vec4(pc.x, pc.y, pc.z, 1.0);
        self.params.ptype().in_bounds(&(normpt.x, normpt.y))
    }
}

impl Drawable for Line {
    fn draw(&self, ctx: &DrawCtx) {
        let mut trans: glm::TMat4<f32> = glm::identity();
        let p1c = pixels_to_trans_vec(&self.p1, &ctx.viewport);
        let p2c = pixels_to_trans_vec(&self.p2, &ctx.viewport);
        trans = glm::translate(&trans, &p1c);
        let color = rgb_to_f32(&self.color);
        unsafe {
            gl::BindVertexArray(ctx.prim_map[&PrimType::Line]);
            let trans_loc = gl::GetUniformLocation(ctx.program.id(), GChar::new("transform").ptr());
            gl::UniformMatrix4fv(trans_loc, 1, gl::FALSE, trans.as_ptr());
            let color_loc = gl::GetUniformLocation(ctx.program.id(), GChar::new("color").ptr());
            gl::Uniform4f(color_loc, color.0, color.1, color.2, 1.0);
            let point2_loc = gl::GetUniformLocation(ctx.program.id(), GChar::new("point2").ptr());
            gl::Uniform2f(point2_loc, p2c.x, p2c.y);
            gl::LineWidth(self.line_width as GLfloat);
            gl::DrawArrays(gl::POINTS, 0, 1);
        }
    }
    fn drag(&mut self, dir: Point) {
        self.p1 += dir;
    }
    fn in_bounds(&self, p: &Point, _: &Point) -> bool {
        let t = (p.x - self.p1.x) as f32 / (self.p2.x - self.p1.x) as f32;
        let ypt = (1. - t) * self.p1.y as f32 + t * self.p2.y as f32;
        t >= 0. && t <= 1. && p.y as f32 - ypt <= self.line_width
    }
}

pub struct DrawCtx {
    prim_map: PrimMap,
    viewport: Point,
    program: Program 
}

impl DrawCtx {
    pub fn new(program: Program, viewport: Point) -> DrawCtx {
        DrawCtx { prim_map: prim_map(), viewport, program }
    }
    pub fn draw<D: Drawable>(&self, s: &D) {
        self.program.set_used();
        s.draw(&self);
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

#[allow(dead_code)]
fn coords_to_pixels(coords: &(f32, f32), vp: &Point) -> (i32, i32) {
    let pt = (vp.x as f32 * (coords.0 + 1.) / 2., vp.y as f32 * (1. - coords.1) / 2.);
    (pt.0 as i32, pt.1 as i32)
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

