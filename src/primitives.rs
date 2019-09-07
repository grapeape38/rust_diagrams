use gl::types::{GLuint, GLint, GLchar, GLfloat, GLenum, GLvoid, GLsizeiptr};
extern crate gl;
extern crate nalgebra_glm;
use nalgebra_glm as glm;
use std::collections::HashMap;
use std::ffi::CString;
use std::f32::{self, consts::PI};
use PrimType as PT;
use crate::render_gl::{Shader, Program};

type PrimMap = HashMap<PrimType, GLuint>;

pub fn prim_map() -> PrimMap {
    let mut m = HashMap::new();
    for prim in &[PT::Triangle, PT::Circle, PT::Rect, PT::Line] {
        m.insert(*prim, prim.buffer_data());
    }
    m
}

type ProgMap<'a> = HashMap<PrimType, &'a Program>;

pub struct PrimPrograms {
    line_prog: Program,
    shape_prog: Program
}

impl PrimPrograms {
    pub fn new() -> PrimPrograms {
        let vert_shader = Shader::from_vert_source(
            &CString::new(include_str!("triangle.vert")).unwrap()
        ).unwrap();

        let frag_shader = Shader::from_frag_source(
            &CString::new(include_str!("triangle.frag")).unwrap()
        ).unwrap();

        let mut shaders = vec![vert_shader, frag_shader];

        let shape_prog = Program::from_shaders(shaders.as_ref()).unwrap();
        let geom_shader = Shader::from_geom_source(
            &CString::new(include_str!("line.geom")).unwrap()
        ).unwrap();

        shaders.insert(1, geom_shader);
        let line_prog = Program::from_shaders(shaders.as_ref()).unwrap();
        PrimPrograms {
            shape_prog,
            line_prog
        }
    }
}

