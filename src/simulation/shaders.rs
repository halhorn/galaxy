use bevy::{asset::uuid_handle, prelude::*};

const GRAVITY_WGSL: &str = include_str!("../../assets/shaders/gravity.wgsl");
const INTEGRATE_WGSL: &str = include_str!("../../assets/shaders/integrate.wgsl");
const BODIES_WGSL: &str = include_str!("../../assets/shaders/bodies.wgsl");

pub const GRAVITY_SHADER: Handle<Shader> = uuid_handle!("a8c31e42-1f0b-4d2a-9e3c-7b5a6d8e9f01");
pub const INTEGRATE_SHADER: Handle<Shader> = uuid_handle!("b9d42f53-2a1c-5e3b-0f4d-8c6b7e0f1a02");
pub const BODIES_SHADER: Handle<Shader> = uuid_handle!("f3a91c2e-8b4d-4a1e-9c2f-1d8e5a6b7c0d");

/// Embeds WGSL at compile time (wasm-safe; no runtime asset fetch).
pub fn register_simulation_shaders(mut shaders: ResMut<Assets<Shader>>) {
    let _ = shaders.insert(
        GRAVITY_SHADER.id(),
        Shader::from_wgsl(GRAVITY_WGSL, "shaders/gravity.wgsl"),
    );
    let _ = shaders.insert(
        INTEGRATE_SHADER.id(),
        Shader::from_wgsl(INTEGRATE_WGSL, "shaders/integrate.wgsl"),
    );
    let _ = shaders.insert(
        BODIES_SHADER.id(),
        Shader::from_wgsl(BODIES_WGSL, "shaders/bodies.wgsl"),
    );
}
