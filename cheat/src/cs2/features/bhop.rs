use crate::{
    config::Config,
    cs2::{entity::player::Player, input::Input, offsets::Offsets},
    os::{keyboard::Keyboard, mouse::Mouse, process::Process},
};
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub struct Bhop {
    previous_yaw: f32,
    was_strafing: bool,
    was_on_ground: bool,
    was_space_pressed: bool,
    was_hotkey_pressed: bool,
    debug_checked: bool,
    verbosity: u8,
    hotkey_release_time: Option<Instant>,
    last_debug_log: Option<String>,
    debug_log_count: u32,
    debug_log_start: Option<Instant>,
    ground_ticks: u32,
    landing_speed: f32,
    jump_toggle: bool,
    last_jump_toggle_time: Option<Instant>,
}

impl Bhop {
    pub fn is_strafing(&self) -> bool {
        self.was_strafing
    }

    fn flush_debug_log(&mut self) {
        if self.verbosity >= 3 {
            if let Some(log) = &self.last_debug_log {
                if self.debug_log_count > 1 {
                    let elapsed = self.debug_log_start.unwrap().elapsed().as_millis();
                    ::utils::info!("{} (x{} over {}ms)", log, self.debug_log_count, elapsed);
                } else {
                    ::utils::info!("{}", log);
                }
            }
        }
        self.last_debug_log = None;
        self.debug_log_count = 0;
    }

