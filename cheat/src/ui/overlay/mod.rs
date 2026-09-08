use std::sync::Arc;

use egui::{Align2, Color32, PaintCallback, Painter, Pos2, Rect, Shape, Stroke, Ui, pos2};
use egui_glow::{CallbackFn, glow};
use glam::{Vec3, vec3};
use shared::{Data, Weapon};

use crate::{
    config::aim::AimbotConfig,
    math::world_to_screen,
    ui::{app::AppState, grenades::Grenade},
};

mod entity;
mod hud;
pub mod model;
mod models;
mod opengl;
mod player;

impl AppState {
    fn aimbot_config(&self, weapon: &Weapon) -> &AimbotConfig {
        if let Some(weapon_config) = self.config.aim.weapons.get(weapon)
            && weapon_config.aimbot.enable_override
        {
            return &weapon_config.aimbot;
        }
        &self.config.aim.global.aimbot
    }

    pub fn overlay(&mut self, ui: &mut Ui, glow: &Arc<glow::Context>) {
        ui.ctx().set_pixels_per_point(1.0);
        let painter = ui.layer_painter(egui::LayerId::background());

        self.update_trails();
        self.update_player_sounds();

        let data = &self.data.lock();

        if self.model_renderer.is_none() {
            match model::ModelRenderer::new(glow.clone()) {
                Ok(renderer) => self.model_renderer = Some(Arc::new(renderer)),
                Err(error) => utils::error!("failed to initialize model renderer: {error}"),
            }
        }
        self.overlay_debug(&painter, data);

        for player in &data.players {
            if data.esp_active {
                self.draw_player(&painter, player, data);
            }
        }

        for player in &data.friendlies {
            if data.esp_active {
                let is_target = !self.config.player.target_player_name.is_empty() && player.name.contains(&self.config.player.target_player_name);
                let force_show = player.has_bomb || is_target;
                if self.config.player.show_friendlies || force_show {
                    self.draw_player(&painter, player, data);
                }
            }
        }

        for entity in &data.entities {
            self.draw_entity(&painter, entity, data);
        }

        self.draw_player_models(&painter, data);
        self.draw_bomb_timer(&painter, data);
        self.draw_fov_circle(&painter, data);
        self.draw_spread_circle(&painter, data);
        self.draw_sniper_crosshair(&painter, data);
        self.draw_recoil_crosshair(&painter, data);
        self.draw_hit_marker(&painter, data);
        self.draw_keybind_list(&painter, data);
        self.draw_spectator_list(&painter, data);
        self.draw_sound_esp(&painter, data);
        self.draw_grenade_warnings(&painter, data);
        self.draw_offscreen_indicators(&painter, data);
        self.draw_floating_damage_text(&painter, data);

        if data.aimbot_active {
            let cat = &self.config.hud.overlay_text.status_text;
            self.text_sized(
                &painter,
                "aimbot active",
                hud::screen_anchor(
                    [data.window_size.x, data.window_size.y],
                    cat.position,
                    8.0,
                    8.0,
                ),
                cat.align.to_align2(),
                cat.color,
                cat.font_size,
            );
        }

        if data.triggerbot_active {
            let cat = &self.config.hud.overlay_text.status_text;
            self.text_sized(
                &painter,
                "trigger active",
                hud::screen_anchor(
                    [data.window_size.x, data.window_size.y],
                    cat.position,
                    8.0,
                    8.0 + cat.font_size,
                ),
                cat.align.to_align2(),
                cat.color,
                cat.font_size,
            );
        }

        self.grenade_manager(data, &painter);
    }

