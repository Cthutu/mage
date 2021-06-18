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
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(in.colour, 1.0);
}



