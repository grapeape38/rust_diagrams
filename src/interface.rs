extern crate sdl2;

use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use sdl2::event::Event;
use crate::primitives::*;
use crate::ShapeProps as SP;

impl Shape {
    fn drag(&mut self, off: &Point) {
        match self.props {
            SP::Line(ref mut draw_line) => {
                draw_line.p1 += *off;
                draw_line.p2 += *off;
            }
            SP::Polygon(ref mut draw_poly) => {
                draw_poly.offset.0 += *off;
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
    /*fn drag_resize(&mut self, off: &Point) {
        self.width += off.x as u32;
        self.height += off.y as u32;
    }*/
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
    fn get_box_selection(&self, r: &Rect, vp: &Point) -> HashSet<u32> {
        self.m.iter().filter(|(_,s)| s.in_select_box(r, vp)).map(|(k, _)| *k).collect()
    }
    pub fn draw_all(&self, ctx: &DrawCtx) {
        self.draw_order.iter().for_each(|idx| {
            let s = &self.m[idx];
            s.draw(ctx);
        });
    }
    pub fn draw_select_box(&self, rect: &Rect, ctx: &DrawCtx) {
        rect.builder().fill(false).get().draw(ctx);
    }
    pub fn draw_shape_select_boxes(&self, selection: &HashSet<u32>, ctx: &DrawCtx) {
        for s in selection.iter().filter_map(|id| self.m.get(id)) {
            let rect = Rect::bounding_box(&s.verts(&ctx.viewport)).builder().color(255,255,255).fill(false).get();
            rect.draw(ctx);
        }
    }
}

pub struct AppState<'a> {
    draw_list: DrawList,
    selection: HashSet<u32>,
    drag_mode: DragMode,
    draw_ctx: DrawCtx<'a>,
}

#[derive(Clone, Copy)]
pub enum DragMode {
    DragNone,
    SelectBox {start_pt: Point, last_pt: Point},
    DragShapes {start_pt: Point, last_pt: Point, click_shape: u32},
    DragResize
}

impl<'a> AppState<'a> {
    pub fn new(draw_list: DrawList, draw_ctx: DrawCtx<'a>) -> AppState<'a> {
        AppState {
            draw_list,
            selection: HashSet::new(),
            drag_mode: DragMode::DragNone,
            draw_ctx
        }
    }
    pub fn handle_mouse_event(&mut self, ev: &Event) {
        match *ev {
            Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                if mouse_btn == sdl2::mouse::MouseButton::Left {
                    let pt = Point{x: x as f32,y: y as f32};
                    if let Some(click_shape) = self.draw_list.click_shape(&pt, &self.draw_ctx.viewport) {
                        if !self.selection.contains(&click_shape) {
                            self.selection.clear();
                            self.selection.insert(click_shape);
                        }
                        self.drag_mode = DragMode::DragShapes {start_pt: pt, last_pt: pt, click_shape };
                    }
                    else {
                        self.selection.clear();
                        self.drag_mode = DragMode::SelectBox{start_pt: pt, last_pt: pt};
                    }
                }
            } 
            Event::MouseButtonUp{mouse_btn, .. } => {
                if mouse_btn == sdl2::mouse::MouseButton::Left {
                    match self.drag_mode {
                        DragMode::DragShapes {start_pt, last_pt, click_shape } => {
                            if start_pt == last_pt {
                                self.selection.clear();
                                self.selection.insert(click_shape);
                            }
                        },
                        _ => {}
                    }
                    self.drag_mode = DragMode::DragNone;
                }
            }
            Event::MouseMotion{ x, y, ..} => {
                let pt = Point{x:x as f32, y:y as f32};
                match self.drag_mode {
                    DragMode::DragShapes {ref mut last_pt, ..} => {
                        for id in self.selection.iter() {
                            self.draw_list.get_mut(id).map(|s| s.drag(&(pt - *last_pt)));
                        }
                        *last_pt = pt;
                    }
                    DragMode::SelectBox {start_pt, ref mut last_pt} => {
                        self.selection = self.draw_list.get_box_selection(&Rect::new(start_pt, pt), &self.draw_ctx.viewport);
                        *last_pt = pt;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    pub fn render(&self) {
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT); }
        self.draw_list.draw_all(&self.draw_ctx);
        if let DragMode::SelectBox{start_pt, last_pt} = self.drag_mode {
            self.draw_list.draw_select_box(&Rect::new(start_pt, last_pt), &self.draw_ctx);
        }
        self.draw_list.draw_shape_select_boxes(&self.selection, &self.draw_ctx);
    }
}

#[derive(Copy, Clone, PartialEq)]
enum ClickResponse {
    Clicked,
    NotClicked
}

/*pub struct ShapeBar<'a> {
    shapes: DrawList<'a>
}

impl Clickable for ShapeCreator {
    fn click(&self, id: u32) -> Box<Fn(&mut DrawList) -> Option<u32>> {
        Box::new(move |dl: &mut DrawList| {
            Some(dl.clone_shape(id))
        })
    }
}

impl Draggable for ShapeCreator {}

impl DrawBounds for ShapeCreator {}

trait Creator {
    fn creator(self) -> ShapeCreator;
}

impl<T: DrawBounds + Sized + 'static> Creator for T {
    fn creator(self) -> ShapeCreator {
        ShapeCreator { s: Box::new(self) }
    }
}*/
