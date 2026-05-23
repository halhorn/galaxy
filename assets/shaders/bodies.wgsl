#import bevy_pbr::view_transformations::position_world_to_clip

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<storage, read> masses: array<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<storage, read> body_colors: array<vec4<f32>>;

struct StarsRenderParams {
    star_visual_scale: f32,
    min_star_visual_scale: f32,
    sun_radius_au: f32,
    _pad: f32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<uniform> params: StarsRenderParams;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(5) body_id: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

fn physical_radius_from_mass(mass: f32) -> f32 {
    return params.sun_radius_au * pow(mass, 1.0 / 3.0);
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let body_id = vertex.body_id;
    let mass = masses[body_id];
    if (mass <= 1e-8) {
        out.clip_position = vec4<f32>(0.0, 0.0, -1e6, 1.0);
        out.color = vec4<f32>(0.0);
        return out;
    }
    let center = positions[body_id].xyz;
    let radius = max(
        physical_radius_from_mass(mass) * params.star_visual_scale,
        params.min_star_visual_scale,
    );
    let world_pos = center + vertex.position * (radius * 2.0);
    let n = normalize(vertex.normal);
    let light = normalize(vec3(0.15, 1.0, 0.25));
    let shade = 0.55 + 0.45 * max(dot(n, light), 0.0);
    let rgb = body_colors[body_id].rgb;
    out.clip_position = position_world_to_clip(world_pos);
    out.color = vec4(rgb * shade, 1.0);
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
