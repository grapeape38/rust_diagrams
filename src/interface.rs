use std::collections::HashMap;
use crate::primitives::*;

pub trait Draggable : Clickable {
    fn drag(&mut self, _: &Point) {}
}

pub trait Clickable {
    fn click(&mut self) {}
}

pub trait DrawClone {
    fn draw_clone(&self) -> Box<DrawBounds>;
}

impl<T: DrawBounds + Clone + 'static> DrawClone for T {
    fn draw_clone(&self) -> Box<DrawBounds + 'static> {
        Box::new(self.clone())
    }
}

/*impl<T: Drawable> Drawable for Box<T> { 
    fn draw(&self, ctx: &DrawCtx) {
        self.draw(ctx);
    }
    fn prim_type(&self) -> PrimType {
        self.prim_type()
    }
}

impl<T: Draggable> Clickable for Box<T> {
    fn click(&mut self) {
        self.click();
    }
}

impl<T: Draggable> Draggable for Box<T> {
    fn drag(&mut self, p: &Point) {
        self.drag(p);
    }
}

impl<T: InBounds> InBounds for Box<T> {
    fn in_bounds(&self, p: &Point, vp: &Point) -> bool {
        self.in_bounds(p, vp)
    }
}*/

/*impl <T: DrawClone + 'static> DrawClone for Box<T> {
    fn draw_clone(&self) -> Box<DrawBounds + 'static> {
        self.draw_clone();
    }
}*/

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
    pub fn click_shape(&mut self, p: &Point, vp: &Point) -> Option<u32> {
        let shape_id = self.m.iter().find(|(_,s)| s.in_bounds(p, vp)).map(|(k,_)| *k);
        if let Some(id) = shape_id {
            self.get_mut(id).unwrap().click();
            let pos = self.draw_order.iter().position(|i| *i == id);
            if let Some(pos) = pos {
                let elem = self.draw_order.remove(pos as usize);
                self.draw_order.push(elem);
            }
        }
        shape_id
    }
    pub fn draw_all(&self, ctx: &DrawCtx) {
        self.draw_order.iter().for_each(|idx| {
            let s = &self.m[idx];
            s.draw(ctx);
        });
    }
}


pub struct ShapeBar<'a> {
    shapes: DrawList<'a>
}

impl<'a> ShapeBar<'a> {
    fn new(viewport: &Point) -> ShapeBar<'a> {
        let y_inc = viewport.y as i32 / 6;
        let left_margin = viewport.x as i32 / 8;
        let square = ShapeBuilder::new().square(20).offset(left_margin, y_inc * 2).get();
        let tri = ShapeBuilder::new().tri(20).offset(left_margin, y_inc * 3).get();
        let circle = ShapeBuilder::new().circle(20).offset(left_margin, y_inc * 3).get();
        let line = LineBuilder::new().points(left_margin as f32, y_inc as f32 * 4., left_margin as f32 + 20., y_inc as f32 * 4.).get();
        let mut shapes = DrawList::new();
        shapes.add(square);
        shapes.add(tri);
        shapes.add(circle);
        shapes.add(line);
        ShapeBar { shapes }
    }
    pub fn get_draggable_id(&mut self, p: &Point, vp: &Point) -> Option<u32> {
        let shapebar_shape = self.shapes.m.iter().take(4).find(|(_,s)| s.in_bounds(p, vp)).map(|(k,_)| *k);
        if let Some(id) = shapebar_shape {
            let s = self.shapes.m[&id];
            //self.shapes.add(s.draw_clone());
        }
        None
    }
}