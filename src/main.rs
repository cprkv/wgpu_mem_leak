use std::mem;

use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

struct FrameResources {
    buffers: Vec<wgpu::Buffer>,
    bind_groups: Vec<wgpu::BindGroup>,
    current_frame: usize,
}

impl FrameResources {
    fn new(device: &wgpu::Device, size: u64, layout: &wgpu::BindGroupLayout) -> Self {
        let mut buffers = Vec::new();
        let mut bind_groups = Vec::new();

        for _ in 0..3 {
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                size,
                mapped_at_creation: false,
            });
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
                label: None,
            });
            buffers.push(buffer);
            bind_groups.push(bind_group);
        }

        Self {
            buffers,
            bind_groups,
            current_frame: 0,
        }
    }

    fn get_current_buffer(&self) -> &wgpu::Buffer {
        &self.buffers[self.current_frame]
    }

    fn get_current_bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_groups[self.current_frame]
    }

    fn next_frame(&mut self) {
        self.current_frame += 1;
        if self.current_frame >= self.buffers.len() {
            self.current_frame = 0;
        }
    }
}

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    frame_resources: FrameResources,
    frame: u64,
    uniform_aligned_size: u64,
    window: &'a Window,
}

impl<'a> State<'a> {
    async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .enumerate_adapters(wgpu::Backends::all())
            .into_iter()
            .filter(|adapter| adapter.is_surface_supported(&surface))
            .next()
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();

        let uniform_size = (mem::size_of::<f32>() * 2) as u64;
        let uniform_alignment = device.limits().min_uniform_buffer_offset_alignment as u64;
        let uniform_aligned_size = uniform_size * uniform_alignment;

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(uniform_size),
                },
                count: None,
            }],
        });

        let frame_resources =
            FrameResources::new(&device, uniform_aligned_size, &bind_group_layout);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let frame = 0;

        println!("state created!");

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            frame_resources,
            uniform_aligned_size,
            frame,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let buffer = &self.frame_resources.get_current_buffer();
        let bind_group = &self.frame_resources.get_current_bind_group();

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        self.frame = self.frame + 1;
        let factor = (self.frame as f32) * 0.01_f32;
        if factor > std::f32::consts::PI * 2_f32 {
            self.frame = 0
        }
        let uniform_data = [factor.sin(), factor.cos()];

        self.queue.write_buffer(&buffer, 0, unsafe {
            std::slice::from_raw_parts(
                uniform_data.as_ptr() as *const u8,
                self.uniform_aligned_size as usize,
            )
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..wgpu::RenderPassDescriptor::default()
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.frame_resources.next_frame();

        Ok(())
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = State::new(&window).await;

    event_loop
        .run(move |event, control_flow| match event {
            Event::AboutToWait => {
                state.window().request_redraw();
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                    ..
                } => control_flow.exit(),
                WindowEvent::Resized(physical_size) => {
                    state.resize(*physical_size);
                }
                WindowEvent::RedrawRequested => match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                    Err(e) => eprintln!("{:?}", e),
                },
                _ => {}
            },
            _ => {}
        })
        .unwrap();
}

fn main() {
    pollster::block_on(run());
}
