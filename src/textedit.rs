extern crate ropey;
extern crate sdl2;

use ropey::Rope;
use std::time::{SystemTime, Duration};
use crate::primitives::{Point, RotateRect, DrawCtx, LineBuilder};
use crate::render_text::{RenderText, TextParams};
use sdl2::keyboard::Keycode;

#[derive(Debug)]
struct TextCursor {
    char_idx: usize
}

impl TextCursor {
    fn new() -> Self {
        TextCursor { char_idx: 0 }
    }
}

pub enum TextCursorDirection {
    Up, Down, Left, Right
}

#[derive(Debug)]
pub struct TextBox {
    text_rope: Rope,
    text_scale: f32,
    top_line: usize,
    cursor: TextCursor,
}

impl TextBox {
    pub fn new() -> Self {
        TextBox {
            text_rope: Rope::new(),
            top_line: 0,
            text_scale: 0.7,
            cursor: TextCursor::new(),
        }
    }
    pub fn insert_char(&mut self, ch: char, draw_rect: &RotateRect, rt: &RenderText) {
        self.text_rope.insert_char(self.cursor.char_idx, ch);
        self.cursor.char_idx += 1;
        let cursor_line = self.text_rope.char_to_line(self.cursor.char_idx);
        let line = self.text_rope.line(cursor_line);
        let cursor_pos = self.cursor.char_idx - self.text_rope.line_to_char(cursor_line);
        if cursor_pos == line.len_chars() &&
            rt.measure(line.as_str().unwrap(), self.text_scale).x > draw_rect.size.x
        {
            if self.cursor.char_idx == self.text_rope.len_chars() {
                self.text_rope.insert_char(self.cursor.char_idx-1, '\n');
                self.cursor.char_idx += 1;
            }
        }
        if self.cursor.char_idx < self.text_rope.len_chars() {
            self.format_text(draw_rect, cursor_line, rt);
        }
        /*if self.text_rope.len_lines() as f32 * rt.line_height(self.text_scale) > draw_rect.size.y {
            self.top_line += 1;
        }*/
    }
    pub fn delete_char(&mut self, draw_rect: &RotateRect, rt: &RenderText) {
        if self.text_rope.len_chars() == 0 {
            return;
        }
        let cursor_line = self.text_rope.char_to_line(self.cursor.char_idx);
        if cursor_line > 0 && self.cursor.char_idx == self.text_rope.line_to_char(cursor_line) {
            self.cursor.char_idx -= 1;
        }
        //println!("Cursor char index: {:?}, line char index: {:?}", self.cursor.char_idx, self.text_rope.line_to_char(cursor_line));
        self.text_rope.remove(self.cursor.char_idx-1..self.cursor.char_idx);
        self.cursor.char_idx -= 1;
        if self.text_rope.len_chars() > 0 && self.cursor.char_idx < self.text_rope.len_chars() - 1 {
            self.format_text(draw_rect, cursor_line, rt);
        }
    }
    pub fn hover_text(&self, pt: &Point, rect: &RotateRect, rt: &RenderText) -> Option<usize> {
        let n_line = ((pt.y - rect.offset.y) / rt.line_height(self.text_scale)) as i32;
        let x_off = pt.x - rect.offset.x;
        if x_off < 0. || x_off > rect.size.x || 
            n_line < 0 || n_line >= self.text_rope.len_lines() as i32 {
            return None;
        }
        let line_idx = n_line as usize;
        let start_char = self.text_rope.line_to_char(line_idx);
        let end_char = self.text_rope.line_to_char(line_idx + 1);
        //println!("Hover Text! Line index: {:?} Start char pos: {:?}, End char pos {:?}", line_idx, start_char, end_char);
        let mut line_x = 0.;
        (start_char+1..end_char)
            .take_while(|i| { 
                line_x += rt.char_size_w_advance(self.text_rope.char(i-1), self.text_scale).x; line_x <= x_off}).last()
    } 
    pub fn set_cursor_pos(&mut self, cursor_idx: usize) {
        self.cursor.char_idx = std::cmp::max(0, std::cmp::min(self.text_rope.len_chars(), cursor_idx));
    }
    pub fn move_cursor(&mut self, dir: TextCursorDirection) {
        let cursor_line = self.text_rope.char_to_line(self.cursor.char_idx);
        let line = self.text_rope.line(cursor_line);
        let cursor_pos = self.cursor.char_idx - self.text_rope.line_to_char(cursor_line);
        //println!("Cursor line: {:?} Cursor Pos: {:?}", line, cursor_pos);
        match dir {
            TextCursorDirection::Left =>  { 
                if self.cursor.char_idx > 0 {
                    if cursor_pos == 0 {
                        self.cursor.char_idx -= 1;
                    }
                    self.cursor.char_idx -= 1;
                }
            }
            TextCursorDirection::Right =>  { 
                if self.cursor.char_idx < self.text_rope.len_chars() {
                    if cursor_pos == line.len_chars() {
                        self.cursor.char_idx += 1;
                    }
                    self.cursor.char_idx += 1;
                }
            }
            TextCursorDirection::Up =>  { 
                if cursor_line > 0 {
                    let prev_line_char = self.text_rope.line_to_char(cursor_line - 1);
                    let prev_line = self.text_rope.line(cursor_line - 1);
                    self.cursor.char_idx = std::cmp::min(prev_line_char + prev_line.len_chars(), prev_line_char + cursor_pos);
                }
            }
            TextCursorDirection::Down =>  { 
                if cursor_line < self.text_rope.len_lines() - 1 {
                    let next_line_char = self.text_rope.line_to_char(cursor_line + 1);
                    let next_line = self.text_rope.line(cursor_line + 1);
                    self.cursor.char_idx = std::cmp::min(next_line_char + next_line.len_chars() - 1, next_line_char + cursor_pos);
                }
            }
        }
    }
    pub fn draw(&self, draw_rect: &RotateRect, select_time: Option<SystemTime>, rt: &RenderText, draw_ctx: &DrawCtx) {
        let cursor_line = self.text_rope.char_to_line(self.cursor.char_idx);
        let line_height = rt.line_height(self.text_scale);
        if self.text_rope.len_chars() > 0 {
            let mut max_lines = (draw_rect.size.y / line_height) as usize;
            max_lines = std::cmp::min(max_lines, self.text_rope.len_lines());
            let start_idx = if self.text_rope.len_lines() == 0 { 0 } 
                else { self.text_rope.line_to_char(self.top_line) };
            let end_idx = self.text_rope.line_to_char(self.top_line + max_lines);
            let text_params = 
                TextParams::new(self.text_rope.slice(start_idx..end_idx).as_str().unwrap(), draw_rect)
                    .scale(self.text_scale);
                    //.color(255, 0, 255)
                    //.offset(&(draw_rect.offset + Point::new(0., line_height)));
            rt.draw(&text_params, draw_ctx);
        }
        if let Some(select_time) = select_time {
            let millis = select_time.elapsed().unwrap().as_millis() % 1000;
            if millis < 500 {
                let before_str = self.text_rope.slice(self.text_rope.line_to_char(cursor_line)..self.cursor.char_idx).as_str().unwrap();
                let cursor_pt = draw_rect.offset + Point::new(
                    rt.measure(before_str, self.text_scale).x, 
                    (cursor_line - self.top_line) as f32 * line_height);
                let cursor_line = LineBuilder::new().points(cursor_pt.x, cursor_pt.y, cursor_pt.x, cursor_pt.y + line_height)
                    .color(0,0,0).get();
                cursor_line.draw(draw_ctx);
            }
        }
    }
    pub fn needs_format(&self, draw_rect: &RotateRect, start_line: usize, rt: &RenderText) -> bool {
        let start_line_width = rt.measure(self.text_rope.line(start_line).as_str().unwrap(), self.text_scale).x;
        if start_line_width > draw_rect.size.x {
            return true;
        }
        if start_line < self.text_rope.len_lines() - 1 {
            let next_line = self.text_rope.line(start_line + 1);
            if next_line.len_chars() > 0 {
                let next_line_char = next_line.char(0);
                let next_line_char_width = rt.char_size(next_line_char, self.text_scale).x;
                return start_line_width + next_line_char_width <= draw_rect.size.x
            }
        }
        false
    }
    pub fn format_text(&mut self, draw_rect: &RotateRect, start_line: usize, rt: &RenderText) {
        if !self.needs_format(draw_rect, start_line, rt) {
            return;
        }
        let start_char = self.text_rope.line_to_char(start_line);
        let mut line_x = 0.;
        let mut line_breaks = Vec::new();
        for (i, c) in self.text_rope.slice(start_char..).chars().enumerate() {
            let add_break = line_x + rt.char_size(c, self.text_scale).x > draw_rect.size.x;
            let was_break = c == '\n';
            if add_break {
                line_x = 0.;
            }
            if add_break != was_break {
                line_breaks.push((i + start_char, add_break));
            }
            line_x += rt.char_size_w_advance(c, self.text_scale).x;
        }
        let mut offset: i32 = 0;
        for (idx, is_add) in line_breaks {
            let uidx = (idx as i32 + offset) as usize;
            if is_add {
                self.text_rope.insert_char(uidx, '\n');
                if uidx < self.cursor.char_idx {
                    self.cursor.char_idx += 1;
                }
                offset += 1;
            }
            else {
                self.text_rope.remove(uidx..uidx+1);
                if uidx < self.cursor.char_idx {
                    self.cursor.char_idx -= 1;
                }
                offset -= 1;
            }
        }
    }
}

pub fn get_char_from_keycode(keycode: Keycode) -> Option<char> {
    let name = keycode.name();
    if name.len() == 1 {
        name.chars().nth(0)
    }
    else if keycode == Keycode::Space {
        Some(' ')
    }
    else {
        None
    }
}

pub fn get_dir_from_keycode(kc: Keycode) -> Option<TextCursorDirection> {
    match kc {
        Keycode::Left => Some(TextCursorDirection::Left),
        Keycode::Right => Some(TextCursorDirection::Right),
        Keycode::Up => Some(TextCursorDirection::Up),
        Keycode::Down => Some(TextCursorDirection::Down),
        _ => None
    }
}