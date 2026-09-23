use egui::{Align2, Color32, Painter, Pos2, Stroke, pos2, vec2};
use shared::{Data, WeaponClass};

use crate::{
    config::aim::KeyMode, config::text::TextPosition, math::world_to_screen, ui::app::AppState,
};

impl AppState {
    pub fn overlay_debug(&self, painter: &Painter, data: &Data) {
        let mut y_offset = 100.0;

        if self.config.hud.debug {
            let pos = data.local_player.position;
            let vel = data.local_player.velocity;
            let speed = (vel.x * vel.x + vel.y * vel.y).sqrt();

            let debug_text = format!(
                "Position: {:.2}, {:.2}, {:.2}\nSpeed: {:.2}\nJump CPS: {} | Total CPS: {}",
                pos.x, pos.y, pos.z, speed, data.jump_cps, data.total_cps
            );

            self.text(
                painter,
                debug_text,
                pos2(10.0, y_offset),
                Align2::LEFT_TOP,
                Some(Color32::WHITE),
            );
            y_offset += 50.0;
        }

        if self.config.hud.raycast_debug {
            let mut raycast_text = String::new();

            if let Some(mat_id) = data.looking_at_material {
                let name = match mat_id {
                    0 => "default",
                    1 => "metal",
                    2 => "cardboard",
                    3 => "wood",
                    4 => "concrete",
                    5 => "rock",
                    6 => "gravel",
                    7 => "dirt",
                    8 => "grass",
                    9 => "tile",
                    10 => "glass",
                    11 => "plaster",
                    12 => "plastic",
                    13 => "cloth",
                    14 => "carpet",
                    15 => "mud",
                    16 => "sand",
                    17 => "snow",
                    18 => "ice",
                    19 => "glass_window",
                    20 => "rubber",
                    21 => "clay",
                    22 => "plasterboard",
                    23 => "wood_plank",
                    24 => "metal_barrel",
                    25 => "floating_water",
                    _ => "unknown",
                };
                raycast_text.push_str(&format!("Material: {} ({})", name, mat_id));
            }

            if let Some(thick) = data.wall_thickness {
                if !raycast_text.is_empty() { raycast_text.push('\n'); }
                raycast_text.push_str(&format!("Wall Thickness: {:.1} in", thick));
            }
            if let Some(dmg) = data.penetration_damage {
                if !raycast_text.is_empty() { raycast_text.push('\n'); }
                raycast_text.push_str(&format!("Penetration Damage: {:.1}", dmg));
            }
            if let Some(hs_dmg) = data.penetration_headshot_damage {
                if !raycast_text.is_empty() { raycast_text.push('\n'); }
                raycast_text.push_str(&format!("Penetration Headshot Damage: {:.1}", hs_dmg));
            }

            let weapon = &data.weapon;
            if *weapon != shared::weapon::Weapon::None {
                if !raycast_text.is_empty() { raycast_text.push('\n'); }
                raycast_text.push_str(&format!(
                    "Held Weapon: {} (Damage: {})",
                    weapon,
                    weapon.damage_description()
                ));
            }

            if !raycast_text.is_empty() { raycast_text.push('\n'); }
            raycast_text.push_str(&format!(
                "BVH Loaded: {} (Tris: {})\nEyePos: {:.2}, {:.2}, {:.2}\nRayDir: {:.3}, {:.3}, {:.3}\nRaycast Hit: {:?}",
                data.bvh_loaded,
                data.bvh_triangles_count,
                data.eye_pos.x, data.eye_pos.y, data.eye_pos.z,
                data.ray_dir.x, data.ray_dir.y, data.ray_dir.z,
                data.raycast_hit
            ));

            self.text(
                painter,
                raycast_text,
                pos2(10.0, y_offset),
                Align2::LEFT_TOP,
                Some(Color32::WHITE),
            );
        }
    }

