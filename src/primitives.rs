use gl::types::{GLuint, GLint, GLfloat, GLenum, GLvoid, GLsizeiptr};
extern crate gl;
extern crate nalgebra_glm;
use nalgebra_glm as glm;
use std::collections::HashMap;
use std::ffi::CString;
use std::f32::{self, consts::PI};
use PrimType as PT;
use ShapeProps as SP;
use crate::render_gl::{Shader, Program, GChar, SendUniforms, SendUniform};

type PrimMap = HashMap<PrimType, GLuint>;

pub fn prim_map() -> PrimMap {
    let mut m = HashMap::new();
    for prim in &[PT::Triangle, PT::Circle, PT::Rect, PT::Ring, PT::Line] {
        m.insert(*prim, prim.buffer_data());
    }
    m
}

type ProgMap<'a> = HashMap<PrimType, &'a Program>;

pub struct PrimPrograms {
    line_prog: Program,
    shape_prog: Program,
}

impl PrimPrograms {
    pub fn new() -> PrimPrograms {
        let vert_shader = Shader::from_vert_source(
            &CString::new(include_str!("shaders/shape2d.vert")).unwrap()
        ).unwrap();

        let frag_shader = Shader::from_frag_source(
            &CString::new(include_str!("shaders/shape2d.frag")).unwrap()
        ).unwrap();

        let line_shader = Shader::from_vert_source(
            &CString::new(include_str!("shaders/line.vert")).unwrap()
        ).unwrap();

        let line_geom_shader = Shader::from_geom_source(
            &CString::new(include_str!("shaders/line.geom")).unwrap()
        ).unwrap();

        let mut shaders = vec![vert_shader, frag_shader];
        let shape_prog = Program::from_shaders(shaders.as_ref()).unwrap();

        shaders[0] = line_shader;
        shaders.insert(1, line_geom_shader);
        let line_prog = Program::from_shaders(shaders.as_ref()).unwrap();

        PrimPrograms {
            shape_prog,
            line_prog,
        }
    }
}

pub fn prog_map(programs: &PrimPrograms) -> ProgMap {
   let mut m = HashMap::new();
   for prim in &[PrimType::Triangle, PT::Circle, PT::Rect, PT::Ring] {
        m.insert(*prim, &programs.shape_prog);
    }
    m.insert(PT::Line, &programs.line_prog);
    m
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum PrimType {
    Triangle,
    Circle,
    Ring,
    Rect,
    Line,
}

const NCIRCLE_VERTS: usize = 30;

impl PrimType {
    fn verts(&self) -> Vec<f32> {
        match self {
            PT::Triangle => { //isosceles
                vec![
                    0., 1.0,
                    1.0, 1.0,
                    0.5, 0.
                ]
            },
            PT::Circle => {
                let mut v = vec![0.5, 0.5];
                v.extend(PT::Ring.verts());
                v
            },
            PT::Ring => {
                let n = NCIRCLE_VERTS as f32;
                (0..NCIRCLE_VERTS).map(|i| 
                    vec![0.5 + 0.5 * f32::cos(2.*PI*i as f32 / (n-1.)), 
                         0.5 + 0.5 * f32::sin(2.*PI*i as f32 / (n-1.))]
                ).flatten().collect()
            }
            PT::Rect => {
               vec![ 
                    0.0, 0.0,
                    1.0, 0.0, 
                    1.0, 1.0,
                    0.0, 1.0
                ]
            },
            PT::Line => {
               vec![0.0, 0.0]
            }
        }
    }
    fn buffer_data(&self) -> GLuint {
        unsafe { buffer_verts(&self.verts().as_slice()) }
    }
    fn mode(&self) -> GLenum {
        match self {
            PT::Triangle => gl::TRIANGLES,
            PT::Rect => gl::QUADS,
            PT::Circle => gl::TRIANGLE_FAN,
            PT::Ring => gl::LINE_STRIP, 
            PT::Line => gl::POINTS
        }
    }
    fn size(&self) -> usize {
        match self {
            PT::Triangle => 3,
            PT::Rect => 4,
            PT::Circle => NCIRCLE_VERTS + 1, 
            PT::Ring => NCIRCLE_VERTS, 
            PT::Line => 1
        }
    }
    fn in_bounds(&self, p: &Point) -> bool {
        match self {
            PT::Triangle => {
                p.x >= 0.0 && p.x <= 1.0 && p.y >= f32::abs(p.x - 0.5) && p.y <= 1.0
            }
            PT::Circle | PT::Ring => {
                (*p - Point{x: 0.5, y: 0.5}).mag() <= 0.5
            }
            PT::Rect => {
                p.x >= 0.0 && p.x <= 1.0 && p.y >= 0.0 && p.y <= 1.0 
            }
            PT::Line => {
                false
            }
        }
    }
}

unsafe fn buffer_verts(verts: &[f32]) -> GLuint {
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
    gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE,
        (2 * std::mem::size_of::<f32>()) as GLint,
        std::ptr::null());
    gl::BindBuffer(gl::ARRAY_BUFFER, 0);
    gl::BindVertexArray(0);
    vao
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32
}

