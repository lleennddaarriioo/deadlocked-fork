use std::time::{Duration, Instant};

use glam::{IVec2, Mat4, Vec2, Vec3};
use rayon::prelude::*;
use shared::{
    bones::Bones,
    data::{Data, PlayerData},
    entity::EntityInfo,
    weapon::Weapon,
};

use crate::{
    config::{
        Config,
        aim::{AimbotConfig, KeyMode, RcsConfig, TriggerbotConfig},
    },
    constants::cs2::{self, TEAM_CT, TEAM_T},
    cs2::{
        entity::{
            Entity, grenade_info,
            planted_c4::PlantedC4,
            player::Player,
            weapon::{weapon_clip_ammo, weapon_reserve_ammo},
        },
        features::{aimbot::Aimbot, bhop::Bhop, counter_strafe::CounterStrafe, esp_toggle::EspToggle, rcs::Recoil, triggerbot::Triggerbot},
        input::Input,
        key_codes::KeyCode,
        offsets::Offsets,
        target::Target,
    },
    math::{angles_from_vector, vec2_clamp, vector_from_angles},
    os::{keyboard::Keyboard, mouse::Mouse, process::Process},
    parser::{bvh::Bvh, read_map},
};

pub mod bones;
pub mod bvh;
pub mod entity;
mod features;
mod find_offsets;
mod input;
pub mod key_codes;
mod offsets;
mod schema;
mod target;

#[derive(Debug)]
pub struct CS2 {
    is_valid: bool,
    process: Process,
    offsets: Offsets,
    input: Input,
    bvh: Option<Bvh>,
    current_bvh: String,
    target: Target,
    players: Vec<Player>,
    dead_players: Vec<Player>,
    entities: Vec<Entity>,
    recoil: Recoil,
    aim: Aimbot,
    trigger: Triggerbot,
    bhop: Bhop,
    counter_strafe: CounterStrafe,
    mic_tone: crate::cs2::features::mic_tone::MicTone,
    esp: EspToggle,
    weapon: Weapon,
    planted_c4: Option<PlantedC4>,
    last_cache: Instant,
    last_bvh: Instant,
    last_bone_vis: Instant,
    cached_bone_vis: std::collections::HashMap<u64, std::collections::HashMap<Bones, bool>>,
    last_bhop: Instant,
    last_aimbot: Instant,
    last_input: Instant,
    last_trigger: Instant,
    aimbot_predicted_damage: Option<(f32, f32)>, // (damage, headshot_damage)
    triggerbot_predicted_damage: Option<(f32, f32)>, // (damage, headshot_damage)
}

impl CS2 {
    pub fn is_valid(&self) -> bool {
        self.is_valid && self.process.is_valid()
    }

    pub fn setup(&mut self) {
        let Some(process) = Process::open(cs2::PROCESS_NAME) else {
            self.is_valid = false;
            return;
        };
        utils::info!("process found, pid: {}", process.pid);
        self.process = process;

        self.offsets = match self.find_offsets() {
            Some(offsets) => offsets,
            None => {
                self.process = Process::new(-1);
                self.is_valid = false;
                return;
            }
        };
        utils::info!("offsets found. Inaccuracy offset: 0x{:X}", self.offsets.weapon.inaccuracy);

        self.is_valid = true;
    }

