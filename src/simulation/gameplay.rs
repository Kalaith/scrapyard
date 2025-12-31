use crate::ship::ship::ModuleType;
use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct ModulesJson {
    modules: HashMap<String, ModuleConfigRaw>
}

#[derive(Debug, Clone, Deserialize)]
struct ModuleConfigRaw {
    name: String,
    base_cost: i32,
    #[serde(default)]
    power_generation: i32,
    #[serde(default)]
    power_consumption: i32,
    max_health: f32,
    #[serde(default)]
    range: f32,
    #[serde(default)]
    damage: f32,
    #[serde(default)]
    fire_rate: f32,
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

        // Load modules config from embedded JSON
        let json_content = include_str!("../../assets/modules.json");
        let config: ModulesJson = serde_json::from_str(json_content)
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to parse modules.json: {}. Using hardcoded defaults.", e);
                // Return empty so defaults below are used, or panic? 
                // Better to panic in dev if assets are broken.
                // But let's return a basic struct to avoid crash if possible, but map lookups will fail.
                ModulesJson { modules: HashMap::new() }
            });

        // Helper to determine module type from string
        fn get_module_type(key: &str) -> Option<ModuleType> {
             match key.to_lowercase().as_str() {
                 "core" => Some(ModuleType::Core),
                 "weapon" => Some(ModuleType::Weapon),
                 "defense" => Some(ModuleType::Defense),
                 "utility" => Some(ModuleType::Utility),
                 "engine" => Some(ModuleType::Engine),
                 "empty" => Some(ModuleType::Empty),
                 _ => None,
             }
        }

        for (key, raw) in config.modules {
             if let Some(mod_type) = get_module_type(&key) {
                 let power = if raw.power_generation > 0 { raw.power_generation } else { -raw.power_consumption };
                 let stats_obj = ModuleStats::new(&raw.name, raw.base_cost, power, raw.max_health)
                     .with_combat(raw.range, raw.damage, raw.fire_rate);
                 stats.insert(mod_type, stats_obj);
             } else {
                 eprintln!("Warning: Unknown module type in JSON: {}", key);
             }
        }

        // Ensure Empty exists if not in JSON
        stats.entry(ModuleType::Empty).or_insert_with(|| ModuleStats::new("Empty Slot", 0, 0, 0.0));

        Self { stats }
    }

    pub fn get(&self, module_type: ModuleType) -> &ModuleStats {
        self.stats.get(&module_type).unwrap_or_else(|| self.stats.get(&ModuleType::Empty).unwrap())
    }
}
