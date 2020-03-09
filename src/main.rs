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
    pixel_delta: f64,
    center_re: f64,
    center_im: f64,
}

fn main() {
    let event_loop = EventLoop::new();

    #[cfg(not(feature = "gl"))]
    let (window, size, surface) = {
        let window = winit::window::Window::new(&event_loop).unwrap();
        let size = window.inner_size();
        let surface = wgpu::Surface::create(&window);
        (window, size, surface)
    };

    #[cfg(feature = "gl")]
    let (window, instance, size, surface) = {
        let wb = winit::WindowBuilder::new();
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

    let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::Default,
        backends: wgpu::BackendBit::PRIMARY,
    })
    .unwrap();

    let (device, mut queue) = adapter.request_device(&wgpu::DeviceDescriptor {
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

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        size: std::mem::size_of::<Locals>() as wgpu::BufferAddress,
        usage: wgpu::BufferUsage::UNIFORM
            | wgpu::BufferUsage::COPY_DST
            | wgpu::BufferUsage::COPY_SRC,
    });

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

    let mut locals = Locals {
        screen_width: size.width as f64,
        screen_height: size.height as f64,
        max_iterations: 1000.0,
        pixel_delta: 0.003_141_5,
        center_re: -0.5,
        center_im: 0.0,
    };

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
                let frame = swap_chain.get_next_texture();

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

                let b = device
                    .create_buffer_mapped(
                        std::mem::size_of::<Locals>(),
                        wgpu::BufferUsage::UNIFORM
                            | wgpu::BufferUsage::COPY_DST
                            | wgpu::BufferUsage::COPY_SRC,
                    )
                    .fill_from_slice(locals.as_bytes());
                encoder.copy_buffer_to_buffer(
                    &b,
                    0,
                    &buffer,
                    0,
                    std::mem::size_of::<Locals>() as u64,
                );

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
                    handle_input(key, &mut locals);
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

fn handle_input(key_code: event::VirtualKeyCode, locals: &mut Locals) {
    let shortest_dim = if locals.screen_width < locals.screen_height {
        locals.screen_width
    } else {
        locals.screen_height
    };

    let step = locals.pixel_delta * f64::from(shortest_dim / 100.0);
    let mut transform_re = 0.0;
    let mut transform_im = 0.0;

    if let event::VirtualKeyCode::A = key_code {
        transform_re -= step;
    }
    if let event::VirtualKeyCode::D = key_code {
        transform_re += step;
    }
    if let event::VirtualKeyCode::W = key_code {
        transform_im += step;
    }
    if let event::VirtualKeyCode::S = key_code {
        transform_im -= step;
    }

    if let event::VirtualKeyCode::Up = key_code {
        locals.pixel_delta /= ZOOM_FACTOR;
    }
    if let event::VirtualKeyCode::Down = key_code {
        locals.pixel_delta *= ZOOM_FACTOR;
    }

    locals.center_re += transform_re;
    locals.center_im += transform_im;
}
