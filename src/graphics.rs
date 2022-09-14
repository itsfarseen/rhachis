use wgpu::{CommandEncoder, Device, Queue, RenderPass, Surface, SurfaceConfiguration, TextureView};
use winit::{dpi::PhysicalSize, window::Window};

use crate::GameData;

pub struct Graphics {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface,
    pub config: SurfaceConfiguration,
}

impl Graphics {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        Self {
            device,
            queue,
            surface,
            config,
        }
    }

    pub(crate) fn render(&mut self, renderer: &mut dyn Renderer) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let render_pass = renderer.make_render_pass(&view, &mut encoder);
            renderer.render(render_pass);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub(crate) fn resize(&mut self, size: PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }
}

pub trait Renderer {
    fn make_render_pass<'a>(
        &'a self,
        view: &'a TextureView,
        encoder: &'a mut CommandEncoder,
    ) -> RenderPass {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        })
    }

    fn render(&self, _: RenderPass) {}
    fn update(&mut self, _: &GameData) {}
    fn resize(&mut self, _: &GameData) {}
}

pub struct EmptyRenderer;

impl Renderer for EmptyRenderer {}
