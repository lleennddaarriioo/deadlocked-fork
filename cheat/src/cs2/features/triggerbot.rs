use std::time::{Duration, Instant};

use glam::Vec2;
use rand::rng;
use shared::bones::Bones;

use crate::{
    config::Config,
    cs2::{
        CS2,
        entity::{player::Player, weapon_class::WeaponClass},
    },
    math::angles_to_fov,
    os::mouse::Mouse,
};

#[derive(Debug, Default)]
pub struct Triggerbot {
    shot_start: Option<Instant>,
    shot_end: Option<Instant>,
    pub active: bool,
}

impl CS2 {
    pub fn triggerbot(&mut self, config: &Config) {
        let hotkey1 = config.aim.triggerbot_hotkey;
        let hotkey2 = config.aim.triggerbot_hotkey2;
        let config = self.triggerbot_config(config);

        if !config.enabled {
            return;
        }

        if !Self::check_hotkey(&self.input, config.mode, hotkey1, hotkey2, &mut self.trigger.active) {
            return;
        }

        if self.trigger.shot_start.is_some() || self.trigger.shot_end.is_some() {
            return;
        }

        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        if config.flash_check && local_player.is_flashed(self) {
            return;
        }

        if config.scope_check
            && local_player.weapon_class(self) == WeaponClass::Sniper
            && !local_player.is_scoped(self)
        {
            return;
        }

        if config.velocity_check && local_player.velocity(self).length() > config.velocity_threshold
        {
            return;
        }

        self.triggerbot_predicted_damage = None;

        let weapon_class = local_player.weapon_class(self);
        let _shots_fired = local_player.shots_fired(self);
        let aim_punch = match (weapon_class, local_player.aim_punch(self) * 2.0) {
            (WeaponClass::Sniper, _) => Vec2::ZERO,
            (_, punch) => punch,
        };

        let Some((player, target_bone, _is_wallbang)) = self.get_triggerbot_target(config, &local_player, &aim_punch) else {
            return;
        };

        if !self.is_ffa() && player.team(self) == local_player.team(self) {
            return;
        }

        // Check visibility / line of sight
        let bone_pos = player.bone_position(self, target_bone.u64());
        let is_visible = player.is_visible_mode(self, &local_player, config.visibility_mode);

        if !is_visible && config.visibility_check {
            return;
        }

        let base_damage = local_player.weapon(self).base_damage().max(30) as f32;
        let predicted_damage = if target_bone == Bones::Head { base_damage * 4.0 } else { base_damage };

        self.triggerbot_predicted_damage = Some((predicted_damage, base_damage * 4.0));

        // Calculate hit chance using 1000 simulated spread checks based on weapon inaccuracy
        let distance = (local_player.eye_position(self) - bone_pos).length();
        let inaccuracy = local_player.inaccuracy(self);
        
        let target_angle = self.angle_to_target(&local_player, &bone_pos, &aim_punch);
        let view_angles = local_player.view_angles(self);
        let fov = angles_to_fov(&view_angles, &target_angle);
        
        let offset_dist = distance * (fov * std::f32::consts::PI / 180.0).tan();
        // Multiply inaccuracy by 1.5 to account for innate weapon spread (m_flSpread) that we aren't reading
        let spread_radius = (inaccuracy * 1.5) * distance;
        // Make the target radius smaller to be more strict and ensure we actually hit
        let target_radius = if config.head_only { 2.0 } else { 8.0 }; // Stricter Approx radius
        
        let mut hits = 0;
        let checks = 1000;
        use rand_distr::{Distribution as _, Uniform};
        let uniform = Uniform::new(0.0, 1.0f32).unwrap();
        let mut r = rng();
        
        for _ in 0..checks {
            let rs = spread_radius * uniform.sample(&mut r).sqrt();
            let theta = uniform.sample(&mut r) * 2.0 * std::f32::consts::PI;
            
            let px = rs * theta.cos();
            let py = rs * theta.sin();
            
            let dx = px - offset_dist;
            let dy = py;
            if dx * dx + dy * dy <= target_radius * target_radius {
                hits += 1;
            }
        }
        
        let hitchance_pct = hits as f32 / checks as f32;
        if hitchance_pct < config.hit_chance {
            return;
        }

        if config.head_only {
            let head = player.bone_position(self, Bones::Head.u64());

            let target_angle = self.angle_to_target(&local_player, &head, &aim_punch);
            let view_angles = local_player.view_angles(self);
            let fov = angles_to_fov(&view_angles, &target_angle);

            let head_radius_fov =
                3.5 / (local_player.position(self) - player.position(self)).length() * 100.0;

            if fov > head_radius_fov {
                return;
            }
        }

        let mean = (*config.delay.start() + *config.delay.end()) as f32 / 2.0;
        let std_dev = (*config.delay.end() - *config.delay.start()) as f32 / 2.0;

        let delay = if std_dev <= 0.0 {
            mean.max(0.0) as u64
        } else {
            let normal = rand_distr::Normal::new(mean, std_dev).unwrap();
            use rand_distr::Distribution as _;
            normal.sample(&mut rng()).max(0.0) as u64
        };

        let now = Instant::now();
        let delay = Duration::from_millis(delay);
        self.trigger.shot_start = Some(now + delay);
        self.trigger.shot_end = Some(now + delay + Duration::from_millis(config.shot_duration as u64));
    }