    fn draw_player_models(&self, painter: &Painter, data: &Data) {
        use crate::config::player::{DrawMode, ModelRenderMode};

        let Some(renderer) = self.model_renderer.as_ref() else {
            return;
        };
        if !data.esp_active || self.config.player.draw_model == DrawMode::None {
            return;
        }

        let view = data.view_matrix.to_cols_array();
        let window = data.window_size;
        let players = data.players.iter().chain(
            self.config
                .player
                .show_friendlies
                .then_some(data.friendlies.iter())
                .into_iter()
                .flatten(),
        );
        for player in players {
            if player.skeleton.is_empty() {
                continue;
            }
            let renderer = renderer.clone();
            let model_name = player.model_name.clone();
            let skeleton = player.skeleton.clone();
            let (visible, invisible) = match self.config.player.draw_model {
                DrawMode::None => unreachable!(),
                DrawMode::Color => (
                    self.config.player.model_visible_color,
                    self.config.player.model_invisible_color,
                ),
                DrawMode::Health => (
                    self.health_color(
                        player.health,
                        player.max_health,
                        self.config.player.model_visible_color.a(),
                    ),
                    self.health_color(
                        player.health,
                        player.max_health,
                        self.config.player.model_invisible_color.a(),
                    ),
                ),
            };
            let mode = match self.config.player.model_mode {
                ModelRenderMode::Filled => model::ModelRenderMode::Filled,
                ModelRenderMode::Wireframe => model::ModelRenderMode::Wireframe,
            };
            let callback = CallbackFn::new(move |info, painter| {
                let viewport = info.viewport_in_pixels();
                renderer.render(
                    painter.gl(),
                    model::ModelRenderParams {
                        model_name: &model_name,
                        skeleton: &skeleton,
                        viewport: (
                            viewport.left_px,
                            viewport.from_bottom_px,
                            viewport.width_px,
                            viewport.height_px,
                        ),
                        view: &view,
                        model: &model::model_matrix(),
                        visible_color: visible.to_normalized_gamma_f32(),
                        invisible_color: invisible.to_normalized_gamma_f32(),
                        mode,
                    },
                );
            });
            painter.add(PaintCallback {
                rect: Rect::from_min_size(Pos2::ZERO, egui::vec2(window.x, window.y)),
                callback: Arc::new(callback),
            });
        }
    }

    fn grenade_manager(&self, data: &Data, painter: &Painter) {
        let position = data.local_player.position;
        let map = &data.map_name;

        let Some(grenades) = self.grenades.get(map) else {
            return;
        };

        let player_weapon = &data.local_player.weapon;
        
        let nearest_grenade = grenades
            .iter()
            .filter(|g| g.weapon == *player_weapon)
            .map(|g| (g, (position - g.position).length()))
            .filter(|(_, dist)| *dist <= 1000.0)
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((grenade, distance)) = nearest_grenade {
            // Draw standing ring on ground
            self.grenade_circle(data, grenade, painter);

            // Draw line from player to standing spot if not standing on it
            if distance > 5.0 {
                if let (Some(player_screen), Some(spot_screen)) = (
                    world_to_screen(&data.local_player.position, data),
                    world_to_screen(&grenade.position, data),
                ) {
                    let stroke = Stroke::new(2.5, Color32::from_rgb(0, 229, 255));
                    painter.line_segment([player_screen, spot_screen], stroke);
                }
            }

            // Always show aim target reticle regardless of distance when holding matching grenade
            self.grenade_indicator(data, grenade, painter);

            // HUD Guidance Box for Movement & Aim Alignment
            self.draw_grenade_guidance_hud(data, grenade, painter, distance);
        }
    }

