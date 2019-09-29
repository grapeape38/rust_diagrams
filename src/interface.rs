extern crate sdl2;

use num_derive::FromPrimitive;    
use num_traits::FromPrimitive;
use std::collections::{HashMap};
use sdl2::event::Event;
use sdl2::keyboard::Mod;
use sdl2::mouse::{Cursor, SystemCursor};
use std::iter::FromIterator;
use crate::primitives::*;
use crate::ShapeProps as SP;

pub struct CursorMap(HashMap<SystemCursor, Cursor>);
impl CursorMap {
    fn new() -> Self {
        let mut m = HashMap::new();
        m.insert(SystemCursor::Arrow, Cursor::from_system(SystemCursor::Arrow).unwrap());
        m.insert(SystemCursor::Hand, Cursor::from_system(SystemCursor::Hand).unwrap());
        m.insert(SystemCursor::Crosshair, Cursor::from_system(SystemCursor::Crosshair).unwrap());
        m.insert(SystemCursor::SizeNESW, Cursor::from_system(SystemCursor::SizeNESW).unwrap());
        m.insert(SystemCursor::SizeNS, Cursor::from_system(SystemCursor::SizeNS).unwrap());
        m.insert(SystemCursor::SizeNWSE, Cursor::from_system(SystemCursor::SizeNWSE).unwrap());
        m.insert(SystemCursor::SizeWE, Cursor::from_system(SystemCursor::SizeWE).unwrap());
        CursorMap(m)
    }
    fn get(&self, cursor: &SystemCursor) -> &Cursor {
        &self.0[cursor]
    }
}

impl Rect {
    fn drag(&mut self, off: &Point) {
        self.c1 += *off;
        self.c2 += *off;
    }
}

#[allow(dead_code)]
impl Shape {
    fn drag(&mut self, off: &Point) {
        match self.props {
            SP::Line(ref mut draw_line) => {
                draw_line.p1 += *off;
                draw_line.p2 += *off;
            }
            SP::Polygon(ref mut draw_poly) => {
                draw_poly.rect.offset += *off;
            }
        }
    }
    fn click(&self, p: &Point, vp: &Point) -> ClickResponse {
        match self.in_bounds(p, vp) {
            true => {
                ClickResponse::Clicked
            }
            false => {
                ClickResponse::NotClicked
            }
        }
    }
    fn in_select_box(&self, r: &Rect, vp: &Point) -> bool {
        self.verts(&vp).iter().any(|v| r.in_bounds(v, vp))
    }
    fn set_rect(&mut self, r: &RotateRect) {
        match self.props {
            SP::Polygon(ref mut draw_poly) => {
                draw_poly.rect = r.clone();
            }
            SP::Line(_) => { }
        }
    }
    fn drag_side(&mut self, r: &Rect) {
        match self.props {
            SP::Polygon(ref mut draw_poly) => {
                draw_poly.rect.offset = r.c1;
                draw_poly.rect.size = Point::new(r.width(), r.height());
            }
            SP::Line(ref mut draw_line) => {
                *draw_line.min_x() = *r.min_x();
                *draw_line.max_x() = *r.max_x();
                *draw_line.min_y() = *r.min_y();
                *draw_line.max_y() = *r.max_y();
            }
        }
    }
}

pub struct DrawList {
    m: HashMap<u32, Shape>,
    draw_order: Vec<u32>,
    next_id: u32
}

