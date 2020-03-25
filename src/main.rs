mod shaders;

use std::fmt::{self, Display};
use winit::{
    event,
    event_loop::{ControlFlow, EventLoop},
};
use zerocopy::AsBytes;

const ZOOM_FACTOR: f64 = 1.05;
const TRANSFORM_STEP: f64 = 0.01;

#[cfg(not(feature = "double"))]
type GPUFloat = f32;
#[cfg(feature = "double")]
type GPUFloat = f64;

struct State {
    window_size: (u32, u32),
    max_iterations: u32,
    scale: f64,
    center: (f64, f64),
}

impl State {
    fn fragment_uniform(&self) -> FragmentUniform {
        FragmentUniform {
            screen_size: [
                self.window_size.0 as GPUFloat,
                self.window_size.1 as GPUFloat,
            ],
            center: [self.center.0 as GPUFloat, self.center.1 as GPUFloat],
            scale: self.scale as GPUFloat,
            max_iterations: self.max_iterations,
            _padding: 0,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State {
            window_size: (800, 600),
            max_iterations: 200,
            scale: 2.0,
            center: (-0.5, 0.0),
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Center: {:?}, Scale: {}, Iterations: {}",
            self.center, self.scale, self.max_iterations
        )
    }
}

#[derive(Copy, Clone, AsBytes)]
#[repr(C)]
struct FragmentUniform {
    screen_size: [GPUFloat; 2],
    center: [GPUFloat; 2],
    scale: GPUFloat,
    max_iterations: u32,
    _padding: u32,
}

fn usage() -> ! {
    println!(
        "Usage:
    mandelbrot [<iterations> <center real> <center imag> <width> <height>]
    or
    mandelbrot [<iterations> <center real> <center imag>]
    or
    mandelbrot [<iterations>]
    "
    );
    std::process::exit(1);
}

fn handle_input(key_code: event::VirtualKeyCode, state: &mut State) -> bool {
    let mut redraw_needed = true;
    let step = TRANSFORM_STEP * state.scale;

    match key_code {
        event::VirtualKeyCode::A => state.center.0 -= step,
        event::VirtualKeyCode::D => state.center.0 += step,
        event::VirtualKeyCode::W => state.center.1 += step,
        event::VirtualKeyCode::S => state.center.1 -= step,
        event::VirtualKeyCode::Up => state.scale /= ZOOM_FACTOR,
        event::VirtualKeyCode::Down => state.scale *= ZOOM_FACTOR,
        event::VirtualKeyCode::Left => {
            if state.max_iterations > 200 {
                state.max_iterations -= 200
            }
        }
        event::VirtualKeyCode::Right => state.max_iterations += 200,
        event::VirtualKeyCode::I => println!("{}", state),
        _ => redraw_needed = false,
    }

    redraw_needed
}

fn main() {
    let mut args = std::env::args();
    args.next();

    let mut state = if args.len() == 5 {
        let values: Vec<_> = args
            .map(|s| s.parse::<f64>().unwrap_or_else(|_| usage()))
            .collect();

        State {
            window_size: (values[3] as u32, values[4] as u32),
            max_iterations: values[0] as u32,
            center: (values[1], values[2]),
            ..Default::default()
        }
    } else if args.len() == 3 {
        let values: Vec<_> = args
            .map(|s| s.parse::<f64>().unwrap_or_else(|_| usage()))
            .collect();

        State {
            max_iterations: values[0] as u32,
            center: (values[1], values[2]),
            ..Default::default()
        }
    } else if args.len() == 1 {
        State {
            max_iterations: args
                .next()
                .unwrap()
                .parse::<u32>()
                .unwrap_or_else(|_| usage()),
            ..Default::default()
        }
    } else if args.len() == 0 {
        State::default()
    } else {
        usage();
    };

    println!("{}", state);

    let event_loop = EventLoop::new();

    #[cfg(not(feature = "gl"))]
    let (window, size, surface) = {
        let window = winit::window::WindowBuilder::new()
            .with_inner_size(winit::dpi::PhysicalSize::<u32>::from(state.window_size))
            .with_title("Mandelbrot Visualization")
            .with_resizable(false)
            .build(&event_loop)
            .unwrap();
        let size = window.inner_size();
        let surface = wgpu::Surface::create(&window);
        (window, size, surface)
    };

    #[cfg(feature = "gl")]
    let (window, instance, size, surface) = {
        let wb = winit::window::WindowBuilder::new()
            .with_inner_size(winit::dpi::PhysicalSize::<u32>::from(state.window_size))
            .with_title("Mandelbrot Visualization")
            .with_resizable(false);
        let cb = wgpu::glutin::ContextBuilder::new().with_vsync(true);
        let context = cb.build_windowed(wb, &event_loop).unwrap();

        let size = context
            .window()
            .get_inner_size()
            .unwrap()
            .to_physical(context.window().get_hidpi_factor());

        let (context, window) = unsafe { context.make_current().unwrap().split() };

        let instance = wgpu::Instance::new(context);
        let surface = instance.get_surface();

        (window, instance, size, surface)
    };

    let adapter = wgpu::Adapter::request(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::Default,
        },
        wgpu::BackendBit::PRIMARY,
    )
    .unwrap();

    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
        limits: wgpu::Limits::default(),
    });

    let vs_module = shaders::vertex_shader_module(&device);
    let fs_module = shaders::fragment_shader_module(&device);

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        bindings: &[wgpu::BindGroupLayoutBinding {
            binding: 0,
            visibility: wgpu::ShaderStage::FRAGMENT,
            ty: wgpu::BindingType::UniformBuffer { dynamic: false },
        }],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
    });

    let buffer = device.create_buffer_with_data(
        state.fragment_uniform().as_bytes(),
        wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
    );

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        bindings: &[wgpu::Binding {
            binding: 0,
            resource: wgpu::BindingResource::Buffer {
                buffer: &buffer,
                range: 0..std::mem::size_of::<FragmentUniform>() as wgpu::BufferAddress,
            },
        }],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        layout: &pipeline_layout,
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        }),
        rasterization_state: Some(wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
        }),
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        color_states: &[wgpu::ColorStateDescriptor {
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }],
        depth_stencil_state: None,
        index_format: wgpu::IndexFormat::Uint16,
        vertex_buffers: &[],
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Vsync,
    };

    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    let mut redraw = true;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            event::Event::MainEventsCleared => window.request_redraw(),
            event::Event::WindowEvent {
                event: event::WindowEvent::Resized(size),
                ..
            } => {
                sc_desc.width = size.width;
                sc_desc.height = size.height;
                swap_chain = device.create_swap_chain(&surface, &sc_desc);
            }
            event::Event::RedrawRequested(_) => {
                if !redraw {
                    return;
                }

                let frame = swap_chain
                    .get_next_texture()
                    .expect("Timeout when acquiring next swap chain texture");

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

                if redraw {
                    let new_buffer = device.create_buffer_with_data(
                        state.fragment_uniform().as_bytes(),
                        wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC,
                    );
                    encoder.copy_buffer_to_buffer(
                        &new_buffer,
                        0,
                        &buffer,
                        0,
                        std::mem::size_of::<FragmentUniform>() as wgpu::BufferAddress,
                    );
                    redraw = false;
                }

                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &frame.view,
                            resolve_target: None,
                            load_op: wgpu::LoadOp::Clear,
                            store_op: wgpu::StoreOp::Store,
                            clear_color: wgpu::Color::GREEN,
                        }],
                        depth_stencil_attachment: None,
                    });
                    rpass.set_pipeline(&render_pipeline);
                    rpass.set_bind_group(0, &bind_group, &[]);
                    rpass.draw(0..3, 0..1);
                    rpass.draw(2..5, 0..1);
                }

                queue.submit(&[encoder.finish()]);
            }
            event::Event::WindowEvent {
                event:
                    event::WindowEvent::KeyboardInput {
                        input:
                            event::KeyboardInput {
                                virtual_keycode: Some(key_code),
                                state: event::ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                if key_code == event::VirtualKeyCode::Escape {
                    *control_flow = ControlFlow::Exit;
                } else {
                    redraw = handle_input(key_code, &mut state);
                }
            }
            event::Event::WindowEvent {
                event: event::WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