    pub fn draw_bomb_timer(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.bomb_timer || !data.bomb.planted {
            return;
        }

        if let Some(pos) = world_to_screen(&data.bomb.position, data) {
            let cat = &self.config.hud.overlay_text.bomb_timer;
            let anchor = point_anchor(pos, cat.position, cat.font_size * 0.3);
            self.text_sized(
                painter,
                format!("{:.3}", data.bomb.timer),
                anchor,
                cat.align.to_align2(),
                cat.color,
                cat.font_size,
            );
            if data.bomb.being_defused {
                self.text_sized(
                    painter,
                    format!("defusing {:.3}", data.bomb.defuse_remain_time),
                    anchor + vec2(0.0, cat.font_size),
                    cat.align.to_align2(),
                    cat.color,
                    cat.font_size,
                );
            }
        }

        let fraction = (data.bomb.timer / 40.0).clamp(0.0, 1.0);
        let color = self.health_color((fraction * 100.0) as i32, 100, 255);
        painter.line(
            vec![
                pos2(0.0, data.window_size.y),
                pos2(data.window_size.x * fraction, data.window_size.y),
            ],
            Stroke::new(self.config.hud.line_width * 3.0, color),
        );
    }

    pub fn draw_fov_circle(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.fov_circle || !data.in_game {
            return;
        }

        let weapon_config = self.aimbot_config(&data.weapon);

        if !weapon_config.enabled || (weapon_config.mode == KeyMode::Toggle && !data.aimbot_active)
        {
            return;
        }

        let aim_fov = weapon_config.fov;

        if weapon_config.distance_adjusted_fov {
            self.draw_distance_scaled_fov_circle(painter, data, aim_fov, 125.0, Color32::GREEN);
            self.draw_distance_scaled_fov_circle(painter, data, aim_fov, 250.0, Color32::YELLOW);
            self.draw_distance_scaled_fov_circle(painter, data, aim_fov, 500.0, Color32::RED);
        } else {
            self.draw_simple_fov_circle(painter, data, aim_fov, Color32::WHITE);
        }
    }

    pub fn draw_spread_circle(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.spread_circle || !data.in_game {
            return;
        }

        let inaccuracy = data.local_player.inaccuracy;
        
        if inaccuracy <= 0.0 {
            return;
        }

        let center = data.window_size / 2.0;
        let fov = data.local_player.fov as f32;
        let fov = if fov == 0.0 { 90.0 } else { fov };
        
        let screen_width = data.window_size.x;
        let pixels_per_degree = screen_width / fov;

        let punch_x = center.x - (data.aim_punch.y * pixels_per_degree);
        let punch_y = center.y + (data.aim_punch.x * pixels_per_degree);
        let pos = pos2(punch_x, punch_y);

        let spread_deg = inaccuracy.to_degrees();
        let radius = spread_deg * pixels_per_degree;

        if radius > 0.1 {
            painter.circle_stroke(
                pos,
                radius,
                Stroke::new(self.config.hud.line_width, self.config.hud.spread_circle_color),
            );
        }
    }

    pub fn draw_keybind_list(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.keybind_list {
            return;
        }

        let cat = &self.config.hud.overlay_text.keybind_list;
        let position = screen_anchor(
            [data.window_size.x, data.window_size.y],
            cat.position,
            10.0,
            0.0,
        );
        let aimbot_color = if data.aimbot_active {
            Color32::GREEN
        } else {
            cat.color
        };
        self.text_sized(
            painter,
            format!("Aimbot: {:?}", self.config.aim.aimbot_hotkey),
            position,
            cat.align.to_align2(),
            aimbot_color,
            cat.font_size,
        );

        let triggerbot_color = if data.triggerbot_active {
            Color32::GREEN
        } else {
            cat.color
        };
        self.text_sized(
            painter,
            format!("Triggerbot: {:?}", self.config.aim.triggerbot_hotkey),
            position + vec2(0.0, cat.font_size),
            cat.align.to_align2(),
            triggerbot_color,
            cat.font_size,
        );
    }

