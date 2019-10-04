pub enum MouseShapeBar {
    DragResize { drag_vertex: DragVertex },
    DragRotate { last_angle: f32 }
}

pub enum MouseCanvas {
    DragShapes { last_pt: Point, clear_select: bool },
}

struct ShapeCreateDrag {
    start_pt: Point,
    last_pt: Point,
}

trait HandleMouseMove {
    fn mouse_move(self, mouse_pt: &Point, id: ShapeID, app_state: &mut AppState) -> Option<Self> where Self: Sized;
}

impl HandleMouseMove for MouseShapeBar {
    fn mouse_move(mut self, mouse_pt: &Point, id: ShapeID, app_state: &mut AppState) -> Option<Self> {
        let vp = app_state.draw_ctx.viewport;
        let select_box = app_state.selection.get_mut(&id); 
        let mut cursor = SystemCursor::Hand;
        if let Some(sbox) = select_box {
            match self {
                MouseShapeBar::DragRotate { ref mut last_angle } => {
                    let angle = sbox.get_rotate_angle(mouse_pt, &vp);
                    let curr_angle = 180. * sbox.0.rot / std::f32::consts::PI;
                    sbox.0.set_radians(curr_angle + angle - *last_angle);
                    app_state.draw_list.get_mut(&id).map(|s| s.set_rect(&sbox.0.clone()));
                    *last_angle = angle;
                }
                MouseShapeBar::DragResize { ref mut drag_vertex } => {
                    cursor = get_drag_hover_cursor(&drag_vertex);
                    *drag_vertex = sbox.drag_side(&drag_vertex, mouse_pt, &vp);
                    app_state.draw_list.get_mut(&id).map(|s| s.set_rect(&sbox.0.clone()));
                }
            }
        }
        app_state.cursors.get(&cursor).set();
        Some(self)
    }
}

impl HandleMouseMove for MouseCanvas {
    fn mouse_move(mut self, mouse_pt: &Point, _: ShapeID, app_state: &mut AppState) -> Option<Self> {
        match self {
            MouseCanvas::DragShapes { ref mut last_pt, ref mut clear_select, .. } => {
                *clear_select = false;
                for (id, rect) in app_state.selection.iter_mut() {
                    app_state.draw_list.get_mut(id).map(|s| s.drag(&(*mouse_pt - *last_pt)));
                    rect.drag(&(*mouse_pt - *last_pt));
                }
                *last_pt = *mouse_pt;
            }
        }
        app_state.cursors.get(&SystemCursor::Hand).set();
        Some(self)
    }
}

impl HandleMouseMove for ShapeCreateDrag {
    fn mouse_move(mut self, mouse_pt: &Point, _: ShapeID, app_state: &mut AppState) -> Option<Self> {
        self.last_pt = *mouse_pt;
        app_state.cursors.get(&SystemCursor::Hand).set();
        Some(self)
    }
}

