//! Game persistence (save/load)
//!
//! Save and load functionality - only available on native platforms (not WASM).

use crate::state::game_state::GameState;
use crate::state::persistence::SaveData;
use crate::enemy::entities::{Enemy, Projectile, Particle, ScrapPile};
use macroquad::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{BufReader, BufWriter};

#[cfg(not(target_arch = "wasm32"))]
use crate::state::persistence::{SavedEnemy, SavedProjectile, SavedParticle, SavedScrapPile};

#[cfg(not(target_arch = "wasm32"))]
impl GameState {
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        let save_data = SaveData {
            ship: self.ship.clone(),
            resources: self.resources.clone(),
            phase: self.phase,
            engine_state: self.engine_state,
            escape_timer: self.escape_timer,
            enemies: self.enemies.iter().map(|e| SavedEnemy {
                id: e.id,
                enemy_type: e.enemy_type.clone(),
                pos: (e.position.x, e.position.y),
                hp: e.health,
                max_hp: e.max_health,
                speed: e.speed,
                damage: e.damage,
                target: e.target_module,
                attached_to: e.attached_to,
                ability_timer: e.ability_timer,
            }).collect(),
            projectiles: self.projectiles.iter().map(|p| SavedProjectile {
                pos: (p.position.x, p.position.y),
                vel: (p.velocity.x, p.velocity.y),
                damage: p.damage,
                active: p.active,
            }).collect(),
            particles: self.particles.iter().map(|p| SavedParticle {
                pos: (p.position.x, p.position.y),
                vel: (p.velocity.x, p.velocity.y),
                life: p.lifetime,
                max_life: p.max_lifetime,
                color: (p.color.r, p.color.g, p.color.b, p.color.a),
                active: p.active,
            }).collect(),
            scrap_piles: self.scrap_piles.iter().map(|p| SavedScrapPile {
                pos: (p.position.x, p.position.y),
                amount: p.amount,
                active: p.active,
            }).collect(),
            upgrades: self.upgrades.clone(),
            frame_count: self.frame_count,
            time_survived: self.time_survived,
            room_repair_states: self.interior.rooms.iter()
                .map(|room| room.repair_points.iter().map(|rp| rp.repaired).collect())
                .collect(),
            player_pos: (self.player.position.x, self.player.position.y),
            view_mode: self.view_mode,
            tutorial_index: self.tutorial_state.current_index,
            tutorial_completed: self.tutorial_state.completed,
        };
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &save_data)?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let save_data: SaveData = serde_json::from_reader(reader)?;
        let mut state = GameState::new();
        state.ship = save_data.ship;
        state.resources = save_data.resources;
        state.phase = save_data.phase;
        state.engine_state = save_data.engine_state;
        state.escape_timer = save_data.escape_timer;
        state.upgrades = save_data.upgrades;
        state.frame_count = save_data.frame_count;
        state.time_survived = save_data.time_survived;
        state.enemies = save_data.enemies.into_iter().map(|s| Enemy {
            id: s.id,
            enemy_type: s.enemy_type,
            position: vec2(s.pos.0, s.pos.1),
            health: s.hp,
            max_health: s.max_hp,
            speed: s.speed,
            damage: s.damage,
            target_module: s.target,
            attached_to: s.attached_to,
            ability_timer: s.ability_timer,
            attacking: false,
        }).collect();
        state.projectiles = save_data.projectiles.into_iter().map(|s| Projectile {
            position: vec2(s.pos.0, s.pos.1),
            velocity: vec2(s.vel.0, s.vel.1),
            damage: s.damage,
            active: s.active,
        }).collect();
        state.particles = save_data.particles.into_iter().map(|s| Particle {
            position: vec2(s.pos.0, s.pos.1),
            velocity: vec2(s.vel.0, s.vel.1),
            lifetime: s.life,
            max_lifetime: s.max_life,
            color: Color::new(s.color.0, s.color.1, s.color.2, s.color.3),
            active: s.active,
        }).collect();
        state.scrap_piles = save_data.scrap_piles.into_iter().map(|s| ScrapPile {
            position: vec2(s.pos.0, s.pos.1),
            amount: s.amount,
            active: s.active,
        }).collect();
        
        // Restore interior repair states
        for (room_idx, repair_states) in save_data.room_repair_states.into_iter().enumerate() {
            if room_idx < state.interior.rooms.len() {
                for (point_idx, repaired) in repair_states.into_iter().enumerate() {
                    if point_idx < state.interior.rooms[room_idx].repair_points.len() {
                        state.interior.rooms[room_idx].repair_points[point_idx].repaired = repaired;
                    }
                }
            }
        }
        
        // Restore player position
        state.player.position = vec2(save_data.player_pos.0, save_data.player_pos.1);
        state.view_mode = save_data.view_mode;
        
        // Restore tutorial state
        state.tutorial_state.current_index = save_data.tutorial_index;
        state.tutorial_state.completed = save_data.tutorial_completed;
        
        Ok(state)
    }

    pub fn get_save_slot_path(slot: usize) -> String {
        format!("save_slot_{}.json", slot)
    }

    pub fn save_to_slot(&self, slot: usize) -> std::io::Result<()> {
        let path = Self::get_save_slot_path(slot);
        self.save(&path)
    }

    pub fn load_from_slot(slot: usize) -> std::io::Result<Self> {
        let path = Self::get_save_slot_path(slot);
        Self::load_from_file(&path)
    }
}
