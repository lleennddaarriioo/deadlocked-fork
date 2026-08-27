use glam::{Vec2, vec2};

use shared::bones::Bones;

use crate::{
    config::Config,
    cs2::{
        CS2,
        entity::{player::Player, weapon_class::WeaponClass},
    },
    math::{angles_to_fov, vec2_clamp},
    os::mouse::Mouse,
};

#[derive(Debug, Default)]
pub struct Aimbot {
    pub active: bool,
    inertia: Vec2,
}

impl CS2 {
    pub fn aimbot(&mut self, config: &Config, mouse: &mut Mouse) -> bool {
        crate::profile_scope!("aimbot");
        let hotkey = config.aim.aimbot_hotkey;
        let config = self.aimbot_config(config);

        if !config.enabled {
            return false;
        }

        if !Self::check_hotkey(&self.input, config.mode, hotkey, crate::cs2::key_codes::KeyCode::None, &mut self.aim.active) {
            return false;
        }

        let Some(local_player) = Player::local_player(self) else {
            return false;
        };

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
                    mouse.move_rel(self.aim.inertia);
                    return true;
                }
            }
            return false;
        }

        let Some(target) = &self.target.player else {
            return false;
        };

        if !target.is_valid(self) {
            return false;
        }

        let disallowed_weapons = [
            WeaponClass::Unknown,
            WeaponClass::Knife,
        ];
        if disallowed_weapons.contains(&weapon_class) {
            return false;
        }

        if config.flash_check && local_player.is_flashed(self) {
            return false;
        }



        if local_player.shots_fired(self) < config.start_bullet {
            return false;
        }

        let max_fov = config.fov
            * if config.distance_adjusted_fov {
                self.distance_scale(self.target.distance)
            } else {
                1.0
            };

        let mut best_bone_damage = None;

        let target_angle = {
            let mut smallest_fov = 360.0;
            let mut smallest_angle = glam::Vec2::ZERO;
            let mut found_bone = false;

            let eye_pos = local_player.eye_position(self);

            for bone in &config.bones {
                let bone_pos = target.bone_position(self, bone.u64());

                if config.visibility_check {
                    let is_vis = match config.visibility_mode {
                        crate::config::aim::VisibilityMode::BoneFast => {
                            if let Some(bvh) = &self.bvh {
                                bvh.has_line_of_sight(eye_pos, bone_pos)
                            } else {
                                target.visible(self, &local_player)
                            }
                        }
                        crate::config::aim::VisibilityMode::BoneLoS => {
                            if let Some(cached_map) = self.cached_bone_vis.get(&target.steam_id(self)) {
                                cached_map.get(bone).copied().unwrap_or_else(|| {
                                    if let Some(bvh) = &self.bvh {
                                        bvh.has_line_of_sight(eye_pos, bone_pos)
                                    } else {
                                        target.visible(self, &local_player)
                                    }
                                })
                            } else if let Some(bvh) = &self.bvh {
                                bvh.has_line_of_sight(eye_pos, bone_pos)
                            } else {
                                target.visible(self, &local_player)
                            }
                        }
                        crate::config::aim::VisibilityMode::PixelLoS => target.is_pixel_visible(self),
                        crate::config::aim::VisibilityMode::Both => {
                            let bone_vis = if let Some(cached_map) = self.cached_bone_vis.get(&target.steam_id(self)) {
                                cached_map.get(bone).copied().unwrap_or_else(|| {
                                    if let Some(bvh) = &self.bvh {
                                        bvh.has_line_of_sight(eye_pos, bone_pos)
                                    } else {
                                        target.visible(self, &local_player)
                                    }
                                })
                            } else if let Some(bvh) = &self.bvh {
                                bvh.has_line_of_sight(eye_pos, bone_pos)
                            } else {
                                target.visible(self, &local_player)
                            };
                            bone_vis && target.is_pixel_visible(self)
                        }
                        crate::config::aim::VisibilityMode::Either => {
                            let bone_vis = if let Some(cached_map) = self.cached_bone_vis.get(&target.steam_id(self)) {
                                cached_map.get(bone).copied().unwrap_or_else(|| {
                                    if let Some(bvh) = &self.bvh {
                                        bvh.has_line_of_sight(eye_pos, bone_pos)
                                    } else {
                                        target.visible(self, &local_player)
                                    }
                                })
                            } else if let Some(bvh) = &self.bvh {
                                bvh.has_line_of_sight(eye_pos, bone_pos)
                            } else {
                                target.visible(self, &local_player)
                            };
                            bone_vis || target.is_pixel_visible(self)
                        }
                    };

                    if !is_vis {
                        continue;
                    }
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
                    if fov <= max_fov {
                        smallest_angle = angle;
                        found_bone = true;
                        best_bone_damage = Some((predicted_damage, base_damage * 4.0));
                        break;
                    }
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
                self.aimbot_predicted_damage = None;
                return false;
            }

            smallest_angle
        };

        self.aimbot_predicted_damage = best_bone_damage;

        let view_angles = local_player.view_angles(self);
        if angles_to_fov(&view_angles, &target_angle)
            > (config.fov
                * if config.distance_adjusted_fov {
                    self.distance_scale(self.target.distance)
                } else {
                    1.0
                })
        {
            return false;
        }

        let mut aim_angles = view_angles - target_angle;
        if aim_angles.y < -180.0 {
            aim_angles.y += 360.0
        }
        vec2_clamp(&mut aim_angles);

        let sensitivity = self.get_sensitivity() * local_player.fov_multiplier(self);

        // Avoid overshooting by enforcing a minimum smoothing divisor.
        // Even at "0" smooth, a divisor of 2.0 ensures we only cover half the distance per tick,
        // preventing the aimbot from snapping past the target due to floating-point/pixel rounding.
        let smooth_factor = (config.smooth + 1.0).max(2.0);
        let mouse_angles = vec2(
            aim_angles.y / sensitivity * 45.45,
            -aim_angles.x / sensitivity * 45.45,
        ) / smooth_factor;

        let alpha = 1.0 - config.inertia.clamp(0.0, 1.0) * 0.5;
        self.aim.inertia += (mouse_angles - self.aim.inertia) * alpha;
        mouse.move_rel(self.aim.inertia);

        self.recoil.previous = local_player.aim_punch(self);

        true
    }
}
