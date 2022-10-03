//! Code specialised in handling graphics. Most of this is universally applicable.

use wgpu::{CommandEncoder, Device, Queue, RenderPass, Surface, SurfaceConfiguration, TextureView};
use winit::{dpi::PhysicalSize, window::Window};

use crate::GameData;

/// A handler over all core graphics components.
pub struct Graphics {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface,
    pub config: SurfaceConfiguration,
}

impl Graphics {
    pub(crate) async fn new(window: &Window) -> Self {
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
            let mut render_pass = renderer.make_render_pass(&view, &mut encoder);
            renderer.render(&mut render_pass);
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

#[allow(unused)]
/// This trait must be implemented on all renderers. It exposes API for rendering a frame.
pub trait Renderer {
    /// Make the default render pass. Most simple renderers don't need to replace this.
    /// This is called at the beginning of each frame.
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

    /// This is called every frame once the renderpass has been created.
    fn render<'a, 'b: 'a>(&'b self, render_pass: &'a mut RenderPass<'b>) {}
    /// This is called every frame after the game updates. This is for any state that the renderer
    /// itself will have to maintain.
    fn update(&mut self, data: &GameData) {}
    /// This is called when the canvas is resized. This is for things such as updating the size of
    /// a perspective projection aspect ratio.
    fn resize(&mut self, data: &GameData) {}
}

/// A renderer that does nothing, useful for some tests.
pub struct EmptyRenderer;

impl Renderer for EmptyRenderer {}
