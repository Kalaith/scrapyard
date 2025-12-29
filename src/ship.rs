use macroquad::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleType {
    Weapon,
    Defense,
    Utility,
    Engine,
    Core,
    Empty, // Slot exists but no module built
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleState {
    Destroyed,
    Offline,
    Active,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub module_type: ModuleType,
    pub state: ModuleState,
    pub level: u8,
    pub health: f32,
    pub max_health: f32,
}

impl Module {
    pub fn new(module_type: ModuleType) -> Self {
        Self {
            module_type,
            state: ModuleState::Destroyed, // Default to destroyed for repair mechanics
            level: 1,
            health: 100.0,
            max_health: 100.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ship {
    pub grid: Vec<Vec<Option<Module>>>,
}

impl Ship {
    pub fn new(width: usize, height: usize) -> Self {
        let mut grid = vec![vec![None; height]; width];
        
        // Initialize starting ship layout
        let cx = width / 2;
        let cy = height / 2;
        
        // Core (Active)
        let mut core = Module::new(ModuleType::Core);
        core.state = ModuleState::Active;
        core.health = 1000.0;
        core.max_health = 1000.0;
        grid[cx][cy] = Some(core);

        // Adjacent slots - mix of weapons and empty
        // Left: Weapon (Active)
        let mut weapon1 = Module::new(ModuleType::Weapon);
        weapon1.state = ModuleState::Active;
        grid[cx-1][cy] = Some(weapon1);
        
        // Right: Weapon (Active)
        let mut weapon2 = Module::new(ModuleType::Weapon);
        weapon2.state = ModuleState::Active;
        grid[cx+1][cy] = Some(weapon2);
        
        // Top: Empty slot (for building)
        grid[cx][cy-1] = Some(Module::new(ModuleType::Empty));
        
        // Bottom: Defense (Destroyed - needs repair)
        let defense = Module::new(ModuleType::Defense);
        grid[cx][cy+1] = Some(defense);
        
        // Corners: More empty slots for expansion
        grid[cx-1][cy-1] = Some(Module::new(ModuleType::Empty));
        grid[cx+1][cy-1] = Some(Module::new(ModuleType::Empty));
        grid[cx-1][cy+1] = Some(Module::new(ModuleType::Empty));
        grid[cx+1][cy+1] = Some(Module::new(ModuleType::Empty));
        
        // Engine (far from core, destroyed - needs repair to escape)
        let engine = Module::new(ModuleType::Engine);
        grid[cx][cy+3] = Some(engine);

        Self { grid }
    }

    /// Check if a grid coordinate is a valid slot (has a module or empty slot).
    pub fn is_valid_slot(&self, x: usize, y: usize) -> bool {
        if x >= self.grid.len() {
            return false;
        }
        if y >= self.grid[x].len() {
            return false;
        }
        self.grid[x][y].is_some()
    }

    /// Find the core position in the grid.
    pub fn find_core(&self) -> Option<(usize, usize)> {
        for (x, row) in self.grid.iter().enumerate() {
            for (y, cell) in row.iter().enumerate() {
                if let Some(module) = cell {
                    if module.module_type == ModuleType::Core {
                        return Some((x, y));
                    }
                }
            }
        }
        None
    }

    /// Calculate path from a starting position to the core using BFS.
    /// Returns the path as a vector of (x, y) coordinates, or None if no path exists.
    pub fn calculate_path_to_core(&self, start: (usize, usize)) -> Option<Vec<(usize, usize)>> {
        use std::collections::{VecDeque, HashMap};

        let core_pos = self.find_core()?;
        if start == core_pos {
            return Some(vec![start]);
        }

        let width = self.grid.len();
        let height = if width > 0 { self.grid[0].len() } else { 0 };

        let mut queue = VecDeque::new();
        let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        
        queue.push_back(start);
        came_from.insert(start, start);

        let directions: [(i32, i32); 4] = [(0, -1), (0, 1), (-1, 0), (1, 0)];

        while let Some(current) = queue.pop_front() {
            if current == core_pos {
                // Reconstruct path
                let mut path = vec![current];
                let mut pos = current;
                while pos != start {
                    pos = *came_from.get(&pos)?;
                    path.push(pos);
                }
                path.reverse();
                return Some(path);
            }

            for (dx, dy) in directions {
                let nx = current.0 as i32 + dx;
                let ny = current.1 as i32 + dy;

                if nx >= 0 && ny >= 0 {
                    let nx = nx as usize;
                    let ny = ny as usize;

                    if nx < width && ny < height && !came_from.contains_key(&(nx, ny)) {
                        // Allow movement through any cell (not just module slots)
                        queue.push_back((nx, ny));
                        came_from.insert((nx, ny), current);
                    }
                }
            }
        }

        None // No path found
    }
}

