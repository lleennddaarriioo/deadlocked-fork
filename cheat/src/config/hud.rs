use egui::Color32;
use serde::{Deserialize, Serialize};

use crate::ui::color::Colors;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HudConfig {
    pub bomb_timer: bool,
    pub fov_circle: bool,
    pub sniper_crosshair: CrosshairConfig,
    pub recoil_crosshair: CrosshairConfig,
    pub dropped_weapons: bool,
    pub item_esp_hotkey: crate::cs2::key_codes::KeyCode,
    pub keybind_list: bool,
    pub spectator_list: bool,
    pub grenade_trails: TrailConfig,
    pub text_outline: bool,
    pub text_color: Color32,
    pub line_width: f32,
    pub font_size: f32,
    pub icon_size: f32,
    pub debug: bool,
    pub raycast_debug: bool,
    pub hit_marker: bool,
    pub spread_circle: bool,
    pub spread_circle_color: Color32,
    pub overlay_offset_x: i32,
    pub overlay_offset_y: i32,
    pub grenade_warning: GrenadeWarningConfig,
    pub floating_damage: FloatingDamageConfig,
    pub hitsound: HitsoundConfig,
}

impl Default for HudConfig {
    fn default() -> Self {
        Self {
            bomb_timer: true,
            fov_circle: false,
            sniper_crosshair: CrosshairConfig::default(),
            recoil_crosshair: CrosshairConfig::default(),
            dropped_weapons: true,
            item_esp_hotkey: crate::cs2::key_codes::KeyCode::None,
            keybind_list: false,
            spectator_list: false,
            grenade_trails: TrailConfig::default(),
            text_outline: true,
            text_color: Colors::TEXT,
            line_width: 2.0,
            font_size: 16.0,
            icon_size: 20.0,
            debug: false,
            raycast_debug: false,
            hit_marker: false,
            spread_circle: false,
            spread_circle_color: Color32::from_rgba_premultiplied(255, 255, 255, 120),
            overlay_offset_x: 0,
            overlay_offset_y: 0,
            grenade_warning: GrenadeWarningConfig::default(),
            floating_damage: FloatingDamageConfig::default(),
            hitsound: HitsoundConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, strum::EnumIter, Serialize, Deserialize)]
pub enum HitsoundPreset {
    RustHeadshot,
    CodHitmarker,
    MetallicBell,
    CsgoDing,
    Bubble,
    Neverlose,
    Skeet,
    Aimware,
    Primordial,
    CustomWav,
}

impl HitsoundPreset {
    pub fn name(&self) -> &'static str {
        match self {
            HitsoundPreset::RustHeadshot => "Rust Headshot",
            HitsoundPreset::CodHitmarker => "COD Hitmarker",
            HitsoundPreset::MetallicBell => "Metallic Bell",
            HitsoundPreset::CsgoDing => "CSGO Ding",
            HitsoundPreset::Bubble => "Bubble",
            HitsoundPreset::Neverlose => "Neverlose (NL)",
            HitsoundPreset::Skeet => "Skeet / Gamesense (GS)",
            HitsoundPreset::Aimware => "Aimware / Mutiny (MS)",
            HitsoundPreset::Primordial => "Primordial",
            HitsoundPreset::CustomWav => "Custom WAV File",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HitsoundConfig {
    pub enabled: bool,
    pub only_local_player: bool,
    pub only_one_tap: bool,
    pub preset: HitsoundPreset,
    pub kill_preset: HitsoundPreset,
    pub volume: f32,
    pub pitch: f32,
    pub custom_wav_name: String,
}

impl Default for HitsoundConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            only_local_player: true,
            only_one_tap: false,
            preset: HitsoundPreset::CodHitmarker,
            kill_preset: HitsoundPreset::RustHeadshot,
            volume: 0.8,
            pitch: 1.0,
            custom_wav_name: "hit.wav".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GrenadeWarningConfig {
    pub enabled: bool,
    pub draw_radius: bool,
    pub danger_banner: bool,
}

impl Default for GrenadeWarningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            draw_radius: true,
            danger_banner: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FloatingDamageConfig {
    pub enabled: bool,
    pub duration_secs: f32,
    pub scale: f32,
}

impl Default for FloatingDamageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            duration_secs: 2.0,
            scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CrosshairConfig {
    pub enabled: bool,
    pub color: Color32,
    pub line_length: f32,
    pub line_width: f32,
    pub gap: f32,
}

impl Default for CrosshairConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            color: Color32::WHITE,
            line_length: 50.0,
            line_width: 2.0,
            gap: 20.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TrailConfig {
    pub enabled: bool,
    pub smoke: Color32,
    pub molotov: Color32,
    pub incendiary: Color32,
    pub flash: Color32,
    pub he: Color32,
    pub decoy: Color32,
}

impl Default for TrailConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            smoke: Color32::LIGHT_GRAY,
            molotov: Color32::RED,
            incendiary: Color32::ORANGE,
            flash: Color32::WHITE,
            he: Color32::DARK_GRAY,
            decoy: Color32::PURPLE,
        }
    }
}