    pub fn triggerbot_shoot(&mut self, mouse: &mut Mouse) {
        let now = Instant::now();

        if let Some(shot_time) = self.trigger.shot_start
            && now >= shot_time
        {
            mouse.left_press();
            self.trigger.shot_start = None;
        }

        if let Some(shot_end) = self.trigger.shot_end
            && now >= shot_end
        {
            mouse.left_release();
            self.trigger.shot_end = None;
        }
    }

    pub fn auto_pistol(&mut self, config: &Config) {
        if !config.aim.auto_pistol {
            return;
        }

        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        if local_player.weapon_class(self) != WeaponClass::Pistol {
            return;
        }

        let hotkey1_pressed = self.input.is_key_pressed(config.aim.triggerbot_hotkey);
        let hotkey2_pressed = self.input.is_key_pressed(config.aim.triggerbot_hotkey2);

        if !hotkey1_pressed && !hotkey2_pressed {
            return;
        }

        if self.trigger.shot_start.is_some() || self.trigger.shot_end.is_some() {
            return;
        }

        // Apply hit chance probability
        let config_trigger = self.triggerbot_config(config);
        
        let weapon_class = local_player.weapon_class(self);
        let _shots_fired = local_player.shots_fired(self);
        let aim_punch = match (weapon_class, local_player.aim_punch(self) * 2.0) {
            (WeaponClass::Sniper, _) => Vec2::ZERO,
            (_, punch) => punch,
        };

        let Some((player, target_bone, _is_wallbang)) = self.get_triggerbot_target(config_trigger, &local_player, &aim_punch) else {
            return;
        };

        if !self.is_ffa() && player.team(self) == local_player.team(self) {
            return;
        }

        let bone_pos = player.bone_position(self, target_bone.u64());
        let is_visible = player.is_visible_mode(self, &local_player, config_trigger.visibility_mode);

        if !is_visible && config_trigger.visibility_check {
            return;
        }

        let base_damage = local_player.weapon(self).base_damage().max(30) as f32;
        let predicted_damage = if target_bone == Bones::Head { base_damage * 4.0 } else { base_damage };

        self.triggerbot_predicted_damage = Some((predicted_damage, base_damage * 4.0));

        let distance = (local_player.eye_position(self) - bone_pos).length();
        let inaccuracy = local_player.inaccuracy(self);
        
        let target_angle = self.angle_to_target(&local_player, &bone_pos, &aim_punch);
        let view_angles = local_player.view_angles(self);
        let fov = angles_to_fov(&view_angles, &target_angle);
        
        let offset_dist = distance * (fov * std::f32::consts::PI / 180.0).tan();
        // Multiply inaccuracy by 1.5 to account for innate weapon spread (m_flSpread) that we aren't reading
        let spread_radius = (inaccuracy * 1.5) * distance;
        // Make the target radius smaller to be more strict and ensure we actually hit
        let target_radius = if config_trigger.head_only { 2.0 } else { 8.0 }; // Stricter Approx radius
        
        let mut hits = 0;
        let checks = 1000;
        use rand_distr::{Distribution as _, Uniform};
        let uniform = Uniform::new(0.0, 1.0f32).unwrap();
        let mut r = rng();
        
        for _ in 0..checks {
            let rs = spread_radius * uniform.sample(&mut r).sqrt();
            let theta = uniform.sample(&mut r) * 2.0 * std::f32::consts::PI;
            
            let px = rs * theta.cos();
            let py = rs * theta.sin();
            
            let dx = px - offset_dist;
            let dy = py;
            if dx * dx + dy * dy <= target_radius * target_radius {
                hits += 1;
            }
        }
        
        let hitchance_pct = hits as f32 / checks as f32;
        if hitchance_pct < config_trigger.hit_chance {
            return;
        }

        let mean = (*config_trigger.delay.start() + *config_trigger.delay.end()) as f32 / 2.0;
        let std_dev = (*config_trigger.delay.end() - *config_trigger.delay.start()) as f32 / 2.0;

        let delay = if std_dev <= 0.0 {
            mean.max(0.0) as u64
        } else {
            let normal = rand_distr::Normal::new(mean, std_dev).unwrap();
            normal.sample(&mut rng()).max(0.0) as u64
        };

        let now = Instant::now();
        self.trigger.shot_start = Some(now + Duration::from_millis(delay));
        self.trigger.shot_end = Some(now + Duration::from_millis(delay) + Duration::from_millis(config_trigger.shot_duration as u64));
    }

