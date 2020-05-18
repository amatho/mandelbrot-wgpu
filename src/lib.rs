mod renderer;
mod shaders;
mod state;

use renderer::Renderer;
use state::State;
use winit::event_loop::EventLoop;

pub async fn run() {
    let state = State::from_args();
    println!("{}", state);

    let event_loop = EventLoop::new();
    let window = create_window(&event_loop, state.window_size);
    let renderer = Renderer::new(state, &window).await;

    renderer.run_event_loop(event_loop, window);
}

fn create_window(event_loop: &EventLoop<()>, size: (u32, u32)) -> winit::window::Window {
    winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::<u32>::from(size))
        .with_title("Mandelbrot Visualization")
        .with_resizable(false)
        .build(&event_loop)
        .unwrap()
}
