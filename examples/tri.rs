use rhachis::{
    graphics::{ColorVertex, Renderer},
    *,
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, RenderPipeline,
};

fn main() {
    Tri::run();
}

struct Tri {
    renderer: TriRenderer,
}

impl Game for Tri {
    fn init(data: &GameData) -> Self {
        Tri {
            renderer: TriRenderer::new(data),
        }
    }

    fn get_renderer(&mut self) -> &mut dyn Renderer {
        &mut self.renderer
    }
}

struct TriRenderer {
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
}

impl TriRenderer {
    fn new(data: &GameData) -> Self {
        let graphics = data.graphics.lock();

        let shader = graphics
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("simple.wgsl").into()),
            });

        Self {
            pipeline: graphics
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Tri Pipeline"),
                    layout: Some(&graphics.device.create_pipeline_layout(
                        &wgpu::PipelineLayoutDescriptor {
                            label: None,
                            bind_group_layouts: &[],
                            push_constant_ranges: &[],
                        },
                    )),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "color_vertex",
                        buffers: &[ColorVertex::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "color_fragment",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: graphics.config.format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        unclipped_depth: false,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        conservative: false,
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                }),
            vertex_buffer: graphics.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Tri Vertices"),
                contents: bytemuck::cast_slice(&[
                    ColorVertex {
                        pos: [0.0, 0.0, 0.0],
                        color: [1.0, 1.0, 1.0, 1.0],
                    },
                    ColorVertex {
                        pos: [1.0, 0.0, 0.0],
                        color: [1.0, 1.0, 1.0, 1.0],
                    },
                    ColorVertex {
                        pos: [1.0, 1.0, 0.0],
                        color: [1.0, 1.0, 1.0, 1.0],
                    },
                ]),
                usage: wgpu::BufferUsages::VERTEX,
            }),
        }
    }
}

impl Renderer for TriRenderer {
    fn render<'a>(&'a self, mut render_pass: wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..3, 0..1);
    }
}
