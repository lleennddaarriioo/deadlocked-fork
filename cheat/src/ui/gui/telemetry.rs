use egui::{Color32, DragValue, Pos2, Stroke, Ui, Vec2};
use std::time::{Duration, Instant};

use crate::game::TELEMETRY;
use crate::ui::app::App;
use crate::ui::gui::helpers::scroll;

impl App {
    pub fn telemetry_settings(&mut self, ui: &mut Ui) {
        scroll(ui, "telemetry_tab_scroll", |ui| {
            ui.heading("Performance & Thread Telemetry");
            ui.separator();
            ui.add_space(4.0);

        // Compute actual FPS & TPS from recorded frame times
        let actual_tps = if self.frame_times.is_empty() {
            0.0f32
        } else {
            let avg_ms = self.frame_times.iter().sum::<Duration>().as_secs_f32() * 1000.0 / self.frame_times.len() as f32;
            if avg_ms > 0.0 { 1000.0 / avg_ms } else { 0.0 }
        };

        // Smooth telemetry updates at configurable pie chart refresh Hz
        let graph_refresh_hz = self.config.graph_refresh_hz.max(1);
        let update_interval = Duration::from_secs_f32(1.0 / graph_refresh_hz as f32);
        if self.smoothed_telemetry.is_none() || self.last_pie_chart_update.elapsed() >= update_interval {
            if let Ok(raw_t) = TELEMETRY.lock() {
                let alpha = 0.35f32; // Smoothing factor for pie chart values
                let mut current = self.smoothed_telemetry.unwrap_or(*raw_t);
                current.cache_entities_ms = current.cache_entities_ms * (1.0 - alpha) + raw_t.cache_entities_ms * alpha;
                current.check_bvh_ms = current.check_bvh_ms * (1.0 - alpha) + raw_t.check_bvh_ms * alpha;
                current.aimbot_ms = current.aimbot_ms * (1.0 - alpha) + raw_t.aimbot_ms * alpha;
                current.rcs_ms = current.rcs_ms * (1.0 - alpha) + raw_t.rcs_ms * alpha;
                current.triggerbot_ms = current.triggerbot_ms * (1.0 - alpha) + raw_t.triggerbot_ms * alpha;
                current.bhop_ms = current.bhop_ms * (1.0 - alpha) + raw_t.bhop_ms * alpha;
                current.counter_strafe_ms = current.counter_strafe_ms * (1.0 - alpha) + raw_t.counter_strafe_ms * alpha;
                current.input_update_ms = current.input_update_ms * (1.0 - alpha) + raw_t.input_update_ms * alpha;
                current.draw_data_ms = current.draw_data_ms * (1.0 - alpha) + raw_t.draw_data_ms * alpha;
                current.other_features_ms = current.other_features_ms * (1.0 - alpha) + raw_t.other_features_ms * alpha;
                current.idle_ms = current.idle_ms * (1.0 - alpha) + raw_t.idle_ms * alpha;
                current.target_frame_ms = raw_t.target_frame_ms;
                current.total_loop_ms = raw_t.total_loop_ms;
                self.smoothed_telemetry = Some(current);
                self.last_pie_chart_update = Instant::now();
            }
        }

        let t = match self.smoothed_telemetry {
            Some(t) => t,
            None => {
                ui.label("Waiting for telemetry data...");
                return;
            }
        };

        // Master Global TPS & Control Toolbar
        ui.horizontal(|ui| {
            if ui.add(DragValue::new(&mut self.config.fps).range(1..=10000).speed(5.0).prefix("Global Master TPS: ")).changed() {
                let g_val = self.config.fps;
                self.config.bhop_tps = g_val;
                self.config.aimbot_tps = g_val;
                self.config.trigger_tps = g_val;
                self.config.bvh_tps = g_val;
                self.config.input_tps = g_val;
                self.config.bone_vis_hz = g_val;
                self.config.cache_hz = g_val.min(500);
                self.send_config();
            }
            ui.add_space(12.0);
            if ui.add(DragValue::new(&mut self.config.graph_refresh_hz).range(1..=240).speed(1.0).prefix("Pie Refresh Hz: ")).changed() {
                self.send_config();
            }
            ui.add_space(12.0);
            if ui.button("🔄 Sync All Rates").clicked() {
                let g_val = self.config.fps;
                self.config.bhop_tps = g_val;
                self.config.aimbot_tps = g_val;
                self.config.trigger_tps = g_val;
                self.config.bvh_tps = g_val;
                self.config.input_tps = g_val;
                self.config.bone_vis_hz = g_val;
                self.config.cache_hz = g_val.min(500);
                self.send_config();
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("⚡ Monitor VSync: {} Hz", self.max_monitor_hz))
                    .strong()
                    .color(Color32::from_rgb(168, 85, 247)),
            );
            ui.add_space(12.0);
            ui.label(
                egui::RichText::new(format!("📊 Actual Loop TPS: {:.1}", actual_tps))
                    .strong()
                    .color(Color32::from_rgb(56, 189, 248)),
            );
            ui.add_space(12.0);
            let num_rayon_threads = rayon::current_num_threads();
            ui.label(
                egui::RichText::new(format!("🧵 Rayon Pool: {} Workers", num_rayon_threads))
                    .strong()
                    .color(Color32::from_rgb(34, 197, 94)),
            );
        });

        ui.add_space(6.0);

        // Record history for thread execution graphs
        push_history(&mut self.thread_history_bhop, t.bhop_ms + t.counter_strafe_ms);
        push_history(&mut self.thread_history_aimbot, t.aimbot_ms + t.rcs_ms);
        push_history(&mut self.thread_history_trigger, t.triggerbot_ms);
        push_history(&mut self.thread_history_input, t.input_update_ms);
        push_history(&mut self.thread_history_bvh, t.check_bvh_ms);
        push_history(&mut self.thread_history_cache, t.cache_entities_ms);
        push_history(&mut self.thread_history_gui, t.draw_data_ms);
        push_history(&mut self.thread_history_other, t.other_features_ms);
        push_history(&mut self.thread_history_loop, t.total_loop_ms);

        ui.add_space(4.0);

        // Real-time per-thread graph section
        egui::CollapsingHeader::new("📈 Real-Time Execution Graphs Per Sub-System Thread")
            .default_open(true)
            .show(ui, |ui| {
                let threads_graphs: &[(&str, &str, &std::collections::VecDeque<f32>, Color32)] = &[
                    ("Thread 1: Bhop & Counter Strafe", "Movement Thread", &self.thread_history_bhop, Color32::from_rgb(234, 179, 8)),
                    ("Thread 2: Aimbot & RCS", "Aimbot Thread", &self.thread_history_aimbot, Color32::from_rgb(239, 68, 68)),
                    ("Thread 3: Triggerbot", "Triggerbot Thread", &self.thread_history_trigger, Color32::from_rgb(249, 115, 22)),
                    ("Thread 4: Input Polling", "Device Polling Thread", &self.thread_history_input, Color32::from_rgb(34, 197, 94)),
                    ("Thread 5: BVH Raycast Engine", "Raycast Thread", &self.thread_history_bvh, Color32::from_rgb(56, 189, 248)),
                    ("Thread 6: Entity Cache Scanner", "Entity Scanner", &self.thread_history_cache, Color32::from_rgb(168, 85, 247)),
                    ("Thread 7: GUI & Draw Data", "Overlay Render Thread", &self.thread_history_gui, Color32::from_rgb(168, 162, 158)),
                    ("Thread 8: Other Features & ESP", "Misc Features", &self.thread_history_other, Color32::from_rgb(99, 102, 241)),
                    ("Main Thread: Total Loop Duration", "Game Loop Master", &self.thread_history_loop, Color32::from_rgb(236, 72, 153)),
                ];

                egui::Grid::new("per_thread_graphs_grid")
                    .striped(true)
                    .min_col_width(240.0)
                    .spacing(Vec2::new(20.0, 8.0))
                    .show(ui, |ui| {
                        for (title, desc, history, color) in threads_graphs {
                            ui.vertical(|ui| {
                                ui.add(egui::Label::new(egui::RichText::new(*title).strong().color(*color)).truncate());
                                ui.add(egui::Label::new(egui::RichText::new(*desc).small().color(Color32::from_rgb(148, 163, 184))).truncate());
                            });

                            let latest_val = history.back().copied().unwrap_or(0.0);
                            ui.add(egui::Label::new(egui::RichText::new(format!("{:.3} ms", latest_val)).strong()).truncate());

                            draw_sparkline_graph(ui, history, *color, Vec2::new(180.0, 32.0));
                            ui.end_row();
                        }
                    });
            });

        ui.add_space(8.0);

        // Structured List View for Sub-System Rates (One Category Per Dedicated Thread)
        egui::CollapsingHeader::new("Sub-System Target Frequency List (Per-Thread Controls)")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("tps_rate_list_grid")
                    .striped(true)
                    .spacing(Vec2::new(20.0, 4.0))
                    .show(ui, |ui| {
                        ui.label("🐰 Bhop & Strafe Rate:");
                        if ui.add(DragValue::new(&mut self.config.bhop_tps).range(1..=10000).speed(5.0).suffix(" TPS")).changed() {
                            self.send_config();
                        }
                        ui.label(egui::RichText::new("Thread 1 (Movement)").color(Color32::from_rgb(234, 179, 8)));
                        ui.end_row();

                        ui.label("🎯 Aimbot & RCS Rate:");
                        if ui.add(DragValue::new(&mut self.config.aimbot_tps).range(1..=10000).speed(5.0).suffix(" TPS")).changed() {
                            self.send_config();
                        }
                        ui.label(egui::RichText::new("Thread 2 (Aimbot)").color(Color32::from_rgb(239, 68, 68)));
                        ui.end_row();

                        ui.label("🔫 Triggerbot Rate:");
                        if ui.add(DragValue::new(&mut self.config.trigger_tps).range(1..=10000).speed(5.0).suffix(" TPS")).changed() {
                            self.send_config();
                        }
                        ui.label(egui::RichText::new("Thread 3 (Triggerbot)").color(Color32::from_rgb(249, 115, 22)));
                        ui.end_row();

                        ui.label("⌨️ Input Polling Rate:");
                        if ui.add(DragValue::new(&mut self.config.input_tps).range(1..=10000).speed(5.0).suffix(" TPS")).changed() {
                            self.send_config();
                        }
                        ui.label(egui::RichText::new("Thread 4 (Input Polling)").color(Color32::from_rgb(34, 197, 94)));
                        ui.end_row();

                        ui.label("🧱 BVH Raycast Rate:");
                        if ui.add(DragValue::new(&mut self.config.bvh_tps).range(1..=10000).speed(5.0).suffix(" TPS")).changed() {
                            self.send_config();
                        }
                        ui.label(egui::RichText::new("Thread 5 (BVH Raycast)").color(Color32::from_rgb(56, 189, 248)));
                        ui.end_row();

                        ui.label("🔍 Entity Cache Scan:");
                        if ui.add(DragValue::new(&mut self.config.cache_hz).range(1..=1000).speed(1.0).suffix(" Hz")).changed() {
                            self.send_config();
                        }
                        ui.label(egui::RichText::new("Thread 6 (Entity Cache)").color(Color32::from_rgb(168, 85, 247)));
                        ui.end_row();

                        ui.label("🦴 Bone Vis Raycasts:");
                        if ui.add(DragValue::new(&mut self.config.bone_vis_hz).range(1..=10000).speed(5.0).suffix(" Hz")).changed() {
                            self.send_config();
                        }
                        ui.label(egui::RichText::new("Thread 9 (Rayon Pool)").color(Color32::from_rgb(236, 72, 153)));
                        ui.end_row();
                    });
            });

