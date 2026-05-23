struct Params {
    n: u32,
    min_mass: f32,
    _pad0: f32,
    _pad1: f32,
}

@group(0) @binding(0) var<storage, read> masses: array<f32>;
@group(0) @binding(1) var<storage, read> merge_aux: array<u32>;
@group(0) @binding(2) var<storage, read_write> body_colors: array<vec4<f32>>;
@group(0) @binding(3) var<uniform> params: Params;

// Matches bodies.wgsl / init.rs mass range.
const MASS_LOG_MIN: f32 = -1.7;
const MASS_LOG_MAX: f32 = 1.0;
const MERGE_FLASH_FRAMES: f32 = 10.0;

fn mass_to_t(mass: f32) -> f32 {
    let log_m = log(max(mass, 1e-8)) / log(10.0);
    return clamp((log_m - MASS_LOG_MIN) / (MASS_LOG_MAX - MASS_LOG_MIN), 0.0, 1.0);
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
    let brightness = 1.0 + t * 3.5;
    return rgb * brightness;
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.n) {
        return;
    }

    let mass = masses[i];
    if (mass <= params.min_mass) {
        body_colors[i] = vec4<f32>(0.0);
        return;
    }

    let base = color_from_mass(mass);
    let remaining = merge_aux[params.n + i] >> 1u;
    var rgb = base;
    if (remaining > 0u) {
        let progress = 1.0 - f32(remaining) / MERGE_FLASH_FRAMES;
        let t = 1.0 - progress;
        let red_weight = t * t;
        rgb = mix(base, vec3<f32>(1.0, 0.0, 0.0), red_weight);
    }
    body_colors[i] = vec4<f32>(rgb, 1.0);
}
