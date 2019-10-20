use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::{Cursor, SystemCursor};
use crate::app::canvas::*;
use crate::primitives::Point;
use std::collections::HashMap;
use std::convert::TryInto;
use std::rc::Rc;
use std::cell::RefCell;

macro_rules! parent_callback(
    { $p_type: ty, $fn: ident $(,$args: expr)* } => {
        ParentCallback(Box::new(move |parent: &mut $p_type|
             { parent.$fn($($args),*); }
        ))
    };
);


pub struct AppState {
    pub cursor_state: CursorPtr,
    pub event_handler: EventPtr,
    pub display_tree: DisplayTree,
}

pub type CursorPtr = Rc<RefCell<CursorState>>;
pub type EventPtr = Rc<RefCell<EventHandler>>;

pub struct EventCtx { 
    pub cursors: CursorPtr,
    pub event_handler: EventPtr,
    pub vp: Point
}

impl EventCtx {
    fn new(vp: &Point, cursors: &CursorPtr, event_handler: &EventPtr) -> Self {
        EventCtx { 
            vp: *vp, 
            cursors: Rc::clone(cursors), 
            event_handler: Rc::clone(event_handler)
        }
    }
}

pub struct ParentCallback<T>(pub Box<dyn Fn(&mut T)>);

impl<T> ParentCallback<T> {
    pub fn none() -> Self { ParentCallback(Box::new(|_: &mut T| {})) }
}

pub fn self_handled<P>() -> ChildResponse<P> {
    Some(ParentCallback::none())
}

pub type ChildResponse<T> = Option<ParentCallback<T>>;

pub trait HandleEvent<P> {
    fn handle_event(&mut self, ev: &Event, ctx: &EventCtx) -> ChildResponse<P>;
}

pub type EVId = u64;

pub trait IntoEVId {
    fn into_id(self) -> EVId; 
}

impl IntoEVId for i32 { fn into_id(self) -> EVId { self.try_into().map_err(|_| "Error converting to u64").unwrap() } }
impl IntoEVId for u32 { fn into_id(self) -> EVId { self.try_into().map_err(|_| "Error converting to u64").unwrap() } }
impl IntoEVId for u64 { fn into_id(self) -> EVId { self.try_into().map_err(|_| "Error converting to u64").unwrap() } }
impl IntoEVId for usize { fn into_id(self) -> EVId { self.try_into().map_err(|_| "Error converting to u64").unwrap() } }
impl<T: IntoEVId + Copy> IntoEVId for &T { fn into_id(self) -> EVId { (*self).into_id() }}


pub type EvChildT<'a, S> = &'a mut dyn HandleEvent<S>;
pub type EvChild<'a, S, U> = (U, EvChildT<'a,S>);
pub type EvChildS<'a, C, U> = (U, &'a mut C);

pub trait IntoEvChildIter<'a, S: 'a, J: Iterator<Item=EvChild<'a,S,U>>, U> : IntoIterator<Item=EvChild<'a,S,U>, IntoIter=J> { }

impl<'a, S: 'a, I, J, U> IntoEvChildIter<'a, S, J, U> for I 
   where I : IntoIterator<Item=EvChild<'a,S,U>, IntoIter=J>, J: Iterator<Item=EvChild<'a,S,U>> {}

pub trait IntoEvChildIterStatic<'a, S: 'a, C: 'a, J : Iterator<Item=EvChildS<'a, C,U>>, U> : 
    IntoIterator<Item=EvChildS<'a,C,U>, IntoIter=J> {}

impl<'a, S: 'a, C: 'a, I, J, U> IntoEvChildIterStatic<'a, S, C, J, U> for I 
   where I : IntoIterator<Item=EvChildS<'a, C, U>, IntoIter=J>, J: Iterator<Item=EvChildS<'a, C, U>> {}

pub type EvChildIterMut<'a, S> = Box<dyn Iterator<Item=EvChild<'a, S, EVId>> + 'a>;

pub struct EvChildIterOwner<'a, S>(EvChildIterMut<'a, S>);

