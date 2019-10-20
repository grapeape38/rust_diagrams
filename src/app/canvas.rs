extern crate sdl2;

use sdl2::keyboard::{Keycode, Mod};
use sdl2::event::Event;
use sdl2::mouse::SystemCursor;
use crate::displaytree::*;
use crate::primitives::{Shape, Point, InBounds};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Selection {
}

impl Selection { 
    fn new() -> Self { Selection { } }
}

impl EventTreeT for Selection {
    type Parent = Canvas;
    fn try_handle_before(&mut self, ev: &Event, _: &EventCtx) -> ChildResponse<Self::Parent> {
        match *ev { 
            Event::MouseButtonDown {x, y, ..} => { 
                Some(parent_callback!(Canvas, respond, CanvasResponse::SelectionClick(x,y)))
            }
            _ => None
        }
    }
}

#[derive(Clone)]
enum CanvasResponse {
    ShapeClick,
    SelectionClick(i32, i32)
}

type ShapeID = u64;

pub type CanvasPtr =  Rc<RefCell<Canvas>>;
pub struct CanvasOwner {
    canvas: CanvasPtr,
    child_resp: Option<(ShapeID, CanvasResponse)>
}

pub struct Canvas {
    shapes: HashMap<ShapeID, Shape>,
    selection: HashMap<ShapeID, Selection>,
    child_resp: Option<CanvasResponse> 
}

impl Canvas {
    pub fn new() -> Self {
        let mut m = HashMap::new();
        m.insert(51, Selection::new());
        m.insert(52, Selection::new());
        Canvas {
            shapes: HashMap::new(),
            selection: m,
            child_resp: None 
        }
    }
}

impl CanvasOwner {
    pub fn new() -> Self {
        CanvasOwner {
            canvas: Rc::new(RefCell::new(Canvas::new())),
            child_resp: None 
        }
    }
    fn respond(&mut self, id: ShapeID, resp: CanvasResponse) {
        println!("changing something");
        self.child_resp = Some((id, resp));
    }
}

impl HandleEvent<DisplayTree> for CanvasOwner {
    fn handle_event(&mut self, ev: &Event, ctx: &EventCtx) -> ChildResponse<DisplayTree> 
    {
        let cb = self.canvas.borrow_mut().handle_event(ev, ctx);
        if let Some(cb) = cb {
            cb.0(self);
        }
        match self.child_resp {
            Some((id, CanvasResponse::ShapeClick)) => {
                ctx.event_handler.borrow_mut().add_mouse_handle_all(Box::new(ShapeDrag::new(id, &self.canvas)));
                ctx.cursors.borrow_mut().change_cursor(SystemCursor::Hand);
            },
            Some((id, CanvasResponse::SelectionClick(x, y))) => { 
                println!("Selection id: {:?} clicked at {:?} {:?}", id, x, y);
                ctx.event_handler.borrow_mut().add_mouse_handle_all(Box::new(RootCounter::new()));
                ctx.cursors.borrow_mut().change_cursor(SystemCursor::Hand);
            }
            _ => {}
        }
        self.child_resp = None; 
        Some(ParentCallback::none())
    }
}

impl EventTreeT for Canvas {
    type Parent = CanvasOwner;
    fn try_handle_before(&mut self, ev: &Event, _: &EventCtx) -> ChildResponse<Self::Parent> {
        match ev {
            Event::MouseButtonDown {..} => { None },
            Event::KeyDown {..} => { None },
            _ => Some(ParentCallback::none())
        }
    }
    fn handle_response(&mut self, id: EVId, _: &Event, _: &EventCtx) -> ChildResponse<Self::Parent> {
        let cr = self.child_resp.clone().map(|cr| 
            parent_callback!(CanvasOwner, respond, id, cr.clone())
        );
        self.child_resp = None;
        cr
    }
    fn ev_children<'a>(&'a mut self) -> EvChildIterOwner<'a, Self> {
        EvChildIterOwner::new().chain_s(self.shapes.iter_mut())
            .chain_s(self.selection.iter_mut())
    }
}

impl Canvas {
    fn respond(&mut self, resp: CanvasResponse) {
        self.child_resp = Some(resp);
    }
}

struct ShapeDrag {
    canvas: CanvasPtr,
    id: ShapeID 
}

impl ShapeDrag {
    fn new(id: ShapeID, ptr: &CanvasPtr) -> Self {
        ShapeDrag { id, canvas: Rc::clone(ptr) }
    }
}

impl MouseMotionT for ShapeDrag {
    fn handle_move(&mut self, x: i32, y: i32, ctx: &EventCtx) -> ChildResponse<EventHandler> {
        let off = Point::new(x as f32, y as f32);
        self.canvas.borrow_mut().shapes.get_mut(&self.id).map(|s| s.drag(&off));
        ctx.cursors.borrow_mut().change_cursor(SystemCursor::Hand);
        self_handled()
    }
}

impl MouseDownT for ShapeDrag {}
impl MouseUpT for ShapeDrag {}
impl MouseHandleT for ShapeDrag {}

enum CreateShapeMode {
    
}

struct CreateShape {
    mode: CreateShapeMode,
    start_pt: Point,
    last_pt: Point,
    canvas: CanvasPtr
}

impl EventTreeT for Shape {
    type Parent = Canvas;
    fn try_handle_before(&mut self, ev: &Event, ctx: &EventCtx) -> ChildResponse<Self::Parent> {
        match *ev {
            Event::MouseButtonDown {x, y, ..} => { 
                if self.in_bounds(&Point::new(x as f32, y as f32), &ctx.vp) {
                    Some(parent_callback!(Canvas, respond, CanvasResponse::ShapeClick))
                }
                else {
                    self_handled()
                }
            },
            _ => None
        }
    }
}

struct KeyboardCounter {
    counter: u32
}

impl KeyboardCounter {
    fn new() -> Self { 
        println!("In keyboard counter mode!!");
        KeyboardCounter { counter: 0} 
    }
}

impl KeyDownT for KeyboardCounter {
    fn handle(&mut self, kc: Option<Keycode>) -> ChildResponse<EventHandler> {
        match kc {
            Some(Keycode::Escape) => {
                println!("Exiting keyboard counter mode.");
                Some(parent_callback!(EventHandler, change_key_down, None))
            }
            _ => { self.counter += 1;
            println!("My counter is now: {:?}", self.counter); self_handled() }
        }
    }
}

struct RootCounter { }

impl RootCounter {
    fn new() -> Self {
        RootCounter { } 
    }
}

impl MouseDownT for RootCounter {}
impl MouseUpT for RootCounter {}
impl MouseMotionT for RootCounter {
    fn handle_move(&mut self, x: i32, y: i32, ctx: &EventCtx) -> ChildResponse<EventHandler> {
        println!("Supposedly dragging at: {:?} {:?}", x, y);
        ctx.cursors.borrow_mut().change_cursor(SystemCursor::Hand);
        self_handled()
    }
}

impl MouseHandleT for RootCounter { }
