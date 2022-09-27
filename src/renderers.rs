//! A collection of rendering structs that could be used for simple
//! or temporary parts of a pipeline.
//!
//! Enough code is written for this module to have an entire functional pipeline,
//! but only pieces may be used if needed.

use std::{
    collections::HashMap, f32::consts::TAU, fmt::Debug, hash::Hash, mem::size_of, num::NonZeroU32,
    path::Path,
};

use anyhow::Result;
use glam::{Mat4, Quat, Vec3};
use image::{DynamicImage, GenericImageView};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, Buffer, RenderPipeline, Sampler, TextureView,
};

use crate::{graphics::Renderer, GameData};

/// An enum offering simpler projection description for renderers.
pub enum SimpleProjection {
    /// A projection with no perspective. Useful for 2D games.
    Orthographic,
    /// A projection which distorts positions to look realistic and 3D.
    /// Use `SimpleProjection::new_perspective` to avoid setting `aspect_ratio`.
    Perspective {
        /// The aspect ratio of the screen.
        aspect_ratio: f32,
    },
    /// A custom projection matrix.
    Other(Mat4),
}

impl SimpleProjection {
    /// Returns a `SimpleProjection::Perspective` with an automatically determined
    /// `aspect_ratio`.
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
                Mat4::orthographic_rh(-1.0, 1.0, -1.0, 1.0, 0.0, 100.0)
            }
            SimpleProjection::Perspective { aspect_ratio } => {
                Mat4::perspective_rh(TAU / 4.0, aspect_ratio, 0.0, 100.0)
            }
            SimpleProjection::Other(proj) => proj,
        }
    }
}

/// A simple renderer with pipelines for both color vertices and texture vertices. No
/// lighting is performed.
pub struct SimpleRenderer {
    color_pipeline: RenderPipeline,
    texture_pipeline: RenderPipeline,
    camera_buffer: Buffer,
    projection_buffer: Buffer,
    projection_bind_group: BindGroup,
    /// A view of the depth texture.
    pub depth_texture_view: TextureView,
    /// A sampler for nearest filters (magnified textures looked pixelated).
    pub nearest_sampler: Sampler,
    /// A sampler for linear filters (magnified textures looked blurry).
    pub linear_sampler: Sampler,
    /// A list of all `Model`s that will be rendered.
    pub models: Vec<Model>,
}

impl SimpleRenderer {
    /// Create a `SimpleRenderer`.
    pub fn new(data: &GameData, projection: SimpleProjection) -> Self {
        let projection = Mat4::from(projection);
        let projection_bind_group_layout = Self::mat4_bind_group_layout(data);

        let camera_buffer = data
            .graphics
            .lock()
            .device
            .create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[Mat4::IDENTITY.to_cols_array_2d()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

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
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: projection_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: camera_buffer.as_entire_binding(),
                        },
                    ],
                    layout: &projection_bind_group_layout,
                });

        Self {
            color_pipeline: Self::color_pipeline(data),
            texture_pipeline: Self::texture_pipeline(data),
            camera_buffer,
            projection_buffer,
            projection_bind_group,
            depth_texture_view: Self::depth_texture(data),
            nearest_sampler: Self::nearest_sampler(data),
            linear_sampler: Self::linear_sampler(data),
            models: Vec::new(),
        }
    }

    /// Replaces the camera of the renderer and updates its
    /// buffer
    pub fn set_camera(&mut self, data: &GameData, camera: Mat4) {
        data.graphics.lock().queue.write_buffer(
            &self.camera_buffer,
            size_of::<[[f32; 4]; 4]>() as u64,
            bytemuck::cast_slice(&[camera.to_cols_array_2d()]),
        )
    }

    /// Replaces the projection of the renderer and updates its
    /// buffer
    pub fn set_projection(&mut self, data: &GameData, projection: Mat4) {
        data.graphics.lock().queue.write_buffer(
            &self.projection_buffer,
            0,
            bytemuck::cast_slice(&[projection.to_cols_array_2d()]),
        )
    }

    /// Makes the default color pipeline.
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

    /// Makes the default color pipeline.
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

    /// Makes the default linear sampler.
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

    /// Makes the default nearest sampler.
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

    /// Makes the default 4x4 matrix bind group layout.
    pub fn mat4_bind_group_layout(data: &GameData) -> wgpu::BindGroupLayout {
        data.graphics
            .lock()
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            })
    }

    /// Makes the default `TextureView` of a depth texture.
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

