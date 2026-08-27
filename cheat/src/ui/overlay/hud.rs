use egui::{Align2, Color32, Painter, Stroke, pos2};
use shared::data::Data;

use crate::{
    config::aim::KeyMode, cs2::entity::weapon_class::WeaponClass, math::world_to_screen,
    ui::app::App,
};

impl App {
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
            if *weapon != shared::weapon::Weapon::Unknown {
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
            self.text(
                painter,
                format!("{:.3}", data.bomb.timer),
                pos,
                Align2::CENTER_CENTER,
                None,
            );
            if data.bomb.being_defused {
                self.text(
                    painter,
                    format!("defusing {:.3}", data.bomb.defuse_remain_time),
                    pos2(pos.x, pos.y + self.config.hud.font_size),
                    Align2::CENTER_CENTER,
                    None,
                );
            }
        }

        let fraction = (data.bomb.timer / 40.0).clamp(0.0, 1.0);
        let color = self.health_color((fraction * 100.0) as i32, 255);
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

        let position = pos2(10.0, data.window_size.y / 2.0);
        let aimbot_color = if data.aimbot_active {
            Color32::GREEN
        } else {
            Color32::WHITE
        };
        self.text(
            painter,
            format!("Aimbot: {:?}", self.config.aim.aimbot_hotkey),
            position,
            Align2::LEFT_TOP,
            Some(aimbot_color),
        );

        let triggerbot_color = if data.triggerbot_active {
            Color32::GREEN
        } else {
            Color32::WHITE
        };
        self.text(
            painter,
            format!("Triggerbot: {:?}", self.config.aim.triggerbot_hotkey),
            position + egui::vec2(0.0, self.config.hud.font_size),
            Align2::LEFT_TOP,
            Some(triggerbot_color),
        );
    }

    pub fn draw_spectator_list(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.spectator_list {
            return;
        }

        let position = pos2(
            10.0,
            data.window_size.y / 2.0 + self.config.hud.font_size * 3.0,
        );
        self.text(
            painter,
            "Spectators:",
            position,
            Align2::LEFT_TOP,
            Some(Color32::WHITE),
        );

        for (i, name) in data.spectators.iter().enumerate() {
            self.text(
                painter,
                format!("> {name}"),
                position + egui::vec2(0.0, self.config.hud.font_size * (i as f32 + 1.0)),
                Align2::LEFT_TOP,
                Some(Color32::WHITE),
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
            || WeaponClass::from_string(data.weapon.as_ref()) != WeaponClass::Sniper
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
}
