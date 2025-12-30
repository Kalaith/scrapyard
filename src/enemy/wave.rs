// wave.rs - Structured state for enemy spawn timing

#[derive(Debug, Clone)]
pub struct WaveState {
    pub spawn_timer: f32,
    pub guard_timer: f32,
}

impl WaveState {
    pub fn new() -> Self {
        Self {
            spawn_timer: 0.0,
            guard_timer: 0.0,
        }
    }
    
    pub fn update(&mut self, dt: f32) {
        self.spawn_timer += dt;
        self.guard_timer += dt;
    }
    
    pub fn reset_spawn_timer(&mut self) {
        self.spawn_timer = 0.0;
    }
    
    pub fn reset_guard_timer(&mut self) {
        self.guard_timer = 0.0;
    }
}
