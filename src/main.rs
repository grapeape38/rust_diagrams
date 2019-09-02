extern crate sdl2;
extern crate gl;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::ffi::{CString};

mod render_gl;
mod primitives;
use render_gl::*;
use primitives::{*};

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    const VIEWPORT: Point = Point{x:1100, y:700};
    let window = video_subsystem
        .window("Game", VIEWPORT.x as u32, VIEWPORT.y as u32)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let gl_attr = video_subsystem.gl_attr();
    let _gl_context = window.gl_create_context().unwrap();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4,5);
    let _gl = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);


    unsafe {
        gl::Viewport(0, 0, VIEWPORT.x, VIEWPORT.y);
        gl::ClearColor(0.3, 0.3, 0.5, 1.0);
    }


    let vert_shader = Shader::from_vert_source(
        &CString::new(include_str!("triangle.vert")).unwrap()
    ).unwrap();

    let frag_shader = Shader::from_frag_source(
        &CString::new(include_str!("triangle.frag")).unwrap()
    ).unwrap();

    let mut shaders = vec![vert_shader, frag_shader];

    let program = Program::from_shaders(shaders.as_ref()).unwrap();

    let geom_shader = Shader::from_geom_source(
        &CString::new(include_str!("line.geom")).unwrap()
    ).unwrap();

    shaders.insert(1, geom_shader);
    let line_prog = Program::from_shaders(shaders.as_ref()).unwrap();

    let shape_ctx = DrawCtx::new(program, VIEWPORT);
    let line_ctx = DrawCtx::new(line_prog, VIEWPORT);

    let mut event_pump = sdl.event_pump().unwrap();
    
    let mut shapes = DrawList::new();
    let mut lines = DrawList::new();

    shapes.add(ShapeBuilder::new() 
        .tri(100)
        .offset(200,200)
        .color(0, 255, 0).get()
    );
    shapes.add(ShapeBuilder::new() 
        .rect(200, 100)
        .offset(400,200)
        .rot(45.)
        .color(0, 0, 255).get()
    );
    shapes.add(ShapeBuilder::new()
        .circle(100)
        .offset(200,400)
        .color(255, 0, 255).get()
    );
    shapes.add(ShapeBuilder::new() 
        .ellipse(200, 100)
        .offset(400,400)
        .color(255, 255, 0).get()
    );

    shapes.add(ShapeBuilder::new() 
        .square(200)
        .offset(600,600)
        .color(255, 255, 255).get()
    );

    shapes.add(ShapeBuilder::new()
        .square(150)
        .rot(45.)
        .color(200,100,200)
        .offset(600,200).get()
    );

    lines.add(LineBuilder::new()
        .points(200,200,400,400)
        .color(0, 255, 255).line_width(6.).get()
    );

    lines.add(LineBuilder::new()
        .points(200,200,400,200)
        .color(255, 255, 255).get()
    );

    lines.add(LineBuilder::new()
        .points(400,200,400,400)
        .color(255, 0, 0).get()
    );

    let mut drag_state = DragState { drag_shape: None, last_pt: Point{x:0,y:0} };

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                ev @ Event::MouseMotion{..} | 
                ev @ Event::MouseButtonDown{..} | 
                ev @ Event::MouseButtonUp{..} => {
                    drag_state.handle_mouse_event(&ev, &mut shapes, &VIEWPORT);
                }
                Event::Quit {..} | 
                Event::KeyDown { keycode: Some(Keycode::Escape), ..} => break 'main,
                _ => {},
            }
            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }

            shapes.draw_all(&shape_ctx);
            lines.draw_all(&line_ctx);

            window.gl_swap_window();
        }
    }
}

struct DragState {
    drag_shape: Option<u32>,
    last_pt: Point
}

impl DragState {
    fn handle_mouse_event(&mut self, ev: &Event, shapes: &mut DrawList, vp: &Point) {
        match *ev {
            Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                if self.drag_shape.is_none() && mouse_btn == sdl2::mouse::MouseButton::Left {
                    self.last_pt = Point{x,y};
                    self.drag_shape = shapes.intersect(&self.last_pt, vp);
                }
            } 
            Event::MouseButtonUp{mouse_btn, .. } => {
                if mouse_btn == sdl2::mouse::MouseButton::Left {
                    self.drag_shape = None
                }
            }
            Event::MouseMotion{ x, y, ..} => {
                if let Some(id) = self.drag_shape {
                    let off = Point{x, y};
                    if let Some(shape) = shapes.get_mut(&id) {
                        shape.drag(off - self.last_pt);
                    }
                    self.last_pt = off;
                }
            }
            _ => {}
        }
    }
}
