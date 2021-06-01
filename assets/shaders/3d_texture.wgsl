var<private> f_color: vec4<f32>;
[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;
var<private> v_tex_coords: vec2<f32>;

fn main_frag() {
    let _e9: vec4<f32> = textureSample(t_diffuse, s_diffuse, v_tex_coords);
    f_color = _e9;
    return;
}

[[stage(fragment)]]
fn main1_frag([[location(0), interpolate(perspective)]] v_tex_coords1: vec2<f32>) -> [[location(0)]] vec4<f32> {
    v_tex_coords = v_tex_coords1;
    main_frag();
    return f_color;
}

[[block]]
struct gl_PerVertex {
    [[builtin(position)]] gl_Position: vec4<f32>;
};

[[block]]
struct Uniforms {
    u_view_position: vec3<f32>;
    u_view_proj: mat4x4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] gl_Position1: vec4<f32>;
    [[location(0), interpolate(perspective)]] member: vec2<f32>;
};

var<private> perVertexStruct: gl_PerVertex;
[[group(1), binding(0)]]
var<uniform> _: Uniforms;
var<private> a_position: vec3<f32>;
var<private> v_tex_coords: vec2<f32>;
var<private> a_tex_coords: vec2<f32>;

fn main_vert() {
    let _e15: vec3<f32> = a_position;
    perVertexStruct.gl_Position = (_.u_view_proj * vec4<f32>(_e15.x, _e15.y, _e15.z, 1.0));
    v_tex_coords = a_tex_coords;
    return;
}

[[stage(vertex)]]
fn main1_vert([[location(0)]] a_position1: vec3<f32>, [[location(1)]] a_tex_coords1: vec2<f32>) -> VertexOutput {
    a_position = a_position1;
    a_tex_coords = a_tex_coords1;
    main_vert();
    return VertexOutput(perVertexStruct.gl_Position, v_tex_coords);
}
