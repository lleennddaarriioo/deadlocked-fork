use egui::{DragValue, Ui};
use shared::Bones;
use strum::IntoEnumIterator as _;

use crate::ui::{
    app::AppState,
    drag_range::DragRange,
    gui::helpers::{checkbox, checkbox_hover, collapsing_open, combo_box, drag, keybind, scroll},
};
use egui_plot::{Line, Plot, PlotPoints, Points, VLine};

#[derive(PartialEq, Default)]
pub enum AimbotTab {
    #[default]
    Global,
    Weapon,
    Flick,
}

impl AppState {
    pub fn aimbot_settings(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.aimbot_tab, AimbotTab::Global, "Global");
            ui.selectable_value(&mut self.aimbot_tab, AimbotTab::Weapon, "Weapon");
            ui.selectable_value(&mut self.aimbot_tab, AimbotTab::Flick, "Flick");
            if self.aimbot_tab == AimbotTab::Weapon {
                combo_box(ui, "aimbot_weapon", "Weapon", &mut self.aimbot_weapon);
            }
            ui.separator();
            ui.toggle_value(&mut self.aimbot_show_curve, "Curve Editor");
        });
        ui.separator();
        
        if self.aimbot_show_curve {
            if self.aimbot_tab == AimbotTab::Flick {
                self.aimbot_curve_editor(ui, true);
            } else {
                self.aimbot_curve_editor(ui, false);
            }
        } else if self.aimbot_tab == AimbotTab::Flick {
            self.aimbot_flick_tab(ui);
        } else {
            ui.columns(2, |cols| {
                let left = &mut cols[0];
                scroll(left, "aimbot_left", |ui| self.aimbot_left(ui));

                let right = &mut cols[1];
                scroll(right, "aimbot_right", |ui| self.aimbot_right(ui));
            });
        }
    }

    fn aimbot_flick_tab(&mut self, ui: &mut Ui) {
        scroll(ui, "aimbot_flick", |ui| {
            collapsing_open(ui, "Flick Mode Configuration", |ui| {
                if checkbox_hover(
                    ui,
                    "Enable Flick Mode",
                    "Rapidly swipe through the target instead of smoothly tracking",
                    &mut self.weapon_config().aimbot.flick_mode,
                ) {
                    self.send_config_game();
                }

                if drag(
                    ui,
                    "Flick Speed",
                    DragValue::new(&mut self.weapon_config().aimbot.flick_speed)
                        .range(0.1..=100.0)
                        .speed(0.1)
                        .max_decimals(2),
                ) {
                    self.send_config_game();
                }
                
                ui.label(egui::RichText::new("Note: When Flick mode is enabled, the smoothing curve is ignored and the aimbot will swipe past the target.").italics().color(egui::Color32::from_rgb(150, 150, 150)));
            });
        });
    }

    fn aimbot_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Aimbot", |ui| {
            if keybind(
                ui,
                "aimbot_hotkey",
                "Hotkey",
                &mut self.config.aim.aimbot_hotkey,
            ) {
                self.send_config_game();
            }

            if self.aimbot_tab == AimbotTab::Weapon
                && checkbox_hover(
                    ui,
                    "Enable Override",
                    "Enable aimbot settings override for a specific weapon",
                    &mut self.weapon_config().aimbot.enable_override,
                )
            {
                self.send_config_game();
            }

            if checkbox(
                ui,
                "Enable Aimbot",
                &mut self.weapon_config().aimbot.enabled,
            ) {
                self.send_config_game();
            }

            if combo_box(
                ui,
                "aimbot_mode",
                "Mode",
                &mut self.weapon_config().aimbot.mode,
            ) {
                self.send_config_game();
            }
        });

        ui.collapsing("Targeting", |ui| {
            if checkbox(
                ui,
                "Target Friendlies",
                &mut self.weapon_config().aimbot.target_friendlies,
            ) {
                self.send_config_game();
            }

            if checkbox_hover(
                ui,
                "Distance-Adjusted FOV",
                "Adjusts FOV based on target distance",
                &mut self.weapon_config().aimbot.distance_adjusted_fov,
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "FOV",
                DragValue::new(&mut self.weapon_config().aimbot.fov)
                    .range(0.1..=360.0)
                    .suffix("°")
                    .speed(0.02)
                    .max_decimals(1),
            ) {
                self.send_config_game();
            }

            if checkbox_hover(
                ui,
                "Silent Aim",
                "Snaps to the target then immediately snaps back",
                &mut self.weapon_config().aimbot.silent_aim,
            ) {
                self.send_config();
            }

            if drag(
                ui,
                "Inertia",
                DragValue::new(&mut self.weapon_config().aimbot.inertia)
                    .range(0.0..=1.0)
                    .speed(0.005)
                    .max_decimals(2),
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Prediction",
                DragValue::new(&mut self.weapon_config().aimbot.prediction_time)
                    .range(0.0..=0.25)
                    .suffix(" s")
                    .speed(0.002)
                    .max_decimals(2),
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Start Bullet",
                DragValue::new(&mut self.weapon_config().aimbot.start_bullet)
                    .range(0..=10)
                    .speed(0.05),
            ) {
                self.send_config_game();
            }

            if combo_box(
                ui,
                "targeting_mode",
                "Targeting Mode",
                &mut self.weapon_config().aimbot.targeting_mode,
            ) {
                self.send_config_game();
            }

            if combo_box(
                ui,
                "bone_mode",
                "Bone Mode",
                &mut self.weapon_config().aimbot.bone_mode,
            ) {
                self.send_config();
            }
        });

        ui.collapsing("Checks", |ui| {
            if checkbox(
                ui,
                "Visibility Check",
                &mut self.weapon_config().aimbot.visibility_check,
            ) {
                self.send_config_game();
            }


            if drag(
                ui,
                "Damage Threshold",
                DragValue::new(&mut self.weapon_config().aimbot.damage_threshold)
                    .range(0.0..=120.0)
                    .speed(0.5)
                    .suffix(" HP"),
            ) {
                self.send_config();
            }

            if checkbox(
                ui,
                "Flash Check",
                &mut self.weapon_config().aimbot.flash_check,
            ) {
                self.send_config_game();
            }
        });

        ui.collapsing("Bones", |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                for bone in Bones::iter() {
                    let index = self
                        .weapon_config()
                        .aimbot
                        .bones
                        .iter()
                        .position(|b| *b == bone);
                    
                    let text = if let Some(idx) = index {
                        format!("[{}] {:?}", idx + 1, bone)
                    } else {
                        format!("{:?}", bone)
                    };

                    if ui.selectable_label(index.is_some(), text).clicked() {
                        if let Some(index) = index {
                            self.weapon_config().aimbot.bones.remove(index);
                        } else {
                            self.weapon_config().aimbot.bones.push(bone);
                        }
                        self.send_config_game();
                    }
                }
            });

            ui.add_space(4.0);
            if ui.button("Clear All").clicked() {
                self.weapon_config().aimbot.bones.clear();
                self.send_config();
            }
        });
    }

    fn aimbot_right(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Triggerbot", |ui| {
            if self.aimbot_tab == AimbotTab::Weapon
                && checkbox(
                    ui,
                    "Enable Override",
                    &mut self.weapon_config().triggerbot.enable_override,
                )
            {
                self.send_config_game();
            }

            if checkbox(
                ui,
                "Enable Triggerbot",
                &mut self.weapon_config().triggerbot.enabled,
            ) {
                self.send_config_game();
            }

            if keybind(
                ui,
                "triggerbot_hotkey",
                "Hotkey 1",
                &mut self.config.aim.triggerbot_hotkey,
            ) {
                self.send_config_game();
            }

            if keybind(
                ui,
                "triggerbot_hotkey2",
                "Hotkey 2",
                &mut self.config.aim.triggerbot_hotkey2,
            ) {
                self.send_config();
            }

            if checkbox(
                ui,
                "Auto Pistol (Uses Triggerbot Hotkey)",
                &mut self.config.aim.auto_pistol,
            ) {
                self.send_config();
            }

            if ui
                .add(DragRange::new(
                    "Delay (ms)",
                    &mut self.weapon_config().triggerbot.delay,
                    0..=999,
                ))
                .changed()
            {
                self.send_config_game();
            }

            if combo_box(
                ui,
                "triggerbot_mode",
                "Mode",
                &mut self.weapon_config().triggerbot.mode,
            ) {
                self.send_config_game();
            }

            if checkbox(
                ui,
                "Head Only",
                &mut self.weapon_config().triggerbot.head_only,
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Hit Chance",
                DragValue::new(&mut self.weapon_config().triggerbot.hit_chance)
                    .range(0.0..=1.0)
                    .speed(0.01),
            ) {
                self.send_config();
            }

            if drag(
                ui,
                "Hold Duration (ms)",
                DragValue::new(&mut self.weapon_config().triggerbot.shot_duration)
                    .range(0..=2000)
                    .speed(10.0),
            ) {
                self.send_config_game();
            }
        });

        ui.collapsing("Checks\u{200b}", |ui| {
            if checkbox_hover(
                ui,
                "Only Shoot When Aimbot Locked",
                "Only shoot if Aimbot is actively locked onto target within lock threshold",
                &mut self.weapon_config().triggerbot.aimbot_lock_only,
            ) {
                self.send_config();
            }

            if drag(
                ui,
                "Lock FOV Threshold",
                DragValue::new(&mut self.weapon_config().triggerbot.lock_fov_threshold)
                    .range(0.1..=10.0)
                    .speed(0.1)
                    .suffix("°"),
            ) {
                self.send_config();
            }

            if drag(
                ui,
                "Damage Threshold",
                DragValue::new(&mut self.weapon_config().triggerbot.damage_threshold)
                    .range(0.0..=120.0)
                    .speed(0.5)
                    .suffix(" HP"),
            ) {
                self.send_config();
            }

            if checkbox(
                ui,
                "Visibility Check",
                &mut self.weapon_config().triggerbot.visibility_check,
            ) {
                self.send_config();
            }


            if checkbox(
                ui,
                "Flash Check",
                &mut self.weapon_config().triggerbot.flash_check,
            ) {
                self.send_config_game();
            }

            if checkbox(
                ui,
                "Scope Check",
                &mut self.weapon_config().triggerbot.scope_check,
            ) {
                self.send_config_game();
            }

            if checkbox_hover(
                ui,
                "Velocity Check",
                "Only shoot if the player moves slower than the specified threshold",
                &mut self.weapon_config().triggerbot.velocity_check,
            ) {
                self.send_config_game();
            }

            if drag(
                ui,
                "Velocity Threshold",
                DragValue::new(&mut self.weapon_config().triggerbot.velocity_threshold)
                    .range(0..=5000),
            ) {
                self.send_config_game();
            }
        });

        collapsing_open(ui, "RCS", |ui| {
            if self.aimbot_tab == AimbotTab::Weapon
                && checkbox(
                    ui,
                    "Enable Override",
                    &mut self.weapon_config().rcs.enable_override,
                )
            {
                self.send_config_game();
            }

            if checkbox(ui, "Enable RCS", &mut self.weapon_config().rcs.enabled) {
                self.send_config_game();
            }

            if ui
                .horizontal(|ui| {
                    let rcs = &mut self.weapon_config().rcs;
                    let x = ui.add(
                        DragValue::new(&mut rcs.strength.x)
                            .prefix("X: ")
                            .range(0.0..=1.0)
                            .speed(0.01),
                    );
                    let y = ui.add(
                        DragValue::new(&mut rcs.strength.y)
                            .prefix("Y: ")
                            .range(0.0..=1.0)
                            .speed(0.01),
                    );
                    ui.label("Strength");
                    (x | y).changed()
                })
                .inner
            {
                self.send_config_game();
            }
        });
    }

    fn aimbot_curve_editor(&mut self, ui: &mut Ui, is_flick: bool) {
        ui.vertical_centered(|ui| {
            if is_flick {
                ui.heading("Flick Speed Curve Editor");
                ui.label("Map Distance to Target (FOV) ➔ Flick Divisor Speed");
            } else {
                ui.heading("Aimbot Smoothing Curve Editor");
                ui.label("Map Distance to Target (FOV) ➔ Smoothing Ticks");
            }
        });
        ui.add_space(10.0);

        let curve = if is_flick {
            &mut self.weapon_config().aimbot.flick_curve
        } else {
            &mut self.weapon_config().aimbot.curve
        };
        
        // Ensure sorted by X
        curve.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));

        let plot = Plot::new("curve_plot")
            .view_aspect(2.0)
            .include_y(0.0)
            .include_x(0.0)
            .allow_drag(false)
            .allow_zoom(false)
            .allow_scroll(false);

        let mut dragged_pos = None;
        
        let response = plot.show(ui, |plot_ui| {
            let points: Vec<[f64; 2]> = curve.iter().map(|p| [p.x as f64, p.y as f64]).collect();
            
            let line = Line::new("Smoothing Curve", PlotPoints::new(points.clone()));
            plot_ui.line(line);
            
            let colors = [
                egui::Color32::RED,
                egui::Color32::GREEN,
                egui::Color32::BLUE,
                egui::Color32::YELLOW,
                egui::Color32::CYAN,
                egui::Color32::MAGENTA,
                egui::Color32::ORANGE,
            ];
            
            for (i, p) in curve.iter().enumerate() {
                let color = colors[i % colors.len()];
                plot_ui.vline(VLine::new(format!("FOV {}", i + 1), p.x as f64).color(color));
            }

            let p_points = Points::new("Control Points", PlotPoints::new(points)).radius(5.0);
            plot_ui.points(p_points);

            if plot_ui.response().dragged() {
                if let Some(pos) = plot_ui.pointer_coordinate() {
                    dragged_pos = Some(egui::vec2(pos.x as f32, pos.y as f32));
                }
            }
        });

        let mut changed = false;

        // Simple drag interaction: move the closest point
        if let Some(pos) = dragged_pos {
            let pos_x = pos.x.clamp(0.0, 180.0);
            let pos_y = pos.y.max(1.0);
            
            let mut min_dist_sq = f32::MAX;
            let mut closest_idx = None;
            for (i, p) in curve.iter().enumerate() {
                // Normalize the distances for interaction
                let dx = p.x - pos_x;
                let dy = (p.y - pos_y) / 10.0;
                let dist = dx * dx + dy * dy;
                if dist < min_dist_sq {
                    min_dist_sq = dist;
                    closest_idx = Some(i);
                }
            }
            
            if min_dist_sq < 4.0 {
                // We grabbed an existing point
                if let Some(i) = closest_idx {
                    let min_x = if i > 0 { curve[i-1].x } else { 0.0 };
                    let max_x = if i + 1 < curve.len() { curve[i+1].x } else { 180.0 };
                    
                    curve[i].x = pos_x.clamp(min_x, max_x);
                    curve[i].y = pos_y;
                    changed = true;
                }
            } else {
                // Freehand draw
                // If we are far enough from the closest point's X to prevent point spam
                if curve.iter().all(|p| (p.x - pos_x).abs() > 0.5) {
                    curve.push(glam::Vec2::new(pos_x, pos_y));
                    curve.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
                    changed = true;
                } else if let Some(i) = closest_idx {
                    // Update Y of the closest point if we're just swiping over it vertically
                    curve[i].y = pos_y;
                    changed = true;
                }
            }
        }

        ui.add_space(10.0);
        ui.separator();
        
        ui.horizontal(|ui| {
            if ui.button("Add Point").clicked() {
                if let Some(last) = curve.last() {
                    curve.push(glam::Vec2::new(last.x + 0.5, last.y));
                } else {
                    curve.push(glam::Vec2::new(0.0, 50.0));
                }
                changed = true;
            }
            if ui.button("Remove Point").clicked() {
                if curve.len() > 2 { // Keep at least 2 points
                    curve.pop();
                    changed = true;
                }
            }
        });

        ui.add_space(5.0);
        
        // Manual editors
        scroll(ui, "curve_scroll", |ui| {
            let mut i = 0;
            while i < curve.len() {
                ui.horizontal(|ui| {
                    ui.label(format!("Point {}", i + 1));
                    
                    let min_x = if i > 0 { curve[i-1].x } else { 0.0 };
                    let max_x = if i + 1 < curve.len() { curve[i+1].x } else { 180.0 };
                    
                    let x_changed = drag(ui, "FOV Dist", DragValue::new(&mut curve[i].x).range(min_x..=max_x).speed(0.1));
                    let y_changed = drag(ui, if is_flick { "Speed" } else { "Ticks" }, DragValue::new(&mut curve[i].y).range(1.0..=1000.0).speed(1.0));
                    
                    if x_changed || y_changed {
                        changed = true;
                    }
                });
                i += 1;
            }
        });

        if changed {
            self.send_config_game();
        }
    }
}
