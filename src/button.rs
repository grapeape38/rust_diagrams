extern crate nalgebra_glm;

use nalgebra_glm as glm;

use crate::interface::{CallbackFn};
use crate::render_text::{RenderText, TextParams};
use crate::primitives::{DrawCtx, Point, Rect, RotateRect, Radians, Border, BorderRect, InBounds};


pub struct Button {
    text: &'static str,
    text_params: TextParams,
    pub callback: CallbackFn,
    fill_color: glm::Vec4,
    border: Border
}

impl Button {
    pub fn new(text: &'static str, text_params: TextParams, 
        border: Border, fill_color: glm::Vec4, callback: CallbackFn) -> Self 
    {
        Button { text, text_params, callback, fill_color, border }
    }
    pub fn measure(&self, rt: &RenderText) -> Point {
       rt.measure(self.text, self.text_params.scale) + self.border.width * Point::new(2., 2.)
    }
    pub fn draw(&self, offset: &Point, ctx: &DrawCtx) {
        let size = ctx.render_text.measure(self.text, self.text_params.scale);
        let r = Rect::new(*offset, *offset + size);
        let rect = BorderRect::new(r, self.fill_color, self.border.clone());
        rect.draw(ctx);
        let rr = RotateRect::from_rect(rect.r, Radians(0.));
        ctx.render_text.draw(&self.text, &self.text_params, &rr, ctx);
    }
}