impl DrawList {
    pub fn new() -> DrawList {
        DrawList {m: HashMap::new(), draw_order: Vec::new(), next_id: 0}
    }
    pub fn add(&mut self, s: Shape) {
        self.m.insert(self.next_id, s);
        self.draw_order.push(self.next_id);
        self.next_id += 1;
    }
    fn get(&self, id: &u32) -> Option<&Shape> {
        self.m.get(id)
    }
    fn get_mut(&mut self, id: &u32) -> Option<&mut Shape> {
        self.m.get_mut(id)
    }
    fn click_shape(&mut self, p: &Point, vp: &Point) -> Option<u32> {
        let draw_idx = self.draw_order.iter().rev().position(|idx| {
            self.m.get(idx).map_or(ClickResponse::NotClicked, |s| s.click(p, vp)) != ClickResponse::NotClicked
        }).map(|idx| self.draw_order.len() - 1 - idx); //go in reverse to get shape that's rendered last
        if let Some(idx) = draw_idx {
            let elem = self.draw_order.remove(idx as usize);
            self.draw_order.push(elem);
            return Some(elem) 
        }
        None
    }
    #[inline]
    fn get_box_selection(&self, r: &Rect, vp: &Point) -> Vec<(u32, &Shape)> {
        self.m.iter().filter(|(_,s)| s.in_select_box(r, vp)).map(|(id, s)| (*id,s)).collect()
    }
    pub fn draw(&self, ctx: &DrawCtx) {
        self.draw_order.iter().for_each(|idx| {
            let s = &self.m[idx];
            s.draw(ctx);
        });
    }
}

type ShapeID = u32;

pub struct AppState<'a> {
    draw_list: DrawList,
    selection: HashMap<ShapeID, ShapeSelectBox>,
    shape_bar: ShapeBar,
    drag_mode: DragMode,
    hover_item: HoverItem,
    draw_ctx: DrawCtx<'a>,
    cursors: CursorMap
    //click_mode: ClickMode,
}

/*#[derive(Clone, Copy)]
enum ClickMode {
    Select,
    CreateShape {shape_id: ShapeID}
}*/

#[derive(Clone, Copy)]
pub enum DragMode {
    DragNone,
    SelectBox {start_pt: Point, last_pt: Point},
    CreateShape { shape_id: ShapeID, start_pt: Point, last_pt: Point },
    DragShapes { last_pt: Point, click_shape: ShapeID, clear_select: bool },
    DragResize { click_box: ShapeID, drag_vertex: DragVertex }
}


#[derive(PartialEq, Clone)]
pub enum HoverItem {
   HoverNone,
   HoverVertex(ShapeID, DragVertex),
   HoverShape(ShapeID, Shape),
   HoverRect(ShapeID) 
}

