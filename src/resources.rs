use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resources {
    pub scrap: i32,
    pub max_scrap: i32,
    pub power: i32,
    pub credits: i32,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            scrap: 50, // Starting scrap (lowered as requested)
            max_scrap: 1000,
            power: 0,
            credits: 0,
        }
    }

    pub fn can_afford(&self, cost: i32) -> bool {
        self.scrap >= cost
    }

    pub fn deduct(&mut self, cost: i32) {
        if self.can_afford(cost) {
            self.scrap -= cost;
        }
    }
    
    pub fn add_scrap(&mut self, amount: i32) {
        self.scrap = (self.scrap + amount).min(self.max_scrap);
    }
}
