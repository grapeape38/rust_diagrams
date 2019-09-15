extern crate sdl2;
extern crate gl;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::{SystemTime, Duration};

mod interface;
mod render_gl;
mod primitives;
use interface::{AppState, DrawList};
use primitives::{*};

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    const VIEWPORT: Point = Point{x:1100., y:700.};
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
        gl::Viewport(0, 0, VIEWPORT.x as i32, VIEWPORT.y as i32);
        gl::ClearColor(0.3, 0.3, 0.5, 1.0);
    }

   let programs = PrimPrograms::new();
   let draw_ctx = DrawCtx::new(&programs, VIEWPORT);

    let mut event_pump = sdl.event_pump().unwrap();
    
    let mut shapes = DrawList::new();

    shapes.add(ShapeBuilder::new() 
        .tri(100, 100)
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

    shapes.add(LineBuilder::new()
        .points(200.,200.,400.,400.)
        .color(0, 255, 255).line_width(6.).get()
    );

    shapes.add(LineBuilder::new()
        .points(834.,338.,1000.,450.)
        .color(255, 255, 255).get()
    );

    shapes.add(LineBuilder::new()
        .points(400.,200.,400.,400.)
        .color(255, 0, 0).get()
    );
    
    let mut app_state = AppState::new(shapes, draw_ctx);

    let mut timer = SystemTime::now();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                ev @ Event::MouseMotion{..} | 
                ev @ Event::MouseButtonDown{..} | 
                ev @ Event::MouseButtonUp{..} => {
                   if timer.elapsed().unwrap() >= Duration::from_millis(5) {
                       app_state.handle_mouse_event(&ev);
                       timer = SystemTime::now();
                   }
                }
                Event::Quit {..} | 
                Event::KeyDown { keycode: Some(Keycode::Escape), ..} => break 'main,
                _ => {},
            }
            app_state.render();
            window.gl_swap_window();
        }
    }
}
