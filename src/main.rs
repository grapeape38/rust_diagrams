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
    const VIEWPORT: Point = Point{x:900, y:700};
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
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | 
                Event::KeyDown { keycode: Some(Keycode::Escape), ..} => break 'main,
                _ => {},
            }
            unsafe {
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
            let mut shapes: Vec<Shape> = Vec::new();
            let mut lines: Vec<Line> = Vec::new();
            shapes.push(ShapeBuilder::new() 
                .tri(100)
                .offset(200,200)
                .color(0, 255, 0).into()
            );
            shapes.push(ShapeBuilder::new() 
                .rect(200, 100)
                .offset(400,200)
                .rot(45.)
                .color(0, 0, 255).into()
            );
            shapes.push(ShapeBuilder::new()
                .circle(100)
                .offset(200,400)
                .color(255, 0, 255).into()
            );
            shapes.push(ShapeBuilder::new() 
                .ellipse(200, 100)
                .offset(400,400)
                .color(255, 255, 0).into()
            );

            shapes.push(ShapeBuilder::new() 
                .square(200)
                .offset(600,600)
                .color(255, 255, 255).into()
            );

            shapes.push(ShapeBuilder::new()
                .square(150)
                .rot(45.)
                .color(200,100,200)
                .offset(600,200).into()
            );

            lines.push(LineBuilder::new()
                .points(200,200,400,400)
                .color(0, 255, 255).line_width(6.).into()
            );

            lines.push(LineBuilder::new()
                .points(200,200,400,200)
                .color(255, 255, 255).into()
            );

            lines.push(LineBuilder::new()
                .points(400,200,400,400)
                .color(255, 0, 0).into()
            );

            for s in shapes {
                shape_ctx.draw(s);
            }

            for l in lines {
                line_ctx.draw(l);
            }

            window.gl_swap_window();
        }
    }
}
