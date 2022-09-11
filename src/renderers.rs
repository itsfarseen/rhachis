use std::mem::size_of;

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, RenderPipeline,
};

use crate::{graphics::Renderer, GameData};

pub struct SimpleRenderer {
    pipeline: RenderPipeline,
    pub models: Vec<Model>,
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
            models: Vec::new(),
        }
    }
}

impl Renderer for SimpleRenderer {
    fn render<'a>(&'a self, mut render_pass: wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        for model in &self.models {
            render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
            render_pass.draw(0..3, 0..1);
        }
    }
}

pub struct Model {
    pub vertex_buffer: Buffer,
}

impl Model {
    pub fn new_color(data: &GameData, vertices: &[ColorVertex]) -> Self {
        Self {
            vertex_buffer: data
                .graphics
                .lock()
                .device
                .create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
        }
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