    pub fn draw_spectator_list(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.spectator_list {
            return;
        }

        let cat = &self.config.hud.overlay_text.spectator_list;
        let position = screen_anchor(
            [data.window_size.x, data.window_size.y],
            cat.position,
            10.0,
            cat.font_size * 3.0,
        );
        self.text_sized(
            painter,
            "Spectators:",
            position,
            cat.align.to_align2(),
            cat.color,
            cat.font_size,
        );

        for (i, name) in data.spectators.iter().enumerate() {
            self.text_sized(
                painter,
                format!("> {name}"),
                position + vec2(0.0, cat.font_size * (i as f32 + 1.0)),
                cat.align.to_align2(),
                cat.color,
                cat.font_size,
            );
        }
    }

    fn get_current_fov(&self) -> f32 {
        (if self.config.misc.fov_changer {
            self.config.misc.desired_fov
        } else {
            crate::constants::cs2::DEFAULT_FOV
        }) as f32
    }

    fn calculate_fov_radius(&self, data: &Data, target_fov: f32) -> f32 {
        let current_fov = self.get_current_fov();
        let screen_width = data.window_size.x;

        let current_fov_tan = (current_fov.to_radians() / 2.0).tan();
        if current_fov_tan == 0.0 {
            return 0.0;
        }

        let target_fov_tan = (target_fov.to_radians() / 2.0).tan();
        (target_fov_tan / current_fov_tan) * (screen_width / 2.0)
    }

    fn draw_fov_circle_impl(&self, painter: &Painter, data: &Data, radius: f32, color: Color32) {
        let center = pos2(data.window_size.x / 2.0, data.window_size.y / 2.0);
        let stroke = Stroke::new(self.config.hud.line_width, color);
        painter.circle_stroke(center, radius, stroke);
    }

    fn get_distance_fov_scale(&self, distance: f32) -> f32 {
        (5.0 - (distance / 125.0)).max(1.0)
    }

    fn draw_simple_fov_circle(
        &self,
        painter: &Painter,
        data: &Data,
        target_fov: f32,
        color: Color32,
    ) {
        let radius = self.calculate_fov_radius(data, target_fov);
        self.draw_fov_circle_impl(painter, data, radius, color);
    }

    fn draw_distance_scaled_fov_circle(
        &self,
        painter: &Painter,
        data: &Data,
        base_aim_fov: f32,
        distance: f32,
        color: Color32,
    ) {
        let scale = self.get_distance_fov_scale(distance);
        let target_fov = base_aim_fov * scale;

        let radius = self.calculate_fov_radius(data, target_fov);
        self.draw_fov_circle_impl(painter, data, radius, color);
    }

