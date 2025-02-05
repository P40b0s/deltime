use fltk::{app, prelude::*, window::Window};

pub fn start() 
{
    let app = app::App::default();
    let mut wind = Window::new(100, 100, 400, 300, "Hello from rust");
    wind.end();
    wind.show();
    app.run().unwrap();
}