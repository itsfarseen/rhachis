use rhachis::{graphics::{Renderer, ColorVertex}, *};
use wgpu::{Buffer, util::{DeviceExt, BufferInitDescriptor}};

fn main() {
    Tri::run();
}

struct Tri {
    renderer: TriRenderer,
}

impl Game for Tri {
    fn init(data: &GameData) -> Self {
        Tri { renderer: TriRenderer::new(data) }
    }

    fn get_renderer(&mut self) -> &mut dyn Renderer {
        &mut self.renderer
    }
}

struct TriRenderer {
    vertex_buffer: Buffer,
}

impl TriRenderer {
    fn new(data: &GameData) -> Self {
        let graphics = data.graphics.lock();

        Self {
            vertex_buffer: graphics.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Triangle Vertices"),
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

impl Renderer for TriRenderer {}