    fn draw_grenade_guidance_hud(
        &self,
        data: &Data,
        grenade: &Grenade,
        painter: &Painter,
        distance: f32,
    ) {
        let screen_center = pos2(data.window_size.x / 2.0, data.window_size.y * 0.76);

        // Calculate relative direction to standing position
        let delta_world = grenade.position - data.local_player.position;
        let yaw_rad = data.view_angles.y.to_radians();

        let forward_dir = vec3(yaw_rad.cos(), yaw_rad.sin(), 0.0);
        let right_dir = vec3(-yaw_rad.sin(), yaw_rad.cos(), 0.0);

        let forward_dist = delta_world.dot(forward_dir);
        let right_dist = delta_world.dot(right_dir);

        // Angle differences
        let mut yaw_diff = grenade.view_angles.y - data.view_angles.y;
        while yaw_diff > 180.0 {
            yaw_diff -= 360.0;
        }
        while yaw_diff < -180.0 {
            yaw_diff += 360.0;
        }
        let pitch_diff = grenade.view_angles.x - data.view_angles.x;

        let in_position = distance <= 5.0;
        let aim_aligned = pitch_diff.abs() < 0.6 && yaw_diff.abs() < 0.6;

        let box_width = 340.0;
        let box_height = 100.0;
        let bg_rect = egui::Rect::from_center_size(screen_center, egui::vec2(box_width, box_height));

        let border_color = if in_position && aim_aligned {
            Color32::from_rgb(0, 255, 157)
        } else if in_position {
            Color32::from_rgb(255, 200, 0)
        } else {
            Color32::from_rgb(0, 229, 255)
        };

        painter.rect(
            bg_rect,
            8.0,
            Color32::from_rgba_unmultiplied(10, 15, 25, 220),
            Stroke::new(2.0, border_color),
            egui::StrokeKind::Middle,
        );

        let mut text_y = bg_rect.min.y + 10.0;

        self.text(
            painter,
            format!("🎯 Lineup: {} ({})", grenade.name, grenade.weapon),
            pos2(screen_center.x, text_y),
            Align2::CENTER_TOP,
            Some(Color32::WHITE),
        );
        text_y += 20.0;

        let pos_str = if in_position {
            "📍 Position: IN POSITION ✓".to_string()
        } else {
            let fwd_str = if forward_dist > 1.0 {
                format!("Forward {:.1}u", forward_dist)
            } else if forward_dist < -1.0 {
                format!("Back {:.1}u", -forward_dist)
            } else {
                "".to_string()
            };

            let side_str = if right_dist > 1.0 {
                format!("Right {:.1}u", right_dist)
            } else if right_dist < -1.0 {
                format!("Left {:.1}u", -right_dist)
            } else {
                "".to_string()
            };

            format!("🚶 Move: {} {} ({:.1}u)", fwd_str, side_str, distance).trim().to_string()
        };

        let pos_color = if in_position {
            Color32::GREEN
        } else {
            Color32::from_rgb(0, 229, 255)
        };
        self.text(
            painter,
            pos_str,
            pos2(screen_center.x, text_y),
            Align2::CENTER_TOP,
            Some(pos_color),
        );
        text_y += 20.0;

        let aim_str = if aim_aligned {
            "🎯 Aim: PERFECT ALIGNMENT ✓".to_string()
        } else {
            let p_str = if pitch_diff > 0.3 {
                format!("DOWN {:.1}°", pitch_diff)
            } else if pitch_diff < -0.3 {
                format!("UP {:.1}°", -pitch_diff)
            } else {
                "".to_string()
            };

            let y_str = if yaw_diff > 0.3 {
                format!("RIGHT {:.1}°", yaw_diff)
            } else if yaw_diff < -0.3 {
                format!("LEFT {:.1}°", -yaw_diff)
            } else {
                "".to_string()
            };

            format!("👀 Aim: {} {}", p_str, y_str).trim().to_string()
        };

        let aim_color = if aim_aligned {
            Color32::GREEN
        } else {
            Color32::LIGHT_YELLOW
        };
        self.text(
            painter,
            aim_str,
            pos2(screen_center.x, text_y),
            Align2::CENTER_TOP,
            Some(aim_color),
        );
        text_y += 20.0;

        if in_position && aim_aligned {
            let throw_mod = match (
                grenade.modifiers.duck,
                grenade.modifiers.jump,
                grenade.modifiers.run,
            ) {
                (false, false, false) => "NORMAL THROW",
                (true, false, false) => "DUCK THROW",
                (false, true, false) => "JUMP THROW",
                (true, true, false) => "DUCK + JUMP THROW",
                (false, true, true) => "JUMP + RUN THROW",
                _ => "SPECIAL THROW",
            };
            self.text(
                painter,
                format!("🔥 READY! RELEASE [{}] NOW!", throw_mod),
                pos2(screen_center.x, text_y),
                Align2::CENTER_TOP,
                Some(Color32::from_rgb(0, 255, 157)),
            );
        }
    }