impl<'a> AppState<'a> {
    pub fn new(draw_list: DrawList, draw_ctx: DrawCtx<'a>) -> AppState<'a> {
        AppState {
            draw_list,
            shape_bar: ShapeBar::new(&draw_ctx.viewport),
            selection: HashMap::new(),
            //click_mode: ClickMode::Select,
            drag_mode: DragMode::DragNone,
            hover_item: HoverItem::HoverNone,
            draw_ctx,
            cursors: CursorMap::new()
        }
    }
    fn get_shape_select_box(&self, s: &Shape) -> ShapeSelectBox {
        ShapeSelectBox(Rect::bounding_box(&s.verts(&self.draw_ctx.viewport)))
    }
    fn fuzzy_hover_rect(&self, p: &Point, vp: &Point) -> Option<ShapeID> {
        self.selection.iter().find(|(_, r)| r.fuzzy_in_bounds(p, vp)).map(|(id, _)| *id)
    }
    pub fn handle_hover_click(&mut self, pt: &Point, clear_select: bool, cursor: &mut SystemCursor) {
        match self.hover_item {
            HoverItem::HoverRect(select_id) => {
                if self.selection[&select_id].in_bounds(pt, &self.draw_ctx.viewport) {
                    self.drag_mode = DragMode::DragShapes { last_pt: *pt, click_shape: select_id, clear_select };
                    *cursor = SystemCursor::Hand;
                }
            }
            HoverItem::HoverVertex(select_id, drag_vertex) => {
                self.drag_mode = DragMode::DragResize { click_box: select_id, drag_vertex };
                *cursor = get_drag_hover_cursor(&drag_vertex);
            }
            HoverItem::HoverShape(shape_id, _) => {
                self.drag_mode = DragMode::CreateShape {shape_id, start_pt: *pt, last_pt: *pt};
                self.hover_item = HoverItem::HoverNone;
                *cursor = SystemCursor::Crosshair;
            }
            _ => {}
        }
    }
    pub fn handle_select(&mut self, pt: &Point, clear_select: bool, cursor: &mut SystemCursor) {
        if clear_select {
            self.selection.clear();
        }
        if let Some(shape_id) = self.shape_bar.click_shape(&pt, &self.draw_ctx.viewport) {
            *cursor =  SystemCursor::Crosshair;
            let r = Rect::new(*pt, *pt);
            let s = self.shape_bar.get_shape(shape_id, &r, false);
            self.hover_item = HoverItem::HoverShape(shape_id, s)
        }
        else if let Some(click_shape) = self.draw_list.click_shape(&pt, &self.draw_ctx.viewport) {
            let s = self.draw_list.get(&click_shape).unwrap();
            self.selection.insert(click_shape, self.get_shape_select_box(s));
            self.drag_mode = DragMode::DragShapes { last_pt: *pt, click_shape, clear_select };
            self.hover_item = HoverItem::HoverRect(click_shape);
            *cursor = SystemCursor::Hand;
        }
        else if clear_select {
            self.drag_mode = DragMode::SelectBox{start_pt: *pt, last_pt: *pt};
        }
    }
    fn handle_drag(&mut self, pt: &Point, cursor: &mut SystemCursor) {
        match self.drag_mode {
            DragMode::DragShapes { ref mut last_pt, ref mut clear_select, .. } => {
                *clear_select = false;
                *cursor = SystemCursor::Hand;
                for (id, ref mut rect) in self.selection.iter_mut() {
                    self.draw_list.get_mut(id).map(|s| s.drag(&(*pt - *last_pt)));
                    rect.drag(&(*pt - *last_pt));
                }
                *last_pt = *pt;
            }
            DragMode::SelectBox {start_pt, ref mut last_pt} => {
                *last_pt = *pt;
                self.selection = 
                    HashMap::from_iter(
                        self.draw_list.get_box_selection(&Rect::new(start_pt, *pt), &self.draw_ctx.viewport).iter().map(|(id, shape)|
                            (*id, self.get_shape_select_box(shape))
                        ));
            }
            DragMode::DragResize { click_box, ref mut drag_vertex } => {
                *cursor = get_drag_hover_cursor(&drag_vertex);
                *drag_vertex = self.selection.get_mut(&click_box).map(|s| s.drag_side(&drag_vertex, &pt))
                    .unwrap_or(*drag_vertex);
                if let Some(ref sbox) = self.selection.get(&click_box) {
                    self.draw_list.get_mut(&click_box).map(|s| s.drag_side(&sbox.0.clone()));
                }
            }
            DragMode::CreateShape { ref mut last_pt, .. } => {
                *last_pt = *pt;
                *cursor = SystemCursor::Crosshair;
            }
            _ => {}
        }
    }
    fn handle_hover(&mut self, pt: &Point, cursor: &mut SystemCursor) {
        if let HoverItem::HoverShape(_, ref mut s) = self.hover_item {
            *cursor = SystemCursor::Crosshair;
            if let ShapeProps::Polygon(ref mut poly) = s.props {
                poly.rect.set_center(pt);
            }
        }
        else if let Some(select_id) = self.fuzzy_hover_rect(pt, &self.draw_ctx.viewport) {
            if let Some(drag_vertex) = self.selection[&select_id].get_drag_vertex(pt) {
                *cursor = get_drag_hover_cursor(&drag_vertex);
                self.hover_item = HoverItem::HoverVertex(select_id, drag_vertex);
            }
            else { 
                *cursor = SystemCursor::Hand;
                self.hover_item = HoverItem::HoverRect(select_id);
            }
        }
        else {
            self.hover_item = HoverItem::HoverNone;
        }
    }
    pub fn handle_mouse_event(&mut self, ev: &Event, kmod: &Mod) {
        match *ev {
            Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                if mouse_btn == sdl2::mouse::MouseButton::Left {
                    let pt = Point{x: x as f32,y: y as f32};
                    let mut use_cursor = SystemCursor::Arrow;
                    let clear_select = (*kmod & Mod::LCTRLMOD) == Mod::NOMOD;
                    if self.hover_item != HoverItem::HoverNone {
                        self.handle_hover_click(&pt, clear_select, &mut use_cursor);
                    }
                    else {
                        self.handle_select(&pt, clear_select, &mut use_cursor);
                    }
                    self.cursors.get(&use_cursor).set();
                }
            } 
            Event::MouseButtonUp{mouse_btn, .. } => {
                if mouse_btn == sdl2::mouse::MouseButton::Left {
                    match self.drag_mode {
                        DragMode::DragShapes { click_shape, clear_select, .. } => {
                            if clear_select { 
                                let s = self.selection[&click_shape].clone();
                                self.selection.clear();
                                self.selection.insert(click_shape, s);
                            }
                        },
                        DragMode::CreateShape { shape_id, start_pt, last_pt } => {
                            let s = self.shape_bar.get_shape(
                                shape_id, &Rect::new(start_pt, last_pt), true);
                            self.draw_list.add(s);
                        }
                        _ => {}
                    }
                    self.drag_mode = DragMode::DragNone;
                }
            }
            Event::MouseMotion{ x, y, ..} => {
                let pt = Point{x:x as f32, y:y as f32};
                let mut use_cursor = SystemCursor::Arrow;
                if let DragMode::DragNone = self.drag_mode {
                    self.handle_hover(&pt, &mut use_cursor);
                }
                else {
                    self.handle_drag(&pt, &mut use_cursor);
                }
                self.cursors.get(&use_cursor).set();
            }
            _ => {}
        }
    }
    fn draw_hover_item(&self) {
        if let HoverItem::HoverShape(_, ref shape) = self.hover_item {
            shape.draw(&self.draw_ctx);
        }
    }
    fn draw_select_box(&self) {
        match self.drag_mode {
            DragMode::SelectBox{start_pt, last_pt} => {
                Rect::new(start_pt, last_pt).builder().color(0,0,0).fill(false).get().draw(&self.draw_ctx);
            }
            DragMode::CreateShape{shape_id, start_pt, last_pt} => {
                let r = Rect::new(start_pt, last_pt);
                self.shape_bar.get_shape(shape_id, &r, false).draw(&self.draw_ctx);
            }
            _ => {}
        }
    }
    fn draw_shape_select_boxes(&self) {
        for r in self.selection.values() {
            r.draw(&self.draw_ctx);
        }
    }
    pub fn render(&self) {
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT); }
        //self.shape_bar.draw(&self.draw_ctx);
        self.draw_list.draw(&self.draw_ctx);
        self.draw_hover_item();
        self.draw_select_box();
        self.draw_shape_select_boxes();
    }
}

