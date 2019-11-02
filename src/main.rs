extern crate sdl2;
extern crate gl;
extern crate rand;
extern crate nalgebra_glm;

use sdl2::event::Event;
use sdl2::keyboard::{Keycode};
use std::time::{SystemTime, Duration};

pub mod interface;
pub mod render_gl;
pub mod render_text;
pub mod primitives;
#[macro_use]
//pub mod app;
pub mod textedit;
use interface::{AppState, DrawList};
use primitives::{*};

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    const VIEWPORT: Point = Point{x:1100., y:700.};
    let window = video_subsystem
        .window("Shapes", VIEWPORT.x as u32, VIEWPORT.y as u32)
        .opengl()
        .resizable()
        .build()
        .unwrap();

    let gl_attr = video_subsystem.gl_attr();
    let _gl_context = window.gl_create_context().unwrap();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4,5);
    let _gl = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

    let bg_color = rgb_to_f32(3, 190, 252);
    unsafe {
        gl::Viewport(0, 0, VIEWPORT.x as i32, VIEWPORT.y as i32);
        gl::ClearColor(bg_color[0], bg_color[1], bg_color[2], bg_color[3]);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    let mut app_state = AppState::new(&VIEWPORT);

    let mut event_pump = sdl.event_pump().unwrap();
    let mut timer = SystemTime::now();

    const FPS: u64 = 120;

    'main: loop {
        for event in event_pump.poll_iter() {
            let kmod = sdl.keyboard().mod_state();
            match event {
                Event::Quit {..} | 
                Event::KeyDown { keycode: Some(Keycode::Escape), ..} => break 'main,
                ev @ Event::MouseMotion{..} | 
                ev @ Event::MouseButtonDown{..} | 
                ev @ Event::MouseButtonUp{..} => { 
                /*    app_state.handle_event(&ev);
                }*/
                    app_state.handle_mouse_event(&ev, &kmod);
                }
                ev @ Event::KeyDown {..} => {
                    app_state.handle_keyboard_event(&ev);
                }/**/
                _ => {},
            }
        }
        unsafe { 
            gl::Clear(gl::COLOR_BUFFER_BIT); 
        }
        if timer.elapsed().unwrap() >= Duration::from_millis(1000 / FPS) {
            app_state.render();
            window.gl_swap_window();
            timer = SystemTime::now();
        }
    }
}
