#import bevy_pbr::view_transformations::position_world_to_clip

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<storage, read> masses: array<f32>;
// merge_aux[n..2n): flash counter in upper bits (see merge.wgsl).
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<storage, read> merge_aux: array<u32>;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(5) body_id: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

// Matches init.rs disk masses ~(0.14/0.5)^3 .. (0.36/0.5)^3 and binary stars at 1.0;
// merged bodies can grow toward ~10 M☉.
const MASS_LOG_MIN: f32 = -1.7; // log10(~0.02)
const MASS_LOG_MAX: f32 = 1.0;  // log10(10) — full red at the top end
const MERGE_FLASH_FRAMES: f32 = 10.0;

fn mass_to_t(mass: f32) -> f32 {
    let log_m = log(max(mass, 1e-8)) / log(10.0);
    return clamp((log_m - MASS_LOG_MIN) / (MASS_LOG_MAX - MASS_LOG_MIN), 0.0, 1.0);
}

fn flash_red_weight(body_id: u32) -> f32 {
    let n = arrayLength(&merge_aux) / 2u;
    let remaining = merge_aux[n + body_id] >> 1u;
    if (remaining == 0u) {
        return 0.0;
    }
    let progress = 1.0 - f32(remaining) / MERGE_FLASH_FRAMES;
    // Ease-out: red drops quickly at first, then eases gently back to mass color.
    let t = 1.0 - progress;
    return t * t;
}

fn color_from_mass(mass: f32) -> vec3<f32> {
    let t = mass_to_t(mass);
    var rgb = vec3<f32>(0.0);
    if (t < 0.5) {
        let s = t * 2.0;
        rgb = vec3<f32>(0.12 + 0.88 * s, 0.32 + 0.68 * s, 1.0);
    } else {
        let s = (t - 0.5) * 2.0;
        rgb = vec3<f32>(1.0, 1.0 - 0.88 * s, 1.0 - 0.92 * s);
    }
    // Bloom なしでも色が潰れない範囲でやや明るく（旧 HDR 1.5+t*14 より控えめ）
    let brightness = 1.0 + t * 3.5;
    return rgb * brightness;
}

fn radius_from_mass(mass: f32) -> f32 {
    return 0.5 * pow(mass, 1.0 / 3.0);
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
    let radius = radius_from_mass(mass);
    let world_pos = center + vertex.position * (radius * 2.0);
    let n = normalize(vertex.normal);
    let light = normalize(vec3(0.15, 1.0, 0.25));
    let shade = 0.55 + 0.45 * max(dot(n, light), 0.0);
    let base = color_from_mass(mass);
    let red_weight = flash_red_weight(body_id);
    let rgb = mix(base, vec3<f32>(1.0, 0.0, 0.0), red_weight);
    out.clip_position = position_world_to_clip(world_pos);
    out.color = vec4(rgb * shade, 1.0);
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
