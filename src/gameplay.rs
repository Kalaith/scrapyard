use crate::ship::ModuleType;
use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct WeaponConfig {
    pub damage: f32,
    pub range: f32,
    pub fire_rate: f32,
}

#[derive(Debug, Clone)]
pub struct ModuleStats {
    pub name: String,
    pub base_cost: i32,
    pub power_consumption: i32,
    pub max_health: f32,
    pub range: f32,
    pub damage: f32,
    pub fire_rate: f32,
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

        // Load weapon config from embedded JSON
        let weapon_json = include_str!("../assets/stats/weapons.json");
        let weapon_configs: HashMap<String, WeaponConfig> = serde_json::from_str(weapon_json)
            .expect("Failed to parse weapons.json");

        let default_weapon = WeaponConfig { damage: 10.0, range: 200.0, fire_rate: 1.0 };
        let pulse_turret = weapon_configs.get("Pulse Turret").unwrap_or(&default_weapon);

        // 1. Core
        stats.insert(ModuleType::Core, ModuleStats::new("Power Core", 0, 10, 1000.0));
        
        // 2. Weapons
        stats.insert(ModuleType::Weapon, 
            ModuleStats::new("Pulse Turret", 20, -2, 100.0)
            .with_combat(pulse_turret.range, pulse_turret.damage, pulse_turret.fire_rate)
        );

        // 3. Defense
        stats.insert(ModuleType::Defense,
            ModuleStats::new("Shield Gen", 30, -3, 150.0)
        );

        // 4. Utility
        stats.insert(ModuleType::Utility,
             ModuleStats::new("Recycler", 25, -1, 80.0)
        );
        
        // 5. Engine
        stats.insert(ModuleType::Engine,
            ModuleStats::new("Hyperdrive", 500, -50, 500.0)
        );

        // 6. Empty
        stats.insert(ModuleType::Empty, ModuleStats::new("Empty Slot", 0, 0, 0.0));

        Self { stats }
    }

    pub fn get(&self, module_type: ModuleType) -> &ModuleStats {
        self.stats.get(&module_type).unwrap_or_else(|| self.stats.get(&ModuleType::Empty).unwrap())
    }
}