        ui.add_space(8.0);

        // Active compute slices mapped to one category per thread
        let compute_slices: &[(&str, &str, f32, Color32)] = &[
            ("Cache Entities", "Thread 6 (Cache)", t.cache_entities_ms, Color32::from_rgb(168, 85, 247)),  // Purple
            ("Check BVH", "Thread 5 (BVH)", t.check_bvh_ms, Color32::from_rgb(56, 189, 248)),        // Cyan
            ("Aimbot & RCS", "Thread 2 (Aim)", t.aimbot_ms + t.rcs_ms, Color32::from_rgb(239, 68, 68)), // Red
            ("Triggerbot", "Thread 3 (Trigger)", t.triggerbot_ms, Color32::from_rgb(249, 115, 22)),      // Orange
            ("Bhop & Strafe", "Thread 1 (Bhop)", t.bhop_ms + t.counter_strafe_ms, Color32::from_rgb(234, 179, 8)), // Yellow
            ("Input & Devices", "Thread 4 (Input)", t.input_update_ms, Color32::from_rgb(34, 197, 94)), // Green
            ("Draw Data", "Thread 7 (GUI)", t.draw_data_ms, Color32::from_rgb(168, 162, 158)),       // Gray
            ("Other Features", "Thread 8 (Misc)", t.other_features_ms, Color32::from_rgb(99, 102, 241)), // Indigo
        ];

