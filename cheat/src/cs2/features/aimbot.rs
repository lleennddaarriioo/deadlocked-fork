use glam::{Vec2, vec2};
use shared::WeaponClass;

use shared::bones::Bones;

use crate::{
    config::Config,
    cs2::{CS2, entity::player::Player},
    math::{angles_to_fov, vec2_clamp},
    os::mouse::Mouse,
};

#[derive(Debug, Default, PartialEq)]
pub enum SilentAimState {
    #[default]
    Idle,
    FlickingToTarget,
    Shooting,
    SnappingBack,
}

#[derive(Debug, Default)]
pub struct Aimbot {
    pub active: bool,
    pub is_locked: bool,
    pub current_target_fov: f32,
    pub inertia: Vec2,
    pub saved_angles: Option<Vec2>,
    pub silent_mouse_angles: Option<Vec2>,
    pub silent_target_angle: Option<Vec2>,
    pub silent_state: SilentAimState,
    pub silent_timer: Option<std::time::Instant>,
    pub was_active: bool,
    pub fractional_mouse: Vec2,
    pub last_tick: Option<std::time::Instant>,
    pub last_framecount: i32,
}

impl CS2 {
    pub fn aimbot(&mut self, config: &Config, mouse: &mut Mouse) -> bool {
        crate::profile_scope!("aimbot");
        let hotkey = config.aim.aimbot_hotkey;
        let global_config = config;
        let config = self.aimbot_config(global_config);

        self.aim.is_locked = false;
        self.aim.current_target_fov = 360.0;

        if !config.enabled {
            return false;
        }

        let hotkey_active = Self::check_hotkey(&self.input, config.mode, hotkey, crate::cs2::key_codes::KeyCode::None, &mut self.aim.active);
        
        let Some(local_player) = Player::local_player(self) else {
            return false;
        };

        let is_silent = config.silent_aim;
        let mut in_progress = false;

        if is_silent && self.aim.silent_state != SilentAimState::Idle {
            in_progress = true;
        }

        if !hotkey_active && !in_progress {
            self.aim.silent_state = SilentAimState::Idle;
            self.aim.saved_angles = None;
            self.aim.was_active = false;
            return false;
        }

        if in_progress {
            match self.aim.silent_state {
                SilentAimState::FlickingToTarget => {
                    if let Some(timer) = self.aim.silent_timer {
                        if std::time::Instant::now() >= timer {
                            mouse.left_press();
                            
                            if let Some(target) = self.aim.silent_target_angle {
                                let actual_angles = local_player.view_angles(self);
                                let mut diff = actual_angles - target;
                                while diff.y < -180.0 { diff.y += 360.0; }
                                while diff.y > 180.0 { diff.y -= 360.0; }
                                crate::math::vec2_clamp(&mut diff);
                                
                                ::utils::info!("[flickbot] SHOT FIRED! Target: ({:.2}, {:.2}) | Actual: ({:.2}, {:.2}) | Miss Offset: ({:.2}, {:.2})", 
                                    target.x, target.y, actual_angles.x, actual_angles.y, diff.x, diff.y);
                            }
                            
                            self.aim.silent_state = SilentAimState::Shooting;
                            self.aim.silent_timer = Some(std::time::Instant::now() + std::time::Duration::from_millis(20));
                        }
                    }
                    return true;
                }
                SilentAimState::Shooting => {
                    if let Some(timer) = self.aim.silent_timer {
                        if std::time::Instant::now() >= timer {
                            mouse.left_release();
                            
                            if let Some(mouse_angles) = self.aim.silent_mouse_angles {
                                // Snap back by perfectly reversing the exact mouse movement we made, ignoring view angles (recoil)
                                mouse.move_rel(vec2(-mouse_angles.x, -mouse_angles.y));
                            }
                            
                            self.aim.silent_state = SilentAimState::SnappingBack;
                            self.aim.silent_timer = Some(std::time::Instant::now() + std::time::Duration::from_millis(150));
                        }
                    }
                    return true;
                }
                SilentAimState::SnappingBack => {
                    if let Some(timer) = self.aim.silent_timer {
                        if std::time::Instant::now() >= timer {
                            self.aim.silent_state = SilentAimState::Idle;
                            self.aim.silent_mouse_angles = None;
                        }
                    }
                    return true;
                }
                _ => {}
            }
        }

        let weapon_class = local_player.weapon_class(self);

        if weapon_class == WeaponClass::Grenade {
            let map_name = self.current_map();
            let position = local_player.position(self);
            let player_weapon = local_player.weapon(self);
            
            let grenade_list = crate::ui::grenades::read_grenades();
            if let Some(grenades) = grenade_list.get(&map_name) {
                let nearest = grenades
                    .iter()
                    .filter(|g| g.weapon == player_weapon)
                    .map(|g| (g, (position - g.position).length()))
                    .filter(|(_, dist)| *dist <= 1000.0)
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

                if let Some((grenade, _)) = nearest {
                    let view_angles = local_player.view_angles(self);
                    let target_angle = grenade.view_angles;

                    let mut aim_angles = view_angles - target_angle;
                    while aim_angles.y < -180.0 {
                        aim_angles.y += 360.0;
                    }
                    while aim_angles.y > 180.0 {
                        aim_angles.y -= 360.0;
                    }
                    vec2_clamp(&mut aim_angles);

                    if aim_angles.length() < 0.02 {
                        self.aim.inertia = Vec2::ZERO;
                        return true;
                    }

                    let sensitivity = self.get_sensitivity() * local_player.fov_multiplier(self);
                    let smooth_factor = 20.0f32;
                    let mouse_angles = vec2(
                        aim_angles.y / sensitivity * 45.45,
                        -aim_angles.x / sensitivity * 45.45,
                    ) / smooth_factor;

                    let alpha = 1.0 - config.inertia.clamp(0.0, 1.0) * 0.5;
                    self.aim.inertia += (mouse_angles - self.aim.inertia) * alpha;
                    
                    self.aim.fractional_mouse += self.aim.inertia;
                    let move_x = self.aim.fractional_mouse.x.trunc();
                    let move_y = self.aim.fractional_mouse.y.trunc();
                    self.aim.fractional_mouse.x -= move_x;
                    self.aim.fractional_mouse.y -= move_y;
                    
                    mouse.move_rel(vec2(move_x, move_y));
                    return true;
                }
            }
            return false;
        }

        let Some(target) = &self.target.player else {
            ::utils::info!("[aimbot miss] no target selected in target manager");
            return false;
        };

        if !target.is_valid(self) {
            ::utils::info!("[aimbot miss] target invalid (dead/dormant)");
            return false;
        }

        let disallowed_weapons = [
            WeaponClass::Unknown,
            WeaponClass::Knife,
        ];
        if disallowed_weapons.contains(&weapon_class) {
            ::utils::info!("[aimbot miss] disallowed weapon class: {}", weapon_class);
            return false;
        }

        if config.flash_check && local_player.is_flashed(self) {
            ::utils::info!("[aimbot miss] player is flashed");
            return false;
        }

        if local_player.shots_fired(self) < config.start_bullet {
            ::utils::info!("[aimbot miss] shots fired ({}) < start_bullet ({})", local_player.shots_fired(self), config.start_bullet);
            return false;
        }

        let max_fov = config.fov * self.distance_scale(self.target.distance);

        let mut best_bone_damage = None;

        let target_angle = {
            let mut smallest_fov = 360.0;
            let mut smallest_angle = glam::Vec2::ZERO;
            let target_velocity = target.velocity(self);
            let prediction_time = config.prediction_time.clamp(0.0, 0.25);
            let mut found_bone = false;

            let eye_pos = local_player.eye_position(self);
            let default_bones = [Bones::Head, Bones::Spine2];
            let aim_bones = if config.bones.is_empty() { &default_bones[..] } else { &config.bones[..] };

            for bone in aim_bones {
                let bone_pos =
                    target.bone_position(self, bone.u64()) + target_velocity * prediction_time;
                let dist_units = eye_pos.distance(bone_pos);
                let dist_meters = dist_units * 0.0254;

                let is_vis = if config.visibility_check {
                    if let Some(bvh) = &self.bvh {
                        bvh.has_line_of_sight(eye_pos, bone_pos) || target.visible(self, &local_player)
                    } else {
                        target.visible(self, &local_player)
                    }
                } else {
                    true
                };

                if !is_vis {
                    if std::env::args().any(|arg| arg == "debug" || arg == "--debug" || arg.starts_with("-v")) {
                        ::utils::info!(
                            "[aimbot miss] target: '{}' | bone: {:?} | dist: {:.1}m ({:.0}u) | LOS block",
                            target.name(self),
                            bone,
                            dist_meters,
                            dist_units,
                        );
                    }
                    continue;
                }

                let player_weapon = local_player.weapon(self);
                let base_damage = player_weapon.base_damage().max(30) as f32;
                let predicted_damage = if bone == &Bones::Head { base_damage * 4.0 } else { base_damage };

                if predicted_damage <= 0.0 {
                    continue;
                }
                let angle =
                    self.angle_to_target(&local_player, &bone_pos, &self.target.previous_aim_punch);
                let fov = angles_to_fov(&local_player.view_angles(self), &angle);
                
                if config.bone_mode == crate::config::aim::BoneMode::Priority {
                    smallest_angle = angle;
                    found_bone = true;
                    best_bone_damage = Some((predicted_damage, base_damage * 4.0));
                    break;
                } else {
                    if fov < smallest_fov {
                        smallest_fov = fov;
                        smallest_angle = angle;
                        found_bone = true;
                        best_bone_damage = Some((predicted_damage, base_damage * 4.0));
                    }
                }
            }

            if !found_bone {
                ::utils::info!("[aimbot miss] no suitable bone found for target");
                self.aimbot_predicted_damage = None;
                return false;
            }

            smallest_angle
        };

        self.aimbot_predicted_damage = best_bone_damage;

        let view_angles = local_player.view_angles(self);

        let current_fov = angles_to_fov(&view_angles, &target_angle);
        self.aim.current_target_fov = current_fov;
        self.aim.is_locked = current_fov <= max_fov;

        let mut aim_angles = view_angles - target_angle;
        if aim_angles.y < -180.0 {
            aim_angles.y += 360.0
        }
        vec2_clamp(&mut aim_angles);

        let now = std::time::Instant::now();
        let dt = if let Some(last) = self.aim.last_tick {
            now.duration_since(last).as_secs_f32().min(0.1) // clamp to 100ms
        } else {
            1.0 / 64.0
        };
        self.aim.last_tick = Some(now);
        
        let framecount = self.framecount();

        let sensitivity = self.get_sensitivity() * local_player.fov_multiplier(self);

        if config.silent_aim {
            if self.aim.silent_state == SilentAimState::Idle {
                let mouse_angles = vec2(
                    aim_angles.y / sensitivity * 45.45,
                    -aim_angles.x / sensitivity * 45.45,
                );
                self.aim.inertia = Vec2::ZERO;
                mouse.move_rel(mouse_angles);
                
                ::utils::info!("[flickbot] FLICK INITIATED! Start: ({:.2}, {:.2}) | Target: ({:.2}, {:.2}) | Aim Delta: ({:.2}, {:.2}) | Mouse Pixels: ({:.2}, {:.2})", 
                    view_angles.x, view_angles.y, target_angle.x, target_angle.y, aim_angles.x, aim_angles.y, mouse_angles.x, mouse_angles.y);
                    
                self.aim.silent_mouse_angles = Some(mouse_angles);
                self.aim.silent_target_angle = Some(target_angle);
                self.aim.silent_state = SilentAimState::FlickingToTarget;
                self.aim.silent_timer = Some(std::time::Instant::now() + std::time::Duration::from_millis(15));
            }
        } else if config.flick_mode {
            let distance = aim_angles.length();
            
            // Sample the FLICK curve
            let smooth_factor = if config.flick_curve.is_empty() {
                config.flick_speed.max(1.0)
            } else if config.flick_curve.len() == 1 {
                config.flick_curve[0].y
            } else {
                let mut factor = config.flick_speed.max(1.0);
                if distance <= config.flick_curve[0].x {
                    factor = config.flick_curve[0].y;
                } else if distance >= config.flick_curve.last().unwrap().x {
                    factor = config.flick_curve.last().unwrap().y;
                } else {
                    for w in config.flick_curve.windows(2) {
                        if distance >= w[0].x && distance <= w[1].x {
                            let t = (distance - w[0].x) / (w[1].x - w[0].x);
                            factor = w[0].y + (w[1].y - w[0].y) * t;
                            break;
                        }
                    }
                }
                factor
            }.max(1.0);

            if smooth_factor <= 1.001 {
                if framecount == self.aim.last_framecount {
                    return false;
                }
                self.aim.last_framecount = framecount;
            }

            let fraction = if smooth_factor <= 1.001 {
                1.0
            } else {
                let rate = 64.0 / smooth_factor;
                1.0 - (-rate * dt).exp()
            };

            let mouse_angles = vec2(
                aim_angles.y / sensitivity * 45.45,
                -aim_angles.x / sensitivity * 45.45,
            ) * fraction;

            self.aim.fractional_mouse += mouse_angles;
            let move_x = self.aim.fractional_mouse.x.trunc();
            let move_y = self.aim.fractional_mouse.y.trunc();
            self.aim.fractional_mouse.x -= move_x;
            self.aim.fractional_mouse.y -= move_y;

            if move_x != 0.0 || move_y != 0.0 {
                ::utils::info!("[aimbot_debug] moving x: {:.2}, y: {:.2} | aim_angles: {:.2}, {:.2} | mouse_angles: {:.2}, {:.2} | fraction: {:.6}", move_x, move_y, aim_angles.x, aim_angles.y, mouse_angles.x, mouse_angles.y, fraction);
            } else if framecount % 64 == 0 {
                ::utils::info!("[aimbot_debug] stuck at 0! fraction: {:.6} | dt: {:.6} | mouse_angles: {:.2}, {:.2} | fractional: {:.2}, {:.2}", fraction, dt, mouse_angles.x, mouse_angles.y, self.aim.fractional_mouse.x, self.aim.fractional_mouse.y);
            }

            mouse.move_rel(vec2(move_x, move_y));
        } else {
            let distance = aim_angles.length();
            
            // Sample the curve
            let smooth_factor = if config.curve.is_empty() {
                1.0
            } else if config.curve.len() == 1 {
                config.curve[0].y
            } else {
                let mut factor = 1.0;
                if distance <= config.curve[0].x {
                    factor = config.curve[0].y;
                } else if distance >= config.curve.last().unwrap().x {
                    factor = config.curve.last().unwrap().y;
                } else {
                    for w in config.curve.windows(2) {
                        if distance >= w[0].x && distance <= w[1].x {
                            let t = (distance - w[0].x) / (w[1].x - w[0].x);
                            factor = w[0].y + (w[1].y - w[0].y) * t;
                            break;
                        }
                    }
                }
                factor
            }.max(1.0);

            if smooth_factor <= 1.001 {
                if framecount == self.aim.last_framecount {
                    return false;
                }
                self.aim.last_framecount = framecount;
            }

            let fraction = if smooth_factor <= 1.001 {
                1.0
            } else {
                let rate = 64.0 / smooth_factor;
                1.0 - (-rate * dt).exp()
            };

            let mouse_angles = vec2(
                aim_angles.y / sensitivity * 45.45,
                -aim_angles.x / sensitivity * 45.45,
            ) * fraction;

            self.aim.fractional_mouse += mouse_angles;
            let move_x = self.aim.fractional_mouse.x.trunc();
            let move_y = self.aim.fractional_mouse.y.trunc();
            self.aim.fractional_mouse.x -= move_x;
            self.aim.fractional_mouse.y -= move_y;

            mouse.move_rel(vec2(move_x, move_y));
        }

        self.recoil.previous = local_player.aim_punch(self);

        true
    }
}
