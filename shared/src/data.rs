use std::collections::HashMap;

use glam::{Mat4, Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::{bones::Bones, entity::EntityInfo, weapon::Weapon};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SoundType {
    Footstep,
    Gunshot,
    Weapon,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Data {
    pub in_game: bool,
    pub is_ffa: bool,
    pub weapon: Weapon,
    pub players: Vec<PlayerData>,
    pub friendlies: Vec<PlayerData>,
    pub spectators: Vec<String>,
    pub local_player: PlayerData,
    pub entities: Vec<EntityInfo>,
    pub bomb: BombData,
    pub map_name: String,
    pub view_matrix: Mat4,
    pub view_angles: Vec2,
    pub window_position: Vec2,
    pub window_size: Vec2,
    pub aimbot_active: bool,
    pub triggerbot_active: bool,
    pub esp_active: bool,
    pub item_esp_active: bool,
    pub total_damage: u32,
    pub aim_punch: Vec2,
    pub looking_at_material: Option<u8>,
    pub bvh_loaded: bool,
    pub bvh_triangles_count: usize,
    pub raycast_hit: Option<(f32, u8)>,
    pub eye_pos: Vec3,
    pub ray_dir: Vec3,
    pub wall_thickness: Option<f32>,
    pub penetration_damage: Option<f32>,
    pub penetration_headshot_damage: Option<f32>,
    pub jump_cps: u32,
    pub total_cps: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerData {
    pub steam_id: u64,
    pub health: i32,
    pub armor: i32,
    pub position: Vec3,
    pub head: Vec3,
    pub name: String,
    pub weapon: Weapon,
    pub ammo: (i32, i32),
    pub bones: HashMap<Bones, Vec3>,
    pub has_defuser: bool,
    pub has_helmet: bool,
    pub has_bomb: bool,
    pub visible: bool,
    pub color: i32,
    pub rotation: f32,
    pub sound: Option<SoundType>,
    pub velocity: Vec3,
    pub fov: i32,
    pub shots_fired: i32,
    pub inaccuracy: f32,
    pub is_defusing: bool,
    pub visible_bones: HashMap<Bones, bool>,
    pub round_kills: i32,
    #[serde(skip)]
    pub chams_segments: Vec<(Vec3, Vec3, bool, f32)>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BombData {
    pub planted: bool,
    pub timer: f32,
    pub being_defused: bool,
    pub position: Vec3,
    pub defuse_remain_time: f32,
}
