// interior.rs - FTL-style ship interior with JSON-loaded room layout

use macroquad::prelude::*;
use serde::Deserialize;
use crate::ship::ship::ModuleType;

/// Room size constants (for default sizing)
pub const ROOM_SIZE: f32 = 64.0;
pub const CORRIDOR_WIDTH: f32 = 32.0;
pub const REPAIR_POINT_SIZE: f32 = 24.0;

/// A repair point within a room (subsystem to repair)
#[derive(Debug, Clone)]
pub struct RepairPoint {
    pub id: usize,
    pub x: f32,         // Position relative to room
    pub y: f32,
    pub repaired: bool, // Whether this point has been repaired
}

impl RepairPoint {
    pub fn new(id: usize, x: f32, y: f32) -> Self {
        Self { id, x, y, repaired: false }
    }
    
    /// Check if position is within this repair point
    pub fn contains(&self, room_x: f32, room_y: f32, pos: Vec2) -> bool {
        let px = room_x + self.x;
        let py = room_y + self.y;
        let half = REPAIR_POINT_SIZE / 2.0;
        pos.x >= px - half && pos.x <= px + half &&
        pos.y >= py - half && pos.y <= py + half
    }
}

/// JSON structure for repair point data
#[derive(Debug, Clone, Deserialize)]
pub struct RepairPointData {
    pub x: f32,
    pub y: f32,
}

/// JSON structure for room data
#[derive(Debug, Clone, Deserialize)]
pub struct RoomData {
    pub id: usize,
    #[serde(rename = "type")]
    pub room_type: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    #[serde(default)]
    pub module: Option<[usize; 2]>,
    #[serde(default)]
    pub connections: Vec<usize>,
    #[serde(default)]
    pub repair_points: Vec<RepairPointData>,
}

/// JSON structure for ship data
#[derive(Debug, Clone, Deserialize)]
pub struct ShipData {
    pub name: String,
    pub description: String,
    pub width: f32,
    pub height: f32,
    pub rooms: Vec<RoomData>,
    pub player_start_room: usize,
}

/// Type of room in the ship interior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomType {
    Module(ModuleType),
    Corridor,
    Storage,
    Cockpit,
    Medbay,
    Empty,
}

impl RoomType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "core" => RoomType::Module(ModuleType::Core),
            "weapon" => RoomType::Module(ModuleType::Weapon),
            "shield" => RoomType::Module(ModuleType::Defense),
            "engine" => RoomType::Module(ModuleType::Engine),
            "utility" => RoomType::Module(ModuleType::Utility),
            "corridor" => RoomType::Corridor,
            "storage" => RoomType::Storage,
            "cockpit" => RoomType::Cockpit,
            "medbay" => RoomType::Medbay,
            _ => RoomType::Empty,
        }
    }
}

/// A room in the ship interior
#[derive(Debug, Clone)]
pub struct Room {
    pub id: usize,
    pub room_type: RoomType,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub module_index: Option<(usize, usize)>,
    pub connections: Vec<usize>,
    pub repair_points: Vec<RepairPoint>,
}

