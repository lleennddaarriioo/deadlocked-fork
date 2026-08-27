use std::{
    sync::Arc,
    thread::sleep,
    time::{Duration, Instant},
};

use shared::data::Data;
use utils::{Channel, Mutex};

use crate::{
    config::Config,
    cs2::CS2,
    message::{GameMessage, GameStatus, UiMessage},
    os::{keyboard::Keyboard, mouse::Mouse},
};

#[derive(Debug, Default, Clone, Copy)]
pub struct TelemetryData {
    pub input_update_ms: f32,
    pub cache_entities_ms: f32,
    pub check_bvh_ms: f32,
    pub triggerbot_ms: f32,
    pub bhop_ms: f32,
    pub counter_strafe_ms: f32,
    pub aimbot_ms: f32,
    pub rcs_ms: f32,
    pub other_features_ms: f32,
    pub draw_data_ms: f32,
    pub total_loop_ms: f32,
    pub idle_ms: f32,
    pub target_frame_ms: f32,
}

impl TelemetryData {
    pub const fn new() -> Self {
        Self {
            input_update_ms: 0.0,
            cache_entities_ms: 0.0,
            check_bvh_ms: 0.0,
            triggerbot_ms: 0.0,
            bhop_ms: 0.0,
            counter_strafe_ms: 0.0,
            aimbot_ms: 0.0,
            rcs_ms: 0.0,
            other_features_ms: 0.0,
            draw_data_ms: 0.0,
            total_loop_ms: 0.0,
            idle_ms: 0.0,
            target_frame_ms: 0.0,
        }
    }
}

pub static TELEMETRY: std::sync::Mutex<TelemetryData> = std::sync::Mutex::new(TelemetryData::new());

pub struct GameManager {
    channel: Channel<UiMessage, GameMessage>,
    data: Arc<Mutex<Data>>,
    config: Config,
    mouse: Mouse,
    keyboard: Keyboard,
    cs2: CS2,
    previous_weapon: shared::weapon::Weapon,
    last_shots_fired: i32,
    last_total_damage: u32,
    last_shot_info: Option<(Instant, f32, f32)>, // (time, predicted_damage, predicted_headshot)
}

impl GameManager {
    pub fn new(channel: Channel<UiMessage, GameMessage>, data: Arc<Mutex<Data>>) -> Self {
        let mouse = match Mouse::open() {
            Ok(mouse) => mouse,
            Err(err) => {
                utils::error!("error creating uinput device: {err}");
                utils::error!("uinput kernel module is not loaded, or user is not in input group.");
                std::process::exit(1);
            }
        };

        let keyboard = match Keyboard::open() {
            Ok(keyboard) => keyboard,
            Err(err) => {
                utils::error!("error creating uinput keyboard device: {err}");
                std::process::exit(1);
            }
        };

        Self {
            channel,
            data,
            config: Config::default(),
            mouse,
            keyboard,
            cs2: CS2::new(),
            previous_weapon: shared::weapon::Weapon::Unknown,
            last_shots_fired: 0,
            last_total_damage: 0,
            last_shot_info: None,
        }
    }

    fn send_message(&self, message: UiMessage) {
        if self.channel.send(message).is_err() {
            std::process::exit(1);
        }
    }

    pub fn run(&mut self) {
        self.send_message(UiMessage::Status(GameStatus::NotStarted));
        let mut previous_status = GameStatus::NotStarted;
        loop {
            let start = Instant::now();
            while let Ok(message) = self.channel.try_receive() {
                self.config = *message.0;
            }

            let mut is_valid = self.cs2.is_valid();
            if !is_valid {
                if previous_status == GameStatus::Working {
                    self.send_message(UiMessage::Status(GameStatus::NotStarted));
                    previous_status = GameStatus::NotStarted;
                }
                self.cs2.setup();
                is_valid = self.cs2.is_valid();
            }

            if is_valid {
                if previous_status == GameStatus::NotStarted {
                    self.send_message(UiMessage::Status(GameStatus::Working));
                    previous_status = GameStatus::Working;
                }
                self.cs2.run(&self.config, &mut self.mouse, &mut self.keyboard);
                let (jump_cps, total_cps) = self.keyboard.get_cps();
                let mut data = self.data.lock();
                data.jump_cps = jump_cps;
                data.total_cps = total_cps;
                let t_data_start = Instant::now();
                self.cs2.data(&self.config, &mut data);

                if data.weapon != self.previous_weapon {
                    if data.weapon != shared::weapon::Weapon::Unknown {
                        utils::info!("[WEAPON] Switched to: {} (Damage: {})", data.weapon, data.weapon.damage_description());
                    }
                    self.previous_weapon = data.weapon.clone();
                }

                // Check shots fired changes to capture predicted damage
                if data.local_player.shots_fired > self.last_shots_fired {
                    let pred_dmg = data.penetration_damage.unwrap_or(data.weapon.base_damage() as f32);
                    let pred_hs_dmg = data.penetration_headshot_damage.unwrap_or(data.weapon.base_damage() as f32 * 4.0);
                    self.last_shot_info = Some((Instant::now(), pred_dmg, pred_hs_dmg));
                    self.last_shots_fired = data.local_player.shots_fired;
                } else if data.local_player.shots_fired < self.last_shots_fired {
                    self.last_shots_fired = data.local_player.shots_fired;
                }

                // Check total damage changes to output predicted vs real damage
                if data.total_damage > self.last_total_damage {
                    let diff = data.total_damage - self.last_total_damage;
                    if let Some((time, pred, pred_hs)) = self.last_shot_info {
                        if time.elapsed() < Duration::from_millis(1000) {
                            utils::info!(
                                "[SHOT-DAMAGE] Predicted Damage: {:.1} (Headshot: {:.1}) | Real Damage: {}",
                                pred, pred_hs, diff
                            );
                        }
                    }
                    self.last_total_damage = data.total_damage;
                } else if data.total_damage < self.last_total_damage {
                    self.last_total_damage = data.total_damage;
                }

                let t_data_val = t_data_start.elapsed().as_secs_f32() * 1000.0;
                if let Ok(mut t) = TELEMETRY.lock() {
                    t.draw_data_ms = t_data_val;
                }
            } else {
                *self.data.lock() = Data::default();
            }

            if is_valid {
                let elapsed = start.elapsed();
                let loop_dur = self.loop_duration();
                let idle_duration = if elapsed < loop_dur {
                    let idle = loop_dur - elapsed;
                    sleep(idle);
                    idle
                } else {
                    utils::debug!(
                        "game loop took {} ms (max {} ms)",
                        elapsed.as_millis(),
                        loop_dur.as_millis()
                    );
                    Duration::ZERO
                };

                if let Ok(mut t) = TELEMETRY.lock() {
                    t.total_loop_ms = elapsed.as_secs_f32() * 1000.0;
                    t.idle_ms = idle_duration.as_secs_f32() * 1000.0;
                    t.target_frame_ms = loop_dur.as_secs_f32() * 1000.0;
                }

                let total_iter_dur = start.elapsed();
                self.send_message(UiMessage::FrameTime(total_iter_dur));
            } else {
                sleep(Duration::from_secs(5));
            }
        }
    }

    fn loop_duration(&self) -> Duration {
        if self.config.fps == 0 {
            Duration::ZERO
        } else {
            Duration::from_secs_f64(1.0 / self.config.fps as f64)
        }
    }
}
