use std::{collections::HashMap, fs::read_to_string};

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use shared::weapon::Weapon;
use uuid::Uuid;

use crate::{config::BASE_PATH, constants::GRENADE_FILE_NAME};

pub type GrenadeList = HashMap<String, Vec<Grenade>>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Grenade {
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub position: Vec3,
    pub view_angles: Vec2,
    pub weapon: Weapon,
    #[serde(default)]
    pub modifiers: GrenadeModifiers,
}

impl Grenade {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct GrenadeModifiers {
    pub jump: bool,
    pub duck: bool,
    pub run: bool,
}

use std::sync::Mutex;
use std::time::Instant;

static GRENADE_CACHE: Mutex<Option<(Instant, GrenadeList)>> = Mutex::new(None);

pub fn read_grenades() -> GrenadeList {
    if let Ok(guard) = GRENADE_CACHE.lock() {
        if let Some((time, ref list)) = *guard {
            if time.elapsed() < std::time::Duration::from_secs(2) {
                return list.clone();
            }
        }
    }

    let path = BASE_PATH.join(GRENADE_FILE_NAME);
    if !path.exists() {
        utils::info!("no grenade list found");
        return GrenadeList::default();
    }

    let grenade_list_file = read_to_string(path).unwrap();
    let grenade_list: GrenadeList = serde_json::from_str(&grenade_list_file).unwrap_or_default();
    
    if let Ok(mut guard) = GRENADE_CACHE.lock() {
        *guard = Some((Instant::now(), grenade_list.clone()));
    }

    grenade_list
}

pub fn write_grenades(grenades: &GrenadeList) {
    let out = serde_json::to_string(grenades).unwrap();
    let path = BASE_PATH.join(GRENADE_FILE_NAME);
    std::fs::write(path, out).unwrap();
    if let Ok(mut guard) = GRENADE_CACHE.lock() {
        *guard = None;
    }
}
