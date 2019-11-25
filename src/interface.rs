extern crate sdl2;

use num_derive::FromPrimitive;    
use num_traits::FromPrimitive;
use std::collections::{HashMap};
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Mod};
use sdl2::mouse::{Cursor, SystemCursor};
use std::iter::FromIterator;
use std::time::SystemTime;
use crate::primitives::*;
//use crate::primitives::ShapeProps as Shape;
use crate::textedit::{TextBox, get_char_from_keycode, get_dir_from_keycode};
use crate::render_text::{TextParams};
use crate::hexcolor::HexColor;
use crate::button::Button;

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
        m.insert(SystemCursor::IBeam, Cursor::from_system(SystemCursor::IBeam).unwrap());
        CursorMap(m)
    }
    fn get(&self, cursor: &SystemCursor) -> &Cursor {
        &self.0[cursor]
    }
}

#[allow(dead_code)]
impl Shape {
    pub fn drag(&mut self, off: &Point) {
        match self {
            Shape::Line(ref mut draw_line) => {
                draw_line.p1 += *off;
                draw_line.p2 += *off;
            }
            Shape::Polygon(ref mut draw_poly) => {
                draw_poly.rect.drag(off);
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
        match self {
            Shape::Polygon(ref mut draw_poly) => {
                draw_poly.rect = r.clone();
            }
            Shape::Line(_) => { }
        }
    }
    fn drag_vertex(&mut self, v: &LineVertex, pt: &Point) {
        match self {
            Shape::Line(ref mut draw_line) => {
                draw_line.drag_vertex(v, pt);
            }
            Shape::Polygon(_) => { }
        }
    }
}

impl DrawLine {
    fn drag_vertex(&mut self, v: &LineVertex, pt: &Point) {
        match v {
            LineVertex::P1 => self.p1 = *pt,
            LineVertex::P2 => self.p2 = *pt,
        };
    }
}

pub struct DrawList {
    m: HashMap<ShapeID, Shape>,
    draw_order: Vec<ShapeID>,
    next_id: ShapeID 
}

impl DrawList {
    pub fn new() -> DrawList {
        DrawList {m: HashMap::new(), draw_order: Vec::new(), next_id: 0}
    }
    pub fn add(&mut self, s: Shape) -> ShapeID {
        self.m.insert(self.next_id, s);
        self.draw_order.push(self.next_id);
        self.next_id += 1;
        self.next_id - 1
    }
    fn get(&self, id: &u32) -> Option<&Shape> {
        self.m.get(id)
    }
    fn get_mut(&mut self, id: &ShapeID) -> Option<&mut Shape> {
        self.m.get_mut(id)
    }
    fn remove(&mut self, id: &ShapeID) {
        if self.m.remove(id).is_some() {
            self.draw_order.iter().position(|idx| *idx == *id)
                .map(|idx| self.draw_order.remove(idx));
        }
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
    fn get_box_selection(&self, r: &Rect, vp: &Point) 
        -> (Vec<(ShapeID, &DrawPolygon)>, Vec<(ShapeID, &DrawLine)>)
    {
        let mut v0 = Vec::new();
        let mut v1 = Vec::new();
        self.m.iter().filter(|(_,s)| s.in_select_box(r, vp)).for_each(|(id, s)| 
            match s {
                Shape::Polygon(ref draw_poly) => v0.push((*id, draw_poly)),
                Shape::Line(ref draw_line) => v1.push((*id, draw_line)) 
            });
        (v0, v1)
    }
    pub fn draw(&self, ctx: &DrawCtx) {
        self.draw_order.iter().for_each(|idx| {
            let s = &self.m[idx];
            s.draw(ctx);
        });
    }
}

type ShapeID = u32;

pub struct AppState {
    draw_list: DrawList,
    selection: HashMap<ShapeID, ShapeSelectBox>,
    line_select: HashMap<ShapeID, SelectLine>,
    text_boxes: HashMap<ShapeID, TextBox>,
    interface: Interface,
    drag_mode: DragMode,
    key_mode: KeyboardMode,
    hover_item: HoverItem,
    pub draw_ctx: DrawCtx,
    cursors: CursorMap
}

#[derive(Clone, Copy)]
pub enum DragMode {
    DragNone,
    SelectBox {start_pt: Point, last_pt: Point},
    CreateShape { shape_id: ShapeBarShape, start_pt: Point, last_pt: Point },
    DragShapes { last_pt: Point, click_shape: ShapeID, clear_select: bool },
    DragResize { click_box: ShapeID, drag_vertex: DragVertex },
    DragRotate { click_box: ShapeID, last_angle: Radians },
    DragLineVertex { shape_id: ShapeID, line_vertex: LineVertex }
}

#[derive(Clone, Copy)]
pub enum KeyboardMode {
    KeyboardNone,
    TextEdit(ShapeID, SystemTime),
}

#[derive(PartialEq, Clone)]
pub enum HoverItem {
   HoverNone,
   HoverVertex(ShapeID, DragVertex),
   HoverLineVertex(ShapeID, LineVertex),
   HoverRotate(ShapeID),
   HoverShape(ShapeBarShape, Shape),
   HoverText(ShapeID, usize),
   HoverRect(ShapeID),
   HoverLine(ShapeID),
   HoverCreateLine {start_pt: Point, last_pt: Point, color: (u8, u8, u8)}
}

pub type CallbackFn = Box<dyn FnOnce(&mut AppState)>;

struct Interface {
    shape_bar: ShapeBar,
    tab_bar: TabBar,
}

impl Interface {
    fn new(ctx: &DrawCtx) -> Self {
        let x_offset = ctx.viewport.x / 5.;
        let tab_bar_offset = Point::new(x_offset, 0.); 
        let tab_bar = TabBar::new(tab_bar_offset);
        let shape_bar_offset = Point::new(x_offset, tab_bar_offset.y + tab_bar.measure(ctx).y);
        let shape_bar = ShapeBar::new(shape_bar_offset, &ctx.viewport);
        Interface { shape_bar, tab_bar }
    }
    fn click(&mut self, pt: &Point, cursor: &mut SystemCursor, vp: &Point) -> Option<CallbackFn> {
         self.shape_bar.click(pt, cursor, vp) 
    }
    fn draw(&self, draw_ctx: &DrawCtx) {
        self.tab_bar.draw(draw_ctx);
        self.shape_bar.draw(draw_ctx);
    }
}

impl AppState {
    pub fn new(viewport: &Point) -> AppState {
        let draw_ctx = DrawCtx::new(viewport);
        let interface = Interface::new(&draw_ctx);
        AppState {
            draw_list: DrawList::new(),
            draw_ctx,
            interface,
            selection: HashMap::new(),
            line_select: HashMap::new(),
            drag_mode: DragMode::DragNone,
            hover_item: HoverItem::HoverNone,
            key_mode: KeyboardMode::KeyboardNone,
            text_boxes: HashMap::new(),
            cursors: CursorMap::new()
        }
    }
    fn get_shape_select_box(&self, s: &DrawPolygon) -> ShapeSelectBox {
        ShapeSelectBox(s.rect.clone())
    }
    fn is_hover_text(&self, p: &Point, vp: &Point) -> Option<(ShapeID, usize)> {
        self.text_boxes.iter().find(|(id, _)| self.draw_list.get(id).unwrap().in_bounds(p, vp))
            .map(|(id, tb)| (id, tb, self.draw_list.get(id).unwrap().rect()))
            .and_then(|(id, tb, rect)| tb.hover_text(p, &rect, &self.draw_ctx.render_text, vp).map(|pos| (*id, pos)))
    }
    fn is_hover_select_box(&self, p: &Point, vp: &Point) -> Option<(ShapeID, BoxHover)> {
        self.selection.iter().filter_map(|(id, sb)| sb.get_hover(p, vp).map(|lh| (*id, lh))).nth(0)
    }
    fn is_hover_line(&self, p: &Point, vp: &Point) -> Option<(ShapeID, LineHover)> {
        self.line_select.iter().filter_map(|(id, l)| l.get_hover(p, vp).map(|lh| (*id, lh))).nth(0)
    }
    pub fn handle_hover_click(&mut self, pt: &Point, clear_select: bool, cursor: &mut SystemCursor) {
        match self.hover_item {
            HoverItem::HoverRect(select_id) => {
                if self.selection[&select_id].in_bounds(pt, &self.draw_ctx.viewport) {
                    self.drag_mode = DragMode::DragShapes { last_pt: *pt, click_shape: select_id, clear_select };
                    *cursor = SystemCursor::Hand;
                }
            }
            HoverItem::HoverLine(select_id) => {
                if self.line_select[&select_id].in_bounds(pt, &self.draw_ctx.viewport) {
                    self.drag_mode = DragMode::DragShapes { last_pt: *pt, click_shape: select_id, clear_select };
                    *cursor = SystemCursor::Hand;
                }
            }
            HoverItem::HoverRotate(select_id) => {
                let last_angle = self.selection[&select_id].get_rotate_angle(pt, &self.draw_ctx.viewport);
                self.drag_mode = DragMode::DragRotate { click_box: select_id, last_angle };
                    *cursor = SystemCursor::Hand;
            }
            HoverItem::HoverVertex(select_id, drag_vertex) => {
                self.drag_mode = DragMode::DragResize { click_box: select_id, drag_vertex };
                *cursor = get_drag_hover_cursor(&drag_vertex);
            }
            HoverItem::HoverShape(shape_id, ref shape) => {
                match shape_id {
                    ShapeBarShape::Line => {
                        self.hover_item = HoverItem::HoverCreateLine { start_pt: *pt, last_pt: *pt, color: shape.rgb() }
                    }
                    _ => {
                        self.drag_mode = DragMode::CreateShape {shape_id, start_pt: *pt, last_pt: *pt};
                        self.hover_item = HoverItem::HoverNone;
                    }
                }
                *cursor = SystemCursor::Crosshair;
            }
            HoverItem::HoverText(tb_id, cursor_pos) => {
                self.text_boxes.get_mut(&tb_id).map(|tb| tb.set_cursor_pos(cursor_pos));
                self.key_mode = KeyboardMode::TextEdit(tb_id, SystemTime::now());
                *cursor = SystemCursor::IBeam;
            }
            HoverItem::HoverLineVertex(shape_id, line_vertex) => {
                self.drag_mode = DragMode::DragLineVertex { shape_id, line_vertex };
                *cursor = SystemCursor::Hand;
            }
            HoverItem::HoverCreateLine { start_pt, last_pt, color } => {
                let id = self.draw_list.add(LineBuilder::new().points2(&start_pt, &last_pt).color(color.0, color.1, color.2).get());
                self.line_select.insert(id, SelectLine::new(start_pt, last_pt));
                self.hover_item = HoverItem::HoverNone;
            }
            HoverItem::HoverNone => {}
        }
    }
    pub fn handle_select(&mut self, pt: &Point, clear_select: bool, cursor: &mut SystemCursor) {
        if clear_select {
            self.clear_selection();
        }
        if let Some(cb) = self.interface.click(pt, cursor, &self.draw_ctx.viewport) {
            (cb)(self);
        }
        else if let Some(click_shape) = self.draw_list.click_shape(&pt, &self.draw_ctx.viewport) {
            let s = self.draw_list.get(&click_shape).unwrap();
            match s {
                Shape::Polygon(ref draw_poly) => {
                    self.selection.insert(click_shape, self.get_shape_select_box(draw_poly));
                }
                Shape::Line(ref draw_line) => {
                    self.line_select.insert(click_shape, SelectLine(draw_line.clone()));
                }
            };
            self.drag_mode = DragMode::DragShapes { last_pt: *pt, click_shape, clear_select };
            //self.hover_item = HoverItem::HoverRect(click_shape);
            *cursor = SystemCursor::Hand;
        }
        else if clear_select {
            self.drag_mode = DragMode::SelectBox{start_pt: *pt, last_pt: *pt};
        }
    }
    fn handle_drag(&mut self, pt: &Point, cursor: &mut SystemCursor) {
        let vp = &self.draw_ctx.viewport;
        match self.drag_mode {
            DragMode::DragShapes { ref mut last_pt, ref mut clear_select, .. } => {
                *clear_select = false;
                *cursor = SystemCursor::Hand;
                for (id, rect) in self.selection.iter_mut() {
                    self.draw_list.get_mut(id).map(|s| s.drag(&(*pt - *last_pt)));
                    rect.drag(&(*pt - *last_pt));
                }
                for (id, line) in self.line_select.iter_mut() {
                    self.draw_list.get_mut(id).map(|s| s.drag(&(*pt - *last_pt)));
                    line.drag(&(*pt - *last_pt));
                }
                *last_pt = *pt;
            }
            DragMode::SelectBox {start_pt, ref mut last_pt} => {
                *last_pt = *pt;
                let (shapes, lines)= self.draw_list.get_box_selection(&Rect::new(start_pt, *pt), vp);
                self.selection = HashMap::from_iter(shapes.iter().map(|(id, shape)|
                            (*id, self.get_shape_select_box(shape))
                    ));
                self.line_select = HashMap::from_iter(lines.into_iter().map(|(id, line)| 
                        (id, SelectLine(line.clone()))
                    ));
            }
            DragMode::DragRotate { click_box, ref mut last_angle } => {
                *cursor = SystemCursor::Hand;
                if let Some(sbox) = self.selection.get_mut(&click_box) {
                    let angle = sbox.get_rotate_angle(pt, vp);
                    sbox.0.set_radians(sbox.0.rot + angle - *last_angle);
                    self.draw_list.get_mut(&click_box).map(|s| s.set_rect(&sbox.0.clone()));
                    *last_angle = angle;
                }
            }
            DragMode::DragResize { click_box, ref mut drag_vertex } => {
                *cursor = get_drag_hover_cursor(&drag_vertex);
                if let Some(sbox) = self.selection.get_mut(&click_box) {
                    *drag_vertex = sbox.drag_side(&drag_vertex, &pt, vp);
                    self.draw_list.get_mut(&click_box).map(|s| s.set_rect(&sbox.0.clone()));
               }
               if let Some(tbox) = self.text_boxes.get_mut(&click_box) {
                   let rect = self.draw_list.get(&click_box).unwrap().rect();
                   tbox.format_text(&rect, 0, &self.draw_ctx.render_text);
               }
            }
            DragMode::DragLineVertex { shape_id, line_vertex } => {
                if let Some(sline) = self.line_select.get_mut(&shape_id) {
                  sline.drag_vertex(&line_vertex, &pt);  
                  self.draw_list.get_mut(&shape_id).map(|s| s.drag_vertex(&line_vertex, &pt));
                }
            }
            DragMode::CreateShape { ref mut last_pt, .. } => {
                *last_pt = *pt;
                *cursor = SystemCursor::Crosshair;
            }
            DragMode::DragNone => {}
        }
    }
    fn handle_hover(&mut self, pt: &Point, cursor: &mut SystemCursor) {
        let vp = &self.draw_ctx.viewport;
        if let HoverItem::HoverShape(_, ref mut s) = self.hover_item {
            *cursor = SystemCursor::Crosshair;
            match s {
                Shape::Polygon(ref mut poly) => poly.rect.set_center(pt),
                Shape::Line(ref mut draw_line) => {
                    let off = *pt - (draw_line.p1 + draw_line.p2) / 2.;
                    draw_line.p1 += off;
                    draw_line.p2 += off;
                }
            }
        }
        else if let HoverItem::HoverCreateLine { ref mut last_pt, .. } = self.hover_item {
            *last_pt = *pt;
            *cursor = SystemCursor::Crosshair;
        }
        else if let Some((select_id, box_hover)) = self.is_hover_select_box(&pt, vp) {
            match box_hover {
                BoxHover::Rect => { 
                    self.hover_item = HoverItem::HoverRect(select_id);
                    *cursor = SystemCursor::Hand;
                },
                BoxHover::RotateVert => { 
                    self.hover_item = HoverItem::HoverRotate(select_id);
                    *cursor = SystemCursor::Hand;
                },
                BoxHover::Drag(drag_vertex) => {
                    self.hover_item = HoverItem::HoverVertex(select_id, drag_vertex);
                    *cursor = get_drag_hover_cursor(&drag_vertex);
                }
            };
        }
        else if let Some((line_id, line_hover)) = self.is_hover_line(&pt, vp) {
            match line_hover {
                LineHover::Line => self.hover_item = HoverItem::HoverLine(line_id),
                LineHover::Vertex(line_vertex) => self.hover_item = HoverItem::HoverLineVertex(line_id, line_vertex)
            };
            *cursor = SystemCursor::Hand;
        }
        else if let Some((tb_id, cursor_pos)) = self.is_hover_text(pt, vp) {
            *cursor = SystemCursor::IBeam;
            self.hover_item = HoverItem::HoverText(tb_id, cursor_pos);
        }
        else {
            self.hover_item = HoverItem::HoverNone;
        }
    }
    fn clear_selection(&mut self) {
        self.selection.clear();
        self.line_select.clear();
        self.key_mode = KeyboardMode::KeyboardNone;
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
                                if let Some(s) = self.selection.get(&click_shape) {
                                    let s = s.clone();
                                    self.selection.clear();
                                    self.line_select.clear();
                                    self.selection.insert(click_shape, s);
                                }
                                else {
                                    let s = self.line_select[&click_shape].clone();
                                    self.selection.clear();
                                    self.line_select.clear();
                                    self.line_select.insert(click_shape, s.clone());
                                }
                            }
                        },
                        DragMode::CreateShape { shape_id, start_pt, last_pt } => {
                            let r = Rect::new(start_pt, last_pt);
                            let fill = shape_id != ShapeBarShape::TextBox;
                            let s = self.interface.shape_bar.get_shape(
                                shape_id, &r, fill);
                            let id = self.draw_list.add(s);
                            if let ShapeBarShape::TextBox = shape_id {
                                self.text_boxes.insert(id, TextBox::new());
                                self.key_mode = KeyboardMode::TextEdit(id, SystemTime::now());
                            }
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
    pub fn handle_keyboard_event(&mut self, ev: &Event) {
        if let KeyboardMode::TextEdit(shape_id, _) = self.key_mode {
            if let Event::KeyDown { keycode: Some(keycode), .. } = *ev {
                if let Some(ch) = get_char_from_keycode(keycode) {
                    let rect = self.draw_list.get(&shape_id).unwrap().rect();
                    self.text_boxes.get_mut(&shape_id).unwrap().insert_char(ch, &rect, &self.draw_ctx.render_text);
                }
                else if let Some(dir) = get_dir_from_keycode(keycode) {
                    self.text_boxes.get_mut(&shape_id).unwrap().move_cursor(dir);
                }
                else if keycode == Keycode::Backspace {
                    let rect = self.draw_list.get(&shape_id).unwrap().rect();
                    self.text_boxes.get_mut(&shape_id).unwrap().delete_char(&rect, &self.draw_ctx.render_text);
                }
            }
        }
        match *ev {
            Event::KeyDown { keycode: Some(Keycode::Delete), .. } => self.delete_selection(),
            _ => {}
        }
    }
    fn delete_selection(&mut self) {
        for id in self.selection.keys() {
            self.draw_list.remove(id); 
            self.text_boxes.remove(id);
        }
        for id in self.line_select.keys() {
            self.draw_list.remove(id); 
        }
        self.line_select.clear();
        self.selection.clear();
    }
    fn draw_hover_item(&self) {
        match self.hover_item {
            HoverItem::HoverShape(_, ref shape) => {
                shape.draw(&self.draw_ctx);
            }
            HoverItem::HoverCreateLine{start_pt, last_pt, color} => {
                self.draw_ctx.draw_line(start_pt, last_pt, rgb_to_f32(color.0, color.1, color.2), 3.);
            }
            _ => {}
        };
    }
    fn draw_drag_item(&self) {
        match self.drag_mode {
            DragMode::SelectBox{start_pt, last_pt} => {
                self.draw_ctx.draw_rect(Rect::new(start_pt, last_pt), rgb_to_f32(0, 0, 0), false, Radians(0.));
            }
            DragMode::CreateShape{shape_id, start_pt, last_pt} => {
                let r = Rect::new(start_pt, last_pt);
                self.interface.shape_bar.get_shape(shape_id, &r, false).draw(&self.draw_ctx);
            }
            _ => {}
        }
    }
    fn draw_shape_select_boxes(&self) {
        for r in self.selection.values() {
            r.draw(&self.draw_ctx);
        }
        for l in self.line_select.values() {
            l.draw(&self.draw_ctx);
        }
    }
    fn draw_text_boxes(&self) {
        for (id, tb) in self.text_boxes.iter() {
            let select_time = match self.key_mode {
                KeyboardMode::TextEdit(edit_id, select_time) => {
                    if edit_id == *id { Some(select_time) } else { None }
                }
                _ => None
            };
            let rect = self.draw_list.get(&id).unwrap().rect();
            tb.draw(&rect, select_time, &self.draw_ctx);
        }
    }
    pub fn render(&self) {
        self.interface.draw(&self.draw_ctx);
        self.draw_list.draw(&self.draw_ctx);
        self.draw_text_boxes();
        self.draw_hover_item();
        self.draw_drag_item();
        self.draw_shape_select_boxes();
    }
}

#[derive(Clone)]
struct SelectLine(DrawLine);


#[derive(Copy, Clone, PartialEq)]
pub enum LineVertex {
    P1, P2
}

#[derive(Copy, Clone, PartialEq)]
pub enum LineHover {
    Line,
    Vertex(LineVertex)
}

impl SelectLine {
    const MIN_VERT_DIST: f32 = 20.;
    fn new(p1: Point, p2: Point) -> Self {
        SelectLine(DrawLine { p1, p2, line_width: 3., color: Point::origin().to_vec4() })
    }
    fn drag(&mut self, off: &Point) {
        self.0.p1 += *off;
        self.0.p2 += *off;
    }
    fn get_hover(&self, pt: &Point, vp: &Point) -> Option<LineHover> {
        if pt.dist(&self.0.p1) <= SelectLine::MIN_VERT_DIST {
            Some(LineHover::Vertex(LineVertex::P1))
        }
        else if pt.dist(&self.0.p2) <= SelectLine::MIN_VERT_DIST {
            Some(LineHover::Vertex(LineVertex::P2))
        }
        else if self.in_bounds(pt, vp) {
            Some(LineHover::Line)
        }
        else { None }
    }
    fn drag_vertex(&mut self, vtx: &LineVertex, pt: &Point) {
        self.0.drag_vertex(vtx, pt);
    }
    fn draw_verts(&self, draw_ctx: &DrawCtx) {
        let radi = 7.;
        &[self.0.p1, self.0.p2].iter().
            for_each(|v| draw_ctx.draw_circle(radi, *v, rgb_to_f32(255, 255, 255), false));
    }
    fn draw(&self, draw_ctx: &DrawCtx) {
        self.draw_verts(draw_ctx);
    }
}

impl InBounds for SelectLine {
    fn in_bounds(&self, p: &Point, vp: &Point) -> bool {
        self.0.in_bounds(p, vp)
    }
}

#[derive(Clone)]
struct ShapeSelectBox(RotateRect);

pub enum BoxHover {
    RotateVert,
    Rect,
    Drag(DragVertex),
}

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
    fn drag_side(&mut self, drag_vertex: &DragVertex, new_pt: &Point, vp: &Point) -> DragVertex {
        let trans = RectTransform::new(&self.0, vp);
        let model_pt: Point = trans.pixel_to_model(new_pt).into();
        let mut r = Rect::default(); 
        let new_vtx = match *drag_vertex {
            DragVertex::TopCenter => {
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::BottomCenter, &mut r.c1.y, &mut r.c2.y, &model_pt.y)
            }
            DragVertex::BottomCenter => {
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::TopCenter, &mut r.c2.y, &mut r.c1.y, &model_pt.y)
            }
            DragVertex::Left => {
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::Right, &mut r.c1.x, &mut r.c2.x, &model_pt.x)
            }
            DragVertex::Right => {
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::Left, &mut r.c2.x, &mut r.c1.x, &model_pt.x)
            }
            DragVertex::TopLeft => {
                let h = (model_pt.x - r.c1.x) * r.height() / r.width();
                let pt = Point{x: model_pt.x, y: r.c1.y + h};
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::BottomRight, &mut r.c1.x, &mut r.c2.x, &pt.x);
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::BottomRight, &mut r.c1.y, &mut r.c2.y, &pt.y)
            }
            DragVertex::BottomRight => {
                let h = (model_pt.x - r.c2.x) * r.height() / r.width();
                let pt = Point{x: model_pt.x, y: r.c2.y + h};
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::TopLeft, &mut r.c2.x, &mut r.c1.x, &pt.x);
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::TopLeft, &mut r.c2.y, &mut r.c1.y, &pt.y)
            }
            DragVertex::TopRight => {
                let h = (r.c2.x - model_pt.x) * r.height() / r.width();
                let pt = Point{x: model_pt.x, y: r.c1.y + h};
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::BottomLeft, &mut r.c2.x, &mut r.c1.x, &pt.x);
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::BottomLeft, &mut r.c1.y, &mut r.c2.y, &pt.y)
            }
            DragVertex::BottomLeft => {
                let h = (r.c1.x - model_pt.x) * r.height() / r.width();
                let pt = Point{x: model_pt.x, y: r.c2.y + h};
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::TopRight, &mut r.c1.x, &mut r.c2.x, &pt.x);
                self.drag_side_swap_vertex(drag_vertex, &DragVertex::TopRight, &mut r.c2.y, &mut r.c1.y, &pt.y)
            }
        };
        self.0.resize(&r, vp);
        new_vtx
    }

    fn get_drag_vertex(&self, pt: &Point, vp: &Point) -> Option<DragVertex> {
        self.get_drag_points(vp).iter().enumerate()
            .find(|(_, p)| (p.dist(&pt) as u32) < ShapeSelectBox::MIN_CORNER_DIST)
            .map(|(i, _)| FromPrimitive::from_usize(i).unwrap())
    }

    #[inline]
    fn get_drag_points(&self, vp: &Point) -> Vec<Point> {
        let mut points = self.0.verts(vp);
        points.push((points[0] + points[1]) / 2.);
        points.push((points[1] + points[2]) / 2.);
        points.push((points[2] + points[3]) / 2.);
        points.push((points[3] + points[0]) / 2.);
        points
    }
    fn get_rotate_points(&self, vp: &Point) -> Vec<Point> {
        let mut r = self.0.clone();
        r.size *= Point::new(1.2,1.2);
        r.set_center(&self.0.center(vp));
        r.verts(vp)
    }
    fn is_hover_rotate(&self, pt: &Point, vp: &Point) -> bool {
        let radi = 12.;
        self.get_rotate_points(vp).iter().any(|p| p.dist(&pt) < radi)
    }
    fn get_rotate_angle(&self, pt: &Point, vp: &Point) -> Radians {
        let center = self.0.center(vp);
        let dist = *pt - center;
        let angle = dist.y.atan2(dist.x);
        Radians(angle)
    }
    fn draw_drag_circles(&self, draw_ctx: &DrawCtx) {
        let radi = 5.;
        self.get_drag_points(&draw_ctx.viewport).iter()
            .for_each(|v| draw_ctx.draw_circle(radi, *v, rgb_to_f32(255, 255, 255), true));
    } 
    fn draw_rotate_circles(&self, draw_ctx: &DrawCtx) {
        let radi = 5.;
        self.get_rotate_points(&draw_ctx.viewport).iter()
            .for_each(|v| draw_ctx.draw_circle(radi, *v, rgb_to_f32(0, 0, 255), false));
    }
    fn draw(&self, draw_ctx: &DrawCtx) {
        //draw box
        self.0.builder().color(255,255,255).fill(false).get().draw(draw_ctx);
        self.draw_drag_circles(draw_ctx);
        self.draw_rotate_circles(draw_ctx);
    }
    fn get_hover(&self, p: &Point, vp: &Point) -> Option<BoxHover> {
        if self.0.in_bounds(p,vp) {
            Some(BoxHover::Rect)
        }
        else if let Some(v) = self.get_drag_vertex(p, vp) {
            Some(BoxHover::Drag(v))
        }
        else if self.is_hover_rotate(p, vp) {
            Some(BoxHover::RotateVert)
        }
        else { None }
    }
}

