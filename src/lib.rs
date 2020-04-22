mod renderer;
mod shaders;
mod state;

use renderer::Renderer;
use state::State;
use winit::event_loop::EventLoop;

pub fn run() {
    let state = State::from_args();
    println!("{}", state);

    let event_loop = EventLoop::new();
    let window = create_window(&event_loop, state.window_size);
    let renderer = Renderer::new(state, &window);

    renderer.run_event_loop(event_loop, window);
}

fn create_window(event_loop: &EventLoop<()>, size: (u32, u32)) -> winit::window::Window {
    #[cfg(not(feature = "gl"))]
    let window = {
        winit::window::WindowBuilder::new()
            .with_inner_size(winit::dpi::PhysicalSize::<u32>::from(size))
            .with_title("Mandelbrot Visualization")
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };

    #[cfg(feature = "gl")]
    let (window, instance) = {
        let wb = winit::window::WindowBuilder::new()
            .with_inner_size(winit::dpi::PhysicalSize::<u32>::from(state.window_size))
            .with_title("Mandelbrot Visualization")
            .with_resizable(false);
        let cb = wgpu::glutin::ContextBuilder::new().with_vsync(true);
        let context = cb.build_windowed(wb, &event_loop).unwrap();

        let (context, window) = unsafe { context.make_current().unwrap().split() };

        let instance = wgpu::Instance::new(context);

        (window, instance)
    };

    window
}