impl Room {
    pub fn new(id: usize, room_type: RoomType, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            id,
            room_type,
            x,
            y,
            width,
            height,
            module_index: None,
            connections: Vec::new(),
            repair_points: Vec::new(),
        }
    }

    /// Count how many repair points are repaired
    pub fn repaired_count(&self) -> usize {
        self.repair_points.iter().filter(|p| p.repaired).count()
    }

    /// Check if all repair points are repaired
    pub fn is_fully_repaired(&self) -> bool {
        !self.repair_points.is_empty() && self.repair_points.iter().all(|p| p.repaired)
    }

    /// Get repair point at position (if any)
    pub fn repair_point_at(&self, pos: Vec2) -> Option<usize> {
        for (i, point) in self.repair_points.iter().enumerate() {
            if point.contains(self.x, self.y, pos) {
                return Some(i);
            }
        }
        None
    }

    pub fn with_module(mut self, gx: usize, gy: usize) -> Self {
        self.module_index = Some((gx, gy));
        self
    }

    pub fn with_connections(mut self, connections: Vec<usize>) -> Self {
        self.connections = connections;
        self
    }

    pub fn contains(&self, pos: Vec2) -> bool {
        pos.x >= self.x && pos.x <= self.x + self.width &&
        pos.y >= self.y && pos.y <= self.y + self.height
    }

    pub fn center(&self) -> Vec2 {
        vec2(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn color(&self) -> Color {
        match self.room_type {
            RoomType::Module(ModuleType::Core) => color_u8!(100, 30, 30, 255),
            RoomType::Module(ModuleType::Weapon) => color_u8!(100, 80, 30, 255),
            RoomType::Module(ModuleType::Defense) => color_u8!(30, 50, 100, 255),
            RoomType::Module(ModuleType::Engine) => color_u8!(80, 30, 100, 255),
            RoomType::Module(ModuleType::Utility) => color_u8!(30, 80, 50, 255),
            RoomType::Module(ModuleType::Empty) => color_u8!(50, 50, 55, 255),
            RoomType::Corridor => color_u8!(40, 40, 45, 255),
            RoomType::Storage => color_u8!(60, 55, 45, 255),
            RoomType::Cockpit => color_u8!(50, 70, 90, 255),
            RoomType::Medbay => color_u8!(80, 80, 100, 255),
            RoomType::Empty => color_u8!(20, 20, 25, 255),
        }
    }

    pub fn name(&self) -> &'static str {
        match self.room_type {
            RoomType::Module(ModuleType::Core) => "REACTOR",
            RoomType::Module(ModuleType::Weapon) => "WEAPONS",
            RoomType::Module(ModuleType::Defense) => "SHIELDS",
            RoomType::Module(ModuleType::Engine) => "ENGINES",
            RoomType::Module(ModuleType::Utility) => "SYSTEMS",
            RoomType::Module(ModuleType::Empty) => "",
            RoomType::Corridor => "",
            RoomType::Storage => "STORAGE",
            RoomType::Cockpit => "COCKPIT",
            RoomType::Medbay => "MEDBAY",
            RoomType::Empty => "",
        }
    }
}

/// The ship interior layout
pub struct ShipInterior {
    pub rooms: Vec<Room>,
    pub width: f32,
    pub height: f32,
}

impl ShipInterior {
    /// Load ship layout from JSON string (embedded at compile time)
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        let data: ShipData = serde_json::from_str(json_str)?;
        
        let rooms: Vec<Room> = data.rooms.iter().map(|rd| {
            let room_type = RoomType::from_str(&rd.room_type);
            let mut room = Room::new(rd.id, room_type, rd.x, rd.y, rd.w, rd.h);
            room.connections = rd.connections.clone();
            if let Some([gx, gy]) = rd.module {
                room.module_index = Some((gx, gy));
            }
            // Load repair points
            room.repair_points = rd.repair_points.iter().enumerate()
                .map(|(i, rp)| RepairPoint::new(i, rp.x, rp.y))
                .collect();
            room
        }).collect();

        Ok(Self {
            rooms,
            width: data.width,
            height: data.height,
        })
    }

    /// Create the starter ship layout from JSON
    pub fn starter_ship() -> Self {
        const SHIP_JSON: &str = include_str!("../../assets/ships/starter_ship.json");
        Self::from_json(SHIP_JSON).unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load starter ship: {}. Using fallback.", e);
            Self {
                rooms: Vec::new(),
                width: 1000.0,
                height: 600.0,
            }
        })
    }

    pub fn player_start_position(&self) -> Vec2 {
        // Room 12 is the core (player start)
        if let Some(room) = self.rooms.iter().find(|r| r.id == 12) {
            room.center()
        } else if let Some(first) = self.rooms.first() {
            first.center()
        } else {
            vec2(200.0, 150.0)
        }
    }

    /// Find the room containing a position
    pub fn room_at(&self, pos: Vec2) -> Option<&Room> {
        self.rooms.iter().find(|r| r.contains(pos))
    }

    /// Check if position is walkable (in a non-Empty room)
    pub fn is_walkable(&self, pos: Vec2) -> bool {
        if let Some(room) = self.room_at(pos) {
            room.room_type != RoomType::Empty
        } else {
            false
        }
    }

    /// Get module room if player is in one
    pub fn module_room_at(&self, pos: Vec2) -> Option<&Room> {
        self.room_at(pos).filter(|r| matches!(r.room_type, RoomType::Module(_)))
    }
}
