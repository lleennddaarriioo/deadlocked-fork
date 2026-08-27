use std::{collections::HashMap, ops::RangeInclusive};

use glam::Vec2;
use serde::{Deserialize, Serialize};
use shared::{bones::Bones, weapon::Weapon};
use strum::{EnumIter, IntoEnumIterator};

use crate::cs2::key_codes::KeyCode;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WeaponConfig {
    pub aimbot: AimbotConfig,
    pub rcs: RcsConfig,
    pub triggerbot: TriggerbotConfig,
}

impl WeaponConfig {
    pub fn enabled(enabled: bool) -> Self {
        let aimbot = AimbotConfig {
            enable_override: enabled,
            ..Default::default()
        };
        Self {
            aimbot,
            rcs: RcsConfig::default(),
            triggerbot: TriggerbotConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, EnumIter)]
pub enum VisibilityMode {
    BoneLoS,
    BoneFast,
    PixelLoS,
    Both,
    Either,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AimbotConfig {
    pub enable_override: bool,
    pub enabled: bool,
    pub mode: KeyMode,
    pub target_friendlies: bool,
    pub distance_adjusted_fov: bool,
    pub start_bullet: i32,
    pub visibility_check: bool,
    pub visibility_mode: VisibilityMode,
    pub flash_check: bool,
    pub fov: f32,
    pub smooth: f32,
    pub inertia: f32,
    pub bones: Vec<Bones>,
    pub bone_mode: BoneMode,
    pub targeting_mode: TargetingMode,
    pub damage_threshold: f32,
}

impl Default for AimbotConfig {
    fn default() -> Self {
        Self {
            enable_override: false,
            enabled: true,
            mode: KeyMode::Hold,
            target_friendlies: false,
            distance_adjusted_fov: true,
            start_bullet: 0,
            visibility_check: true,
            visibility_mode: VisibilityMode::Either,
            flash_check: true,
            fov: 2.5,
            smooth: 5.0,
            inertia: 1.0,
            bones: vec![
                Bones::Head,
                Bones::Neck,
                Bones::Spine4,
                Bones::Spine3,
                Bones::Spine2,
                Bones::Spine1,
                Bones::Hip,
            ],
            bone_mode: BoneMode::Nearest,
            targeting_mode: TargetingMode::Fov,
            damage_threshold: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RcsConfig {
    pub enable_override: bool,
    pub enabled: bool,
    pub strength: Vec2,
}

impl Default for RcsConfig {
    fn default() -> Self {
        Self {
            enable_override: false,
            enabled: false,
            strength: Vec2::splat(0.5),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, EnumIter)]
pub enum KeyMode {
    Hold,
    Toggle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumIter)]
pub enum BoneMode {
    Nearest,
    Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumIter)]
pub enum TargetingMode {
    Fov,
    Distance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TriggerbotConfig {
    pub enable_override: bool,
    pub enabled: bool,
    pub delay: RangeInclusive<u64>,
    pub shot_duration: u64,
    pub mode: KeyMode,
    pub visibility_check: bool,
    pub visibility_mode: VisibilityMode,
    pub flash_check: bool,
    pub scope_check: bool,
    pub velocity_check: bool,
    pub velocity_threshold: f32,
    pub head_only: bool,
    pub hit_chance: f32,
    pub damage_threshold: f32,
}

impl Default for TriggerbotConfig {
    fn default() -> Self {
        Self {
            enable_override: false,
            enabled: false,
            delay: 100..=200,
            shot_duration: 200,
            mode: KeyMode::Hold,
            visibility_check: true,
            visibility_mode: VisibilityMode::Either,
            flash_check: true,
            scope_check: true,
            velocity_check: true,
            velocity_threshold: 100.0,
            head_only: false,
            hit_chance: 1.0,
            damage_threshold: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AimConfig {
    pub aimbot_hotkey: KeyCode,
    pub triggerbot_hotkey: KeyCode,
    pub triggerbot_hotkey2: KeyCode,
    pub auto_pistol: bool,
    pub global: WeaponConfig,
    pub weapons: HashMap<Weapon, WeaponConfig>,
}

impl Default for AimConfig {
    fn default() -> Self {
        let mut weapons = HashMap::new();
        for weapon in Weapon::iter() {
            weapons.insert(weapon, WeaponConfig::default());
        }

        Self {
            aimbot_hotkey: KeyCode::Mouse5,
            triggerbot_hotkey: KeyCode::Mouse4,
            triggerbot_hotkey2: KeyCode::LeftAlt,
            auto_pistol: false,
            global: WeaponConfig::enabled(true),
            weapons,
        }
    }
}
