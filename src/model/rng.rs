/// Deterministic RNG for initial conditions (no `getrandom` on wasm).
pub struct SimpleRng(u64);

impl SimpleRng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn next_u32(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.0 >> 32) as u32
    }

    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        let u = (self.next_u32() as f64) / (u32::MAX as f64);
        min + (max - min) * u as f32
    }
}
