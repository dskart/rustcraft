var<private> f_color: vec4<f32>;
var<private> v_color: vec3<f32>;

fn main_frag() {
    let _e7: vec3<f32> = v_color;
    f_color = vec4<f32>(_e7.x, _e7.y, _e7.z, 1.0);
    return;
}

[[stage(fragment)]]
fn main1_frag([[location(0), interpolate(perspective)]] v_color1: vec3<f32>) -> [[location(0)]] vec4<f32> {
    v_color = v_color1;
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
    [[location(0), interpolate(perspective)]] member: vec3<f32>;
    [[builtin(position)]] gl_Position1: vec4<f32>;
};

var<private> v_color: vec3<f32>;
var<private> a_color: vec3<f32>;
var<private> perVertexStruct: gl_PerVertex;
[[group(0), binding(0)]]
var<uniform> _: Uniforms;
var<private> a_position: vec3<f32>;

fn main_vert() {
    v_color = a_color;
    let _e16: vec3<f32> = a_position;
    perVertexStruct.gl_Position = (_.u_view_proj * vec4<f32>(_e16.x, _e16.y, _e16.z, 1.0));
    return;
}

[[stage(vertex)]]
fn main1_vert([[location(1)]] a_color1: vec3<f32>, [[location(0)]] a_position1: vec3<f32>) -> VertexOutput {
    a_color = a_color1;
    a_position = a_position1;
    main_vert();
    return VertexOutput(v_color, perVertexStruct.gl_Position);
}