#[allow(dead_code)]
impl Point {
    pub fn new() -> Point {
        Point {x:0., y:0.}
    }
    pub fn mag(&self) -> f32 {
        f32::sqrt((self.x*self.x + self.y*self.y) as f32)
    }
    pub fn dist(&self, p2: &Point) -> f32 {
        let d = *self - *p2;
        d.mag()
    }
    pub fn to_vec2(&self) -> glm::Vec2 {
        glm::vec2(self.x, self.y)
    }
    pub fn to_vec3(&self) -> glm::Vec3 {
        glm::vec3(self.x, self.y, 0.)
    }
    pub fn to_vec4(&self) -> glm::Vec4 {
        glm::vec4(self.x, self.y, 0., 1.)
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

impl std::ops::SubAssign for Point {
    fn sub_assign(&mut self, other: Point) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl std::ops::Mul for Point {
    type Output = Point;
    fn mul(self, other: Point) -> Self::Output {
        Point {x: self.x*other.x, y: self.y * other.y}
    }
}

impl std::ops::MulAssign for Point {
    fn mul_assign(&mut self, other: Point) {
        self.x *= other.x;
        self.y *= other.y;
    }
}

impl std::ops::Div<f32> for Point {
    type Output = Point;
    fn div(self, rhs: f32) -> Self {
        Point {x: self.x / rhs, y: self.y / rhs}
    }
}

#[derive(Debug)]
pub struct Line {
    a: f32, b: f32, c: f32 
}

#[allow(dead_code)]
fn det(a: f32, b: f32, c: f32, d: f32) -> f32 {
    a * d - b * c
}

#[allow(dead_code)]
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
        let (a,b,c) = (self.a, self.b, self.c);
        f32::abs(a*p.x + b*p.y + c) / f32::sqrt(a*a + b*b)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct DrawLine {
    pub p1: Point, 
    pub p2: Point,
    pub line_width: f32,
}

impl DrawLine {
    pub fn min_x(&mut self) -> &mut f32 {
        if self.p1.x < self.p2.x { &mut self.p1.x } else { &mut self.p2.x }
    }
    pub fn max_x(&mut self) -> &mut f32 {
        if self.p1.x > self.p2.x { &mut self.p1.x } else { &mut self.p2.x }
    }
    pub fn min_y(&mut self) -> &mut f32 {
        if self.p1.y < self.p2.y { &mut self.p1.y } else { &mut self.p2.y }
    }
    pub fn max_y(&mut self) -> &mut f32 {
        if self.p1.y > self.p2.y { &mut self.p1.y } else { &mut self.p2.y }
    }
}

impl Default for DrawLine {
    fn default() -> DrawLine {
        DrawLine {
            p1: Point::new(),
            p2: Point::new(),
            line_width: 3.
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct DrawPolygon {
    pub prim: PrimType,
    pub fill: bool,
    pub offset: Point,
    pub width: u32,
    pub height: u32,
    pub rot: f32,
}

impl Default for DrawPolygon {
    fn default() -> Self {
        DrawPolygon {
            prim: PT::Triangle,
            offset: Point::new(),
            fill: true,
            width: 5,
            height: 5,
            rot: 0.,
        }
    }
}

impl DrawPolygon {
    pub fn from_prim(ptype: PrimType) -> Self {
        DrawPolygon {
            prim: ptype,
            ..DrawPolygon::default()
        }
    }
    pub fn set_center(&mut self, pt: &Point) {
        self.offset = *pt - Point { x: self.width as f32 / 2., y: self.height as f32 / 2. };
    }
}

#[derive(SendUniforms)]
struct PolyTransform {
    projection: glm::Mat4,
    model: glm::Mat4,
}
    
#[allow(dead_code)]
impl PolyTransform {
    fn new(s: &DrawPolygon, vp: &Point) -> Self {
        let projection = glm::ortho(0., vp.x, vp.y, 0., -1., 1.);
        let rad = 180. * s.rot / PI;
        let mut model = glm::translate(&glm::identity(), &s.offset.to_vec3());

        model = glm::translate(&model, 
            &glm::vec3(0.5 * s.width as f32, 0.5 * s.height as f32, 0.));
        model = glm::rotate(&model, rad, &glm::vec3(0., 0., 1.));
        model = glm::translate(&model, 
            &glm::vec3(-0.5 * s.width as f32, -0.5 * s.height as f32, 0.));

        model = glm::scale(&model, &glm::vec3(s.width as f32, s.height as f32, 1.));
        PolyTransform {projection, model}
    }
    fn get(&self) -> glm::Mat4 {
        self.projection * self.model
    }
    fn transform(&self, coords: &glm::Vec4) -> glm::Vec4 {
        self.get() * coords
    }
    fn pixels_to_coords(&self, pt: &Point) -> glm::Vec4{
        self.projection * pt.to_vec4()
    }
    fn coords_to_pixels(&self, coords: &glm::Vec4) -> Point {
        let px = glm::inverse(&self.projection) * coords;
        Point {x: px[0], y: px[1]}
    }
    fn inv(&self) -> glm::Mat4 {
        glm::inverse(&self.get())
    }
}

#[derive(SendUniforms)]
struct LineTransform {
    point1: glm::Vec2, point2: glm::Vec2, projection: glm::Mat4
}

impl LineTransform {
    fn new(l: &DrawLine, vp: &Point) -> Self {
        LineTransform {
            point1: l.p1.to_vec2(),
            point2: l.p2.to_vec2(),
            projection: glm::ortho(0., vp.x, vp.y, 0., -1., 1.)
        }
    }
}

pub trait InBounds {
    fn in_bounds(&self, p: &Point, vp: &Point) -> bool;
}

#[derive(Clone, Debug)]
pub struct Rect {
    //upper left corner, lower right corner
    pub c1: Point, pub c2: Point
}

impl Rect {
    pub fn new(c1: Point, c2: Point) -> Self {
        Rect::bounding_box(&[c1, c2])
    }
    pub fn bounding_box(pts: &[Point]) -> Self {
        if pts.is_empty() { return Rect{c1: Point::new(), c2: Point::new()} }
        let mut min_pt = pts[0]; let mut max_pt = min_pt;
        for p in pts {
            if p.x < min_pt.x { min_pt.x = p.x };
            if p.x > max_pt.x { max_pt.x = p.x };
            if p.y < min_pt.y { min_pt.y = p.y };
            if p.y > max_pt.y { max_pt.y = p.y };
        }
        Rect {c1: min_pt, c2: max_pt}
    }
    pub fn builder(&self) -> ShapeBuilder {
        ShapeBuilder::new().rect(self.width() as u32, self.height() as u32)
            .offset(self.c1.x as i32, self.c1.y as i32)
    }
    pub fn center(&self) -> Point {
        (self.c1 + self.c2) / 2.
    }
    pub fn width(&self) -> f32 {
        f32::abs(self.c2.x - self.c1.x)
    }
    pub fn height(&self) -> f32 {
        f32::abs(self.c2.y - self.c1.y)
    }
    pub fn verts(&self) -> Vec<Point> {
        vec![self.c1, 
             self.ur(),
             self.c2,
             self.bl()] //clockwise
    }
    pub fn ur(&self) -> Point {
        Point{x: *self.max_x(), y: *self.min_y()}
    }
    pub fn bl(&self) -> Point {
        Point{x: *self.min_x(), y: *self.max_y()}
    }
    pub fn min_x(&self) -> &f32 {
        &self.c1.x
    }
    pub fn min_y(&self) -> &f32 {
        &self.c1.y
    }
    pub fn max_x(&self) -> &f32 {
        &self.c2.x
    }
    pub fn max_y(&self) -> &f32 {
        &self.c2.y
    }
}

impl InBounds for Rect {
    fn in_bounds(&self, p: &Point, _: &Point) -> bool {
        let min_x = if self.c1.x < self.c2.x { &self.c1.x } else { &self.c2.x };
        let max_x = if self.c1.x < self.c2.x { &self.c2.x } else { &self.c1.x };
        let min_y = if self.c1.y < self.c2.y { &self.c1.y } else { &self.c2.y };
        let max_y = if self.c1.y < self.c2.y { &self.c2.y } else { &self.c1.y };
        p.x >= *min_x && p.x <= *max_x && p.y >= *min_y && p.y <= *max_y
    }
}

impl InBounds for DrawPolygon {
    fn in_bounds(&self, p: &Point, vp: &Point) -> bool {
        let trans = PolyTransform::new(self, vp);
        let normpt = trans.inv() * trans.pixels_to_coords(p);
        self.prim.in_bounds(&Point {x: normpt.x, y: normpt.y})
    }
}

impl InBounds for DrawLine {
     fn in_bounds(&self, p: &Point, vp: &Point) -> bool {
        let mut c1 = self.p1;
        let mut c2 = self.p2;
        if c1.x == c2.x {
            c1.x -= self.line_width / 2.;
            c2.x += self.line_width / 2.;
        }
        if c2.y == c2.y {
            c1.y -= self.line_width / 2.;
            c2.y += self.line_width / 2.;
        }
        if !(Rect {c1, c2}).in_bounds(p, vp) {
            return false
        }
        let l = Line::from_pts(self.p1, self.p2);
        let dst = l.dist_to_pt(p);
        dst <= self.line_width
    }
}

#[derive(Clone, PartialEq)]
pub enum ShapeProps {
    Line(DrawLine),
    Polygon(DrawPolygon)
}

#[derive(Clone, PartialEq)]
pub struct Shape {
    pub props: ShapeProps,
    pub color: (u8, u8, u8), //rgb
    pub alpha: f32
}

impl Default for Shape {
    fn default() -> Self {
        Shape {
            props: ShapeProps::Polygon(DrawPolygon::default()),
            color: (0,0,0),
            alpha: 1.0
        }
    }
}

impl Shape {
    pub fn from_props(props: ShapeProps) -> Self {
        Shape {
            props,
            ..Shape::default()
        }
    }
    pub fn verts(&self, vp: &Point) -> Vec<Point> {
        match &self.props {
            SP::Polygon(ref draw_poly) => {
                let trans = PolyTransform::new(&draw_poly, vp);
                let v = self.prim_type().verts().chunks(2).map(|s| { 
                    let v = trans.transform(&glm::vec4(s[0], s[1], 0.0, 1.0));
                    trans.coords_to_pixels(&v)
                }).collect();
                v
            }
            SP::Line(draw_line) => {
                vec![draw_line.p1, draw_line.p2]
            }
        }
    }
    fn transform(&self, vp: &Point) -> Box<dyn SendUniforms> {
        match &self.props {
            SP::Polygon(draw_poly) => Box::new(PolyTransform::new(&draw_poly, vp)),
            SP::Line(draw_line) => Box::new(LineTransform::new(&draw_line, vp))
        }
    }
    pub fn draw(&self, ctx: &DrawCtx) {
        ctx.prog_map[&self.prim_type()].set_used();
        let trans = self.transform(&ctx.viewport);
        let color = rgb_to_f32(&self.color);
        let ptype = self.prim_type();
        let prog_id = ctx.prog_map[&ptype].id();
        let vao = ctx.prim_map[&ptype];
        let line_width = if let SP::Line(ref draw_line) = self.props { draw_line.line_width } else { 3. };
        if let SP::Polygon(ref draw_poly) = self.props {
            let poly_mode: GLuint = 
                if draw_poly.fill { gl::FILL } else { gl::LINE };
            unsafe { gl::PolygonMode(gl::FRONT_AND_BACK, poly_mode); }
        }
        unsafe {
            trans.send_uniforms(prog_id).unwrap();
            let color_loc = gl::GetUniformLocation(prog_id, GChar::new("color").ptr());
            gl::Uniform4f(color_loc, color.0, color.1, color.2, self.alpha);
            gl::LineWidth(line_width as GLfloat);
            gl::BindVertexArray(vao);
            gl::DrawArrays(self.mode(), 0, self.size());
        }
    }
    pub fn prim_type(&self) -> PrimType {
        match &self.props {
            SP::Polygon(draw_poly) => draw_poly.prim,
            SP::Line(_) => PT::Line
        }
    }
    fn mode(&self) -> GLenum { self.prim_type().mode() }
    fn size(&self) -> GLint { self.prim_type().size() as GLint }
}

impl InBounds for Shape {
    fn in_bounds(&self, p: &Point, vp: &Point) -> bool {
        match &self.props {
            SP::Polygon(draw_poly) => draw_poly.in_bounds(p,vp),
            SP::Line(draw_line) => draw_line.in_bounds(p,vp)
        }
    }
}

pub struct DrawCtx<'a> {
    prim_map: PrimMap,
    prog_map: ProgMap<'a>,
    pub viewport: Point,
}

impl<'a> DrawCtx<'a> {
    pub fn new(programs: &'a PrimPrograms, viewport: Point) -> DrawCtx<'a> {
        DrawCtx { prim_map: prim_map(), prog_map: prog_map(programs), viewport }
    }
}

pub struct ShapeBuilder {
    s: Shape,
    p: DrawPolygon
}

#[allow(dead_code)]
impl ShapeBuilder {
    pub fn new() -> Self {
        ShapeBuilder { s: Shape::default(), p: DrawPolygon::default() }
    }
    pub fn offset(mut self, x: i32, y: i32) -> Self {
        self.p.offset = Point {x: x as f32,y: y as f32};
        self
    }
    pub fn rot(mut self, rot: f32) -> ShapeBuilder {
        self.p.rot = rot;
        self
    }
    pub fn color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.s.color = (r,g,b);
        self
    }
    pub fn alpha(mut self, a: f32) -> Self {
        self.s.alpha = a;
        self
    }
    pub fn circle(mut self, rad: u32) -> Self {
        self.p.prim = PT::Circle;
        self.p.width = rad;
        self.p.height = rad;
        self
    }
    pub fn ellipse(mut self, rad_x: u32, rad_y: u32) -> Self {
        self.p.prim = PT::Circle;
        self.p.width = rad_x;
        self.p.height = rad_y;
        self
    }
    pub fn rect(mut self, width: u32, height: u32) -> Self {
        self.p.prim = PT::Rect;
        self.p.width = width;
        self.p.height = height;
        self
    }
    pub fn square(mut self, side: u32) -> Self {
        self.p.prim = PT::Rect;
        self.p.width = side;
        self.p.height = side;
        self
    }
    pub fn tri(mut self, base: u32, height: u32) -> Self {
        self.p.prim = PT::Triangle;
        self.p.width = base;
        self.p.height = height;
        self
    }
    pub fn fill(mut self, fill: bool) -> Self {
        self.p.fill = fill;
        self
    }
    pub fn get(mut self) -> Shape { 
        if let PT::Circle = self.p.prim {
            if !self.p.fill {
                self.p.prim = PT::Ring
            }
        }
        if let PT::Ring = self.p.prim {
            if self.p.fill {
                self.p.prim = PT::Circle
            }
        }
        self.s.props = SP::Polygon(self.p); 
        self.s 
    }
}

pub struct LineBuilder {
    s: Shape,
    l: DrawLine
}

#[allow(dead_code)]
impl LineBuilder {
    pub fn new() -> LineBuilder {
        LineBuilder { s: Shape::default(), l: DrawLine::default() }
    }
    pub fn points(mut self, x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        self.l.p1 = Point {x: x1, y: y1};
        self.l.p2 = Point {x: x2, y: y2};
        self
    }
    pub fn color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.s.color = (r,g,b);
        self
    }
    pub fn alpha(mut self, a: f32) -> Self {
        self.s.alpha = a;
        self
    }
    pub fn line_width(mut self, width: f32) -> Self {
        self.l.line_width = width;
        self
    }
    pub fn get(mut self) -> Shape { self.s.props = SP::Line(self.l); self.s }
}

pub fn rgb_to_f32(rgb: &(u8, u8, u8)) -> (f32, f32, f32) {
    (rgb.0 as f32 / 255., rgb.1 as f32 / 255., rgb.2 as f32 / 255.)
}