    pub fn draw_sniper_crosshair(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.sniper_crosshair.enabled
            || data.weapon.weapon_class() != WeaponClass::Sniper
        {
            return;
        }

        let length = self.config.hud.sniper_crosshair.line_length;
        let gap = self.config.hud.sniper_crosshair.gap / 2.0;
        let center = data.window_size / 2.0;

        let stroke = Stroke::new(
            self.config.hud.sniper_crosshair.line_width,
            self.config.hud.sniper_crosshair.color,
        );

        painter.line_segment(
            [
                pos2(center.x + gap, center.y),
                pos2(center.x + gap + length, center.y),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x, center.y + gap),
                pos2(center.x, center.y + gap + length),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x - gap, center.y),
                pos2(center.x - gap - length, center.y),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x, center.y - gap),
                pos2(center.x, center.y - gap - length),
            ],
            stroke,
        );
    }

    pub fn draw_hit_marker(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.hit_marker {
            return;
        }

        let elapsed = self.hit_marker_time.elapsed().as_secs_f32();
        if elapsed > 1.0 {
            return;
        }

        let alpha = (1.0 - elapsed) * 255.0;
        let color = Color32::from_rgba_unmultiplied(255, 255, 255, alpha as u8);

        let center = data.window_size / 2.0;
        let gap = 5.0;
        let length = 10.0;
        let stroke = Stroke::new(self.config.hud.line_width, color);

        painter.line_segment(
            [
                pos2(center.x - gap, center.y - gap),
                pos2(center.x - gap - length, center.y - gap - length),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x + gap, center.y - gap),
                pos2(center.x + gap + length, center.y - gap - length),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x - gap, center.y + gap),
                pos2(center.x - gap - length, center.y + gap + length),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x + gap, center.y + gap),
                pos2(center.x + gap + length, center.y + gap + length),
            ],
            stroke,
        );
    }

    pub fn draw_recoil_crosshair(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.recoil_crosshair.enabled {
            return;
        }

        if data.local_player.shots_fired == 0 {
            return;
        }

        let center = data.window_size / 2.0;
        let fov = data.local_player.fov as f32;
        let fov = if fov == 0.0 { 90.0 } else { fov };
        
        let screen_width = data.window_size.x;
        let pixels_per_degree = screen_width / fov;

        // Note: The y axis might need to be subtracted or added depending on if pitch is positive or negative.
        // Usually pitch up is negative, so adding it to y (which goes down) correctly matches the screen going up!
        let punch_x = center.x - (data.aim_punch.y * pixels_per_degree);
        let punch_y = center.y + (data.aim_punch.x * pixels_per_degree);

        let pos = pos2(punch_x, punch_y);

        let length = self.config.hud.recoil_crosshair.line_length;
        let gap = self.config.hud.recoil_crosshair.gap / 2.0;

        let stroke = Stroke::new(
            self.config.hud.recoil_crosshair.line_width,
            self.config.hud.recoil_crosshair.color,
        );

        painter.line_segment([pos2(pos.x + gap, pos.y), pos2(pos.x + gap + length, pos.y)], stroke);
        painter.line_segment([pos2(pos.x, pos.y + gap), pos2(pos.x, pos.y + gap + length)], stroke);
        painter.line_segment([pos2(pos.x - gap, pos.y), pos2(pos.x - gap - length, pos.y)], stroke);
        painter.line_segment([pos2(pos.x, pos.y - gap), pos2(pos.x, pos.y - gap - length)], stroke);
    }

    pub fn draw_sound_esp(&self, painter: &Painter, data: &Data) {
        if !self.config.player.sound.enabled {
            return;
        }

        for event in &data.sound_events {
            let fade = (1.0 - event.age_secs / 2.0).clamp(0.0, 1.0);
            if fade <= 0.0 {
                continue;
            }

            let max_r = match event.event_type {
                shared::data::SoundEventType::Footstep => self.config.player.sound.footstep_diameter * 0.5,
                shared::data::SoundEventType::Gunshot => self.config.player.sound.gunshot_diameter * 0.5,
                _ => self.config.player.sound.weapon_diameter * 0.5,
            };

            let current_r = max_r * (1.0 - fade * 0.5);
            let alpha = (180.0 * fade) as u8;

            let color = match event.event_type {
                shared::data::SoundEventType::Footstep => Color32::from_rgba_unmultiplied(255, 200, 0, alpha),
                shared::data::SoundEventType::Gunshot => Color32::from_rgba_unmultiplied(255, 50, 50, alpha),
                shared::data::SoundEventType::BombPlant | shared::data::SoundEventType::BombDefuse => Color32::from_rgba_unmultiplied(255, 0, 255, alpha),
                _ => Color32::from_rgba_unmultiplied(0, 229, 255, alpha),
            };

            // Draw 3D floor circle
            let mut pts = Vec::with_capacity(16);
            for i in 0..16 {
                let rad = (i as f32 / 16.0) * std::f32::consts::TAU;
                let world_pt = event.position + glam::vec3(current_r * rad.cos(), current_r * rad.sin(), 0.0);
                if let Some(screen_pt) = world_to_screen(&world_pt, data) {
                    pts.push(screen_pt);
                }
            }

            if pts.len() >= 3 {
                painter.add(egui::Shape::convex_polygon(
                    pts.clone(),
                    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha / 4),
                    Stroke::new(1.5, color),
                ));
            }

            if let Some(icon_pos) = world_to_screen(&event.position, data) {
                let icon_str = match event.event_type {
                    shared::data::SoundEventType::Footstep => "👣",
                    shared::data::SoundEventType::Gunshot => "💥",
                    shared::data::SoundEventType::Weapon => "🔫",
                    shared::data::SoundEventType::BombPlant => "💣",
                    shared::data::SoundEventType::BombDefuse => "✂️",
                    shared::data::SoundEventType::Reload => "🔄",
                    shared::data::SoundEventType::Scope => "🔍",
                };
                self.text(painter, icon_str, icon_pos, Align2::CENTER_CENTER, Some(color));
            }
        }
    }

    pub fn draw_grenade_warnings(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.grenade_warning.enabled {
            return;
        }

        let mut max_danger_dmg = 0;
        let mut danger_gname = "HE GRENADE";

        for warning in &data.grenade_warnings {
            if self.config.hud.grenade_warning.draw_radius {
                let (r, g, b) = match warning.grenade_type {
                    shared::data::GrenadeType::HE => (255, 50, 50),
                    shared::data::GrenadeType::Molotov => (255, 140, 0),
                    shared::data::GrenadeType::Smoke => (180, 180, 180),
                    shared::data::GrenadeType::Flash => (255, 255, 200),
                    shared::data::GrenadeType::Decoy => (180, 100, 255),
                };
                let stroke_color = Color32::from_rgba_unmultiplied(r, g, b, 200);
                let fill_color = Color32::from_rgba_unmultiplied(r, g, b, 40);

                let mut pts = Vec::with_capacity(16);
                for i in 0..16 {
                    let rad = (i as f32 / 16.0) * std::f32::consts::TAU;
                    let world_pt = warning.position + glam::vec3(warning.blast_radius * rad.cos(), warning.blast_radius * rad.sin(), 0.0);
                    if let Some(screen_pt) = world_to_screen(&world_pt, data) {
                        pts.push(screen_pt);
                    }
                }

                if pts.len() >= 3 {
                    painter.add(egui::Shape::convex_polygon(
                        pts,
                        fill_color,
                        Stroke::new(2.0, stroke_color),
                    ));
                }
            }

            if warning.is_danger && warning.estimated_damage > max_danger_dmg {
                max_danger_dmg = warning.estimated_damage;
                danger_gname = match warning.grenade_type {
                    shared::data::GrenadeType::HE => "HE GRENADE",
                    shared::data::GrenadeType::Molotov => "MOLOTOV / INFERNO",
                    _ => "EXPLOSIVE",
                };
            }
        }

        if self.config.hud.grenade_warning.danger_banner && max_danger_dmg > 0 {
            let banner_center = pos2(data.window_size.x / 2.0, 120.0);
            let rect = egui::Rect::from_center_size(banner_center, egui::vec2(360.0, 42.0));
            painter.rect(
                rect,
                6.0,
                Color32::from_rgba_unmultiplied(180, 20, 20, 230),
                Stroke::new(2.0, Color32::YELLOW),
                egui::StrokeKind::Middle,
            );
            self.text(
                painter,
                format!("⚠️ DANGER: {} BLAST ZONE (-{} HP)", danger_gname, max_danger_dmg),
                banner_center,
                Align2::CENTER_CENTER,
                Some(Color32::WHITE),
            );
        }
    }

    pub fn draw_offscreen_indicators(&self, painter: &Painter, data: &Data) {
        if !self.config.player.offscreen.enabled {
            return;
        }

        let center = pos2(data.window_size.x / 2.0, data.window_size.y / 2.0);
        let r = self.config.player.offscreen.radius_px;
        let s = self.config.player.offscreen.size;

        for player in &data.offscreen_players {
            if self.config.player.offscreen.hide_when_onscreen && player.is_onscreen {
                continue;
            }

            let angle = -player.angle_rad - std::f32::consts::FRAC_PI_2;
            let tip = pos2(center.x + r * angle.cos(), center.y + r * angle.sin());

            let left = pos2(tip.x - s * (angle + 0.4).cos(), tip.y - s * (angle + 0.4).sin());
            let right = pos2(tip.x - s * (angle - 0.4).cos(), tip.y - s * (angle - 0.4).sin());

            let color = if player.visible {
                Color32::YELLOW
            } else if player.team_is_friendly {
                Color32::from_rgb(0, 229, 255)
            } else {
                Color32::RED
            };

            painter.add(egui::Shape::convex_polygon(
                vec![tip, left, right],
                color,
                Stroke::new(1.0, Color32::BLACK),
            ));

            if self.config.player.offscreen.show_distance {
                let dist_pos = pos2(tip.x + (s * 0.8) * angle.cos(), tip.y + (s * 0.8) * angle.sin());
                self.text_sized(
                    painter,
                    format!("{:.0}m", player.distance_m),
                    dist_pos,
                    Align2::CENTER_CENTER,
                    Color32::WHITE,
                    12.0,
                );
            }
        }
    }

    pub fn draw_floating_damage_text(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.floating_damage.enabled {
            return;
        }

        for marker in &data.hit_damage_markers {
            let fade = (1.0 - marker.age_secs / self.config.hud.floating_damage.duration_secs).clamp(0.0, 1.0);
            if fade <= 0.0 {
                continue;
            }

            let float_up = marker.age_secs * 40.0;
            let pos = pos2(marker.screen_pos.x, marker.screen_pos.y - 30.0 - float_up);

            let (text_str, color) = if marker.is_headshot {
                (
                    format!("-{}", marker.damage),
                    Color32::from_rgba_unmultiplied(255, 30, 30, (255.0 * fade) as u8),
                )
            } else {
                (
                    format!("-{}", marker.damage),
                    Color32::from_rgba_unmultiplied(255, 180, 50, (255.0 * fade) as u8),
                )
            };

            self.text_sized(
                painter,
                text_str,
                pos,
                Align2::CENTER_CENTER,
                color,
                22.0 * self.config.hud.floating_damage.scale,
            );
        }
    }
}

