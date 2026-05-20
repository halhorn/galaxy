struct Params {
    n: u32,
    g: f32,
    softening_sq: f32,
    _pad: f32,
}

@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> masses: array<f32>;
@group(0) @binding(2) var<storage, read_write> accelerations_new: array<vec4<f32>>;
@group(0) @binding(3) var<uniform> params: Params;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.n) {
        return;
    }

    let pos_i = positions[i].xyz;
    var acc = vec3<f32>(0.0, 0.0, 0.0);

    for (var j = 0u; j < params.n; j++) {
        if (j == i) {
            continue;
        }
        let r = positions[j].xyz - pos_i;
        let dist_sq = dot(r, r) + params.softening_sq;
        let inv_dist3 = params.g / (dist_sq * sqrt(dist_sq));
        acc += r * inv_dist3 * masses[j];
    }

    accelerations_new[i] = vec4<f32>(acc, 0.0);
}
