use crate::{
    shaders,
    state::{FragmentUniform, State},
};
use wgpu::{
    BindGroup, Buffer, Device, Queue, RenderPipeline, Surface, SwapChain, SwapChainDescriptor,
};
use winit::{
    event,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use zerocopy::AsBytes;

pub struct Renderer {
    surface: Surface,
    device: Device,
    queue: Queue,
    buffer: Buffer,
    bind_group: BindGroup,
    render_pipeline: RenderPipeline,
    sc_desc: SwapChainDescriptor,
    swap_chain: SwapChain,
    state: State,
}

impl Renderer {
    pub async fn new(state: State, window: &Window) -> Self {
        let surface = wgpu::Surface::create(window);

        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: wgpu::Limits::default(),
            })
            .await;

        let vs_module = shaders::vertex_shader_module(&device);
        let fs_module = shaders::fragment_shader_module(&device);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            }],
            label: None,
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
            label: None,
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
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: state.window_size.0,
            height: state.window_size.1,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        Renderer {
            surface,
            device,
            queue,
            buffer,
            bind_group,
            render_pipeline,
            sc_desc,
            swap_chain,
            state,
        }
    }

    pub fn run_event_loop(mut self, event_loop: EventLoop<()>, window: Window) {
        let mut redraw = true;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                event::Event::MainEventsCleared => window.request_redraw(),
                event::Event::WindowEvent {
                    event: event::WindowEvent::Resized(size),
                    ..
                } => {
                    self.sc_desc.width = size.width;
                    self.sc_desc.height = size.height;
                    self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
                }
                event::Event::RedrawRequested(_) => {
                    if !redraw {
                        return;
                    }

                    let frame = self
                        .swap_chain
                        .get_next_texture()
                        .expect("Timeout when acquiring next swap chain texture");

                    let mut encoder = self
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                    if redraw {
                        let new_buffer = self.device.create_buffer_with_data(
                            self.state.fragment_uniform().as_bytes(),
                            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_SRC,
                        );
                        encoder.copy_buffer_to_buffer(
                            &new_buffer,
                            0,
                            &self.buffer,
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
                        rpass.set_pipeline(&self.render_pipeline);
                        rpass.set_bind_group(0, &self.bind_group, &[]);
                        rpass.draw(0..3, 0..1);
                        rpass.draw(2..5, 0..1);
                    }

                    self.queue.submit(&[encoder.finish()]);
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
                        redraw = self.state.handle_input(key_code);
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
}
