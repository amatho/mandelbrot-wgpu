use winit::{
    event,
    event_loop::{ControlFlow, EventLoop},
};

use zerocopy::{AsBytes, FromBytes};

const ZOOM_FACTOR: f64 = 1.05;

#[derive(Copy, Clone, AsBytes, FromBytes)]
#[repr(C)]
struct Locals {
    screen_width: f64,
    screen_height: f64,
    max_iterations: f64,
    scale: f64,
    center_re: f64,
    center_im: f64,
}

fn usage() -> ! {
    println!(
        "Usage:
    mandelbrot <width> <height>"
    );
    std::process::exit(1);
}

fn main() {
    let args = std::env::args();
    let (requested_width, requested_height) = if args.len() == 3 {
        let values: Vec<_> = args
            .skip(1)
            .map(|s| s.parse::<u32>().unwrap_or_else(|_| usage()))
            .collect();
        (values[0], values[1])
    } else {
        (960, 720)
    };

    let event_loop = EventLoop::new();

    #[cfg(not(feature = "gl"))]
    let (window, size, surface) = {
        let window = winit::window::WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize::new(
                requested_width,
                requested_height,
            ))
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
            .with_inner_size(winit::dpi::LogicalSize::new(
                requested_width,
                requested_height,
            ))
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

    let vs = wgpu::read_spirv(
        glsl_to_spirv::compile(
            include_str!("shader.vert"),
            glsl_to_spirv::ShaderType::Vertex,
        )
        .unwrap(),
    )
    .unwrap();
    let vs_module = device.create_shader_module(&vs);

    let fs = wgpu::read_spirv(
        glsl_to_spirv::compile(
            include_str!("shader.frag"),
            glsl_to_spirv::ShaderType::Fragment,
        )
        .unwrap(),
    )
    .unwrap();
    let fs_module = device.create_shader_module(&fs);

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

    let mut locals = Locals {
        screen_width: size.width as f64,
        screen_height: size.height as f64,
        max_iterations: 1000.0,
        scale: 0.003,
        center_re: -0.5,
        center_im: 0.0,
    };
    let buffer = device.create_buffer_with_data(
        locals.as_bytes(),
        wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
    );

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        bindings: &[wgpu::Binding {
            binding: 0,
            resource: wgpu::BindingResource::Buffer {
                buffer: &buffer,
                range: 0..std::mem::size_of::<Locals>() as wgpu::BufferAddress,
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
                let frame = swap_chain
                    .get_next_texture()
                    .expect("Timeout when acquiring next swap chain texture");

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

                if redraw {
                    let new_buffer = device.create_buffer_with_data(
                        locals.as_bytes(),
                        wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC,
                    );
                    encoder.copy_buffer_to_buffer(
                        &new_buffer,
                        0,
                        &buffer,
                        0,
                        std::mem::size_of::<Locals>() as wgpu::BufferAddress,
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
            event::Event::DeviceEvent {
                event:
                    event::DeviceEvent::Key(event::KeyboardInput {
                        state: event::ElementState::Pressed,
                        virtual_keycode: Some(key),
                        ..
                    }),
                ..
            } => {
                if key == event::VirtualKeyCode::Escape {
                    *control_flow = ControlFlow::Exit;
                } else {
                    redraw = handle_input(key, &mut locals);
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

fn handle_input(key_code: event::VirtualKeyCode, locals: &mut Locals) -> bool {
    let mut redraw_needed = true;
    let shortest_dim = if locals.screen_width < locals.screen_height {
        locals.screen_width
    } else {
        locals.screen_height
    };
    let step = locals.scale * shortest_dim / 100.0;

    match key_code {
        event::VirtualKeyCode::A => locals.center_re -= step,
        event::VirtualKeyCode::D => locals.center_re += step,
        event::VirtualKeyCode::W => locals.center_im += step,
        event::VirtualKeyCode::S => locals.center_im -= step,
        event::VirtualKeyCode::Up => locals.scale /= ZOOM_FACTOR,
        event::VirtualKeyCode::Down => locals.scale *= ZOOM_FACTOR,
        _ => redraw_needed = false,
    }

    redraw_needed
}
