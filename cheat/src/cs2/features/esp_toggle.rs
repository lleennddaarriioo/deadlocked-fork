use crate::{
    config::{Config, aim::KeyMode},
    cs2::CS2,
};

#[derive(Debug)]
pub struct EspToggle {
    pub active: bool,
    pub item_active: bool,
}

impl Default for EspToggle {
    fn default() -> Self {
        Self { active: true, item_active: true }
    }
}

impl CS2 {
    pub fn esp_toggle(&mut self, config: &Config) {
        let hotkey = config.player.esp_hotkey;
        Self::check_hotkey(&self.input, KeyMode::Toggle, hotkey, crate::cs2::key_codes::KeyCode::None, &mut self.esp.active);

        let item_hotkey = config.hud.item_esp_hotkey;
        Self::check_hotkey(&self.input, KeyMode::Toggle, item_hotkey, crate::cs2::key_codes::KeyCode::None, &mut self.esp.item_active);
    }

    pub fn esp_enabled(&self, config: &Config) -> bool {
        config.player.enabled && self.esp.active
    }

    pub fn item_esp_enabled(&self, _config: &Config) -> bool {
        self.esp.item_active
    }
}