    pub fn run(&mut self, config: &Config, mouse: &mut Mouse, keyboard: &mut Keyboard) {
        let total_start = Instant::now();
        if !self.process.is_valid() {
            self.is_valid = false;
            utils::debug!("process is no longer valid");
            return;
        }

        self.aimbot_predicted_damage = None;
        self.triggerbot_predicted_damage = None;

        let input_interval_ms = if config.input_tps == 0 { 0 } else { 1000 / config.input_tps.max(1) };
        let mut t_input_val = 0.0;
        if self.last_input.elapsed() >= Duration::from_millis(input_interval_ms as u64) {
            let t_input_start = Instant::now();
            self.input.update(&self.process, &self.offsets);
            t_input_val = t_input_start.elapsed().as_secs_f32() * 1000.0;
            self.last_input = Instant::now();
        }

        let cache_interval_ms = if config.cache_hz == 0 { 0 } else { 1000 / config.cache_hz.max(1) };
        let bvh_interval_ms = if config.bvh_tps == 0 { 0 } else { 1000 / config.bvh_tps.max(1) };

        let mut t_cache_val = 0.0;
        let mut t_bvh_val = 0.0;

        if self.last_cache.elapsed() >= Duration::from_millis(cache_interval_ms as u64) {
            let t_cache_start = Instant::now();
            self.cache_entities();
            t_cache_val = t_cache_start.elapsed().as_secs_f32() * 1000.0;
            self.last_cache = Instant::now();
        }

        if self.last_bvh.elapsed() >= Duration::from_millis(bvh_interval_ms as u64) {
            let t_bvh_start = Instant::now();
            self.check_bvh();
            t_bvh_val = t_bvh_start.elapsed().as_secs_f32() * 1000.0;
            self.last_bvh = Instant::now();
        }

        let t_other_features_start = Instant::now();
        for entity in &self.entities {
            if let Entity::Smoke(smoke) = entity {
                if config.misc.no_smoke {
                    smoke.disable(self);
                }

                if config.misc.change_smoke_color {
                    smoke.color(self, &config.misc.smoke_color);
                }
            }
        }

        self.no_flash(config);
        self.fov_changer(config);

        self.esp_toggle(config);
        let t_other_features_val = t_other_features_start.elapsed().as_secs_f32() * 1000.0;

        let trigger_interval_ms = if config.trigger_tps == 0 { 0 } else { 1000 / config.trigger_tps.max(1) };
        let mut t_trigger_val = 0.0;
        if self.last_trigger.elapsed() >= Duration::from_millis(trigger_interval_ms as u64) {
            let t_trigger_start = Instant::now();
            self.triggerbot(config);
            self.auto_pistol(config);
            self.triggerbot_shoot(mouse);
            t_trigger_val = t_trigger_start.elapsed().as_secs_f32() * 1000.0;
            self.last_trigger = Instant::now();
        }

        if let Some(lp) = Player::local_player(self) {
            self.weapon = lp.weapon(self);
        }

        self.find_target(config);

        let bhop_interval_ms = if config.bhop_tps == 0 { 0 } else { 1000 / config.bhop_tps.max(1) };
        let mut t_bhop_val = 0.0;
        let mut t_counter_strafe_val = 0.0;
        if self.last_bhop.elapsed() >= Duration::from_millis(bhop_interval_ms as u64) {
            if let Some(local_player) = Player::local_player(self) {
                let t_bhop_start = Instant::now();
                self.bhop.run(
                    &self.process,
                    &self.offsets,
                    &self.input,
                    config,
                    keyboard,
                    mouse,
                    &local_player,
                );
                t_bhop_val = t_bhop_start.elapsed().as_secs_f32() * 1000.0;

                let is_move_assist_holding = self.input.is_key_pressed(config.misc.grenade_move_hotkey) && 
                    local_player.weapon_class(self) == crate::cs2::entity::weapon_class::WeaponClass::Grenade;

                if !is_move_assist_holding {
                    let t_counter_strafe_start = Instant::now();
                    self.counter_strafe.run(
                        &self.process,
                        &self.offsets,
                        &self.input,
                        config,
                        keyboard,
                        &local_player,
                    );
                    t_counter_strafe_val = t_counter_strafe_start.elapsed().as_secs_f32() * 1000.0;
                } else {
                    self.counter_strafe.reset();
                }
            }
            self.last_bhop = Instant::now();
        }

        let mut move_assisted_this_tick = false;

        // Grenade Move Assistant: automatically walk towards the nearest lineup position when holding a grenade and holding configured hotkey
        if self.input.is_key_pressed(config.misc.grenade_move_hotkey) {
            if let Some(local_player) = Player::local_player(self) {
                let weapon_class = local_player.weapon_class(self);
                if weapon_class == crate::cs2::entity::weapon_class::WeaponClass::Grenade {
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

                        if let Some((grenade, distance)) = nearest {
                            if distance > 5.0 && distance <= 1000.0 {
                                move_assisted_this_tick = true;
                                let waypoint = crate::cs2::features::pathfinding::find_path(
                                    position,
                                    grenade.position,
                                    self.bvh.as_ref(),
                                );
                                let delta_world = waypoint - position;
                                let dist_2d = delta_world.length();

                                if dist_2d > 0.001 {
                                    let dir_norm = delta_world / dist_2d;
                                    let view_angles = local_player.view_angles(self);
                                    let yaw_rad = view_angles.y.to_radians();

                                    let forward_dir = glam::vec3(yaw_rad.cos(), yaw_rad.sin(), 0.0);
                                    let right_dir = glam::vec3(yaw_rad.sin(), -yaw_rad.cos(), 0.0);

                                    let fwd = dir_norm.dot(forward_dir);
                                    let side = dir_norm.dot(right_dir);

                                    utils::info!(
                                        "[MOVE ASSIST A*] Current: ({:.1}, {:.1}) -> Waypoint: ({:.1}, {:.1}) -> Goal '{}' ({:.1}, {:.1}) | Dist: {:.1} | Fwd: {:.2}, Side: {:.2}",
                                        position.x, position.y,
                                        waypoint.x, waypoint.y,
                                        grenade.name, grenade.position.x, grenade.position.y,
                                        distance, fwd, side
                                    );

                                    if fwd > 0.15 {
                                        keyboard.w_press();
                                    } else if fwd < -0.15 {
                                        keyboard.s_press();
                                    } else {
                                        keyboard.w_release();
                                        keyboard.s_release();
                                    }

                                    if side > 0.15 {
                                        keyboard.d_press();
                                    } else if side < -0.15 {
                                        keyboard.a_press();
                                    } else {
                                        keyboard.a_release();
                                        keyboard.d_release();
                                    }
                                }
                            } else {
                                keyboard.w_release();
                                keyboard.s_release();
                                keyboard.a_release();
                                keyboard.d_release();

                                // Automatically align cursor / view_angles to the exact grenade lineup angle once arrived
                                let view_angles = local_player.view_angles(self);
                                let target_angle = grenade.view_angles;

                                let mut aim_angles = view_angles - target_angle;
                                while aim_angles.y < -180.0 {
                                    aim_angles.y += 360.0;
                                }
                                while aim_angles.y > 180.0 {
                                    aim_angles.y -= 360.0;
                                }
                                crate::math::vec2_clamp(&mut aim_angles);

                                if aim_angles.length() >= 0.02 {
                                    let sensitivity = self.get_sensitivity() * local_player.fov_multiplier(self);
                                    let smooth_factor = 10.0f32;
                                    let mouse_angles = glam::vec2(
                                        aim_angles.y / sensitivity * 45.45,
                                        -aim_angles.x / sensitivity * 45.45,
                                    ) / smooth_factor;
                                    mouse.move_rel(mouse_angles);
                                }
                            }
                        }
                    }
                }
            }
        }

