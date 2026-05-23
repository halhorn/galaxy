struct ForceTerm {
    sign: i32,
    exponent: i32,
    coefficient: f32,
    _pad: u32,
}

struct Params {
    n: u32,
    term_count: u32,
    softening_sq: f32,
    min_mass: f32,
    terms: array<ForceTerm, 8>,
}

@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> masses: array<f32>;
@group(0) @binding(2) var<storage, read_write> accelerations_new: array<vec4<f32>>;
@group(0) @binding(3) var<uniform> params: Params;

fn pow_d(exp: i32, dist_sq: f32, d: f32) -> f32 {
    switch exp {
        case -5: { return 1.0 / (dist_sq * dist_sq * d); }
        case -4: { return 1.0 / (dist_sq * dist_sq); }
        case -3: { return 1.0 / (dist_sq * d); }
        case -2: { return 1.0 / dist_sq; }
        case -1: { return 1.0 / d; }
        case 0: { return 1.0; }
        case 1: { return d; }
        case 2: { return dist_sq; }
        default: { return 0.0; }
    }
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.n || masses[i] <= params.min_mass) {
        accelerations_new[i] = vec4<f32>(0.0);
        return;
    }

    let pos_i = positions[i].xyz;
    var acc = vec3<f32>(0.0, 0.0, 0.0);

    for (var j = 0u; j < params.n; j++) {
        if (j == i || masses[j] <= params.min_mass) {
            continue;
        }
        let r = positions[j].xyz - pos_i;
        let dist_sq = dot(r, r) + params.softening_sq;
        let d = sqrt(dist_sq);

        for (var k = 0u; k < params.term_count; k++) {
            let term = params.terms[k];
            if (term.coefficient == 0.0) {
                continue;
            }
            let sign = f32(term.sign);
            let power = pow_d(term.exponent, dist_sq, d);
            acc += r * sign * term.coefficient * power * masses[j];
        }
    }

    accelerations_new[i] = vec4<f32>(acc, 0.0);
}
