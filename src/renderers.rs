use std::{f32::consts::TAU, fmt::Debug, mem::size_of, num::NonZeroU32, path::Path};

use anyhow::Result;
use glam::{Mat4, Quat, Vec3};
use image::{DynamicImage, GenericImageView};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, Buffer, RenderPipeline, Sampler, TextureView,
};

use crate::{graphics::Renderer, GameData};

pub enum SimpleProjection {
    Orthographic,
    Perspective { aspect_ratio: f32 },
    Other(Mat4),
}

impl SimpleProjection {
    pub fn new_perspective(data: &GameData) -> Self {
        let size = data.window.lock().inner_size();
        let aspect_ratio = size.width as f32 / size.height as f32;
        Self::Perspective { aspect_ratio }
    }
}

impl From<SimpleProjection> for Mat4 {
    fn from(proj: SimpleProjection) -> Self {
        match proj {
            SimpleProjection::Orthographic => {
                Mat4::orthographic_rh(-1.0, 1.0, -1.0, 1.0, 0.1, 100.0)
            }
            SimpleProjection::Perspective { aspect_ratio } => {
                Mat4::perspective_rh(TAU / 4.0, aspect_ratio, 0.1, 100.0)
            }
            SimpleProjection::Other(proj) => proj,
        }
    }
}

pub struct SimpleRenderer {
    color_pipeline: RenderPipeline,
    texture_pipeline: RenderPipeline,
    projection_buffer: Buffer,
    projection_bind_group: BindGroup,
    pub depth_texture_view: TextureView,
    pub nearest_sampler: Sampler,
    pub linear_sampler: Sampler,
    pub models: Vec<Model>,
}

impl SimpleRenderer {
    pub fn new(data: &GameData, projection: SimpleProjection) -> Self {
        let projection = Mat4::from(projection);
        let projection_bind_group_layout = Self::mat4_bind_group_layout(data);

        let projection_buffer =
            data.graphics
                .lock()
                .device
                .create_buffer_init(&BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&[projection.to_cols_array_2d()]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let projection_bind_group =
            data.graphics
                .lock()
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: projection_buffer.as_entire_binding(),
                    }],
                    layout: &projection_bind_group_layout,
                });

        Self {
            color_pipeline: Self::color_pipeline(data),
            texture_pipeline: Self::texture_pipeline(data),
            projection_buffer,
            projection_bind_group,
            depth_texture_view: Self::depth_texture(data),
            nearest_sampler: Self::nearest_sampler(data),
            linear_sampler: Self::linear_sampler(data),
            models: Vec::new(),
        }
    }

    pub fn set_projection(&mut self, data: &GameData, projection: Mat4) {
        data.graphics.lock().queue.write_buffer(
            &self.projection_buffer,
            0,
            bytemuck::cast_slice(&[projection.to_cols_array_2d()]),
        )
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

        let mat4_bind_group_layout = Self::mat4_bind_group_layout(data);

        let color_pipeline_layout =
            data.graphics
                .lock()
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&mat4_bind_group_layout],
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
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
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

        let mat4_bind_group_layout = Self::mat4_bind_group_layout(data);

        let texture_pipeline_layout =
            data.graphics
                .lock()
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&mat4_bind_group_layout, &texture_bind_group_layout],
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
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            })
    }

    pub fn linear_sampler(data: &GameData) -> Sampler {
        data.graphics
            .lock()
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
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

    pub fn mat4_bind_group_layout(data: &GameData) -> wgpu::BindGroupLayout {
        data.graphics
            .lock()
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            })
    }

    pub fn depth_texture(data: &GameData) -> TextureView {
        let width = data.graphics.lock().config.width;
        let height = data.graphics.lock().config.height;

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = data
            .graphics
            .lock()
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
            });
        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
}

