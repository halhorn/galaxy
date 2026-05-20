#import bevy_pbr::view_transformations::position_world_to_clip

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<storage, read> masses: array<f32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

fn color_from_mass(mass: f32) -> vec3<f32> {
    let t = clamp((log(mass) / log(10.0) + 1.0) / 3.0, 0.0, 1.0);
    let brightness = 1.5 + t * 14.0;
    var rgb = vec3<f32>(0.0);
    if (t < 0.5) {
        let s = t * 2.0;
        rgb = vec3<f32>(0.1 + 0.9 * s, 0.3 + 0.7 * s, 1.0) * brightness;
    } else {
        let s = (t - 0.5) * 2.0;
        rgb = vec3<f32>(1.0, 1.0 - 0.9 * s, 1.0 - 0.95 * s) * brightness;
    }
    return rgb;
}

@vertex
fn vertex(@builtin(vertex_index) body_id: u32) -> VertexOutput {
    var out: VertexOutput;
    let mass = masses[body_id];
    out.clip_position = position_world_to_clip(positions[body_id].xyz);
    out.color = vec4<f32>(color_from_mass(mass), 1.0);
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
