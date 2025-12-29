use crate::ship::ModuleType;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ModuleStats {
    pub name: String,
    pub base_cost: i32,
    pub power_consumption: i32, // Positive = Generation, Negative = Consumption
    pub max_health: f32,
    pub range: f32,     // 0 for non-weapons
    pub damage: f32,    // 0 for non-weapons
    pub fire_rate: f32, // Rounds per second
}

impl ModuleStats {
    pub fn new(name: &str, cost: i32, power: i32, hp: f32) -> Self {
        Self {
            name: name.to_string(),
            base_cost: cost,
            power_consumption: power,
            max_health: hp,
            range: 0.0,
            damage: 0.0,
            fire_rate: 0.0,
        }
    }

    pub fn with_combat(mut self, range: f32, damage: f32, rate: f32) -> Self {
        self.range = range;
        self.damage = damage;
        self.fire_rate = rate;
        self
    }
}

pub struct ModuleRegistry {
    stats: HashMap<ModuleType, ModuleStats>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        let mut stats = HashMap::new();

        // 1. Core
        stats.insert(ModuleType::Core, ModuleStats::new("Power Core", 0, 10, 1000.0));
        
        // 2. Weapons
        stats.insert(ModuleType::Weapon, 
            ModuleStats::new("Pulse Turret", 20, -2, 100.0)
            .with_combat(200.0, 10.0, 1.0)
        );

        // 3. Defense
        stats.insert(ModuleType::Defense,
            ModuleStats::new("Shield Gen", 30, -3, 150.0)
        );

        // 4. Utility (Generator implied by positive power? Or separate type?)
        // For now, let's say "Utility" includes Solar Panels for specific implementation, 
        // but if generic Utility, maybe just a recycler. 
        // Let's assume there is a PowerGenerator module type missing from the enum or Utility can be one.
        // Let's make Utility a "Recycler" for now (Consumer).
        stats.insert(ModuleType::Utility,
             ModuleStats::new("Recycler", 25, -1, 80.0)
        );
        
        // 5. Engine
        stats.insert(ModuleType::Engine,
            ModuleStats::new("Hyperdrive", 500, -50, 500.0)
        );

        // 6. Empty - No stats needed usually, or dummy
        stats.insert(ModuleType::Empty, ModuleStats::new("Empty Slot", 0, 0, 0.0));

        Self { stats }
    }

    pub fn get(&self, module_type: ModuleType) -> &ModuleStats {
        self.stats.get(&module_type).unwrap_or_else(|| self.stats.get(&ModuleType::Empty).unwrap())
    }
}