impl Renderer for SimpleRenderer {
    fn render<'a>(&'a self, mut render_pass: wgpu::RenderPass<'a>) {
        render_pass.set_bind_group(0, &self.projection_bind_group, &[]);
        for model in &self.models {
            match &model.vertex_type {
                VertexType::ColorVertex => render_pass.set_pipeline(&self.color_pipeline),
                VertexType::TextureVertex(texture) => {
                    render_pass.set_pipeline(&self.texture_pipeline);
                    render_pass.set_bind_group(1, &texture.diffuse, &[]);
                }
            }
            render_pass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, model.transform_buffer.slice(..));
            render_pass.set_index_buffer(model.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..model.index_count, 0, 0..model.transform_count);
        }
    }

    fn update(&mut self, data: &GameData) {
        for model in &mut self.models {
            if model.transforms_outdated {
                model.update_transforms(data);
            }
        }
    }

    fn make_render_pass<'a>(
        &'a self,
        view: &'a TextureView,
        encoder: &'a mut wgpu::CommandEncoder,
    ) -> wgpu::RenderPass {
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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        })
    }

    fn resize(&mut self, data: &GameData) {
        self.depth_texture_view = Self::depth_texture(data);
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
    pub transforms: Vec<Transform>,
    pub transforms_outdated: bool,
    pub transform_buffer: Buffer,
    pub transform_count: u32,
}

impl Model {
    pub fn new(
        data: &GameData,
        vertices: VertexSlice,
        indices: &[u16],
        transforms: Vec<Transform>,
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
                    contents: bytemuck::cast_slice(
                        &transforms
                            .iter()
                            .map(Transform::matrix)
                            .collect::<Vec<[[f32; 4]; 4]>>(),
                    ),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

        Self {
            vertex_buffer,
            vertex_type: vertices.into(),
            index_buffer,
            index_count: indices.len() as u32,
            transform_count: transforms.len() as u32,
            transforms,
            transforms_outdated: false,
            transform_buffer,
        }
    }

    pub fn from_obj<P: AsRef<Path> + Debug>(
        data: &GameData,
        path: P,
        sampler: &Sampler,
        transforms: Vec<Transform>,
    ) -> Result<Vec<Self>> {
        let (models, materials) = tobj::load_obj(
            &path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )?;

        let to_ret = models
            .into_iter()
            .map(|model| {
                let indices = model
                    .mesh
                    .indices
                    .into_iter()
                    .map(|x| x as u16)
                    .collect::<Vec<u16>>();

                match model.mesh.material_id {
                    Some(..) => {
                        let positions = model.mesh.positions.chunks(3);
                        let tex_coords = model.mesh.texcoords.chunks(2);

                        let vertices = iter_tools::zip(positions, tex_coords)
                            .map(|(pos, tex_coords)| TextureVertex {
                                pos: [pos[0], pos[1], pos[2]],
                                tex_coords: [tex_coords[0], -tex_coords[1]],
                            })
                            .collect::<Vec<TextureVertex>>();

                        let texture_path = &materials.as_ref().unwrap()[0].diffuse_texture;
                        let texture =
                            Texture::new(data, &image::open(&texture_path).unwrap(), sampler);

                        Self::new(
                            data,
                            VertexSlice::TextureVertices(&vertices, texture),
                            &indices,
                            transforms.clone(),
                        )
                    }
                    None => {
                        let vertices = model
                            .mesh
                            .positions
                            .chunks(3)
                            .map(|pos| ColorVertex {
                                pos: [pos[0], pos[1], pos[2]],
                                color: [1.0, 1.0, 1.0, 1.0],
                            })
                            .collect::<Vec<ColorVertex>>();

                        Self::new(
                            data,
                            VertexSlice::ColorVertices(&vertices),
                            &indices,
                            transforms.clone(),
                        )
                    }
                }
            })
            .collect();

        Ok(to_ret)
    }