#[derive(Clone)]
struct ShapeSelectBox(Rect);

#[derive(PartialEq, Debug, Copy, Clone, FromPrimitive)]
pub enum DragVertex {
    TopLeft = 0,
    TopRight = 1,
    BottomRight = 2,
    BottomLeft = 3,
    TopCenter = 4,
    Right = 5,
    BottomCenter = 6,
    Left = 7,
}

fn get_drag_hover_cursor(drag_vertex: &DragVertex) -> SystemCursor {
    match drag_vertex {
        DragVertex::Left | DragVertex::Right => {
            SystemCursor::SizeWE
        }
        DragVertex::TopCenter | DragVertex::BottomCenter => {
            SystemCursor::SizeNS
        }
        DragVertex::TopLeft | DragVertex::BottomRight => {
            SystemCursor::SizeNWSE
        }
        DragVertex::TopRight | DragVertex::BottomLeft => {
            SystemCursor::SizeNESW
        }
    }
}

impl ShapeSelectBox {
    const MIN_CORNER_DIST: u32 = 10;

    fn drag(&mut self, off: &Point) {
        self.0.drag(off);
    }

    fn drag_side_swap_vertex(&mut self, vertex1: &DragVertex, vertex2: &DragVertex, start: &mut f32, min_max: &mut f32, new: &f32) 
        -> DragVertex
    {
        if *start < *min_max {
            *start = *new;
            if *new > *min_max {
                std::mem::swap(start, min_max);
                *vertex2
            }
            else {
                *vertex1
            }
        }
        else {
            *start = *new;
            if *new < *min_max {
                std::mem::swap(start, min_max);
                *vertex2
            }
            else {
                *vertex1
            }
        }
    }
    #[allow(dead_code)]
    fn drag_corner_swap_vertex(&mut self, vertex1: &DragVertex, vertex2: &DragVertex, start: &mut Point, min_max: &mut Point, new: &Point)
        -> DragVertex
    {
        let mut pt = *new;
        let width = f32::abs(start.x - min_max.x);
        let height = f32::abs(start.y - min_max.y);
        //if y is shrinking, or both shrinking or both expanding, base on x
        if  ((new.y <= start.y) != (start.y <= min_max.y) ||
            (new.x <= start.x) == (start.x <= min_max.x) && (new.y <= start.y) == (start.y <= min_max.y))
            && (f32::abs(new.x - start.x) < ShapeSelectBox::MIN_CORNER_DIST as f32)
        {
            pt.y = start.y + (new.x - start.x) * height / width;
        }
        //x shrinking, base on y
        else {
            pt.x = start.x + (new.y - start.y) * width / height;
        }
        self.drag_side_swap_vertex(vertex1, vertex2, &mut start.x, &mut min_max.x, &pt.x);
        self.drag_side_swap_vertex(vertex1, vertex2, &mut start.y, &mut min_max.y, &pt.y)
    }
    fn drag_side(&mut self, drag_vertex: &DragVertex, new_pt: &Point) -> DragVertex {
        let mut r = self.0.clone();
        let new_vtx = match *drag_vertex {
            DragVertex::TopCenter => {
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::BottomCenter, &mut r.c1.y, &mut r.c2.y, &new_pt.y)
            }
            DragVertex::BottomCenter => {
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::TopCenter, &mut r.c2.y, &mut r.c1.y, &new_pt.y)
            }
            DragVertex::Left => {
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::Right, &mut r.c1.x, &mut r.c2.x, &new_pt.x)
            }
            DragVertex::Right => {
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::Left, &mut r.c2.x, &mut r.c1.x, &new_pt.x)
            }
            DragVertex::TopLeft => {
                let h = (new_pt.x - r.c1.x) * r.height() / r.width();
                let pt = Point{x: new_pt.x, y: r.c1.y + h};
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::BottomRight, &mut r.c1.x, &mut r.c2.x, &pt.x);
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::BottomRight, &mut r.c1.y, &mut r.c2.y, &pt.y)
            }
            DragVertex::BottomRight => {
                let h = (new_pt.x - r.c2.x) * r.height() / r.width();
                let pt = Point{x: new_pt.x, y: r.c2.y + h};
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::TopLeft, &mut r.c2.x, &mut r.c1.x, &pt.x);
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::TopLeft, &mut r.c2.y, &mut r.c1.y, &pt.y)
            }
            DragVertex::TopRight => {
                let h = (r.c2.x - new_pt.x) * r.height() / r.width();
                let pt = Point{x: new_pt.x, y: r.c1.y + h};
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::BottomLeft, &mut r.c2.x, &mut r.c1.x, &pt.x);
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::BottomLeft, &mut r.c1.y, &mut r.c2.y, &pt.y)
            }
            DragVertex::BottomLeft => {
                let h = (r.c1.x - new_pt.x) * r.height() / r.width();
                let pt = Point{x: new_pt.x, y: r.c2.y + h};
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::TopRight, &mut r.c1.x, &mut r.c2.x, &pt.x);
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::TopRight, &mut r.c2.y, &mut r.c1.y, &pt.y)
            }
        };
        self.0 = r;
        new_vtx
    }

    fn get_drag_vertex(&self, pt: &Point) -> Option<DragVertex> {
        self.get_drag_points().iter().enumerate()
            .find(|(_, p)| (p.dist(&pt) as u32) < ShapeSelectBox::MIN_CORNER_DIST)
            .map(|(i, _)| FromPrimitive::from_usize(i).unwrap())
    }

    #[inline]
    fn get_drag_points(&self) -> Vec<Point> {
        let mut points = self.0.verts();
        points.push((points[0] + points[1]) / 2.);
        points.push((points[1] + points[2]) / 2.);
        points.push((points[2] + points[3]) / 2.);
        points.push((points[3] + points[0]) / 2.);
        points
    }
    fn draw_drag_circles(&self, draw_ctx: &DrawCtx) {
        let radi = 7.;
        self.get_drag_points().iter()
            .map(|v| ShapeBuilder::new().color(255,255,255).circle(radi as u32)
                .offset((v.x - radi/2.) as i32, (v.y - radi/2.) as i32).get())
            .for_each(|s| s.draw(draw_ctx));
    } 
    fn draw(&self, draw_ctx: &DrawCtx) {
        //draw box
        self.0.builder().color(255,255,255).fill(false).get().draw(draw_ctx);
        self.draw_drag_circles(draw_ctx);
    }
    fn fuzzy_in_bounds(&self, p: &Point, vp: &Point) -> bool {
        let mut r = self.0.clone();
        let padding = ShapeSelectBox::MIN_CORNER_DIST as f32;
        r.c1 -= Point {x: padding, y: padding};
        r.c2 += Point {x: padding, y: padding};
        r.in_bounds(p, vp)
    }
}

