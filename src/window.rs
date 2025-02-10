use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

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

    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) 
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