    pub fn quad(data: &GameData, transforms: Vec<Transform>) -> Self {
        Self::new(
            data,
            VertexSlice::ColorVertices(&[
                ColorVertex {
                    pos: [0.0, 0.0, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                },
                ColorVertex {
                    pos: [1.0, 0.0, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                },
                ColorVertex {
                    pos: [0.0, 1.0, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                },
                ColorVertex {
                    pos: [1.0, 1.0, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                },
            ]),
            &[0, 1, 2, 1, 3, 2],
            transforms,
        )
    }

    pub fn quad_texture(data: &GameData, texture: Texture, transforms: Vec<Transform>) -> Self {
        Self::new(
            data,
            VertexSlice::TextureVertices(
                &[
                    TextureVertex {
                        pos: [0.0, 0.0, 0.0],
                        tex_coords: [0.0, 1.0],
                    },
                    TextureVertex {
                        pos: [1.0, 0.0, 0.0],
                        tex_coords: [1.0, 1.0],
                    },
                    TextureVertex {
                        pos: [0.0, 1.0, 0.0],
                        tex_coords: [0.0, 0.0],
                    },
                    TextureVertex {
                        pos: [1.0, 1.0, 0.0],
                        tex_coords: [1.0, 0.0],
                    },
                ],
                texture,
            ),
            &[0, 1, 2, 1, 3, 2],
            transforms,
        )
    }

    pub fn update_transforms(&mut self, data: &GameData) {
        if self.transforms.len() as u32 != self.transform_count {
            self.transform_buffer =
                data.graphics
                    .lock()
                    .device
                    .create_buffer_init(&BufferInitDescriptor {
                        label: None,
                        contents: bytemuck::cast_slice(
                            &self
                                .transforms
                                .iter()
                                .map(Transform::matrix)
                                .collect::<Vec<[[f32; 4]; 4]>>(),
                        ),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    });
            self.transform_count = self.transforms.len() as u32;
        } else {
            data.graphics.lock().queue.write_buffer(
                &self.transform_buffer,
                0,
                bytemuck::cast_slice(
                    &self
                        .transforms
                        .iter()
                        .map(Transform::matrix)
                        .collect::<Vec<[[f32; 4]; 4]>>(),
                ),
            );
        }

        self.transforms_outdated = false;
    }

    pub fn set_transform(&mut self, index: usize, transform: Transform) {
        *self.transforms.get_mut(index).unwrap() = transform;
        self.transforms_outdated = true;
    }

    pub fn with_transform(mut self, index: usize, transform: Transform) -> Self {
        *self.transforms.get_mut(index).unwrap() = transform;
        self.transforms_outdated = true;
        self
    }

    pub fn set_transforms(&mut self, transforms: Vec<Transform>) {
        self.transforms = transforms;
        self.transforms_outdated = true;
    }

    pub fn with_transforms(mut self, transforms: Vec<Transform>) -> Self {
        self.transforms = transforms;
        self.transforms_outdated = true;
        self
    }

    pub fn modify_transforms<F: FnOnce(&mut [Transform])>(&mut self, modify: F) {
        modify(&mut self.transforms);
        self.transforms_outdated = true;
    }

    pub fn with_modify_transforms<F: FnOnce(&mut [Transform])>(mut self, modify: F) -> Self {
        modify(&mut self.transforms);
        self.transforms_outdated = true;
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

macro_rules! transform_methods {
    ($($i: ident: $t: ty),*) => {
        $(
            paste::paste! {
                pub fn [<set_ $i>](&mut self, $i: $t) {
                    self.$i = $i;
                }

                pub fn [<with_ $i>](mut self, $i: $t) -> Self {
                    self.$i = $i;
                    self
                }

                pub fn $i($i: $t) -> Self {
                    Self {
                        $i,
                        ..Default::default()
                    }
                }
            }
        )*
    };
}

macro_rules! with_set_translation {
    ($($i: ident),*) => {
        $(
            paste::paste! {
                pub fn [<set_ $i>](&mut self, $i: f32) {
                    self.translation.$i = $i;
                }

                pub fn [<with_ $i>](mut self, $i: f32) -> Self {
                    self.translation.$i = $i;
                    self
                }
            }
        )*
    };
}

impl Transform {
    transform_methods!(translation: Vec3, rotation: Quat, scale: Vec3);

    with_set_translation!(x, y, z);

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