impl InBounds for ShapeSelectBox {
    fn in_bounds(&self, p: &Point, vp: &Point) -> bool {
        self.0.in_bounds(p, vp)
    }
}

struct ShapeBar {
    shapes: HashMap<ShapeID, Shape>,
    selected: Option<ShapeID>,
    draw_rect: Rect
}

impl ShapeBar {
    fn new(viewport: &Point) -> Self {
        let draw_rect = Rect::new(
            Point { x: viewport.x / 5., y: 0. }, 
            Point { x: 4. * viewport.x / 5., y: viewport.y / 12.});
        let mut shapes = HashMap::new();
        let ptypes = [PrimType::Circle, PrimType::Triangle, PrimType::Rect];
        let shapes_rect = Rect::new( 
            draw_rect.c1 + Point {x: draw_rect.width() / 4., y: draw_rect.height() / 5.},
            draw_rect.c2 - Point {x: draw_rect.width() / 4., y: draw_rect.height() / 5. } 
        );
        let npoly = ptypes.len() as u32;
        for (i, s) in ptypes.iter().enumerate() {
            let mut poly = DrawPolygon::from_prim(*s);
            poly.rect.size = Point::new(30.,30.);
            poly.rect.set_center(&Point {
                x: shapes_rect.c1.x +
                    (i as u32 * shapes_rect.width() as u32 / (npoly - 1)) as f32,
                y: shapes_rect.center().y
            });
            let mut shape = Shape::from_props(ShapeProps::Polygon(poly));
            shape.color = rgb_to_f32(255,0,0);
            shapes.insert(i as u32, shape);
        }
        ShapeBar {
            shapes,
            draw_rect,
            selected: None
        }
    }
    fn get_shape(&self, id: ShapeID, r: &Rect, fill: bool) -> Shape {
        const DEFAULT_SIZE: f32 = 30.;
        let empty = r.c1 == r.c2;
        let size = 
            if empty { Point::new(DEFAULT_SIZE, DEFAULT_SIZE) }
            else { Point::new(r.width(), r.height()) };
        let offset = r.c1;
        let mut rect = RotateRect::new(offset, size, 0.);
        if empty {
            rect.set_center(&offset);
        }
        let mut props = self.shapes[&id].clone().props; 
        if let ShapeProps::Polygon(ref poly) = props {
            let prim = if !fill && poly.prim == PrimType::Circle { PrimType::Ring } else { poly.prim };
            props = ShapeProps::Polygon(
                DrawPolygon { rect, fill, prim, ..DrawPolygon::default()});
        }
        let mut s = Shape::from_props(props);
        s.color = rgb_to_f32(255, 0, 0);
        s
    }
    fn click_shape(&mut self, p: &Point, vp: &Point) -> Option<ShapeID> {
        self.shapes.iter().find(|(_, s)| s.in_bounds(p, vp)).map(|(id, _)| *id)
    }
    fn draw(&self, draw_ctx: &DrawCtx) {
        self.draw_rect.builder().color(120,50,200).get().draw(draw_ctx);
        self.shapes.values().for_each(|s| s.draw(draw_ctx));
    }
}

#[derive(Copy, Clone, PartialEq)]
enum ClickResponse {
    Clicked,
    NotClicked
}