struct Params {
    n: u32,
    dt: f32,
    _pad0: f32,
    _pad1: f32,
}

@group(0) @binding(0) var<storage, read_write> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read_write> velocities: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> accelerations: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> accelerations_new: array<vec4<f32>>;
@group(0) @binding(4) var<uniform> params: Params;

@compute @workgroup_size(256)
fn position_step(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.n) {
        return;
    }
    let dt = params.dt;
    let dt2 = dt * dt;
    let pos = positions[i].xyz;
    let vel = velocities[i].xyz;
    let acc = accelerations[i].xyz;
    positions[i] = vec4<f32>(pos + vel * dt + 0.5 * acc * dt2, 0.0);
}

@compute @workgroup_size(256)
fn velocity_step(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.n) {
        return;
    }
    let dt = params.dt;
    let a_old = accelerations[i].xyz;
    let a_new = accelerations_new[i].xyz;
    velocities[i] = vec4<f32>(velocities[i].xyz + 0.5 * (a_old + a_new) * dt, 0.0);
    accelerations[i] = vec4<f32>(a_new, 0.0);
}
