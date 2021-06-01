[[group(0), binding(0)]]
var t_Color: texture_2d<f32>;
[[group(0), binding(1)]]
var s_Color: sampler;
var<private> v_TexCoord: vec2<f32>;
var<private> o_Target: vec4<f32>;

fn main_frag() {
    var color: vec4<f32>;

    let _e11: vec4<f32> = textureSample(t_Color, s_Color, v_TexCoord);
    color = _e11;
    let _e13: vec3<f32> = color.xyz;
    o_Target = vec4<f32>(_e13.x, _e13.y, _e13.z, color[3u]);
    return;
}

[[stage(fragment)]]
fn main1_frag([[location(0), interpolate(perspective)]] v_TexCoord1: vec2<f32>) -> [[location(0)]] vec4<f32> {
    v_TexCoord = v_TexCoord1;
    main_frag();
    return o_Target;
}

[[block]]
struct gl_PerVertex {
    [[builtin(position)]] gl_Position: vec4<f32>;
};

struct VertexOutput {
    [[location(0), interpolate(perspective)]] member: vec2<f32>;
    [[builtin(position)]] gl_Position1: vec4<f32>;
};

var<private> v_TexCoord: vec2<f32>;
var<private> a_TexCoord: vec2<f32>;
var<private> perVertexStruct: gl_PerVertex;
var<private> a_Pos: vec4<f32>;

fn main_vert() {
    v_TexCoord = a_TexCoord;
    perVertexStruct.gl_Position = a_Pos;
    return;
}

[[stage(vertex)]]
fn main1_vert([[location(1)]] a_TexCoord1: vec2<f32>, [[location(0)]] a_Pos1: vec4<f32>) -> VertexOutput {
    a_TexCoord = a_TexCoord1;
    a_Pos = a_Pos1;
    main_vert();
    return VertexOutput(v_TexCoord, perVertexStruct.gl_Position);
}