#[allow(dead_code)]
impl<'a, S: 'a> EvChildIterOwner<'a, S> {
    pub fn new() -> Self where 
    {
        let v: Vec<EvChild<'a, S, EVId>> = Vec::new();
        EvChildIterOwner(Box::new(v.into_iter()))
    }
    pub fn one<U, C>(self, id: U, c: &'a mut C) -> Self where
        C: HandleEvent<S> + 'a,
        U: IntoEVId
    {
        let v: Vec<EvChild<'a, S, EVId>> = vec![(id.into_id(), c as EvChildT<'a, S>)];
        EvChildIterOwner(Box::new(self.0.chain(v.into_iter())))
    }
    pub fn chain<I, J, U>(self, it: I) -> Self where 
        I : IntoEvChildIter<'a, S, J, U>, 
        J: Iterator<Item=EvChild<'a, S, U>> + 'a,
        U: IntoEVId 
    {
        EvChildIterOwner(Box::new(self.0.chain(it.into_iter().map(|c| (c.0.into_id(), c.1)))))
    }
    pub fn chain_s<I, J, C: HandleEvent<S> + 'a, U>(self, it: I) -> Self where 
        I : IntoEvChildIterStatic<'a, S, C, J, U>,
        J: Iterator<Item=EvChildS<'a, C, U>> + 'a,
        U: IntoEVId 
    {
        EvChildIterOwner(Box::new(self.0.chain(it.into_iter().map(|c| (c.0.into_id(), c.1 as EvChildT<'a, S>)))))
    }
}

pub trait EventTreeT where Self: Sized {
    type Parent;
    fn try_handle_before(&mut self, _: &Event, _: &EventCtx) -> ChildResponse<Self::Parent> {
       None 
    }
    fn handle_response(&mut self, _: EVId, _: &Event, _: &EventCtx) 
        -> ChildResponse<Self::Parent> 
    {
        None
    }
    fn handle_none(&mut self, _: &Event, _: &EventCtx) -> ChildResponse<Self::Parent>
    {
        None
    }
    fn ev_children<'a>(&'a mut self) -> EvChildIterOwner<'a, Self> {
       EvChildIterOwner::new()
    }
}

impl<D, P> HandleEvent<P> for D where D: EventTreeT<Parent=P> {
    fn handle_event(&mut self, ev: &Event, ctx: &EventCtx) -> ChildResponse<P> {
        if let Some(handled) = self.try_handle_before(ev, ctx) {
            return Some(handled);
        }
        let resp = self.ev_children().0.into_iter()
            .map(|c| (c.0, c.1.handle_event(ev, ctx)))
            .find(|(_, resp)| resp.is_some());

        resp.and_then(|(id, cr)| {
            cr.map(|parent_cb| { parent_cb.0(self); }); 
            self.handle_response(id, ev, ctx)
        }).or_else(|| self.handle_none(ev, ctx))
    }
}

pub trait MouseDownT {
    fn handle_click(&mut self, _: i32, _: i32, _: &EventCtx) -> ChildResponse<EventHandler> {
        None
    }
}

pub trait MouseUpT {
    fn handle_up(&mut self, _: i32, _: i32, ctx: &EventCtx) -> ChildResponse<EventHandler> {
        ctx.cursors.borrow_mut().change_cursor(SystemCursor::Arrow);
        Some(parent_callback!(EventHandler, change_mouse_handle, None))
    }
}

pub trait MouseMotionT {
    fn handle_move(&mut self, _: i32, _: i32, _: &EventCtx) -> ChildResponse<EventHandler> {
        None
    }
}

pub trait MouseHandleT : MouseDownT + MouseUpT + MouseMotionT {}

pub enum EMouseHandle {
    HandleAll(Box<dyn MouseHandleT>),
    HandleDown(Box<dyn MouseDownT>),
    HandleMove(Box<dyn MouseMotionT>),
    HandleUp(Box<dyn MouseUpT>)
}

pub struct MouseHandle {
    handle: EMouseHandle,
}

impl HandleEvent<EventHandler> for MouseHandle {
    fn handle_event(&mut self, ev: &Event, ctx: &EventCtx) -> ChildResponse<EventHandler> {
        match ev.clone() {
            Event::MouseButtonDown {x, y, ..} => 
                match self.handle {
                    EMouseHandle::HandleAll(ref mut mh) => mh.handle_click(x,y,ctx),
                    EMouseHandle::HandleDown(ref mut md) => md.handle_click(x,y,ctx),
                    _ => None 
                }
            Event::MouseMotion{x, y, ..} =>
                match self.handle {
                    EMouseHandle::HandleAll(ref mut mh) => mh.handle_move(x,y, ctx),
                    EMouseHandle::HandleMove(ref mut mm) => mm.handle_move(x,y, ctx),
                    _ => None 
                }
            Event::MouseButtonUp{x, y, ..} => 
                match self.handle {
                    EMouseHandle::HandleAll(ref mut mh) => mh.handle_up(x,y, ctx),
                    EMouseHandle::HandleUp(ref mut mu) => mu.handle_up(x,y, ctx),
                    _ => None 
                }
            _ => None 
        }
    }
}

pub trait KeyDownT {
    fn handle(&mut self, _: Option<Keycode>) -> ChildResponse<EventHandler> {
        None
    }
}

pub struct EventHandler {
    mouse_handle: Option<MouseHandle>,
    key_down: Option<Box<dyn KeyDownT>>,
}

impl EventHandler {
    pub fn new() -> Self {
        EventHandler { 
            mouse_handle: None,
            key_down: None, 
        }
    }
    pub fn change_mouse_handle(&mut self, mh: Option<EMouseHandle>) {
        self.mouse_handle = mh.map(|mh| MouseHandle { handle: mh });
    }
    pub fn add_mouse_handle_all(&mut self, mh: Box<dyn MouseHandleT>) {
        self.mouse_handle = Some(MouseHandle { handle: EMouseHandle::HandleAll(mh) });
    }
    pub fn change_key_down(&mut self, kd: Option<Box<dyn KeyDownT>>) {
        self.key_down = kd;
    }
    pub fn add_key_down(&mut self, kd: Box<dyn KeyDownT>) {
        self.key_down = Some(kd);
    }
}

impl EventTreeT for EventHandler {
    type Parent = AppState;
    fn try_handle_before(&mut self, ev: &Event, ctx: &EventCtx) -> ChildResponse<Self::Parent> {
        let resp = match ev {
            ev @ Event::MouseButtonDown {..} |
            ev @ Event::MouseMotion {..} |
            ev @ Event::MouseButtonUp {..} => { 
                self.mouse_handle.as_mut().and_then(|mh| mh.handle_event(ev, ctx))
            },
            Event::KeyDown {keycode, ..} => self.key_down.as_mut().and_then(|m| m.handle(*keycode)),
            _ => None
        };
        resp.map(|pcb| {
            pcb.0(self);
            ParentCallback::none()
        })
    }
}

pub struct CursorState {
    m: HashMap<SystemCursor, Cursor>,
    cursor_changed: bool
}

impl CursorState {
    pub fn new() -> Self {
        let mut m = HashMap::new();
        m.insert(SystemCursor::Arrow, Cursor::from_system(SystemCursor::Arrow).unwrap());
        m.insert(SystemCursor::Hand, Cursor::from_system(SystemCursor::Hand).unwrap());
        m.insert(SystemCursor::Crosshair, Cursor::from_system(SystemCursor::Crosshair).unwrap());
        m.insert(SystemCursor::SizeNESW, Cursor::from_system(SystemCursor::SizeNESW).unwrap());
        m.insert(SystemCursor::SizeNS, Cursor::from_system(SystemCursor::SizeNS).unwrap());
        m.insert(SystemCursor::SizeNWSE, Cursor::from_system(SystemCursor::SizeNWSE).unwrap());
        m.insert(SystemCursor::SizeWE, Cursor::from_system(SystemCursor::SizeWE).unwrap());
        m.insert(SystemCursor::IBeam, Cursor::from_system(SystemCursor::IBeam).unwrap());
        CursorState {m, cursor_changed: false}
    }
    fn get(&self, cursor: &SystemCursor) -> &Cursor {
        &self.m[cursor]
    }
    pub fn change_cursor(&mut self, cur: SystemCursor) {
        self.get(&cur).set();
        self.cursor_changed = true;
    }
}

impl HandleEvent<AppState> for CursorState {
    fn handle_event(&mut self, ev: &Event, _: &EventCtx) -> ChildResponse<AppState> {
        if let Event::MouseMotion {..} = ev {
            if !self.cursor_changed {
                self.get(&SystemCursor::Arrow).set();
            }
            else { self.cursor_changed = false; }
        }
        Some(ParentCallback::none())
    }
}

pub struct DisplayTree {
    canvas: CanvasOwner,
}

impl EventTreeT for DisplayTree { 
    type Parent = AppState;
    fn ev_children<'a>(&'a mut self) -> EvChildIterOwner<'a, Self> {
        EvChildIterOwner::new().one(0, &mut self.canvas)
    }
}

impl DisplayTree {
    pub fn new() -> Self {
        DisplayTree {
            canvas: CanvasOwner::new(),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        AppState { 
            cursor_state: Rc::new(RefCell::new(CursorState::new())),
            event_handler: Rc::new(RefCell::new(EventHandler::new())),
            display_tree: DisplayTree::new()
        }
    }
    pub fn handle_event(&mut self, ev: &Event) {
        //tmp
        let vp = Point::new(1000., 600.);
        let ctx = EventCtx::new(&vp, &self.cursor_state, &self.event_handler);
        let cb = self.event_handler.borrow_mut().handle_event(ev, &ctx);
        if let Some(cb) = cb {
            cb.0(self);
        }
        else if let Some(callback) = self.display_tree.handle_event(ev, &ctx) {
            callback.0(self);
        }
        else {
            self.cursor_state.borrow_mut().handle_event(ev, &ctx);
        }
    }
}