pub fn prog_map(programs: &PrimPrograms) -> ProgMap {
   let mut m = HashMap::new();
   for prim in &[PrimType::Triangle, PrimType::Circle, PT::Rect] {
        m.insert(*prim, &programs.shape_prog);
    }
    m.insert(PT::Line, &programs.line_prog);
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
            PT::Triangle => { //isosceles
                vec![
                    -0.5, -0.5, 0.0,
                    0.5, -0.5, 0.0,
                    0.0, 0.5, 0.0
                ]
            },
            PT::Circle => {
                let n = NCIRCLE_VERTS as f32;
                let mut v = vec![0.0, 0.0, 0.0];
                v.extend((0..NCIRCLE_VERTS).map(|i| 
                    vec![f32::cos(2.*PI*i as f32 / (n-1.)), f32::sin(2.*PI*i as f32 / (n-1.)), 0.0]
                ).flatten());
                v
            },
            PT::Rect => {
               vec![ 
                    -0.5, -0.5, 0.0,
                    -0.5, 0.5, 0.0,
                    0.5, 0.5, 0.0,
                    -0.5, -0.5, 0.0,
                    0.5, 0.5, 0.0,
                    0.5, -0.5, 0.0
                ]
            },
            PT::Line => {
               vec![0.0, 0.0, 0.0]
            }
        }
    }
    fn buffer_data(&self) -> GLuint {
        unsafe { buffer_verts(&self.verts()) }
    }
    fn mode(&self) -> GLenum {
        match self {
            PT::Triangle | PT::Rect => gl::TRIANGLES,
            PT::Circle => gl::TRIANGLE_FAN,
            PT::Line => gl::LINES
        }
    }
    fn size(&self) -> usize {
        match self {
            PT::Triangle => 3,
            PT::Rect => 6,
            PT::Circle => 3 * NCIRCLE_VERTS,
            PT::Line => 1
        }
    }
    fn in_bounds(&self, p: &Point) -> bool {
        match self {
            PT::Triangle => {
                p.x >= -0.5 && p.x <= 0.5 && p.y >= -0.5 && p.y <= (0.5 - f32::abs(p.x))
            }
            PT::Circle => {
                p.mag() <= 1.
            }
            PT::Rect => {
                p.x >= -0.5 && p.x <= 0.5 && p.y >= -0.5 && p.y <= 0.5 
            }
            PT::Line => {
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
            TypeParams::Triangle {..} => PT::Triangle,
            TypeParams::Rect {..} => PT::Rect,
            TypeParams::Ellipse {..} => PT::Circle
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32
}

impl Point {
    fn mag(&self) -> f32 {
        f32::sqrt((self.x*self.x + self.y*self.y) as f32)
    }
    fn dist(&self, p2: &Point) -> f32 {
        let d = *self - *p2;
        d.mag()
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

#[derive(Debug)]
pub struct Line {
    a: f32, b: f32, c: f32 
}

fn det(a: f32, b: f32, c: f32, d: f32) -> f32 {
    a * d - b * c
}

impl Line {
    fn from_pts(p1: Point, p2: Point) -> Line {
        let a = p1.y - p2.y;
        let b = p2.x - p1.x;
        Line {
            a, b, c: -(a * p1.x + b * p1.y)
        }
    }
    fn from_pt_slope(p: &Point, l: Line) -> Line {
        Line { c: -(l.a * p.x + l.b * p.y), ..l }
    }
    fn intersect(&self, l2: &Line) -> Option<Point> {
        let zn = det(self.a, self.b, l2.a, l2.b);
        if f32::abs(zn) < 1e-7 {
            return None;
        }
        Some(Point {
            x: -det(self.c, self.b, l2.c, l2.b) / zn,
            y: -det(self.a, self.c, l2.a, l2.c) / zn
        })
    }
    fn dist_to_pt(&self, p: &Point) -> f32 {
        let opp = Line { a: -self.b, b: self.a, c: 0.0 };
        let l2 = Line::from_pt_slope(p, opp);
        let inter = self.intersect(&l2).unwrap();
        p.dist(&inter)
    }
}

pub struct DrawLine {
    p1: Point,
    p2: Point,
    color: (u8, u8, u8),
    line_width: f32
}

impl DrawLine {
    pub fn new() -> DrawLine {
        DrawLine {
            p1: Point {x: 0., y: 0.},
            p2: Point {x: 0., y: 0.},
            color: (0, 0, 0),
            line_width: 3.
        }
    }
}

pub struct LineBuilder {
    l: DrawLine
}

impl LineBuilder {
    pub fn new() -> LineBuilder {
        LineBuilder { l: DrawLine::new() }
    }
    pub fn points(mut self, x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        self.l.p1 = Point {x: x1, y: y1};
        self.l.p2 = Point {x: x2, y: y2};
        self
    }
    pub fn color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.l.color = (r,g,b);
        self
    }
    pub fn line_width(mut self, width: f32) -> Self {
        self.l.line_width = width;
        self
    }
    pub fn get(self) -> DrawLine { self.l }
}

impl From<LineBuilder> for DrawLine {
    fn from(lb: LineBuilder) -> Self {
        lb.l
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
    pub fn offset(mut self, x: i32, y: i32) -> Self {
        self.s.offset = Point {x: x as f32,y: y as f32};
        self
    }
    pub fn rot(mut self, rot: f32) -> ShapeBuilder {
        self.s.rot = rot;
        self
    }
    pub fn color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.s.color = (r,g,b);
        self
    }
    pub fn line_width(mut self, width: f32) -> Self {
        self.s.line_width = width;
        self
    }
    pub fn circle(mut self, rad: u32) -> Self {
        self.s.params = TypeParams::Ellipse{
            rad_x: rad,
            rad_y: rad
        };
        self
    }
    pub fn ellipse(mut self, rad_x: u32, rad_y: u32) -> Self {
        self.s.params = TypeParams::Ellipse{
            rad_x,
            rad_y
        };
        self
    }
    pub fn rect(mut self, width: u32, height: u32) -> Self {
        self.s.params = TypeParams::Rect {
            width,
            height
        };
        self
    }
    pub fn square(mut self, side: u32) -> Self {
        self.s.params = TypeParams::Rect {
            width: side,
            height: side
        };
        self
    }
    pub fn tri(mut self, base: u32) -> Self {
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
            offset: Point{x:0.,y:0.},
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
        let mut trans = glm::translate(&glm::identity(), &pixels_to_trans_vec(&self.offset, viewport));
        trans = glm::scale(&trans, &self.scale(viewport));
        glm::rotate(&trans, 180. * self.rot / PI, &glm::vec3(0.0, 0.0, 1.0))
    }
    fn mode(&self) -> GLenum { self.params.ptype().mode() }
    fn size(&self) -> GLint { self.params.ptype().size() as GLint }
}

pub trait InBounds {
    fn in_bounds(&self, p: &Point, vp: &Point) -> bool;
}

pub trait Draggable {
    fn drag(&mut self, to: Point);
}

pub trait Drawable {
    fn draw(&self, ctx: &DrawCtx);
    fn prim_type(&self) -> PrimType;
}

pub trait DrawBounds: InBounds + Drawable + Draggable { }

impl Drawable for Shape {
    fn draw(&self, ctx: &DrawCtx) {
        let trans = self.trans(&ctx.viewport);
        let color = rgb_to_f32(&self.color);
        let prog_id = ctx.prog_map[&self.prim_type()].id();
        let vao = ctx.prim_map[&self.prim_type()];
        unsafe {
            let trans_loc = gl::GetUniformLocation(prog_id, GChar::new("transform").ptr());
            gl::UniformMatrix4fv(trans_loc, 1, gl::FALSE, trans.as_ptr());
            let color_loc = gl::GetUniformLocation(prog_id, GChar::new("color").ptr());
            gl::Uniform4f(color_loc, color.0, color.1, color.2, 1.0);
            gl::LineWidth(self.line_width as GLfloat);
            gl::BindVertexArray(vao);
            gl::DrawArrays(self.mode(), 0, self.size());
        }
    }
    fn prim_type(&self) -> PrimType {
        self.params.ptype()
    }
}

impl InBounds for Shape {
     fn in_bounds(&self, p: &Point, vp: &Point) -> bool {
        let trans_inv = glm::inverse(&self.trans(vp));
        let pc = pixels_to_trans_vec(p, vp);
        let normpt = trans_inv * glm::vec4(pc.x, pc.y, pc.z, 1.0);
        self.params.ptype().in_bounds(&Point {x: normpt.x, y: normpt.y})
    }
}

impl Draggable for Shape {
    fn drag(&mut self, off: Point) {
        self.offset += off;
    }
}

impl Drawable for DrawLine {
    fn draw(&self, ctx: &DrawCtx) {
        let mut trans: glm::TMat4<f32> = glm::identity();
        let p1c = pixels_to_trans_vec(&self.p1, &ctx.viewport);
        let p2c = pixels_to_trans_vec(&self.p2, &ctx.viewport);
        trans = glm::translate(&trans, &p1c);
        let color = rgb_to_f32(&self.color);
        let prog_id = ctx.prog_map[&self.prim_type()].id();
        let vao = ctx.prim_map[&self.prim_type()];
        unsafe {
            let trans_loc = gl::GetUniformLocation(prog_id, GChar::new("transform").ptr());
            gl::UniformMatrix4fv(trans_loc, 1, gl::FALSE, trans.as_ptr());
            let color_loc = gl::GetUniformLocation(prog_id, GChar::new("color").ptr());
            gl::Uniform4f(color_loc, color.0, color.1, color.2, 1.0);
            let point2_loc = gl::GetUniformLocation(prog_id, GChar::new("point2").ptr());
            gl::Uniform2f(point2_loc, p2c.x, p2c.y);
            gl::LineWidth(self.line_width as GLfloat);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::POINTS, 0, 1);
        }
    }
    fn prim_type(&self) -> PrimType {
        PT::Line
    }
}

impl InBounds for DrawLine {
    fn in_bounds(&self, p: &Point, _: &Point) -> bool {
        let mn = if self.p1.x < self.p2.x { self.p1.x } else { self.p2.x };
        let mx = if self.p1.x < self.p2.x { self.p2.x } else { self.p1.x };
        if p.x < mn || p.x > mx {
            return false;
        }
        let l = Line::from_pts(self.p1, self.p2);
        let dst = l.dist_to_pt(p);
        dst <= self.line_width
    }
}

impl Draggable for DrawLine {
    fn drag(&mut self, off: Point) {
        self.p1 += off;
        self.p2 += off;
    }
}

impl DrawBounds for Shape {}
impl DrawBounds for DrawLine {}

pub struct DrawCtx<'a> {
    prim_map: PrimMap,
    prog_map: ProgMap<'a>,
    viewport: Point,
}

impl<'a> DrawCtx<'a> {
    pub fn new(programs: &'a PrimPrograms, viewport: Point) -> DrawCtx<'a> {
        DrawCtx { prim_map: prim_map(), prog_map: prog_map(programs), viewport }
    }
}

pub struct DrawList<'a> {
    m: HashMap<u32, Box<DrawBounds + 'a>>,
    next_id: u32
}

impl<'a> DrawList<'a> {
    pub fn new() -> DrawList<'a> {
        DrawList {m: HashMap::new(), next_id: 0}
    }
    pub fn add<D: DrawBounds + 'a>(&mut self, s: D) {
        self.m.insert(self.next_id, Box::new(s));
        self.next_id += 1;
    }
    pub fn get_mut(&mut self, id: u32) -> Option<&mut Box<DrawBounds +'a>> {
        self.m.get_mut(&id)
    }
    pub fn click_shape(&'a self, p: &Point, vp: &Point) -> Option<u32> {
        self.m.iter().find(|(_,s)| s.in_bounds(p, vp)).map(|(k,_)| *k)
    }
    pub fn draw_all(&self, ctx: &DrawCtx) {
        self.m.iter().for_each(|(_, s)| {
            ctx.prog_map[&s.prim_type()].set_used();
            s.draw(ctx);
        });
    }
}

fn rgb_to_f32(rgb: &(u8, u8, u8)) -> (f32, f32, f32) {
    (rgb.0 as f32 / 255., rgb.1 as f32 / 255., rgb.2 as f32 / 255.)
}

fn pixels_to_trans_vec(pixels: &Point, vp: &Point) -> glm::TVec3<f32> {
    let coords = pixels_to_coords(pixels, vp);
    glm::vec3(coords.x, coords.y, 0.0)
}

fn pixels_to_coords(pixels: &Point, vp: &Point) -> Point {
    Point {x: -1. + 2. * pixels.x  / vp.x , y: 1. - 2. * pixels.y  / vp.y}
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