    fn grenade_circle(&self, data: &Data, grenade: &Grenade, painter: &Painter) {
        let Some(center_screen) = world_to_screen(&grenade.position, data) else {
            return;
        };
        let center = &grenade.position;

        // player hitbox width and length
        const WIDTH: f32 = 24.0;
        const HALF_WIDTH: f32 = WIDTH / 2.0;

        const V1: Vec3 = vec3(WIDTH, HALF_WIDTH, 0.0);
        const V2: Vec3 = vec3(HALF_WIDTH, WIDTH, 0.0);
        const V3: Vec3 = vec3(-HALF_WIDTH, WIDTH, 0.0);
        const V4: Vec3 = vec3(-WIDTH, HALF_WIDTH, 0.0);

        let points: Vec<Pos2> = [
            center + V1,
            center + V2,
            center + V3,
            center + V4,
            center - V1,
            center - V2,
            center - V3,
            center - V4,
        ]
        .iter()
        .filter_map(|p| world_to_screen(p, data))
        .collect();

        let shape = Shape::convex_polygon(
            points,
            Color32::from_rgba_unmultiplied(0, 255, 0, 127),
            Stroke::NONE,
        );
        painter.add(shape);

        painter.circle_filled(
            center_screen,
            WIDTH / 8.0,
            Color32::from_rgba_unmultiplied(255, 0, 0, 127),
        );
    }

    fn grenade_indicator(&self, data: &Data, grenade: &Grenade, painter: &Painter) {
        let position = grenade.position + (data.local_player.head - data.local_player.position);
        let view_angles = grenade.view_angles;

        let pitch = view_angles.x.to_radians();
        let yaw = view_angles.y.to_radians();

        let forward = vec3(
            pitch.cos() * yaw.cos(),
            pitch.cos() * yaw.sin(),
            -pitch.sin(),
        )
        .normalize();

        const CROSS_DISTANCE: f32 = 1000.0;
        let center = position + forward * CROSS_DISTANCE;

        const WORLD_UP: Vec3 = vec3(0.0, 0.0, 1.0);
        const CROSS_SIZE: f32 = 24.0;

        let right = forward.cross(WORLD_UP).normalize();
        let up = right.cross(forward).normalize();

        let v1 = center + right * CROSS_SIZE + up * CROSS_SIZE;
        let v2 = center + right * CROSS_SIZE - up * CROSS_SIZE;
        let v3 = center - right * CROSS_SIZE + up * CROSS_SIZE;
        let v4 = center - right * CROSS_SIZE - up * CROSS_SIZE;

        let stroke = Stroke::new(
            self.config.hud.line_width,
            self.config.hud.overlay_text.grenade_lineup.color,
        );
        let stroke_bg = Stroke::new(self.config.hud.line_width * 2.0, Color32::BLACK);

        let Some(v1) = world_to_screen(&v1, data) else {
            return;
        };
        let Some(v2) = world_to_screen(&v2, data) else {
            return;
        };
        let Some(v3) = world_to_screen(&v3, data) else {
            return;
        };
        let Some(v4) = world_to_screen(&v4, data) else {
            return;
        };

        painter.line_segment([v1, v4], stroke_bg);
        painter.line_segment([v2, v3], stroke_bg);

        painter.line_segment([v1, v4], stroke);
        painter.line_segment([v2, v3], stroke);

        let text_center = center - up * CROSS_SIZE;
        if let Some(screen) = world_to_screen(&text_center, data) {
            let cat = &self.config.hud.overlay_text.grenade_lineup;
            let anchor = hud::point_anchor(screen, cat.position, cat.font_size * 0.3);
            let align = cat.align.to_align2();
            self.text_sized(
                painter,
                &grenade.name,
                anchor,
                align,
                cat.color,
                cat.font_size,
            );
            let mut offset = cat.font_size;
            self.text_sized(
                painter,
                format!("{}", grenade.weapon),
                anchor + egui::vec2(0.0, offset),
                align,
                cat.color,
                cat.font_size,
            );
            offset += cat.font_size;
            let text = match (
                grenade.modifiers.duck,
                grenade.modifiers.jump,
                grenade.modifiers.run,
            ) {
                (false, false, false) => "",
                (true, false, false) => "Duck",
                (true, true, false) => "Duck/Jump",
                (true, true, true) => "Duck/Jump/Run",
                (true, false, true) => "Duck/Run",
                (false, true, false) => "Jump",
                (false, true, true) => "Jump/Run",
                (false, false, true) => "Run",
            };
            if !text.is_empty() {
                self.text_sized(
                    painter,
                    text,
                    anchor + egui::vec2(0.0, offset),
                    align,
                    cat.color,
                    cat.font_size,
                );
                offset += cat.font_size;
            }
            if !grenade.description.is_empty() {
                self.text_sized(
                    painter,
                    &grenade.description,
                    anchor + egui::vec2(0.0, offset),
                    align,
                    cat.color,
                    cat.font_size,
                );
            }
        }
    }