/// A slice of any of the supported standard vertices.
pub enum VertexSlice<'a> {
    /// A wrapper around a slice of color vertices.
    ColorVertices(&'a [ColorVertex]),
    /// A wrapper around a slice of texture vertices and also the texture that they map to.
    TextureVertices(&'a [TextureVertex], Texture),
}

impl VertexSlice<'_> {
    /// Exposes the vertex list that wraps around it.
    pub fn contents(&self) -> &[u8] {
        match *self {
            Self::ColorVertices(vertices) => bytemuck::cast_slice(vertices),
            Self::TextureVertices(vertices, ..) => bytemuck::cast_slice(vertices),
        }
    }
}

/// The type of vertex
pub enum VertexType {
    /// A vertex that features a position and a color.
    ColorVertex,
    /// A vertex that features a position and a texture coordinate.
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

/// A set of vertices and transforms for a 3D model.
pub struct Model {
    /// The list of vertices.
    pub vertex_buffer: Buffer,
    /// The type of the vertices in `vertex_buffer`.
    pub vertex_type: VertexType,
    /// The list of indices.
    pub index_buffer: Buffer,
    /// The number of indices in `index_buffer`.
    pub index_count: u32,
    /// The list of instances of the model that will be visible. Every
    /// transform will be a new copy of the model without duplicating memory
    /// use.
    pub transforms: Vec<Transform>,
    /// When a transform is modified then this is set to `true`. Whenever the model is
    /// next rendered it will rebuild the buffer and set this to `false`.
    pub transforms_outdated: bool,
    /// The handle to the transform buffer.
    pub transform_buffer: Buffer,
    /// The count of the transforms on the buffer. This has to be included because
    /// `transforms.len()` might be different to the previous size and then the buffer would need
    /// to be reallocated.
    pub transform_count: u32,
}

impl Model {
    /// Make a model from the described features.
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

    /// Load a model from an obj file.
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

    /// Makes a model that is a plain white square.
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

    /// Makes a model that is a square with a texture on it.
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

    /// Put values of the transforms from the `Vec` on the struct to the
    /// actual buffer for rendering.
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

    /// Modified the value of the transform and marks it as outdated.
    pub fn set_transform(&mut self, index: usize, transform: Transform) {
        *self.transforms.get_mut(index).unwrap() = transform;
        self.transforms_outdated = true;
    }

    /// Modified the value of the transform, marks it as outdated, then returns the model.
    pub fn with_transform(mut self, index: usize, transform: Transform) -> Self {
        *self.transforms.get_mut(index).unwrap() = transform;
        self.transforms_outdated = true;
        self
    }

    /// Modified the value of the transforms and marks it as outdated.
    pub fn set_transforms(&mut self, transforms: Vec<Transform>) {
        self.transforms = transforms;
        self.transforms_outdated = true;
    }

    /// Modified the value of the transforms, marks it as outdated, then returns the model.
    pub fn with_transforms(mut self, transforms: Vec<Transform>) -> Self {
        self.transforms = transforms;
        self.transforms_outdated = true;
        self
    }

    /// Calls the function `modify` on the list of transforms and marks it as outdated.
    pub fn modify_transforms<F: FnOnce(&mut [Transform])>(&mut self, modify: F) {
        modify(&mut self.transforms);
        self.transforms_outdated = true;
    }

    /// Calls the function `modify` on the list of transforms, marks it as outdated, then returns
    /// the model.
    pub fn with_modify_transforms<F: FnOnce(&mut [Transform])>(mut self, modify: F) -> Self {
        modify(&mut self.transforms);
        self.transforms_outdated = true;
        self
    }
}

#[derive(Clone, Copy, Debug)]
/// A set of modifications to a model's vertices.
pub struct Transform {
    /// The value that each vertex has added to it.
    pub translation: Vec3,
    /// The rotation of a vertex from the model's origin.
    pub rotation: Quat,
    /// The value that each vertex is mulitplied again.
    pub scale: Vec3,
}

macro_rules! transform_methods {
    ($($i: ident: $t: ty),*) => {
        $(
            paste::paste! {
                pub fn [<$i _mut>](&mut self) -> &mut $t {
                    &mut self.$i
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
                pub fn [<$i _mut>](&mut self) -> &mut f32 {
                    &mut self.translation.$i
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

    /// Construct matrices from transform values.
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
/// A vertex used for colored models
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

    pub fn load<'a, P: AsRef<Path> + Clone + Eq + Hash>(
        data: &GameData,
        path: P,
        cache: &'a mut HashMap<P, Texture>,
        sampler: &Sampler,
    ) -> &'a Texture {
        cache
            .entry(path.clone())
            .or_insert_with(|| Texture::new(data, &image::open(path).unwrap(), sampler))
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
/// A vertex used for textured models
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