    pub fn run(
        &mut self,
        process: &Process,
        offsets: &Offsets,
        input: &Input,
        config: &Config,
        keyboard: &mut Keyboard,
        _mouse: &mut Mouse,
        local_player: &Player,
    ) {
        if !self.debug_checked {
            let mut verbosity = 0;
            for arg in std::env::args() {
                if arg == "debug" || arg == "--debug" {
                    verbosity = verbosity.max(1);
                } else if arg.starts_with("-v") {
                    let v_count = arg.chars().filter(|&c| c == 'v').count() as u8;
                    verbosity = verbosity.max(v_count);
                }
            }
            self.verbosity = verbosity;
            self.debug_checked = true;
        }

        let velocity = process.read::<glam::Vec3>(local_player.pawn + offsets.pawn.velocity);
        let is_space_pressed = input.is_key_pressed(crate::cs2::key_codes::KeyCode::Space);
        self.was_space_pressed = is_space_pressed;

        let health = process.read::<i32>(local_player.pawn + offsets.pawn.health);
        let life_state = process.read::<u8>(local_player.pawn + offsets.pawn.life_state);
        if health <= 0 || life_state != 0 {
            return;
        }

        let is_in_air = local_player.is_in_air_direct(process, offsets);
        let view_angles = local_player.view_angles_direct(process, offsets);

        // Auto Strafe
        if config.misc.autostrafe && is_in_air {
            self.was_strafing = true;
            // Let go of W/S in mid-air for optimal air accel
            keyboard.w_release();
            keyboard.s_release();

            let diff = view_angles.y - self.previous_yaw;
            let diff = if diff > 180.0 {
                diff - 360.0
            } else if diff < -180.0 {
                diff + 360.0
            } else {
                diff
            };

            if diff > 0.0 {
                keyboard.a_press();
                keyboard.d_release();
            } else if diff < 0.0 {
                keyboard.d_press();
                keyboard.a_release();
            }
        } else if self.was_strafing {
            keyboard.a_release();
            keyboard.d_release();
            self.was_strafing = false;

            // Restore physical WASD state upon landing to prevent momentum loss
            let (is_w, is_a, is_s, is_d) = keyboard.get_physical_wasd();
            if is_w { keyboard.w_press(); }
            if is_s { keyboard.s_press(); }
            if is_a { keyboard.a_press(); }
            if is_d { keyboard.d_press(); }
        }

        // Edge Jump
        if config.misc.edge_jump && input.is_key_pressed(config.misc.edge_jump_hotkey) {
            if self.was_on_ground && is_in_air {
                if self.verbosity >= 1 {
                    ::utils::info!("[BHOP-DEBUG] Edge jump triggered!");
                }
                keyboard.end_press();
                keyboard.space_press();
            }
        }

        // Bunnyhop & Space Pass-through
        let hotkey = config.misc.bhop_hotkey;
        let is_space_hotkey = hotkey == crate::cs2::key_codes::KeyCode::Space;
        let physical_space = keyboard.is_physical_space_pressed();

        let mut raw_hotkey_pressed = if is_space_hotkey {
            physical_space || input.is_key_pressed(hotkey)
        } else {
            input.is_key_pressed(hotkey)
        };
        // SAFETY: Prevent infinite feedback loop if user mapped Bhop Hotkey to END in GUI!
        if hotkey == crate::cs2::key_codes::KeyCode::End {
            raw_hotkey_pressed = physical_space || is_space_pressed;
        }

        // Debounce the hotkey by 50ms to fix X11 auto-repeat fluttering
        if raw_hotkey_pressed {
            self.hotkey_release_time = None;
        } else if self.hotkey_release_time.is_none() {
            self.hotkey_release_time = Some(Instant::now());
        }

        let is_hotkey_pressed = if raw_hotkey_pressed {
            true
        } else if let Some(release_time) = self.hotkey_release_time {
            release_time.elapsed() < Duration::from_millis(50)
        } else {
            false
        };

        // Performance Tracking
        let xy_vel = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
        if !is_in_air && self.was_on_ground {
            self.ground_ticks += 1;
        } else if !is_in_air && !self.was_on_ground {
            self.ground_ticks = 1;
            self.landing_speed = xy_vel;
        } else if is_in_air && self.was_on_ground {
            if is_hotkey_pressed && self.verbosity >= 1 && self.ground_ticks > 0 {
                let speed_change = xy_vel - self.landing_speed;
                let stamina = if offsets.pawn.movement_services != 0 && offsets.pawn.stamina != 0 {
                    let mv_services =
                        process.read::<u64>(local_player.pawn + offsets.pawn.movement_services);
                    if mv_services != 0 {
                        process.read::<f32>(mv_services + offsets.pawn.stamina)
                    } else {
                        0.0
                    }
                } else if offsets.pawn.stamina != 0 {
                    process.read::<f32>(local_player.pawn + offsets.pawn.stamina)
                } else {
                    0.0
                };
                ::utils::info!(
                    "[BHOP] Ground: {} ticks | Speed: {:.1} -> {:.1} ({:+.1}) | Stamina: {:.2}",
                    self.ground_ticks,
                    self.landing_speed,
                    xy_vel,
                    speed_change,
                    stamina
                );
            }
            self.ground_ticks = 0;
        }

        if is_hotkey_pressed && !self.was_hotkey_pressed {
            if self.verbosity >= 1 {
                ::utils::info!("[BHOP-DEBUG] Bhop Hotkey Pressed!");
            }
        } else if !is_hotkey_pressed && self.was_hotkey_pressed {
            self.flush_debug_log();
            if self.verbosity >= 1 {
                ::utils::info!("[BHOP-DEBUG] Bhop Hotkey Released!");
            }
        }
        self.was_hotkey_pressed = is_hotkey_pressed;

        let is_any_space = physical_space || input.is_key_pressed(crate::cs2::key_codes::KeyCode::Space);

        if config.misc.bunnyhop {
            if (is_hotkey_pressed || is_any_space) && crate::os::keyboard::Keyboard::is_cs2_focused() {
                if !is_in_air {
                    if !self.was_on_ground {
                        // The exact frame we touched the ground.
                        keyboard.end_press();
                        self.jump_toggle = true;
                        self.last_jump_toggle_time = Some(Instant::now());
                        if self.verbosity >= 3 {
                            ::utils::info!("[BHOP-VVV] JUMP PRESSED | Reason: On Ground (Pure Reaction)");
                        }
                    } else if !self.jump_toggle {
                        // We are on the ground, but key is released. 
                        // Wait 16ms since last release to avoid SDL squash.
                        let safe_to_press = self.last_jump_toggle_time.map_or(true, |t| t.elapsed() > Duration::from_millis(16));
                        if safe_to_press {
                            keyboard.end_press();
                            self.jump_toggle = true;
                            self.last_jump_toggle_time = Some(Instant::now());
                            if self.verbosity >= 3 {
                                ::utils::info!("[BHOP-VVV] JUMP PRESSED | Reason: Standstill / Recovery");
                            }
                        }
                    } else {
                        // We are on the ground, and key is pressed.
                        // If we've been stuck like this for 60ms (>3 ticks), the engine ignored it.
                        let stuck = self.last_jump_toggle_time.map_or(false, |t| t.elapsed() > Duration::from_millis(60));
                        if stuck {
                            keyboard.end_release();
                            self.jump_toggle = false;
                            self.last_jump_toggle_time = Some(Instant::now());
                            if self.verbosity >= 3 {
                                ::utils::info!("[BHOP-VVV] JUMP RELEASED | Reason: Stuck Recovery");
                            }
                        }
                    }
                } else {
                    if self.was_on_ground {
                        keyboard.end_release();
                        self.jump_toggle = false;
                        self.last_jump_toggle_time = Some(Instant::now());
                        if self.verbosity >= 3 {
                            ::utils::info!("[BHOP-VVV] JUMP RELEASED | Reason: In Air");
                        }
                    }
                }
            } else {
                keyboard.end_release();
                self.jump_toggle = false;
                self.last_jump_toggle_time = None;
            }
        } else {
            // Bhop disabled: mirror physical Space to virtual END so they can jump normally
            if is_any_space && crate::os::keyboard::Keyboard::is_cs2_focused() {
                keyboard.end_press();
                self.jump_toggle = true;
            } else {
                keyboard.end_release();
                self.jump_toggle = false;
            }
        }

        self.previous_yaw = view_angles.y;
        self.was_on_ground = !is_in_air;
    }
}