        let active_compute_ms: f32 = compute_slices.iter().map(|(_, _, ms, _)| *ms).sum();
        let target_frame_ms = if t.target_frame_ms > 0.0 { t.target_frame_ms } else { 1.0 };

        // Full slices including idle (total = target_frame_ms)
        let full_slices: Vec<(&str, &str, f32, Color32)> = compute_slices
            .iter()
            .copied()
            .chain(std::iter::once(("Idle (Sleep)", "System Wait", t.idle_ms.max(0.0), Color32::from_rgb(71, 85, 105))))
            .collect();

        ui.horizontal(|ui| {
            // Dynamic canvas size responsive to eGUI window size
            let avail_width = ui.available_width();
            let pie_dim = (avail_width * 0.38).clamp(160.0, 420.0);
            let desired_size = Vec2::splat(pie_dim);

            let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
            let painter = ui.painter_at(rect);

            let center = rect.center();
            let radius = (rect.width().min(rect.height()) / 2.0 - 10.0).max(30.0);

            let mut start_angle = -std::f32::consts::FRAC_PI_2; // Start at 12 o'clock

            for &(_, _, ms, color) in &full_slices {
                let slice_pct = (ms / target_frame_ms).clamp(0.0, 1.0);
                if slice_pct <= 0.0001 {
                    continue;
                }
                let sweep_angle = slice_pct * std::f32::consts::TAU;
                let end_angle = start_angle + sweep_angle;

                let num_points = ((sweep_angle.abs() / 0.1).ceil() as usize).max(4);
                let mut points = Vec::with_capacity(num_points + 2);
                points.push(center);

                for i in 0..=num_points {
                    let a = start_angle + (sweep_angle * (i as f32 / num_points as f32));
                    points.push(Pos2::new(center.x + radius * a.cos(), center.y + radius * a.sin()));
                }

                painter.add(egui::Shape::convex_polygon(
                    points,
                    color,
                    Stroke::new(1.0, Color32::from_black_alpha(100)),
                ));

                start_angle = end_angle;
            }

            // Dynamic donut hole cutout
            painter.circle_filled(center, radius * 0.45, Color32::from_rgb(20, 20, 26));
            painter.circle_stroke(center, radius * 0.45, Stroke::new(1.0, Color32::from_white_alpha(30)));

            let idle_pct = (t.idle_ms / target_frame_ms * 100.0).clamp(0.0, 100.0);

            // Numeric Breakdown Legend & Per-Thread Assignment
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(format!("Target Tick Budget: {:.2} ms ({:.0} TPS)", target_frame_ms, 1000.0 / target_frame_ms)).strong());
                ui.label(egui::RichText::new(format!("Active Compute Time: {:.2} ms", active_compute_ms)).color(Color32::from_rgb(56, 189, 248)));
                ui.label(egui::RichText::new(format!("Idle / Sleeping: {:.2} ms ({:.1}%)", t.idle_ms.max(0.0), idle_pct)).color(Color32::from_rgb(148, 163, 184)));
                ui.add_space(4.0);

                egui::Grid::new("telemetry_pie_legend")
                    .striped(true)
                    .spacing(Vec2::new(16.0, 3.0))
                    .show(ui, |ui| {
                        for &(label, thread_tag, ms, color) in &full_slices {
                            let pct = (ms / target_frame_ms * 100.0).clamp(0.0, 100.0);

                            // Color badge
                            let (badge_rect, _) = ui.allocate_exact_size(Vec2::new(12.0, 12.0), egui::Sense::hover());
                            ui.painter().rect_filled(badge_rect, 2.0, color);

                            ui.label(label);
                            ui.label(egui::RichText::new(thread_tag).strong().color(color));
                            ui.label(format!("{:.3} ms", ms));
                            ui.label(format!("{:.1}%", pct));
                            ui.end_row();
                        }
                    });
            });
        });

        ui.add_space(16.0);
        ui.heading("🔬 Microsecond Subsystem Profiler & Bottlenecks");
        ui.separator();
        ui.add_space(4.0);

        if let Ok(profiler) = crate::profiler::GLOBAL_PROFILER.lock() {
            let stats = profiler.get_all();
            if stats.is_empty() {
                ui.label("Collecting profiler scope metrics...");
            } else {
                let mut sorted_stats: Vec<(&'static str, crate::profiler::ScopeStats)> = stats.into_iter().collect();
                sorted_stats.sort_by(|a, b| b.1.avg_time_us.partial_cmp(&a.1.avg_time_us).unwrap_or(std::cmp::Ordering::Equal));

                egui::Grid::new("profiler_metrics_grid")
                    .striped(true)
                    .spacing(Vec2::new(16.0, 4.0))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("Scope Name").strong());
                        ui.label(egui::RichText::new("Calls").strong());
                        ui.label(egui::RichText::new("Avg (us)").strong());
                        ui.label(egui::RichText::new("Last (us)").strong());
                        ui.label(egui::RichText::new("Min (us)").strong());
                        ui.label(egui::RichText::new("Max (ms)").strong());
                        ui.label(egui::RichText::new("Spikes (>2ms)").strong());
                        ui.end_row();

                        for (name, scope) in sorted_stats {
                            let avg_us = scope.avg_time_us;
                            let last_us = scope.last_time.as_secs_f32() * 1_000_000.0;
                            let min_us = scope.min_time.as_secs_f32() * 1_000_000.0;
                            let max_ms = scope.max_time.as_secs_f32() * 1000.0;

                            let spike_color = if scope.spike_count > 0 {
                                Color32::from_rgb(239, 68, 68)
                            } else {
                                Color32::from_rgb(34, 197, 94)
                            };

                            ui.label(egui::RichText::new(name).strong());
                            ui.label(format!("{}", scope.count));
                            ui.label(format!("{:.1} us", avg_us));
                            ui.label(format!("{:.1} us", last_us));
                            ui.label(format!("{:.1} us", min_us));
                            ui.label(format!("{:.2} ms", max_ms));
                            ui.label(egui::RichText::new(format!("{}", scope.spike_count)).color(spike_color));
                            ui.end_row();
                        }
                    });
            }
        }
        });
    }
}

fn push_history(queue: &mut std::collections::VecDeque<f32>, val: f32) {
    if queue.len() >= 60 {
        queue.pop_front();
    }
    queue.push_back(val);
}

fn draw_sparkline_graph(ui: &mut Ui, history: &std::collections::VecDeque<f32>, color: Color32, size: Vec2) {
    let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 3.0, Color32::from_rgb(15, 20, 28));
    painter.rect_stroke(rect, 3.0, Stroke::new(1.0, Color32::from_white_alpha(20)), egui::StrokeKind::Inside);

    if history.len() < 2 {
        return;
    }

    let max_val = history.iter().copied().fold(0.001f32, f32::max);
    let pts: Vec<Pos2> = history
        .iter()
        .enumerate()
        .map(|(i, &val)| {
            let x = rect.left() + (i as f32 / (history.len() - 1) as f32) * rect.width();
            let norm_y = (val / max_val).clamp(0.0, 1.0);
            let y = rect.bottom() - norm_y * (rect.height() - 4.0) - 2.0;
            Pos2::new(x, y)
        })
        .collect();

    for i in 0..pts.len() - 1 {
        painter.line_segment([pts[i], pts[i + 1]], Stroke::new(1.5, color));
    }
}
