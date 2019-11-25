extern crate sdl2;
extern crate gl;
extern crate rand;
extern crate nalgebra_glm;

extern crate sem_graph;

use sem_graph::interface::*;
use sem_graph::render_text::*;
use sem_graph::primitives::*;

use sdl2::event::Event;
use sdl2::keyboard::{Keycode};
use std::time::{SystemTime, Duration};

use nalgebra_glm as glm;
use rand::{Rng};

#[allow(dead_code)]
fn rand_color() -> (u8, u8, u8) {
    let mut rng = rand::thread_rng();
    (rng.gen_range(0, 255), rng.gen_range(0, 255), rng.gen_range(0, 255))
}

#[allow(dead_code)]
fn add_test_lines(draw_list: &mut DrawList) {
    draw_list.add(LineBuilder::new()
        .points(200.,200.,400.,400.)
        .color(0, 255, 255).line_width(6.).get()
    );
    draw_list.add(LineBuilder::new()
        .points(834.,338.,1000.,450.)
        .color(255, 255, 255).get()
    );
    draw_list.add(LineBuilder::new()
        .points(400.,200.,400.,400.)
        .color(255, 0, 0).get()
    );
}
#[allow(dead_code)]
fn add_rotated_shapes(draw_list: &mut DrawList) {
    draw_list.add(ShapeBuilder::new() 
        .square(200)
        .offset(300,100)
        .rot(Degrees(45.))
        .color(255, 255, 255).get()
    );
    draw_list.add(ShapeBuilder::new()
        .square(150)
        .color(200,100,200)
        .rot(Degrees(30.))
        .offset(600,200).get()
    );
    draw_list.add(ShapeBuilder::new() 
        .tri(100, 100)
        .offset(200,200)
        .rot(Degrees(60.))
        .color(0, 255, 0).get()
    );
}
#[allow(dead_code)]
fn add_test_shapes(draw_list: &mut DrawList) {
    draw_list.add(ShapeBuilder::new() 
        .tri(100, 100)
        .offset(200,200)
        .color(0, 255, 0).get()
    );
    
    draw_list.add(ShapeBuilder::new() 
        .rect(200, 100)
        .offset(400,200)
        .color(0, 0, 255).get()
    );
    draw_list.add(ShapeBuilder::new()
        .circle(100)
        .offset(200,400)
        .fill(false)
        .color(255, 0, 255).get()
    );
    draw_list.add(ShapeBuilder::new() 
        .ellipse(200, 100)
        .offset(400,400)
        .color(255, 255, 0).get()
    );
}
#[allow(dead_code)]
fn add_random_shapes(draw_list: &mut DrawList, vp: &Point, n: u8) {
    const MIN_DIM: u32 = 10;
    let max_width = vp.x as u32 / 6;
    let max_height = vp.y as u32 / 6;
    let rand_pt = || {
        let mut rng = rand::thread_rng(); 
        let x = rng.gen_range(vp.x as i32 / 11, vp.x as i32 * 10 / 11);
        let y = rng.gen_range(vp.y as i32 / 11, vp.y as i32 * 10 / 11);
        (x,y)
    };
    /*let rand_rot = || {
        let mut rng = rand::thread_rng();
        rng.gen_range(0, 360)
    };*/
    let rand_shape = || {
        let mut rng = rand::thread_rng(); 
        let p1 = rand_pt();
        let r = rng.gen_range(0,3);
        let color = rand_color();
        if r == 3 {
            let p2 = rand_pt();
            LineBuilder::new().points(p1.0 as f32, p1.1 as f32, p2.0 as f32, p2.1 as f32).color(color.0, color.1, color.2).get()
        }
        else {
            let width = rng.gen_range(MIN_DIM, max_width + 1);
            let height = rng.gen_range(MIN_DIM, max_height + 1);
            let mut sb = ShapeBuilder::new();
            sb = match r {
                0 => sb.tri(width, height),
                1 => sb.rect(width, height),
                _ => sb.ellipse(width, height)
            };
            sb.color(color.0, color.1, color.2).offset(p1.0, p1.1).get()
        }
    };
    for _ in 0..n {
        draw_list.add(rand_shape());
    }
}

#[test]
fn test_draw() {
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

    let draw_list = DrawList::new();
    //let mut draw_list = DrawList::new();
    //add_rotated_shapes(&mut draw_list);
    //add_test_lines(&mut draw_list);
    //add_random_shapes(&mut draw_list, &VIEWPORT, 10);

    let mut app_state = AppState::new(&VIEWPORT);

    let mut event_pump = sdl.event_pump().unwrap();
    let mut timer = SystemTime::now();
    let render_text = RenderText::new().unwrap();

    let test_str = "This is a test sentence!\n let's test this sentence!\n haha! woo hoo!";
    let text_params = TextParams::new(test_str).offset(&Point::new(300.,300.)).color(255, 0, 255).rot(Degrees(0.).into());
    'main: loop {
        for event in event_pump.poll_iter() {
            let kmod = sdl.keyboard().mod_state();
            match event {
                ev @ Event::MouseMotion{..} => { 
                   if timer.elapsed().unwrap() >= Duration::from_millis(5) { //don't always handle mouse move
                        app_state.handle_mouse_event(&ev, &kmod);
                        timer = SystemTime::now();
                    }
                }
                ev @ Event::MouseButtonDown{..} | 
                ev @ Event::MouseButtonUp{..} => { //always handle mouse down and up
                    app_state.handle_mouse_event(&ev, &kmod);
                }
                Event::Quit {..} | 
                Event::KeyDown { keycode: Some(Keycode::Escape), ..} => break 'main,
                ev @ Event::KeyDown {..} => {
                    app_state.handle_keyboard_event(&ev);
                }
                _ => {},
            }
            unsafe { 
                gl::Clear(gl::COLOR_BUFFER_BIT); 
            }
            app_state.render();
            render_text.draw(&text_params);
            window.gl_swap_window();
        }
    }
}