    fn get_triggerbot_target(&self, config: &crate::config::aim::TriggerbotConfig, local_player: &Player, aim_punch: &Vec2) -> Option<(Player, Bones, bool)> {
        if let Some(player) = local_player.crosshair_entity(self) {
            let target_bone = if config.head_only { Bones::Head } else { Bones::Spine2 };
            let is_visible = player.is_visible_mode(self, local_player, config.visibility_mode);

            if is_visible || !config.visibility_check {
                return Some((player, target_bone, false));
            }
        }

        let eye_pos = local_player.eye_position(self);
        let view_angles = local_player.view_angles(self);

        for p in &self.players {
            if !self.is_ffa() && p.team(self) == local_player.team(self) {
                continue;
            }
            if p.pawn == local_player.pawn {
                continue;
            }

            let bones_to_check = if config.head_only {
                vec![(Bones::Head, 3.5)]
            } else {
                vec![
                    (Bones::Head, 3.5),
                    (Bones::Neck, 3.0),
                    (Bones::Spine4, 5.0),
                    (Bones::Spine2, 6.0),
                    (Bones::Hip, 7.0),
                ]
            };

            for &(bone, radius) in &bones_to_check {
                let bone_pos = p.bone_position(self, bone.u64());

                if config.visibility_check {
                    let pixel_vis = p.is_pixel_visible(self);
                    let bone_vis = if let Some(bvh) = &self.bvh {
                        bvh.has_line_of_sight(eye_pos, bone_pos)
                    } else {
                        p.visible(self, local_player)
                    };

                    let is_vis = match config.visibility_mode {
                        crate::config::aim::VisibilityMode::BoneLoS | crate::config::aim::VisibilityMode::BoneFast => bone_vis,
                        crate::config::aim::VisibilityMode::PixelLoS => pixel_vis,
                        crate::config::aim::VisibilityMode::Both => bone_vis && pixel_vis,
                        crate::config::aim::VisibilityMode::Either => bone_vis || pixel_vis,
                    };

                    if !is_vis {
                        continue;
                    }
                }

                let target_angle = self.angle_to_target(local_player, &bone_pos, aim_punch);
                let fov = angles_to_fov(&view_angles, &target_angle);

                let distance = (eye_pos - bone_pos).length();
                if distance < 10.0 {
                    continue;
                }
                let bone_radius_fov = (radius / distance).atan().to_degrees();
                if fov <= bone_radius_fov {
                    return Some((p.clone(), bone, true));
                }
            }
        }
        None
    }
}
