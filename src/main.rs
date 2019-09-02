extern crate sdl2;
extern crate gl;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::ffi::{CString};

mod render_gl;
mod primitives;
use render_gl::*;
use primitives::{*, TypeParams::*};

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

    let program = Program::from_shaders(&[vert_shader, frag_shader]).unwrap();
    //program.set_used();

    let shape_ctx = ShapeCtx::new(program.id(), VIEWPORT);

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
            program.set_used();
            let mut shapes = Vec::new();
            shapes.push(ShapeBuilder::new() 
                .params(Triangle { base: 100 })
                .offset(200,200)
                .color(0, 255, 0).into()
            );
            shapes.push(ShapeBuilder::new() 
                .params(Rect { height: 200, width: 100 })
                .offset(400,200)
                .rot(45.)
                .color(0, 0, 255).into()
            );
            shapes.push(ShapeBuilder::new()
                .params(Circle { radius: 100 })
                .offset(200,400)
                .color(255, 0, 255).into()
            );
            shapes.push(ShapeBuilder::new() 
                .params(Ellipse { rad_x: 200, rad_y: 100 })
                .offset(400,400)
                .color(255, 255, 0).into()
            );
            shapes.push(ShapeBuilder::new()
                .params(new_line(200,200,400,400))
                .color(0, 255, 255).into()
            );

            for s in shapes {
                shape_ctx.draw_shape(s);
            }

            window.gl_swap_window();
        }
    }
}
