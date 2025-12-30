# Improvement 8: Optimize Performance Bottlenecks

## Problem
Several performance bottlenecks exist in the codebase that could cause frame rate drops, especially with many enemies or complex scenes.

## Current Issues

### Inefficient Collision Detection (`combat.rs`)
The projectile-enemy collision uses nested loops with spatial bucketing that has bugs:

```rust
for proj in state.projectiles.iter_mut() {
    // ... bounds check ...
    let bx = (proj.position.x / bucket_size).floor() as i32;
    let by = (proj.position.y / bucket_size).floor() as i32;
    
    // Check 3x3 grid
    for dx in -1..=1 {
        for dy in -1..=1 {
            if let Some(enemy_indices) = buckets.get(&(bx + dx, by + dy)) {
                for &idx in enemy_indices {
                    // Double bounds check needed
                    if idx >= state.enemies.len() { continue; }
                    // ... collision logic ...
                }
            }
        }
    }
}
```

### Expensive Core Position Calculation
`get_core_screen_position()` is called every frame and recalculates unnecessarily.

### Pathfinding Cache Issues
The pathfinding cache in `ship.rs` may not invalidate properly when the ship structure changes.

### Particle System Inefficiency
Particles are stored in a Vec and cleaned up with retain, but could be optimized.

## Solution
Implement proper spatial partitioning, caching, and algorithmic improvements.

## Implementation Steps

1. **Fix spatial partitioning** in collision detection
2. **Add caching** for expensive calculations
3. **Optimize particle system**
4. **Improve pathfinding cache invalidation**

## Code Changes

### Fix Collision Detection (`combat.rs`)
Replace the buggy spatial partitioning with a proper implementation:

```rust
fn update_projectiles(state: &mut GameState, dt: f32, events: &mut EventBus) {
    // 1. Move projectiles
    for proj in &mut state.projectiles {
        proj.position += proj.velocity * dt;
        
        // Bounds check with larger bounds for off-screen projectiles
        if proj.position.x < -200.0 || proj.position.x > screen_width() + 200.0 ||
           proj.position.y < -200.0 || proj.position.y > screen_height() + 200.0 {
            proj.active = false;
        }
    }
    
    // 2. Build spatial index for enemies
    let bucket_size = 64.0; // Smaller buckets for better performance
    let mut spatial_index: HashMap<(i32, i32), Vec<usize>> = HashMap::new();
    
    for (i, enemy) in state.enemies.iter().enumerate() {
        if !enemy.health > 0.0 { continue; }
        
        let bx = (enemy.position.x / bucket_size).floor() as i32;
        let by = (enemy.position.y / bucket_size).floor() as i32;
        
        spatial_index.entry((bx, by)).or_default().push(i);
        
        // Add to adjacent buckets if enemy spans multiple buckets
        let radius = match enemy.enemy_type {
            EnemyType::Boss => 40.0,
            EnemyType::Nanoguard => 15.0,
            _ => 10.0,
        };
        
        let bucket_span = (radius / bucket_size).ceil() as i32;
        for dx in -bucket_span..=bucket_span {
            for dy in -bucket_span..=bucket_span {
                if dx == 0 && dy == 0 { continue; }
                spatial_index.entry((bx + dx, by + dy)).or_default().push(i);
            }
        }
    }
    
    // 3. Process collisions using spatial index
    for proj in state.projectiles.iter_mut() {
        if !proj.active { continue; }
        
        let bx = (proj.position.x / bucket_size).floor() as i32;
        let by = (proj.position.y / bucket_size).floor() as i32;
        
        // Check projectile's bucket and immediate neighbors
        let mut checked_enemies = std::collections::HashSet::new();
        
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(enemy_indices) = spatial_index.get(&(bx + dx, by + dy)) {
                    for &enemy_idx in enemy_indices {
                        if checked_enemies.contains(&enemy_idx) { continue; }
                        checked_enemies.insert(enemy_idx);
                        
                        if enemy_idx >= state.enemies.len() { continue; }
                        let enemy = &mut state.enemies[enemy_idx];
                        
                        if enemy.health <= 0.0 { continue; }
                        
                        let hit_radius = match enemy.enemy_type {
                            EnemyType::Boss => 40.0,
                            EnemyType::Nanoguard => 15.0,
                            _ => 10.0,
                        };
                        
                        if proj.position.distance(enemy.position) < hit_radius {
                            enemy.health -= proj.damage;
                            proj.active = false;
                            
                            // ... existing kill logic ...
                            break;
                        }
                    }
                }
                if !proj.active { break; }
            }
            if !proj.active { break; }
        }
    }
    
    // 4. Cleanup
    state.projectiles.retain(|p| p.active);
    state.enemies.retain(|e| e.health > 0.0);
}
```