impl InBounds for ShapeSelectBox {
    fn in_bounds(&self, p: &Point, vp: &Point) -> bool {
        self.0.in_bounds(p, vp)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ShapeBarShape {
    Circle = 0,
    Triangle = 1,
    Rect = 2,
    TextBox = 3,
    Line = 4,
    ColorPicker = 5,
}

pub enum ShapeBarItem {
    Shape(Shape),
    ColorPicker(HexColor)
}

impl ShapeBarItem {
    fn draw(&self, draw_ctx: &DrawCtx) {
        match self {
            ShapeBarItem::Shape(shape) => shape.draw(draw_ctx),
            ShapeBarItem::ColorPicker(hexcolor) => hexcolor.draw(draw_ctx)
        };
    }
}

impl ShapeBarShape {
    const DEFAULT_SIZE: f32 = 30.;
    const DEFAULT_COLOR: (u8, u8, u8) = (255, 0, 0);
    fn prim_type(&self) -> PrimType {
        match self {
            ShapeBarShape::Circle => PrimType::Circle,
            ShapeBarShape::Triangle => PrimType::Triangle,
            ShapeBarShape::Rect | ShapeBarShape::TextBox => PrimType::Rect,
            ShapeBarShape::Line => PrimType::Line,
            _ => PrimType::Rect
        }
    }
    fn get_item(&self, r: &Rect, fill: bool) -> ShapeBarItem {
        let color = 
            match self {
                ShapeBarShape::TextBox => (255, 255, 255),
                _ => ShapeBarShape::DEFAULT_COLOR
            };
        match self {
            ShapeBarShape::Line => {
                ShapeBarItem::Shape(
                    Shape::Line(DrawLine { p1: r.left_center(), p2: r.right_center(), line_width: 3., 
                        color: rgb_to_f32(color.0, color.1, color.2)}))
            },
            ShapeBarShape::ColorPicker => {
                ShapeBarItem::ColorPicker(HexColor::new(RotateRect::new(r.c1, r.size(), Radians(0.))))
            },
            _ => {
                let rect = RotateRect::new(r.c1, r.size(), Radians(0.));
                let ptype = self.prim_type();
                let prim = if !fill && ptype == PrimType::Circle { PrimType::Ring } else { ptype };
                ShapeBarItem::Shape(
                    Shape::Polygon(
                        DrawPolygon { rect, fill, prim, color: rgb_to_f32(color.0, color.1, color.2) })
                )
            }
        }
    }
}

struct ShapeBar {
    items: HashMap<ShapeBarShape, ShapeBarItem>,
    click_rects: HashMap<ShapeBarShape, Rect>,
    draw_rect: Rect
}

impl ShapeBar {
    fn new(offset: Point, viewport: &Point) -> Self {
        let draw_rect = Rect::new(
            offset,
            offset + Point { x: 3. * viewport.x / 5., y: viewport.y / 12.});
            //Point { x: viewport.x / 5., y: 0. }, 
            //Point { x: 4. * viewport.x / 5., y: viewport.y / 12.});
        let mut items = HashMap::new();
        let mut click_rects = HashMap::new();
        let shape_bar_shapes = [ShapeBarShape::Circle, ShapeBarShape::Triangle, 
                                ShapeBarShape::Rect, ShapeBarShape::TextBox, ShapeBarShape::Line,
                                ShapeBarShape::ColorPicker];
        let shapes_rect = Rect::new( 
            draw_rect.c1 + Point {x: draw_rect.width() / 4., y: draw_rect.height() / 5.},
            draw_rect.c2 - Point {x: draw_rect.width() / 4., y: draw_rect.height() / 5. } 
        );
        let npoly = shape_bar_shapes.len() as u32;
        let rect_size = Point::new(ShapeBarShape::DEFAULT_SIZE, ShapeBarShape::DEFAULT_SIZE);
        for (i, s) in shape_bar_shapes.iter().enumerate() {
            let center = Point::new(
                shapes_rect.c1.x + (i as u32 * shapes_rect.width() as u32 / (npoly - 1)) as f32,
                shapes_rect.center().y);
            let rect = Rect::new(center - rect_size / 2., center + rect_size / 2.);
            click_rects.insert(s.clone(), rect.clone());
            let fill = *s != ShapeBarShape::TextBox;
            items.insert(s.clone(), s.get_item(&rect, fill));
        }
        ShapeBar {
            items,
            click_rects,
            draw_rect,
        }
    }
    fn measure(&self) -> Point {
        self.draw_rect.size()
    }
    fn get_shape(&self, id: ShapeBarShape, r: &Rect, fill: bool) -> Shape {
        match id.get_item(r, fill) {
            ShapeBarItem::Shape(s) => s,
            _ => Shape::Polygon(DrawPolygon::default())
        }
    }
    fn click(&mut self, p: &Point, cursor: &mut SystemCursor, vp: &Point) -> Option<CallbackFn> {
        let bar_item = self.click_rects.iter().find(|(_, r)| r.in_bounds(p, vp)).map(|(id, _)| *id);
        match bar_item {
            None => None,
            Some(id) => {
                match id {
                    ShapeBarShape::ColorPicker => None,
                    _ => { 
                        let size = Point::new(ShapeBarShape::DEFAULT_SIZE, ShapeBarShape::DEFAULT_SIZE);
                        let r = Rect::new(*p - size / 2., *p + size / 2.);
                        let s = self.get_shape(id, &r, false);
                        *cursor = SystemCursor::Crosshair;
                        Some(Box::new(move |app: &mut AppState| {
                            app.hover_item = HoverItem::HoverShape(id, s);
                        }))
                    }
                }
            }
        }
    }
    fn draw(&self, draw_ctx: &DrawCtx) {
        draw_ctx.draw_rect(self.draw_rect.clone(), rgb_to_f32(120, 50, 200), true, Radians(0.)); 
        self.items.values().for_each(|s| s.draw(draw_ctx));
    }
}

#[derive(Copy, Clone, PartialEq)]
enum ClickResponse {
    Clicked,
    NotClicked
}

pub struct TabBar {
    offset: Point,
    tabs: Vec<Button>
}

impl TabBar {
    pub fn new(offset: Point) -> Self { 
        let border = Border::new(Point::new(5., 5.), rgb_to_f32(0, 0, 0));
        let canvas_button = Button::new(
            "Canvas",
            TextParams::new(),
            border.clone(),
            rgb_to_f32(0, 255, 255),
            Box::new(|_: &mut AppState| { println!("Clicked!"); })
        );
        let graph_button = Button::new(
            "Graph",
            TextParams::new(),
            border.clone(),
            rgb_to_f32(0, 255, 255),
            Box::new(|_: &mut AppState| { println!("Clicked!"); })
        );
        let tabs = vec![canvas_button, graph_button];
        TabBar { offset, tabs }
    }
    pub fn measure(&self, ctx: &DrawCtx) -> Point {
        self.tabs.iter().fold(Point::origin(), |size, tab| {
            let m = tab.measure(&ctx.render_text);
            Point::new(
                size.x + m.x,
                size.y.max(m.y) 
            )
        })
    }
    pub fn click(&self, pt: &Point, ctx: &DrawCtx) -> Option<CallbackFn> {
        None
        /*let mut off = self.offset;
        self.tabs.iter().map(|t| 
            (t, Rect::new(off, off + t.measure(&ctx.render_text)))).
            find(|(_, r)| r.in_bounds(pt, &ctx.viewport)).map(|(t, _)| t.callback.clone()) */
    }
    pub fn draw(&self, ctx: &DrawCtx) {
        let mut off = self.offset;
        for t in &self.tabs { 
            t.draw(&off, ctx);
            off.x += t.measure(&ctx.render_text).x;
        }
    }
}
