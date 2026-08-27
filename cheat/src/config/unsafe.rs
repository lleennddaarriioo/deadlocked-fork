use egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UnsafeConfig {
    pub bunnyhop: bool,
    pub bhop_hotkey: crate::cs2::key_codes::KeyCode,
    pub autostrafe: bool,
    pub counter_strafe: bool,
    pub edge_jump: bool,
    pub edge_jump_hotkey: crate::cs2::key_codes::KeyCode,
    pub grenade_move_hotkey: crate::cs2::key_codes::KeyCode,
    pub mic_tone: bool,
    pub mic_tone_hotkey: crate::cs2::key_codes::KeyCode,
    pub mic_tone_mode: usize,
    pub mic_tone_frequency: f32,
    pub mic_tone_volume: f32,
    pub mic_hw_boost: f32,
    pub no_flash: bool,
    pub max_flash_alpha: f32,
    pub fov_changer: bool,
    pub desired_fov: u32,
    pub no_smoke: bool,
    pub change_smoke_color: bool,
    pub smoke_color: Color32,
    pub radar: bool,
}

impl Default for UnsafeConfig {
    fn default() -> Self {
        Self {
            bunnyhop: false,
            bhop_hotkey: crate::cs2::key_codes::KeyCode::Space,
            autostrafe: false,
            counter_strafe: false,
            edge_jump: false,
            edge_jump_hotkey: crate::cs2::key_codes::KeyCode::Space,
            grenade_move_hotkey: crate::cs2::key_codes::KeyCode::V,
            mic_tone: false,
            mic_tone_hotkey: crate::cs2::key_codes::KeyCode::Z,
            mic_tone_mode: 1,
            mic_tone_frequency: 4000.0,
            mic_tone_volume: 10.0,
            mic_hw_boost: 500.0,
            no_flash: false,
            max_flash_alpha: 127.0,
            fov_changer: false,
            desired_fov: 90,
            no_smoke: false,
            change_smoke_color: false,
            smoke_color: Color32::RED,
            radar: false,
        }
    }
}
