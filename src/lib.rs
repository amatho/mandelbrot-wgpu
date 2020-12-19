mod fragment;
mod state;

use fragment::FragmentState;
use state::State;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

pub async fn run() {
    let fragment_state = FragmentState::from_args();
    println!("{}", fragment_state);

    let event_loop = EventLoop::new();
    let window = create_window(&event_loop, fragment_state.size);
    let state = State::new(fragment_state, &window).await;

    run_event_loop(state, event_loop, window);
}

fn create_window(
    event_loop: &EventLoop<()>,
    size: winit::dpi::PhysicalSize<u32>,
) -> winit::window::Window {
    winit::window::WindowBuilder::new()
        .with_inner_size(size)
        .with_title("Mandelbrot Visualization")
        .with_resizable(false)
        .build(&event_loop)
        .unwrap()
}

fn run_event_loop(mut state: State, event_loop: EventLoop<()>, window: winit::window::Window) {
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => {
                            *control_flow = ControlFlow::Exit;
                        }
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SwapChainError::Lost) => state.resize(state.fragment_state().size),
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
