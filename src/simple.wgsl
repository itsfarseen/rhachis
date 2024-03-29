struct Transform {
    @location(2) data0: vec4<f32>,
    @location(3) data1: vec4<f32>,
    @location(4) data2: vec4<f32>,
    @location(5) data3: vec4<f32>,
}

struct ColorInput {
    @location(0) pos: vec3<f32>,
    @location(1) color: vec4<f32>,
}

struct ColorOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
}

struct TextureInput {
    @location(0) pos: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct TextureOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@group(0)@binding(0)
var<uniform> projection: mat4x4<f32>;
@group(0)@binding(1)
var<uniform> camera: mat4x4<f32>;

@vertex
fn color_vertex(input: ColorInput, transform: Transform) -> ColorOutput {
    let transform_matrix = mat4x4<f32>(
        transform.data0,
        transform.data1,
        transform.data2,
        transform.data3,
    );

    var output: ColorOutput;
    output.pos = projection * camera * transform_matrix * vec4<f32>(input.pos, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn color_fragment(output: ColorOutput) -> @location(0) vec4<f32> {
    return output.color;
}

@vertex
fn texture_vertex(input: TextureInput, transform: Transform) -> TextureOutput {
    let transform_matrix = mat4x4<f32>(
        transform.data0,
        transform.data1,
        transform.data2,
        transform.data3,
    );

    var output: TextureOutput;
    output.pos = projection * camera * transform_matrix * vec4<f32>(input.pos, 1.0);
    output.tex_coords = input.tex_coords;
    return output;
}

@group(1)@binding(0)
var texture: texture_2d<f32>;
@group(1)@binding(1)
var texture_sampler: sampler;

@fragment
fn texture_fragment(output: TextureOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, output.tex_coords);
}
