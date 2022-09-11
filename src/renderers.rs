use std::{mem::size_of, num::NonZeroU32};

use glam::{Mat4, Quat, Vec3};
use image::{DynamicImage, GenericImageView};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, Buffer, RenderPipeline, Sampler,
};

use crate::{graphics::Renderer, GameData};

pub struct SimpleRenderer {
    color_pipeline: RenderPipeline,
    texture_pipeline: RenderPipeline,
    pub nearest_sampler: Sampler,
    pub models: Vec<Model>,
}

impl SimpleRenderer {
    pub fn new(data: &GameData) -> Self {
        Self {
            color_pipeline: Self::color_pipeline(data),
            texture_pipeline: Self::texture_pipeline(data),
            nearest_sampler: Self::nearest_sampler(data),
            models: Vec::new(),
        }
    }

    pub fn color_pipeline(data: &GameData) -> RenderPipeline {
        let shader =
            data.graphics
                .lock()
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Wgsl(include_str!("simple.wgsl").into()),
                });

        let color_pipeline_layout =
            data.graphics
                .lock()
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let fragment_format = data.graphics.lock().config.format;

        data.graphics
            .lock()
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Tri Color Pipeline"),
                layout: Some(&color_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "color_vertex",
                    buffers: &[ColorVertex::desc(), Transform::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "color_fragment",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: fragment_format,
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

    pub fn texture_pipeline(data: &GameData) -> RenderPipeline {
        let texture_bind_group_layout = Texture::bind_group_layout(data);

        let shader =
            data.graphics
                .lock()
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Wgsl(include_str!("simple.wgsl").into()),
                });

        let texture_pipeline_layout =
            data.graphics
                .lock()
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&texture_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let fragment_format = data.graphics.lock().config.format;

        data.graphics
            .lock()
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Tri Texture Pipeline"),
                layout: Some(&texture_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "texture_vertex",
                    buffers: &[TextureVertex::desc(), Transform::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "texture_fragment",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: fragment_format,
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

    pub fn nearest_sampler(data: &GameData) -> Sampler {
        data.graphics
            .lock()
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            })
    }
}

impl Renderer for SimpleRenderer {
    fn render<'a>(&'a self, mut render_pass: wgpu::RenderPass<'a>) {
        for model in &self.models {
            match &model.vertex_type {
                VertexType::ColorVertex => render_pass.set_pipeline(&self.color_pipeline),
                VertexType::TextureVertex(texture) => {
                    render_pass.set_pipeline(&self.texture_pipeline);
                    render_pass.set_bind_group(0, &texture.diffuse, &[]);
                }
            }
            render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, model.transform_buffer.slice(..));
            render_pass.set_index_buffer(model.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..model.index_count, 0, 0..model.transform_count);
        }
    }
}

pub enum VertexSlice<'a> {
    ColorVertices(&'a [ColorVertex]),
    TextureVertices(&'a [TextureVertex], Texture),
}

impl VertexSlice<'_> {
    pub fn contents(&self) -> &[u8] {
        match *self {
            Self::ColorVertices(vertices) => bytemuck::cast_slice(vertices),
            Self::TextureVertices(vertices, ..) => bytemuck::cast_slice(vertices),
        }
    }
}

pub enum VertexType {
    ColorVertex,
    TextureVertex(Texture),
}

impl From<VertexSlice<'_>> for VertexType {
    fn from(slice: VertexSlice) -> Self {
        match slice {
            VertexSlice::ColorVertices(..) => Self::ColorVertex,
            VertexSlice::TextureVertices(_, texture) => Self::TextureVertex(texture),
        }
    }
}

pub struct Model {
    pub vertex_buffer: Buffer,
    pub vertex_type: VertexType,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub transform_buffer: Buffer,
    pub transform_count: u32,
}

impl Model {
    pub fn new(
        data: &GameData,
        vertices: VertexSlice,
        indices: &[u16],
        transforms: &[[[f32; 4]; 4]],
    ) -> Self {
        let vertex_buffer = data
            .graphics
            .lock()
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: vertices.contents(),
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
            vertex_type: vertices.into(),
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

pub struct Texture {
    pub diffuse: BindGroup,
}

impl Texture {
    pub fn new(data: &GameData, image: &DynamicImage, sampler: &Sampler) -> Texture {
        let (width, height) = image.dimensions();

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let diffuse_texture =
            data.graphics
                .lock()
                .device
                .create_texture(&wgpu::TextureDescriptor {
                    label: None,
                    size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                });

        data.graphics.lock().queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            image.as_rgba8().unwrap(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * width),
                rows_per_image: NonZeroU32::new(height),
            },
            size,
        );

        let bind_group_layout = Texture::bind_group_layout(data);

        let diffuse = data
            .graphics
            .lock()
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(sampler),
                    },
                ],
            });

        Texture { diffuse }
    }

    pub fn bind_group_layout(data: &GameData) -> wgpu::BindGroupLayout {
        let graphics = data.graphics.lock();
        graphics
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            })
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