        if !move_assisted_this_tick {
            let (p_w, p_a, p_s, p_d) = keyboard.get_physical_wasd();
            let is_autostrafing = self.bhop.is_strafing();
            if !p_w && keyboard.w_pressed() && !self.counter_strafe.is_w_active() {
                keyboard.w_release();
            }
            if !p_s && keyboard.s_pressed() && !self.counter_strafe.is_s_active() {
                keyboard.s_release();
            }
            if !p_a && keyboard.a_pressed() && !self.counter_strafe.is_a_active() && !is_autostrafing {
                keyboard.a_release();
            }
            if !p_d && keyboard.d_pressed() && !self.counter_strafe.is_d_active() && !is_autostrafing {
                keyboard.d_release();
            }
        }

        self.mic_tone.run(&self.input, config);

        let aimbot_interval_ms = if config.aimbot_tps == 0 { 0 } else { 1000 / config.aimbot_tps.max(1) };
        let mut t_aim_val = 0.0;
        let mut t_rcs_val = 0.0;

        if self.last_aimbot.elapsed() >= Duration::from_millis(aimbot_interval_ms as u64) {
            let t_aim_start = Instant::now();
            let aimbot_ran = self.aimbot(config, mouse);
            t_aim_val = t_aim_start.elapsed().as_secs_f32() * 1000.0;

            if !aimbot_ran {
                let t_rcs_start = Instant::now();
                self.rcs(config, mouse);
                t_rcs_val = t_rcs_start.elapsed().as_secs_f32() * 1000.0;
            }
            self.last_aimbot = Instant::now();
        }