    fn health_color(&self, health: i32, max_health: i32, alpha: u8) -> Color32 {
        let max_health = max_health.max(1);
        let health = health.clamp(0, max_health);
        let percent = health as f32 / max_health as f32;

        let (r, g) = if percent <= 0.5 {
            let factor = percent * 2.0;
            (255, (255.0 * factor) as u8)
        } else {
            let factor = 1.0 - (percent - 0.5) * 2.0;
            ((255.0 * factor) as u8, 255)
        };

        Color32::from_rgba_unmultiplied(r, g, 0, alpha)
    }

    fn text_sized(
        &self,
        painter: &Painter,
        text: impl AsRef<str>,
        position: Pos2,
        align: Align2,
        color: Color32,
        font_size: f32,
    ) {
        use egui::FontId;

        let font = FontId::proportional(font_size);
        if self.config.hud.text_outline {
            for (pos, color) in outline(position, color) {
                painter.text(pos, align, text.as_ref(), font.clone(), color);
            }
        } else {
            painter.text(position, align, text.as_ref(), font, color);
        }
    }

    fn text(
        &self,
        painter: &Painter,
        text: impl AsRef<str>,
        position: Pos2,
        align: Align2,
        color: impl Into<Option<Color32>>,
    ) {
        let color = color.into().unwrap_or(self.config.accent_color);
        self.text_sized(
            painter,
            text,
            position,
            align,
            color,
            16.0,
        );
    }
}

const OUTLINE_WIDTH: f32 = 2.0;
fn outline(pos: Pos2, color: Color32) -> [(Pos2, Color32); 2] {
    let outline_color = Color32::from_rgba_unmultiplied(0, 0, 0, color.a());
    [
        (
            pos2(pos.x + OUTLINE_WIDTH, pos.y + OUTLINE_WIDTH),
            outline_color,
        ),
        (pos, color),
    ]
}

fn convex_hull(points: &[Vec3]) -> Vec<Vec3> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    let mut sorted_points: Vec<Vec3> = points.iter().filter(|p| !p.is_nan()).copied().collect();
    sorted_points.sort_by(|a, b| {
        a.x.partial_cmp(&b.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
    });

    let mut deduped: Vec<Vec3> = Vec::with_capacity(sorted_points.len());
    for point in sorted_points {
        let is_duplicate = deduped
            .last()
            .is_some_and(|last| point.x == last.x && point.y == last.y);
        if !is_duplicate {
            deduped.push(point);
        }
    }

    if deduped.len() <= 2 {
        return deduped;
    }

    let mut lower = Vec::new();
    for point in &deduped {
        while lower.len() >= 2
            && cross(&lower[lower.len() - 2], &lower[lower.len() - 1], point) <= 0.0
        {
            lower.pop();
        }
        lower.push(*point);
    }

    let mut upper = Vec::new();
    for point in deduped.iter().rev() {
        while upper.len() >= 2
            && cross(&upper[upper.len() - 2], &upper[upper.len() - 1], point) <= 0.0
        {
            upper.pop();
        }
        upper.push(*point);
    }

    upper.pop();
    lower.pop();

    lower.append(&mut upper);
    lower
}

fn cross(o: &Vec3, a: &Vec3, b: &Vec3) -> f32 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}
