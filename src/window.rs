//use fltk::{app, enums::Color, prelude::*, window::Window};


use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

// fn alpha_composited_color(col: Color, bg_col: Color, alpha: f32) -> Color 
// {
//     let tup = col.to_rgb();
//     let bg_col = bg_col.to_rgb();
//     let r = alpha * tup.0 as f32 + (1.0 - alpha) * bg_col.0 as f32;
//     let r = r as u8;
//     let g = alpha * tup.1 as f32 + (1.0 - alpha) * bg_col.1 as f32;
//     let g = g as u8;
//     let b = alpha * tup.2 as f32 + (1.0 - alpha) * bg_col.2 as f32;
//     let b = b as u8;
//     Color::from_rgb(r, g, b)

// }
// pub fn start2() 
// {
//     let app = app::App::default();
    
//     let mut wind = Window::new(0, 0, 100, 100, "Hello from rust");
//     wind.set_color(Color::Free);
//     wind.end();
//     wind.show();
//     wind.set_opacity(0.0);
    
//     while app.wait()
//     {

//     }
//     //app.run().unwrap();
// }
#[derive(Default)]
struct App 
{
    window: Option<Window>,
}

impl ApplicationHandler for App 
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) 
    {
        let w = event_loop.create_window(Window::default_attributes()).unwrap();
        w.set_visible(false);
        self.window = Some(w);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) 
    {
        ()
    }
}

pub fn start() 
{
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::default();
    event_loop.run_app(&mut app);
}