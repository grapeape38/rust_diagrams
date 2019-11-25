use gl::types::{GLuint, GLint, GLfloat, GLenum, GLvoid, GLsizeiptr};
extern crate gl;
extern crate nalgebra_glm;
extern crate newtype_derive;
extern crate macro_attr;
use nalgebra_glm as glm;
use std::collections::HashMap;
use std::ffi::CString;
use std::f32::{self, consts::PI};
use PrimType as PT;
use crate::render_gl::{Shader, Program, SendUniforms, SendUniform};
use crate::render_text::{RenderText};
use crate::hexcolor::HexColor;
use sem_graph_derive::SendUniforms;
use macro_attr::{macro_attr, macro_attr_impl};
use newtype_derive::*;
use std::rc::Rc;
use std::cell::RefCell;


type PrimMap = HashMap<PrimType, GLuint>;
type ProgMap = HashMap<PrimType, Rc<Program>>;

pub fn prim_map() -> PrimMap {
    let mut m = HashMap::new();
    for prim in &[PT::Triangle, PT::Circle, PT::Rect, PT::Ring, PT::Line, PT::HexColor] {
        m.insert(*prim, prim.buffer_data());
    }
    m
}

pub fn prog_map() -> ProgMap {
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

    let shapecolor_vert = Shader::from_vert_source(
        &CString::new(include_str!("shaders/shapecolor.vert")).unwrap()
    ).unwrap();

    let shapecolor_frag = Shader::from_frag_source(
        &CString::new(include_str!("shaders/shapecolor.frag")).unwrap()
    ).unwrap();

    let mut shaders = vec![vert_shader, frag_shader];
    let shape_prog = Rc::new(Program::from_shaders(shaders.as_ref()).unwrap());

    shaders[0] = line_shader;
    shaders.insert(1, line_geom_shader);
    let line_prog = Rc::new(Program::from_shaders(shaders.as_ref()).unwrap());

    shaders = vec![shapecolor_vert, shapecolor_frag];
    let shapecolor_prog = Rc::new(Program::from_shaders(shaders.as_ref()).unwrap());

    let mut m = HashMap::new();
    for prim in &[PrimType::Triangle, PT::Circle, PT::Rect, PT::Ring] {
        m.insert(*prim, Rc::clone(&shape_prog));
    }
    m.insert(PT::Line, line_prog);
    m.insert(PT::HexColor, shapecolor_prog);

    m
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum PrimType {
    Triangle,
    Circle,
    Ring,
    Rect,
    Line,
    HexColor,
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
            PT::HexColor => {
                let colors = &[
                    [255., 0., 0.],
                    [0., 255., 0.],
                    [0., 0., 255.],
                ];
                PT::Triangle.verts().chunks(2).enumerate().map(|(i, c)|
                    vec![c[0], c[1],
                    colors[i][0] / 255., colors[i][1] / 255., colors[i][2] / 255.]
                ).flatten().collect()
            }
        }
    }
    fn buffer_data(&self) -> GLuint {
        unsafe {
            match self {
                PT::HexColor => { HexColor::buffer_verts(&self.verts().as_slice()) }
                _ => buffer_verts(&self.verts().as_slice()) 
            }
        }
    }
    fn mode(&self) -> GLenum {
        match self {
            PT::Triangle => gl::TRIANGLES,
            PT::Rect => gl::QUADS,
            PT::Circle => gl::TRIANGLE_FAN,
            PT::Ring => gl::LINE_STRIP, 
            PT::Line => gl::POINTS,
            PT::HexColor=> gl::TRIANGLES,
        }
    }
    pub fn size(&self) -> usize {
        match self {
            PT::Triangle => 3,
            PT::Rect => 4,
            PT::Circle => NCIRCLE_VERTS + 1, 
            PT::Ring => NCIRCLE_VERTS, 
            PT::Line => 1,
            PT::HexColor => 3 
        }
    }
    fn in_bounds(&self, p: &Point) -> bool {
        match self {
            PT::Triangle => {
                p.x >= 0.0 && p.x <= 1.0 && p.y >= f32::abs(p.x - 0.5) && p.y <= 1.0
            }
            PT::Circle | PT::Ring | PT::HexColor => {
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

impl From<glm::Vec4> for Point {
    fn from(v: glm::Vec4) -> Point {
        Point {x: v[0], y: v[1]}
    }
}

impl From<glm::TVec2<i32>> for Point {
    fn from(v: glm::TVec2<i32>) -> Point {
        Point {x: v[0] as f32, y: v[1] as f32}
    }
}

#[allow(dead_code)]
impl Point {
    pub fn origin() -> Point {
        Point::new(0.,0.)
    }
    pub fn new(x: f32, y: f32) -> Point {
        Point {x, y}
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

impl std::ops::Div<Point> for Point {
    type Output = Point;
    fn div(self, rhs: Point) -> Self {
        Point {x: self.x / rhs.x, y: self.y / rhs.y}
    }
}

macro_attr! {
    #[derive(Copy, Clone, PartialEq, NewtypeAdd!, NewtypeSub!)]
    pub struct Radians(pub f32);
}

#[derive(Copy, Clone, PartialEq)]
pub struct Degrees(pub f32);

impl From<Degrees> for Radians {
    fn from(deg: Degrees) -> Self {
        Radians(deg.0 * PI / 180.)
    }
}

#[derive(Clone, PartialEq)]
pub struct RotateRect {
    pub offset: Point,
    pub size: Point,
    pub rot: Radians,
    pub trans: TransformCache<(Point, Point, Radians), RectTransform>
}

impl RotateRect {
    pub fn new(offset: Point, size: Point, rot: Radians) -> Self {
        RotateRect { offset, size, rot, trans: TransformCache::new() }
    }
    pub fn from_rect(rect: Rect, rot: Radians) -> Self {
        RotateRect { offset: rect.c1, size: Point::new(rect.width(), rect.height()), rot, trans: TransformCache::new() }
    }
    pub fn drag(&mut self, offset: &Point) {
        self.offset += *offset;
    }
    pub fn center(&self, vp: &Point) -> Point {
        self.verts(vp).iter().fold(Point::origin(), |acc, curr| { acc + *curr }) / 4.
    }
    pub fn set_radians(&mut self, mut radians: Radians) {
        radians.0 -= 2. * PI * (radians.0 / 2. / PI).floor();
        self.rot = radians;
    }
    pub fn set_corner(&mut self, c: &Point, vp: &Point) {
        self.offset = Point::origin();
        let t = RectTransform::new(&self, vp);
        let c0 = t.model_to_pixel(&Point::origin().to_vec4());
        self.offset = *c - c0;
    }
    pub fn set_center(&mut self, pt: &Point) {
        self.offset = *pt - (self.size / 2.);
    }
    pub fn set_size(&mut self, size: &Point) {
        self.size = *size;
    }
    pub fn resize(&mut self, model_rect: &Rect, vp: &Point) {
        let corner = self.transform(vp).model_to_pixel(&model_rect.c1.to_vec4());
        self.set_size(&(model_rect.size() * self.size));
        self.set_corner(&corner, vp);
    }
    pub fn verts(&self, vp: &Point) -> Vec<Point>  {
        let trans = self.transform(vp);
        let v = PrimType::Rect.verts().chunks(2).map(|s| { 
            trans.model_to_pixel(&glm::vec4(s[0], s[1], 0.0, 1.0))
        }).collect();
        v
    }
    pub fn in_bounds(&self, p: &Point, vp: &Point) -> bool {
        PrimType::Rect.in_bounds(&self.transform(vp).pixel_to_model(p).into())
    }
    pub fn builder(&self) -> ShapeBuilder {
        ShapeBuilder { p: self.to_poly(), ..ShapeBuilder::new() }
    }
    pub fn transform(&self, vp: &Point) -> RectTransform {
        self.trans.transform(
            (self.offset, self.size, self.rot),
            Box::new(move || RectTransform::new(self, vp)))
    }
    pub fn to_poly(&self) -> DrawPolygon {
        DrawPolygon {
            prim: PrimType::Rect, rect: self.clone(), ..DrawPolygon::default() 
        }
    }
}

impl Default for RotateRect {
    fn default() -> Self {
        RotateRect {
            offset: Point::origin(),
            size: Point::new(5.,5.),
            rot: Radians(0.),
            trans: TransformCache::new() 
        }
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
    pub color: glm::Vec4
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
    pub fn draw(&self, ctx: &DrawCtx) {
        let ptype = &PrimType::Line;
        ctx.prog_map[ptype].set_used();
        let trans = LineTransform::new(self, &ctx.viewport);
        let prog_id = ctx.prog_map[&ptype].id();
        let vao = ctx.prim_map[&ptype];
        let line_width = self.line_width; 
        unsafe {
            trans.send_uniforms(prog_id).unwrap();
            self.color.send_uniform(prog_id, "color").unwrap();
            gl::LineWidth(line_width as GLfloat);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::POINTS, 0, 1);
        }
    }
}

impl Default for DrawLine {
    fn default() -> DrawLine {
        DrawLine {
            p1: Point::origin(),
            p2: Point::origin(),
            line_width: 3.,
            color: glm::vec4(0., 0., 0., 1.)
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct DrawPolygon {
    pub prim: PrimType,
    pub fill: bool,
    pub rect: RotateRect,
    pub color: glm::Vec4
}

impl Default for DrawPolygon {
    fn default() -> Self {
        DrawPolygon {
            prim: PT::Triangle,
            rect: RotateRect::default(),
            fill: true,
            color: glm::vec4(0., 0., 0., 1.)
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
    pub fn verts(&self, vp: &Point) -> Vec<Point> {
        let trans = self.rect.transform(vp);
        let v = self.prim.verts().chunks(2).map(|s| { 
            trans.model_to_pixel(&glm::vec4(s[0], s[1], 0.0, 1.0))
        }).collect();
        v
    }
    pub fn draw(&self, ctx: &DrawCtx) {
        let ptype = &self.prim;
        ctx.prog_map[ptype].set_used();
        let trans = self.rect.transform(&ctx.viewport);
        let prog_id = ctx.prog_map[ptype].id();
        let vao = ctx.prim_map[ptype];
        let poly_mode: GLuint = if self.fill { gl::FILL } else { gl::LINE };
        unsafe {
            gl::PolygonMode(gl::FRONT_AND_BACK, poly_mode); 
            trans.send_uniforms(prog_id).unwrap();
            self.color.send_uniform(prog_id, "color").unwrap();
            gl::BindVertexArray(vao);
            gl::DrawArrays(ptype.mode(), 0, ptype.size() as i32);
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct TransformCache<C: PartialEq + Clone, T: SendUniforms + PartialEq + Clone> {
    pub trans: RefCell<Option<(C, T)>>,
}

impl<C: PartialEq + Clone, T: SendUniforms + PartialEq + Clone> TransformCache<C, T> {
    pub fn new() -> TransformCache<C, T> {
        TransformCache { trans: RefCell::new(None) }
    }
    pub fn transform<U: FnOnce() -> T>(&self, key: C, create: U) -> T {
        let trans = {
            if let Some((cache, tr)) = self.trans.replace(None) {
                if key != cache { create() }
                else { tr }
            }
            else { create() }
        };
        self.trans.replace(Some((key.clone(), trans.clone())));
        trans
    }
}

#[derive(SendUniforms, Clone, PartialEq)]
pub struct RectTransform {
    pub projection: glm::Mat4,
    pub model: glm::Mat4,
}
    
#[allow(dead_code)]
impl RectTransform {
    pub fn new(r: &RotateRect, vp: &Point) -> Self {
        let projection = glm::ortho(0., vp.x, vp.y, 0., -1., 1.);
        let mut model = glm::translate(&glm::identity(), &r.offset.to_vec3());

        model = glm::translate(&model, &(r.size / 2.).to_vec3());
        model = glm::rotate(&model, r.rot.0, &glm::vec3(0., 0., 1.));
        model = glm::translate(&model, &(-r.size / 2.).to_vec3());

        model = glm::scale(&model, &glm::vec3(r.size.x, r.size.y, 1.));
        RectTransform {projection, model}
    }
    fn get(&self) -> glm::Mat4 {
        self.projection * self.model
    }
    fn transform(&self, coords: &glm::Vec4) -> glm::Vec4 {
        self.get() * coords
    }
    pub fn pixel_to_model(&self, pt: &Point) -> glm::Vec4 {
        glm::inverse(&self.model) * pt.to_vec4() 
    }
    pub fn model_to_pixel(&self, coords: &glm::Vec4) -> Point {
        (self.model * coords).into()
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

impl Default for Rect {
    fn default() -> Self {
        Rect { c1: Point{x:0.,y:0.}, c2: Point{x:1., y:1.} }
    }
}

#[allow(dead_code)]
impl Rect {
    pub fn empty() -> Self {
        Rect::new(Point::origin(), Point::origin())
    }
    pub fn new(c1: Point, c2: Point) -> Self {
        Rect::bounding_box(&[c1, c2])
    }
    pub fn bounding_box(pts: &[Point]) -> Self {
        if pts.is_empty() { return Rect{c1: Point::origin(), c2: Point::origin()} }
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
    pub fn size(&self) -> Point {
        Point::new(self.width(), self.height())
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
    pub fn left_center(&self) -> Point {
        Point {x: *self.min_x(), y: (self.min_y() + self.max_y()) / 2. }
    }
    pub fn top_center(&self) -> Point {
        Point {x: (self.min_x() + self.max_x()) / 2., y: *self.min_y() }
    }
    pub fn right_center(&self) -> Point {
        Point {x: *self.max_x(), y: (self.min_y() + self.max_y()) / 2. }
    }
    pub fn bot_center(&self) -> Point {
        Point {x: (self.min_x() + self.max_x()) / 2., y: *self.max_y() }
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
        let trans = self.rect.transform(vp);
        self.prim.in_bounds(&trans.pixel_to_model(p).into())
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
pub enum Shape {
    Line(DrawLine),
    Polygon(DrawPolygon)
}

impl Shape {
    pub fn verts(&self, vp: &Point) -> Vec<Point> {
        match self {
            Shape::Polygon(ref draw_poly) => {
                draw_poly.verts(vp)
            }
            Shape::Line(draw_line) => {
                vec![draw_line.p1, draw_line.p2]
            }
        }
    }
    pub fn rect(&self) -> RotateRect {
        match self {
            Shape::Polygon(ref draw_poly) => draw_poly.rect.clone(),
            Shape::Line(_) => RotateRect::default()
        }
    }
    pub fn draw(&self, ctx: &DrawCtx) {
        match self {
            Shape::Polygon(draw_poly) => draw_poly.draw(ctx),
            Shape::Line(draw_line) => draw_line.draw(ctx)
        }
    }
    pub fn rgb(&self) -> (u8, u8, u8) {
        let color = match self {
            Shape::Polygon(draw_poly) => draw_poly.color,
            Shape::Line(draw_line) => draw_line.color
        };
        ((color[0] * 255.) as u8, (color[1] * 255.) as u8, (color[2] * 255.) as u8)
    }
}

impl InBounds for Shape {
    fn in_bounds(&self, p: &Point, vp: &Point) -> bool {
        match self {
            Shape::Polygon(draw_poly) => draw_poly.in_bounds(p,vp),
            Shape::Line(draw_line) => draw_line.in_bounds(p,vp)
        }
    }
}

pub struct DrawCtx {
    pub prim_map: PrimMap,
    pub prog_map: ProgMap,
    pub viewport: Point,
    pub render_text: RenderText
}

impl DrawCtx {
    pub fn new(viewport: &Point) -> DrawCtx {
        DrawCtx { prim_map: prim_map(), prog_map: prog_map(), viewport: *viewport, render_text: RenderText::new().unwrap() }
    }
    #[allow(dead_code)]
    pub fn draw_circle(&self, radius: f32, center: Point, color: glm::Vec4, fill: bool) {
        let rect = RotateRect::new(Point::new(center.x - radius, center.y - radius), Point::new(radius * 2., radius * 2.), Radians(0.));
        let prim = if fill { PrimType::Circle } else { PrimType::Ring };
        DrawPolygon { prim, fill, color, rect }.draw(self);
    }
    #[allow(dead_code)]
    pub fn draw_rect(&self, rect: Rect, color: glm::Vec4, fill: bool, rot: Radians) {
        let rect = RotateRect::from_rect(rect, rot);
        DrawPolygon { prim: PrimType::Rect, fill, color, rect }.draw(self);
    }
    #[allow(dead_code)]
    pub fn draw_square(&self, side: f32, offset: Point, color: glm::Vec4, fill: bool, rot: Radians) {
        let rect = RotateRect::new(offset, Point::new(side, side), rot);
        DrawPolygon { prim: PrimType::Rect, fill, color, rect }.draw(self);
    }
    #[allow(dead_code)]
    pub fn draw_line(&self, p1: Point, p2: Point, color: glm::Vec4, line_width: f32) {
        DrawLine { p1, p2, line_width, color }.draw(self);
    }
}

pub struct ShapeBuilder {
    p: DrawPolygon
}

#[allow(dead_code)]
impl ShapeBuilder {
    pub fn new() -> Self {
        ShapeBuilder { p: DrawPolygon::default() }
    }
    pub fn offset(mut self, x: i32, y: i32) -> Self {
        self.p.rect.offset = Point {x: x as f32,y: y as f32};
        self
    }
    pub fn rot<T: Into<Radians>>(mut self, rot: T) -> ShapeBuilder {
        self.p.rect.rot = rot.into();
        self
    }
    pub fn color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.p.color = rgb_to_f32(r,g,b);
        self
    }
    pub fn alpha(mut self, a: f32) -> Self {
        self.p.color[3] = a;
        self
    }
    pub fn circle(mut self, rad: u32) -> Self {
        self.p.prim = PT::Circle;
        self.p.rect.size = Point::new(rad as f32, rad as f32);
        self
    }
    pub fn ellipse(mut self, rad_x: u32, rad_y: u32) -> Self {
        self.p.prim = PT::Circle;
        self.p.rect.size = Point::new(rad_x as f32, rad_y as f32);
        self
    }
    pub fn rect(mut self, width: u32, height: u32) -> Self {
        self.p.prim = PT::Rect;
        self.p.rect.size = Point::new(width as f32, height as f32);
        self
    }
    pub fn square(mut self, side: u32) -> Self {
        self.p.prim = PT::Rect;
        self.p.rect.size = Point::new(side as f32, side as f32);
        self
    }
    pub fn tri(mut self, base: u32, height: u32) -> Self {
        self.p.prim = PT::Triangle;
        self.p.rect.size = Point::new(base as f32, height as f32);
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
        Shape::Polygon(self.p)
    }
}

pub struct LineBuilder {
    l: DrawLine
}

#[allow(dead_code)]
impl LineBuilder {
    pub fn new() -> LineBuilder {
        LineBuilder { l: DrawLine::default() }
    }
    pub fn points2(mut self, p1: &Point, p2: &Point) -> Self {
        self.l.p1 = *p1;
        self.l.p2 = *p2;
        self
    }
    pub fn points(mut self, x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        self.l.p1 = Point {x: x1, y: y1};
        self.l.p2 = Point {x: x2, y: y2};
        self
    }
    pub fn color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.l.color = rgb_to_f32(r,g,b);
        self
    }
    pub fn alpha(mut self, a: f32) -> Self {
        self.l.color[3] = a;
        self
    }
    pub fn line_width(mut self, width: f32) -> Self {
        self.l.line_width = width;
        self
    }
    pub fn get(self) -> Shape { Shape::Line(self.l) }
}

pub fn rgb_to_f32(r: u8, g: u8, b: u8) -> glm::Vec4 {
    glm::vec4(r as f32 / 255., g as f32 / 255., b as f32 / 255., 1.)
}

#[derive(Clone)]
pub struct Border {
    pub width: Point,
    pub color: glm::Vec4
}

impl Border {
    pub fn new(width: Point, color: glm::Vec4) -> Self {
        Border { width, color }
    }
}

pub struct BorderRect {
    pub r: Rect,
    fill_color: glm::Vec4,
    pub border: Border
}

impl BorderRect { 
    pub fn new(r: Rect, fill_color: glm::Vec4, border: Border) -> Self {
        BorderRect { r, fill_color, border }
    }
    pub fn draw(&self, draw_ctx: &DrawCtx) {
        let border = Rect::new(self.r.c1 - self.border.width, self.r.c2 + self.border.width);
        draw_ctx.draw_rect(border.clone(), self.fill_color, true, Radians(0.)); 
        draw_ctx.draw_rect(border.clone(), self.border.color, false, Radians(0.)); 
    }
}

/*fn draw_triangle(radius: f32, center: &Point, color: (u8, u8, u8), draw_ctx: &DrawCtx) {
    let rect = RotateRect::new(Point::new(center.x - radius, center.y - radius), Point::new(radius * 2., radius * 2.), Radians(0.));
    DrawPolygon { prim: PrimType::Circle, fill: true, color: rgb_to_f32(color.0, color.1, color.2), rect }.draw(draw_ctx);
}*/