### Add Core Position Caching (`state.rs`)
Cache the core position to avoid repeated searches:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    // ... existing fields ...
    #[serde(skip)]
    core_position_cache: Option<Vec2>,
    // ... rest ...
}

impl GameState {
    // ... existing code ...
    
    pub fn get_core_position(&mut self) -> Option<Vec2> {
        if self.core_position_cache.is_none() {
            if let Some((x, y)) = self.ship.find_core() {
                self.core_position_cache = Some(crate::layout::Layout::grid_to_screen_center(x, y));
            }
        }
        self.core_position_cache
    }
    
    pub fn invalidate_core_cache(&mut self) {
        self.core_position_cache = None;
        self.ship.invalidate_cache(); // Also invalidate path cache
    }
}
```

### Optimize Particle System
Use a more efficient data structure for particles:

```rust
#[derive(Debug, Clone)]
pub struct ParticleSystem {
    particles: Vec<Particle>,
    free_indices: Vec<usize>, // Reuse dead particle slots
}

impl ParticleSystem {
    pub fn new(capacity: usize) -> Self {
        Self {
            particles: Vec::with_capacity(capacity),
            free_indices: Vec::new(),
        }
    }
    
    pub fn add_particle(&mut self, position: Vec2, velocity: Vec2, lifetime: f32, color: Color) {
        let particle = Particle::new(position, velocity, lifetime, color);
        
        if let Some(index) = self.free_indices.pop() {
            self.particles[index] = particle;
        } else if self.particles.len() < self.particles.capacity() {
            self.particles.push(particle);
        }
        // Silently drop if at capacity
    }
    
    pub fn update(&mut self, dt: f32) {
        for (i, particle) in self.particles.iter_mut().enumerate() {
            if particle.active {
                particle.lifetime -= dt;
                particle.position += particle.velocity * dt;
                
                if particle.lifetime <= 0.0 {
                    particle.active = false;
                    self.free_indices.push(i);
                }
            }
        }
    }
    
    pub fn draw(&self) {
        for particle in &self.particles {
            if particle.active {
                // ... draw particle ...
            }
        }
    }
}
```

### Improve Pathfinding Cache (`ship.rs`)
Better cache invalidation and key generation:

```rust
impl Ship {
    pub fn calculate_path_to_core(&self, start: (usize, usize)) -> Option<Vec<(usize, usize)>> {
        // Use a more specific cache key
        let cache_key = start;
        
        if let Some(path) = self.path_cache.borrow().get(&cache_key) {
            return Some(path.clone());
        }
        
        // ... existing pathfinding logic ...
        
        // Cache with better key
        self.path_cache.borrow_mut().insert(cache_key, path.clone());
        Some(path)
    }
    
    /// Invalidate cache when modules are added/removed/repaired
    pub fn invalidate_path_cache(&self) {
        self.path_cache.borrow_mut().clear();
    }
}
```

## Benefits
- Better frame rates with many enemies/projectiles
- Reduced CPU usage
- Smoother gameplay experience
- More scalable for larger battles

## Testing
- Performance test with 100+ enemies
- Measure frame time improvements
- Verify collision accuracy
- Test cache invalidation scenarios
- Check memory usage with particle system</content>
<parameter name="filePath">h:\WebHatchery\games\scrapyard\plans\improvement_08_performance_optimization.md