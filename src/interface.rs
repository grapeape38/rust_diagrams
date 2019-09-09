use std::collections::HashMap;
use crate::primitives::*;

pub trait Draggable : Clickable {
    fn drag(&mut self, _: &Point) {}
}

pub trait Clickable {
    fn click(&self, _: u32) -> Box<Fn(&mut DrawList) -> Option<u32>> {
        Box::new(|_: &mut DrawList| { None })
    }
}

pub trait DrawClone {
    fn draw_clone(&self) -> Box<DrawBounds>;
}

impl<T: DrawBounds + Clone + 'static> DrawClone for T {
    fn draw_clone(&self) -> Box<DrawBounds> {
        Box::new(self.clone())
    }
}

pub trait DrawBounds : Draggable + Drawable + InBounds + DrawClone {}

impl Clickable for Shape {}

impl Draggable for Shape {
    fn drag(&mut self, off: &Point) {
        self.offset += *off;
    }
}

impl Clickable for DrawLine {}

impl Draggable for DrawLine {
    fn drag(&mut self, off: &Point) {
        self.p1 += *off;
        self.p2 += *off;
    }
}

impl DrawBounds for Shape {}
impl DrawBounds for DrawLine {}

pub struct DrawList<'a> {
    m: HashMap<u32, Box<DrawBounds + 'a>>,
    draw_order: Vec<u32>,
    next_id: u32
}

impl<'a> DrawList<'a> {
    pub fn new() -> DrawList<'a> {
        DrawList {m: HashMap::new(), draw_order: Vec::new(), next_id: 0}
    }
    pub fn add<D: DrawBounds + 'a>(&mut self, s: D) {
        self.m.insert(self.next_id, Box::new(s));
        self.draw_order.push(self.next_id);
        self.next_id += 1;
    }
    pub fn get_mut(&mut self, id: u32) -> Option<&mut Box<DrawBounds +'a>> {
        self.m.get_mut(&id)
    }
    pub fn clone_shape(&mut self, id: u32) -> u32 {
        let clone = self.m.get(&id).map(|s| s.draw_clone());
        if let Some(c) = clone {
            self.m.insert(self.next_id, c);
            self.draw_order.push(self.next_id);
            self.next_id += 1;
        }
        self.next_id - 1
    }
    pub fn click_shape(&mut self, p: &Point, vp: &Point) -> Option<u32> {
        let shape_id = self.m.iter().find(|(_,s)| s.in_bounds(p, vp)).map(|(k,_)| *k);
        if let Some(mut id) = shape_id {
            id = self.get_mut(id).unwrap().click(id)(self).unwrap_or(id);
            let pos = self.draw_order.iter().position(|i| *i == id);
            if let Some(pos) = pos {
                let elem = self.draw_order.remove(pos as usize);
                self.draw_order.push(elem);
            }
            return Some(id);
        }
        None
    }
    pub fn draw_all(&self, ctx: &DrawCtx) {
        self.draw_order.iter().for_each(|idx| {
            let s = &self.m[idx];
            s.draw(ctx);
        });
    }
}

pub struct ShapeCreator {
    pub s: Box<DrawBounds>
}

impl DrawClone for ShapeCreator {
    fn draw_clone(&self) -> Box<DrawBounds> {
        self.s.draw_clone()
    }
}

impl Drawable for ShapeCreator {
    fn draw(&self, ctx: &DrawCtx) {
        self.s.draw(ctx);
    }
    fn prim_type(&self) -> PrimType {
        self.s.prim_type()
    }
}

impl InBounds for ShapeCreator {
    fn in_bounds(&self, p: &Point, vp: &Point) -> bool {
        self.s.in_bounds(p, vp)
    }
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
}

pub fn create_shape_bar(dl: &mut DrawList, viewport: &Point) {
    let y_inc = viewport.y as i32 / 6;
    let left_margin = viewport.x as i32 / 8;
    let square = ShapeBuilder::new().square(20).offset(left_margin, y_inc * 2).color(255, 0, 0).get().creator();
    let tri = ShapeBuilder::new().tri(20).offset(left_margin, y_inc * 3).color(0, 255, 0).get().creator();
    let circle = ShapeBuilder::new().circle(20).offset(left_margin, y_inc * 4).color(122, 15, 62).get().creator();
    let line = LineBuilder::new().points(left_margin as f32, y_inc as f32 * 5., left_margin as f32 + 20., y_inc as f32 * 5.).color(255, 255, 255).get().creator();
    dl.add(square);
    dl.add(tri);
    dl.add(circle);
    dl.add(line);
}