        if let Ok(mut t) = crate::game::TELEMETRY.lock() {
            t.input_update_ms = t_input_val;
            if t_cache_val > 0.0 {
                t.cache_entities_ms = t_cache_val;
            }
            if t_bvh_val > 0.0 {
                t.check_bvh_ms = t_bvh_val;
            }
            t.triggerbot_ms = t_trigger_val;
            t.bhop_ms = t_bhop_val;
            t.counter_strafe_ms = t_counter_strafe_val;
            t.aimbot_ms = t_aim_val;
            t.rcs_ms = t_rcs_val;
            t.other_features_ms = t_other_features_val;
            t.total_loop_ms = total_start.elapsed().as_secs_f32() * 1000.0;
        }
    }



    pub fn data(&mut self, config: &Config, data: &mut Data) {
        data.players.clear();
        data.friendlies.clear();
        data.spectators.clear();
        data.entities.clear();

        let sdl_window = self.process.read::<u64>(self.offsets.direct.sdl_window);
        if sdl_window == 0 {
            data.window_position = Vec2::ZERO;
            data.window_size = Vec2::ONE;
        } else {
            data.window_position = self.process.read::<IVec2>(sdl_window + 0x18).as_vec2();
            data.window_size = self
                .process
                .read::<IVec2>(sdl_window + 0x18 + 0x08)
                .as_vec2();
        }

        let Some(local_player) = Player::local_player(self) else {
            data.weapon = Weapon::default();
            data.in_game = false;
            return;
        };
        let local_team = local_player.team(self);
        if local_team != TEAM_T && local_team != TEAM_CT {
            data.weapon = Weapon::default();
            data.in_game = false;
            return;
        }
        let is_ffa = self.is_ffa();
        let spectator_target = local_player.spectator_target(self);
        let active_pawn = if let Some(target) = spectator_target {
            target.pawn
        } else {
            local_player.pawn
        };

        let bone_vis_interval_ms = if config.bone_vis_hz == 0 { 0 } else { 1000 / config.bone_vis_hz.max(1) };
        let update_bone_vis = self.last_bone_vis.elapsed() >= Duration::from_millis(bone_vis_interval_ms as u64);
        if update_bone_vis {
            self.last_bone_vis = Instant::now();
        }

        let player_results: Vec<(bool, PlayerData)> = self
            .players
            .par_iter()
            .filter_map(|player| {
                if spectator_target.is_some() && player.pawn == active_pawn {
                    return None;
                }

                let steam_id = player.steam_id(self);
                let bones = player.all_bones(self);
                let visible_bones = if update_bone_vis {
                    let mut map = std::collections::HashMap::new();
                    if let Some(bvh) = &self.bvh {
                        let eye_pos = local_player.eye_position(self);
                        for (bone, pos) in &bones {
                            map.insert(*bone, bvh.has_line_of_sight(eye_pos, *pos));
                        }
                    }
                    map
                } else {
                    self.cached_bone_vis.get(&steam_id).cloned().unwrap_or_default()
                };

                let mut chams_segments = Vec::new();
                if config.player.draw_chams {
                    if let Some(bvh) = &self.bvh {
                        let eye_pos = local_player.eye_position(self);

                        let connections = [
                            (Bones::Hip, Bones::Spine1, 4, 1.0),
                            (Bones::Spine1, Bones::Spine2, 4, 1.2),
                            (Bones::Spine2, Bones::Spine3, 4, 1.2),
                            (Bones::Spine3, Bones::Spine4, 4, 1.2),
                            (Bones::Spine4, Bones::Neck, 3, 0.8),
                            (Bones::Neck, Bones::LeftShoulder, 3, 0.6),
                            (Bones::LeftShoulder, Bones::LeftElbow, 5, 0.5),
                            (Bones::LeftElbow, Bones::LeftHand, 5, 0.4),
                            (Bones::Neck, Bones::RightShoulder, 3, 0.6),
                            (Bones::RightShoulder, Bones::RightElbow, 5, 0.5),
                            (Bones::RightElbow, Bones::RightHand, 5, 0.4),
                            (Bones::Hip, Bones::LeftHip, 4, 0.8),
                            (Bones::LeftHip, Bones::LeftKnee, 6, 0.7),
                            (Bones::LeftKnee, Bones::LeftFoot, 6, 0.5),
                            (Bones::Hip, Bones::RightHip, 4, 0.8),
                            (Bones::RightHip, Bones::RightKnee, 6, 0.7),
                            (Bones::RightKnee, Bones::RightFoot, 6, 0.5),
                        ];

                        for &(a_bone, b_bone, samples, thickness) in &connections {
                            if let (Some(a_pos), Some(b_pos)) = (bones.get(&a_bone), bones.get(&b_bone)) {
                                let mut prev_pt = *a_pos;
                                let mut prev_vis = bvh.has_line_of_sight(eye_pos, prev_pt);
                                
                                for i in 1..=samples {
                                    let t = i as f32 / samples as f32;
                                    let pt = *a_pos + (*b_pos - *a_pos) * t;
                                    let vis = bvh.has_line_of_sight(eye_pos, pt);
                                    
                                    chams_segments.push((prev_pt, pt, prev_vis, thickness));
                                    
                                    prev_pt = pt;
                                    prev_vis = vis;
                                }
                            }
                        }
                    }
                }

                let is_friendly = !is_ffa && player.team(self) == local_team;
                let player_data = PlayerData {
                    steam_id,
                    health: player.health(self),
                    armor: player.armor(self),
                    position: player.position(self),
                    velocity: player.velocity(self),
                    head: player.bone_position(self, Bones::Head.u64()),
                    name: player.name(self),
                    weapon: player.weapon(self),
                    ammo: (player.clip_ammo(self), player.reserve_ammo(self)),
                    bones,
                    has_defuser: player.has_defuser(self),
                    has_helmet: player.has_helmet(self),
                    has_bomb: player.has_bomb(self),
                    is_defusing: player.is_defusing(self),
                    visible: player.visible(self, &local_player),
                    visible_bones,
                    color: player.color(self),
                    rotation: player.rotation(self),
                    sound: player.is_making_sound(self),
                    fov: player.fov(self) as i32,
                    shots_fired: player.shots_fired(self),
                    inaccuracy: player.inaccuracy(self),
                    round_kills: player.round_kills(self).unwrap_or(0),
                    chams_segments,
                };

                Some((is_friendly, player_data))
            })
            .collect();

        if update_bone_vis {
            for (_, pd) in &player_results {
                self.cached_bone_vis.insert(pd.steam_id, pd.visible_bones.clone());
            }
        }

        for (is_friendly, player_data) in player_results {
            if is_friendly {
                data.friendlies.push(player_data);
            } else {
                data.players.push(player_data);
            }
        }

        for player in &self.dead_players {
            if let Some(target) = player.spectator_target(self)
                && target.pawn == active_pawn
            {
                data.spectators.push(player.name(self));
            }
        }

        let active_player = spectator_target.as_ref().unwrap_or(&local_player);

        data.total_damage = active_player.round_damage(self).unwrap_or(0.0) as u32;
        data.aim_punch = active_player.aim_punch(self);

        let bones = active_player.all_bones(self);
        let mut visible_bones = std::collections::HashMap::new();
        if let Some(bvh) = &self.bvh {
            let eye_pos = active_player.eye_position(self);
            for (bone, pos) in &bones {
                visible_bones.insert(*bone, bvh.has_line_of_sight(eye_pos, *pos));
            }
        }

        data.local_player = PlayerData {
            steam_id: active_player.steam_id(self),
            health: active_player.health(self),
            armor: active_player.armor(self),
            position: active_player.position(self),
            velocity: active_player.velocity(self),
            head: active_player.bone_position(self, Bones::Head.u64()),
            name: active_player.name(self),
            weapon: active_player.weapon(self),
            ammo: (
                active_player.clip_ammo(self),
                active_player.reserve_ammo(self),
            ),
            bones,
            has_defuser: active_player.has_defuser(self),
            has_helmet: active_player.has_helmet(self),
            has_bomb: active_player.has_bomb(self),
            is_defusing: active_player.is_defusing(self),
            visible: true,
            visible_bones,
            color: active_player.color(self),
            rotation: active_player.rotation(self),
            sound: active_player.is_making_sound(self),
            fov: active_player.fov(self) as i32,
            shots_fired: active_player.shots_fired(self),
            inaccuracy: active_player.inaccuracy(self),
            round_kills: active_player.round_kills(self).unwrap_or(0),
            chams_segments: Vec::new(),
        };

        data.entities.clear();
        for entity in &self.entities {
            data.entities.push(match entity {
                Entity::Weapon { weapon, entity } => EntityInfo::Weapon {
                    weapon: weapon.clone(),
                    position: Player::entity(*entity).position(self),
                    ammo: (
                        weapon_clip_ammo(*entity, self),
                        weapon_reserve_ammo(*entity, self),
                    ),
                },
                Entity::Inferno(inferno) => EntityInfo::Inferno(inferno.info(self)),
                Entity::Smoke(smoke) => EntityInfo::Smoke(smoke.info(self)),
                Entity::Molotov(molotov) => EntityInfo::Molotov(molotov.info(self)),
                Entity::Flashbang(entity) => {
                    EntityInfo::Flashbang(grenade_info(*entity, "Flashbang", self))
                }
                Entity::HeGrenade(entity) => {
                    EntityInfo::HeGrenade(grenade_info(*entity, "HE Grenade", self))
                }
                Entity::Decoy(entity) => EntityInfo::Decoy(grenade_info(*entity, "Decoy", self)),
            });
        }

        data.weapon = active_player.weapon(self);
        data.in_game = true;
        data.is_ffa = is_ffa;
        data.map_name = self.current_map();
        data.aimbot_active = if self.aimbot_config(config).mode == KeyMode::Toggle {
            self.aim.active
        } else {
            false
        };
        data.triggerbot_active = if self.triggerbot_config(config).mode == KeyMode::Toggle {
            self.trigger.active
        } else {
            false
        };
        data.esp_active = self.esp_enabled(config);
        data.item_esp_active = self.item_esp_enabled(config);

        data.view_matrix = self.process.read::<Mat4>(self.offsets.direct.view_matrix);
        data.view_angles = active_player.view_angles(self);

        if let Some(bomb) = &self.planted_c4 {
            data.bomb.planted = bomb.is_planted(self);
            data.bomb.timer = bomb.time_to_explosion(self);
            data.bomb.position = bomb.position(self);
            data.bomb.being_defused = bomb.is_being_defused(self);
            data.bomb.defuse_remain_time = bomb.time_to_defuse(self);
        } else {
            data.bomb.planted = false;
        }

        data.bvh_loaded = self.bvh.is_some();
        data.bvh_triangles_count = self.bvh.as_ref().map(|b| b.all_triangles().len()).unwrap_or(0);
        if let Some(bvh) = &self.bvh {
            let eye_pos = active_player.eye_position(self);
            let view_angles = active_player.view_angles(self);
            let dir = vector_from_angles(view_angles.x, view_angles.y);
            data.eye_pos = eye_pos;
            data.ray_dir = dir;
            let hit = bvh.raycast(eye_pos, dir, 2.0, 8192.0);
            let mut verbosity = 0;
            for arg in std::env::args() {
                if arg.starts_with("-v") {
                    let v_count = arg.chars().filter(|&c| c == 'v').count() as u8;
                    verbosity = verbosity.max(v_count);
                }
            }

            if let Some((t, tri)) = hit {
                data.looking_at_material = Some(tri.material);
                data.raycast_hit = Some((t, tri.material));
                
                let start = eye_pos + dir * 2.0;
                let end = eye_pos + dir * (t + 256.0);
                let (thickness, wall_count) = bvh.wall_thickness(start, end);
                data.wall_thickness = if thickness > 0.0 { Some(thickness) } else { None };

                let weapon_penetration = data.weapon.penetration();
                let base_damage = data.weapon.base_damage() as f32;
                if thickness > 0.0 && base_damage > 0.0 && weapon_penetration > 0.0 {
                    let penetration = weapon_penetration.max(0.1);
                    let layer_penalty = (wall_count as f32 - 1.0).max(0.0) * 15.0;
                    let damage_drop = (thickness * 2.5) / penetration + layer_penalty;
                    let final_damage = (base_damage * (1.0 - damage_drop / 100.0)).max(0.0);
                    let hs_dmg = final_damage * 4.0;
                    data.penetration_damage = Some(final_damage);
                    data.penetration_headshot_damage = Some(hs_dmg);
                } else {
                    data.penetration_damage = None;
                    data.penetration_headshot_damage = None;
                }
            } else {
                data.looking_at_material = None;
                data.raycast_hit = None;
                data.wall_thickness = None;
                data.penetration_damage = None;
                data.penetration_headshot_damage = None;
            }
        } else {
            data.looking_at_material = None;
            data.raycast_hit = None;
            data.wall_thickness = None;
            data.penetration_damage = None;
            data.penetration_headshot_damage = None;
            data.eye_pos = glam::Vec3::ZERO;
            data.ray_dir = glam::Vec3::ZERO;
        }

        // Override penetration damage with aimbot or triggerbot predicted damage if targeting/firing
        if self.aim.active {
            if let Some((dmg, hs_dmg)) = self.aimbot_predicted_damage {
                data.penetration_damage = Some(dmg);
                data.penetration_headshot_damage = Some(hs_dmg);
            }
        } else if self.trigger.active {
            if let Some((dmg, hs_dmg)) = self.triggerbot_predicted_damage {
                data.penetration_damage = Some(dmg);
                data.penetration_headshot_damage = Some(hs_dmg);
            }
        }
    }

    pub fn new() -> Self {
        Self {
            is_valid: false,
            process: Process::new(-1),
            offsets: Offsets::default(),
            input: Input::new(),
            bvh: None,
            current_bvh: String::new(),
            target: Target::default(),
            players: Vec::with_capacity(64),
            dead_players: Vec::with_capacity(12),
            entities: Vec::with_capacity(128),
            recoil: Recoil::default(),
            aim: Aimbot::default(),
            trigger: Triggerbot::default(),
            bhop: Bhop::default(),
            counter_strafe: CounterStrafe::default(),
            mic_tone: crate::cs2::features::mic_tone::MicTone::default(),
            esp: EspToggle::default(),
            weapon: Weapon::default(),
            planted_c4: None,
            last_cache: Instant::now(),
            last_bvh: Instant::now(),
            last_bone_vis: Instant::now(),
            cached_bone_vis: std::collections::HashMap::new(),
            last_bhop: Instant::now(),
            last_aimbot: Instant::now(),
            last_input: Instant::now(),
            last_trigger: Instant::now(),
            aimbot_predicted_damage: None,
            triggerbot_predicted_damage: None,
        }
    }

    fn aimbot_config<'a>(&self, config: &'a Config) -> &'a AimbotConfig {
        if let Some(weapon_config) = config.aim.weapons.get(&self.weapon)
            && weapon_config.aimbot.enable_override
        {
            return &weapon_config.aimbot;
        }
        &config.aim.global.aimbot
    }

    fn rcs_config<'a>(&self, config: &'a Config) -> &'a RcsConfig {
        if let Some(weapon_config) = config.aim.weapons.get(&self.weapon)
            && weapon_config.rcs.enable_override
        {
            return &weapon_config.rcs;
        }
        &config.aim.global.rcs
    }

    fn triggerbot_config<'a>(&self, config: &'a Config) -> &'a TriggerbotConfig {
        if let Some(weapon_config) = config.aim.weapons.get(&self.weapon)
            && weapon_config.triggerbot.enable_override
        {
            return &weapon_config.triggerbot;
        }
        &config.aim.global.triggerbot
    }

    fn angle_to_target(&self, local_player: &Player, position: &Vec3, aim_punch: &Vec2) -> Vec2 {
        let eye_position = local_player.eye_position(self);
        let forward = (position - eye_position).normalize();

        let mut angles = angles_from_vector(&forward) - aim_punch;
        vec2_clamp(&mut angles);

        angles
    }

    fn entity_has_owner(&self, entity: u64) -> bool {
        self.process
            .read::<i32>(entity + self.offsets.controller.owner_entity)
            != -1
    }

    // convars
    fn get_sensitivity(&self) -> f32 {
        self.process.read(self.offsets.convar.sensitivity + 0x58)
    }

    fn is_ffa(&self) -> bool {
        self.process.read::<u8>(self.offsets.convar.ffa + 0x58) == 1
    }

    fn current_time(&self) -> f32 {
        let global_vars: u64 = self.process.read(self.offsets.direct.global_vars);
        self.process.read(global_vars + 0x30)
    }

    fn current_map(&self) -> String {
        let global_vars: u64 = self.process.read(self.offsets.direct.global_vars);
        self.process
            .read_string(self.process.read(global_vars + 0x198))
    }

    fn distance_scale(&self, distance: f32) -> f32 {
        if distance > 500.0 {
            1.0
        } else {
            5.0 - (distance / 125.0)
        }
    }

    fn check_bvh(&mut self) {
        crate::profile_scope!("check_bvh");
        let current_map = self.current_map();
        if current_map.is_empty() || current_map == "<empty>" {
            if !self.current_bvh.is_empty() {
                self.current_bvh.clear();
                self.bvh = None;
            }
            return;
        }

        if current_map != self.current_bvh {
            // Map changed or not loaded yet.
            // Wait for local player pawn to ensure map physics are fully loaded in memory.
            if Player::local_player(self).is_none() {
                return;
            }

            if let Some(bvh) = read_map(self) {
                self.current_bvh = current_map.clone();
                utils::info!("Loaded bvh for {current_map}");

                // generate walls json for radar
                let mut lines: Vec<i32> = Vec::new();
                for t in bvh.all_triangles() {
                    let v0 = t.v0;
                    let v1 = t.v1;
                    let v2 = t.v2;

                    let e1 = v1 - v0;
                    let e2 = v2 - v0;
                    let normal = e1.cross(e2).normalize_or_zero();

                    if normal.z.abs() < 0.1 {
                        // Project onto XY and find the longest segment (since vertical walls are a line in XY)
                        let d01 = (v0.x - v1.x).powi(2) + (v0.y - v1.y).powi(2);
                        let d12 = (v1.x - v2.x).powi(2) + (v1.y - v2.y).powi(2);
                        let d20 = (v2.x - v0.x).powi(2) + (v2.y - v0.y).powi(2);
                        
                        let (p1, p2) = if d01 >= d12 && d01 >= d20 {
                            (v0, v1)
                        } else if d12 >= d01 && d12 >= d20 {
                            (v1, v2)
                        } else {
                            (v2, v0)
                        };

                        // Ignore very short segments (< 5 units) to reduce noise and file size
                        if (p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2) > 25.0 {
                            lines.push(p1.x as i32);
                            lines.push(p1.y as i32);
                            lines.push(p2.x as i32);
                            lines.push(p2.y as i32);
                        }
                    }
                }

                if let Ok(json) = serde_json::to_string(&lines) {
                    let _ = std::fs::write(format!("radar/server/assets/{}.json", current_map), json);
                }

                self.bvh = Some(bvh);
            }
        } else if self.bvh.is_none() {
            // Map hasn't changed but BVH is missing (maybe previous attempts failed)
            if Player::local_player(self).is_none() {
                return;
            }

            if let Some(bvh) = read_map(self) {
                utils::info!("Loaded bvh for {current_map}");

                // generate walls json for radar
                let mut lines: Vec<i32> = Vec::new();
                for t in bvh.all_triangles() {
                    let v0 = t.v0;
                    let v1 = t.v1;
                    let v2 = t.v2;

                    let e1 = v1 - v0;
                    let e2 = v2 - v0;
                    let normal = e1.cross(e2).normalize_or_zero();

                    if normal.z.abs() < 0.1 {
                        // Project onto XY and find the longest segment (since vertical walls are a line in XY)
                        let d01 = (v0.x - v1.x).powi(2) + (v0.y - v1.y).powi(2);
                        let d12 = (v1.x - v2.x).powi(2) + (v1.y - v2.y).powi(2);
                        let d20 = (v2.x - v0.x).powi(2) + (v2.y - v0.y).powi(2);
                        
                        let (p1, p2) = if d01 >= d12 && d01 >= d20 {
                            (v0, v1)
                        } else if d12 >= d01 && d12 >= d20 {
                            (v1, v2)
                        } else {
                            (v2, v0)
                        };

                        // Ignore very short segments (< 5 units) to reduce noise and file size
                        if (p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2) > 25.0 {
                            lines.push(p1.x as i32);
                            lines.push(p1.y as i32);
                            lines.push(p2.x as i32);
                            lines.push(p2.y as i32);
                        }
                    }
                }

                if let Ok(json) = serde_json::to_string(&lines) {
                    let _ = std::fs::write(format!("radar/server/assets/{}.json", current_map), json);
                }

                self.bvh = Some(bvh);
            }
        }
    }

    fn check_hotkey(input: &Input, mode: KeyMode, key1: KeyCode, key2: KeyCode, active: &mut bool) -> bool {
        match mode {
            KeyMode::Hold => input.is_key_pressed(key1) || input.is_key_pressed(key2),
            KeyMode::Toggle => {
                if input.key_just_pressed(key1) || input.key_just_pressed(key2) {
                    *active = !*active;
                }
                *active
            }
        }
    }
}
