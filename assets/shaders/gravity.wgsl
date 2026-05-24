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

fn pow_d(exp: i32, inv_dist: f32, inv_dist_sq: f32) -> f32 {
    switch exp {
        case -5: { let inv_d4 = inv_dist_sq * inv_dist_sq; return inv_d4 * inv_dist; }
        case -4: { return inv_dist_sq * inv_dist_sq; }
        case -3: { return inv_dist_sq * inv_dist; }
        case -2: { return inv_dist_sq; }
        case -1: { return inv_dist; }
        case 0: { return 1.0; }
        case 1: { return 1.0 / inv_dist; }
        case 2: { return 1.0 / inv_dist_sq; }
        default: { return 0.0; }
    }
}

fn accumulate_pair(
    acc: ptr<function, vec3<f32>>,
    pos_i: vec3<f32>,
    pos_j: vec3<f32>,
    mass_j: f32,
    inv_dist: f32,
    inv_dist_sq: f32,
    sign: f32,
    coeff: f32,
    exp: i32,
) {
    let r = pos_j - pos_i;
    let power = pow_d(exp, inv_dist, inv_dist_sq);
    *acc += r * sign * coeff * power * mass_j;
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

    if (params.term_count == 1u) {
        let term = params.terms[0];
        if (term.coefficient != 0.0) {
            let sign = f32(term.sign);
            let coeff = term.coefficient;
            let exp = term.exponent;
            for (var j = 0u; j < params.n; j++) {
                if (j == i || masses[j] <= params.min_mass) {
                    continue;
                }
                let r = positions[j].xyz - pos_i;
                let dist_sq = dot(r, r) + params.softening_sq;
                let inv_dist = inverseSqrt(dist_sq);
                let inv_dist_sq = inv_dist * inv_dist;
                accumulate_pair(&acc, pos_i, positions[j].xyz, masses[j], inv_dist, inv_dist_sq, sign, coeff, exp);
            }
        }
    } else {
        for (var j = 0u; j < params.n; j++) {
            if (j == i || masses[j] <= params.min_mass) {
                continue;
            }
            let r = positions[j].xyz - pos_i;
            let dist_sq = dot(r, r) + params.softening_sq;
            let inv_dist = inverseSqrt(dist_sq);
            let inv_dist_sq = inv_dist * inv_dist;

            for (var k = 0u; k < params.term_count; k++) {
                let term = params.terms[k];
                if (term.coefficient == 0.0) {
                    continue;
                }
                let sign = f32(term.sign);
                accumulate_pair(
                    &acc,
                    pos_i,
                    positions[j].xyz,
                    masses[j],
                    inv_dist,
                    inv_dist_sq,
                    sign,
                    term.coefficient,
                    term.exponent,
                );
            }
        }
    }

    accelerations_new[i] = vec4<f32>(acc, 0.0);
}
