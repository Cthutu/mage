struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] colour: vec3<f32>;
};

// 1+-------+3
//  |       |
//  |       |
//  |       |
// 0+-------+2

// 0    0, 0    Black
// 1    0, 1    Green
// 2    1, 0    Red
// 3    1, 1    Yellow

[[group(0), binding(0)]]
var t_fore: texture_2d<f32>;
[[group(0), binding(1)]]
var t_back: texture_2d<f32>;
[[group(0), binding(2)]]
var t_text: texture_2d<f32>;

[[block]]
struct Uniforms {
    font_width: u32;
    font_height: u32;
};

[[group(1), binding(0)]]
var<uniform> uniforms: Uniforms;


[[stage(vertex)]]
fn main(
    [[builtin(vertex_index)]] in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    let i = u32(in_vertex_index);
    let x = i & 2u;
    let y = i & 1u;

    var fx: f32 = f32(x);
    var fy: f32 = f32(y);
    out.colour = vec3<f32>(fx, fy, 0.0);

    fx = (fx * 2.0) - 1.0;
    fy = (fy * 2.0) - 1.0;
    out.clip_position = vec4<f32>(fx, fy, 0.0, 1.0);

    return out;
}

[[stage(fragment)]]

fn main([[builtin(position)]] pos: vec4<f32>) -> [[location(0)]] vec4<f32> {
    let p = vec2<f32>(pos.x - 0.5, pos.y - 0.5);
    let cp = vec2<i32>(i32(p.x / f32(uniforms.font_width)), i32(p.y / f32(uniforms.font_height)));

    // let colour = vec4<f32>(x, y, 0.0, 1.0);
    let colour = textureLoad(t_back, cp, 0);

    return colour;
}

