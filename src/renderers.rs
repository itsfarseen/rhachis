use std::mem::size_of;

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, RenderPipeline,
};

use crate::{graphics::Renderer, GameData};

pub struct SimpleRenderer {
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
}

impl SimpleRenderer {
    pub fn new(data: &GameData) -> Self {
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

impl Renderer for SimpleRenderer {
    fn render<'a>(&'a self, mut render_pass: wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..3, 0..1);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorVertex {
    pub pos: [f32; 3],
    pub color: [f32; 4],
}

impl ColorVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: size_of::<[f32; 3]>() as u64,
                    shader_location: 1,
                },
            ],
        }
    }
}
