use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub base_cost: i32,
    pub cost_multiplier: f32,
    pub max_level: u32,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameUpgrades {
    pub levels: std::collections::HashMap<String, u32>,
}

impl GameUpgrades {
    pub fn new() -> Self {
        Self {
            levels: std::collections::HashMap::new(),
        }
    }

    pub fn get_level(&self, id: &str) -> u32 {
        *self.levels.get(id).unwrap_or(&0)
    }

    pub fn get_cost(&self, template: &UpgradeTemplate) -> i32 {
        let level = self.get_level(&template.id);
        (template.base_cost as f32 * template.cost_multiplier.powi(level as i32)) as i32
    }
}
