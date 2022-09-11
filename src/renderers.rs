use std::mem::size_of;

use glam::{Mat4, Quat, Vec3};
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
        Self {
            pipeline: Self::pipeline(data),
            models: Vec::new(),
        }
    }

    pub fn pipeline(data: &GameData) -> RenderPipeline {
        let graphics = data.graphics.lock();

        let shader = graphics
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(include_str!("simple.wgsl").into()),
            });

        graphics
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
                    buffers: &[ColorVertex::desc(), Transform::desc()],
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
            })
    }
}

impl Renderer for SimpleRenderer {
    fn render<'a>(&'a self, mut render_pass: wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        for model in &self.models {
            render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, model.transform_buffer.slice(..));
            render_pass.set_index_buffer(model.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..model.index_count, 0, 0..model.transform_count);
        }
    }
}

pub struct Model {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub transform_buffer: Buffer,
    pub transform_count: u32,
}

impl Model {
    pub fn new_color(
        data: &GameData,
        vertices: &[ColorVertex],
        indices: &[u16],
        transforms: &[[[f32; 4]; 4]],
    ) -> Self {
        let vertex_buffer = data
            .graphics
            .lock()
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = data
            .graphics
            .lock()
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });
        let transform_buffer =
            data.graphics
                .lock()
                .device
                .create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(transforms),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
            transform_buffer,
            transform_count: transforms.len() as u32,
        }
    }
}

pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn matrix(&self) -> [[f32; 4]; 4] {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
            .to_cols_array_2d()
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<[[f32; 4]; 4]>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: size_of::<[f32; 4]>() as u64,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: size_of::<[f32; 8]>() as u64,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: size_of::<[f32; 12]>() as u64,
                    shader_location: 5,
                },
            ],
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::new(1.0, 1.0, 1.0),
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

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextureVertex {
    pub pos: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl TextureVertex {
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
                    format: wgpu::VertexFormat::Float32x2,
                    offset: size_of::<[f32; 3]>() as u64,
                    shader_location: 1,
                },
            ],
        }
    }
}