pub fn point_anchor(point: Pos2, position: TextPosition, offset: f32) -> Pos2 {
    match position {
        TextPosition::TopLeft => point + vec2(-offset, -offset),
        TextPosition::TopCenter => point + vec2(0.0, -offset),
        TextPosition::TopRight => point + vec2(offset, -offset),
        TextPosition::CenterLeft => point + vec2(-offset, 0.0),
        TextPosition::Center => point,
        TextPosition::CenterRight => point + vec2(offset, 0.0),
        TextPosition::BottomLeft => point + vec2(-offset, offset),
        TextPosition::BottomCenter => point + vec2(0.0, offset),
        TextPosition::BottomRight => point + vec2(offset, offset),
    }
}

pub fn screen_anchor(size: [f32; 2], position: TextPosition, pad_x: f32, offset_y: f32) -> Pos2 {
    let [w, h] = size;
    match position {
        TextPosition::TopLeft => pos2(pad_x, offset_y),
        TextPosition::TopCenter => pos2(w / 2.0, offset_y),
        TextPosition::TopRight => pos2(w - pad_x, offset_y),
        TextPosition::CenterLeft => pos2(pad_x, h / 2.0 + offset_y),
        TextPosition::Center => pos2(w / 2.0, h / 2.0 + offset_y),
        TextPosition::CenterRight => pos2(w - pad_x, h / 2.0 + offset_y),
        TextPosition::BottomLeft => pos2(pad_x, h + offset_y),
        TextPosition::BottomCenter => pos2(w / 2.0, h + offset_y),
        TextPosition::BottomRight => pos2(w - pad_x, h + offset_y),
    }
}
