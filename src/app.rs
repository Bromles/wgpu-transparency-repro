use std::sync::Arc;

use tokio::runtime::Runtime;
use wgpu::{Device, Queue, RenderPipeline, Surface, SurfaceConfiguration};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::window::Window;

pub struct App {
    pub window: Option<Arc<Window>>,
    pub runtime: Runtime,
    pub render_pipeline: Option<RenderPipeline>,
    pub surface: Option<Surface<'static>>,
    pub surface_config: Option<SurfaceConfiguration>,
    pub device: Option<Device>,
    pub queue: Option<Queue>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attr = Window::default_attributes().with_transparent(true);
        self.window = Some(Arc::new(event_loop.create_window(window_attr).unwrap()));
        let Some(ref window) = self.window else {
            return;
        };

        let size = window.inner_size();

        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = self
            .runtime
            .block_on(async {
                instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::default(),
                        force_fallback_adapter: false,
                        // Request an adapter which can render to our surface
                        compatible_surface: Some(&surface),
                    })
                    .await
            })
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = self
            .runtime
            .block_on(async {
                adapter
                    .request_device(
                        &wgpu::DeviceDescriptor {
                            label: None,
                            required_features: wgpu::Features::empty(),
                            required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                                .using_resolution(adapter.limits()),
                        },
                        None,
                    )
                    .await
            })
            .expect("Failed to create device");

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats[0];

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(surface_format.into())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        self.device = Some(device);
        self.surface = Some(surface);
        self.surface_config = Some(surface_config);
        self.render_pipeline = Some(render_pipeline);
        self.queue = Some(queue);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(new_size) => {
                // Reconfigure the surface with the new size
                self.surface_config.as_mut().unwrap().width = new_size.width.max(1);
                self.surface_config.as_mut().unwrap().height = new_size.height.max(1);
                self.surface.as_mut().unwrap().configure(
                    self.device.as_ref().unwrap(),
                    self.surface_config.as_ref().unwrap(),
                );
                // On macos the window needs to be redrawn manually after resizing
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let frame = self
                    .surface
                    .as_ref()
                    .unwrap()
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = self
                    .device
                    .as_ref()
                    .unwrap()
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    rpass.set_pipeline(self.render_pipeline.as_ref().unwrap());
                    rpass.draw(0..3, 0..1);
                }

                self.queue.as_ref().unwrap().submit(Some(encoder.finish()));
                frame.present();
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }
    }
}
