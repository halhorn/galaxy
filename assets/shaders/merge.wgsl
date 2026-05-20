// Parallel merge (≤10 storage buffers for WebGPU). Scratch: [0..n) pos+mass, [n..2n) vel.

const INVALID: u32 = 0xFFFFFFFFu;

struct Params {
    n: u32,
    merge_radius_factor: f32,
    inv_cell_size: f32,
    min_mass: f32,
}

@group(0) @binding(0) var<storage, read_write> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> velocities: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> masses: array<f32>;
@group(0) @binding(3) var<storage, read_write> accelerations: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read_write> scratch: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> bucket_heads: array<atomic<u32>>;
@group(0) @binding(6) var<storage, read_write> bucket_next: array<u32>;
@group(0) @binding(7) var<storage, read_write> absorbed: array<u32>;
@group(0) @binding(8) var<storage, read_write> merge_owner: array<atomic<u32>>;
@group(0) @binding(9) var<uniform> params: Params;

fn snap_pos(i: u32) -> vec3<f32> {
    return scratch[i].xyz;
}

fn snap_mass(i: u32) -> f32 {
    return scratch[i].w;
}

fn snap_vel(i: u32) -> vec3<f32> {
    return scratch[params.n + i].xyz;
}

fn radius_from_mass(mass: f32) -> f32 {
    return 0.5 * pow(mass, 1.0 / 3.0);
}

fn hash_cell(cx: i32, cy: i32, cz: i32) -> u32 {
    let hx = bitcast<u32>(cx);
    let hy = bitcast<u32>(cy);
    let hz = bitcast<u32>(cz);
    let h = hx * 73856093u ^ hy * 19349663u ^ hz * 83492791u;
    return h % arrayLength(&bucket_heads);
}

fn cell_coords(pos: vec3<f32>) -> vec3<i32> {
    let s = params.inv_cell_size;
    return vec3<i32>(
        i32(floor(pos.x * s)),
        i32(floor(pos.y * s)),
        i32(floor(pos.z * s)),
    );
}

fn mergeable(i: u32, j: u32) -> bool {
    if (j <= i || absorbed[j] != 0u || snap_mass(j) <= params.min_mass) {
        return false;
    }
    if (snap_mass(i) <= params.min_mass) {
        return false;
    }
    let dist = length(snap_pos(i) - snap_pos(j));
    let ri = radius_from_mass(snap_mass(i));
    let rj = radius_from_mass(snap_mass(j));
    let touch = (ri + rj) * params.merge_radius_factor;
    return dist < touch;
}

fn absorb(i: u32, j: u32) {
    let mi = snap_mass(i);
    let mj = snap_mass(j);
    let new_mass = mi + mj;
    velocities[i] = vec4<f32>((snap_vel(i) * mi + snap_vel(j) * mj) / new_mass, 0.0);
    positions[i] = vec4<f32>((snap_pos(i) * mi + snap_pos(j) * mj) / new_mass, 0.0);
    masses[i] = new_mass;
    masses[j] = 0.0;
    accelerations[i] = vec4<f32>(0.0);
    absorbed[j] = 1u;
}

@compute @workgroup_size(256)
fn prepare(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n) {
        return;
    }
    absorbed[i] = 0u;
    bucket_next[i] = INVALID;
    scratch[i] = vec4<f32>(positions[i].xyz, masses[i]);
    scratch[params.n + i] = velocities[i];
}

@compute @workgroup_size(256)
fn clear_buckets(@builtin(global_invocation_id) gid: vec3<u32>) {
    let b = gid.x;
    if (b >= arrayLength(&bucket_heads)) {
        return;
    }
    atomicStore(&bucket_heads[b], INVALID);
}

@compute @workgroup_size(256)
fn init_owner(@builtin(global_invocation_id) gid: vec3<u32>) {
    let j = gid.x;
    if (j >= params.n) {
        return;
    }
    atomicStore(&merge_owner[j], params.n);
}

@compute @workgroup_size(256)
fn build_grid(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n || snap_mass(i) <= params.min_mass) {
        return;
    }
    let c = cell_coords(snap_pos(i));
    let b = hash_cell(c.x, c.y, c.z);
    var prev = atomicLoad(&bucket_heads[b]);
    loop {
        bucket_next[i] = prev;
        let result = atomicCompareExchangeWeak(&bucket_heads[b], prev, i);
        if (result.exchanged) {
            break;
        }
        prev = result.old_value;
    }
}

@compute @workgroup_size(256)
fn find_owner(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n || snap_mass(i) <= params.min_mass) {
        return;
    }
    let c = cell_coords(snap_pos(i));
    for (var dx = -1; dx <= 1; dx++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dz = -1; dz <= 1; dz++) {
                let b = hash_cell(c.x + dx, c.y + dy, c.z + dz);
                var j = atomicLoad(&bucket_heads[b]);
                while (j != INVALID) {
                    let j_next = bucket_next[j];
                    if (mergeable(i, j)) {
                        atomicMin(&merge_owner[j], i);
                    }
                    j = j_next;
                }
            }
        }
    }
}

@compute @workgroup_size(256)
fn apply_merge(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n || snap_mass(i) <= params.min_mass) {
        return;
    }
    let c = cell_coords(snap_pos(i));
    for (var dx = -1; dx <= 1; dx++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dz = -1; dz <= 1; dz++) {
                let b = hash_cell(c.x + dx, c.y + dy, c.z + dz);
                var j = atomicLoad(&bucket_heads[b]);
                while (j != INVALID) {
                    let j_next = bucket_next[j];
                    if (atomicLoad(&merge_owner[j]) == i && absorbed[j] == 0u && mergeable(i, j)) {
                        absorb(i, j);
                    }
                    j = j_next;
                }
            }
        }
    }
}
