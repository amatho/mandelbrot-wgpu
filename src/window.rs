use crate::state::State;
use wgpu::Surface;
use winit::event_loop::EventLoop;

pub struct Window {
    inner: winit::window::Window,
    surface: Surface,
}

impl Window {
    pub fn new(state: &State, event_loop: &EventLoop<()>) -> Self {
        #[cfg(not(feature = "gl"))]
        let (window, surface) = {
            let window = winit::window::WindowBuilder::new()
                .with_inner_size(winit::dpi::PhysicalSize::<u32>::from(state.window_size))
                .with_title("Mandelbrot Visualization")
                .with_resizable(false)
                .build(&event_loop)
                .unwrap();
            let surface = wgpu::Surface::create(&window);
            (window, surface)
        };

        #[cfg(feature = "gl")]
        let (window, instance, surface) = {
            let wb = winit::window::WindowBuilder::new()
                .with_inner_size(winit::dpi::PhysicalSize::<u32>::from(state.window_size))
                .with_title("Mandelbrot Visualization")
                .with_resizable(false);
            let cb = wgpu::glutin::ContextBuilder::new().with_vsync(true);
            let context = cb.build_windowed(wb, &event_loop).unwrap();

            let (context, window) = unsafe { context.make_current().unwrap().split() };

            let instance = wgpu::Instance::new(context);
            let surface = instance.get_surface();

            (window, instance, surface)
        };

        Window {
            inner: window,
            surface,
        }
    }

    pub fn inner_window(&self) -> &winit::window::Window {
        &self.inner
    }

    pub fn surface(&self) -> &Surface {
        &self.surface
    }
}
