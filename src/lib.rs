mod renderer;
mod shaders;
mod state;
mod window;

use renderer::Renderer;
use state::State;
use window::Window;
use winit::event_loop::EventLoop;

pub fn run() {
    let state = State::from_args();
    println!("{}", state);

    let event_loop = EventLoop::new();
    let window = Window::new(&state, &event_loop);
    let renderer = Renderer::new(window, state);

    renderer.run_event_loop(event_loop);
}
