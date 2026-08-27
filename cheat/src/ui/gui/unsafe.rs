use egui::{DragValue, Ui};

use crate::ui::{
    app::App,
    gui::helpers::{collapsing_open, color_picker, keybind},
};

impl App {
    pub fn unsafe_settings(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .id_salt("unsafe_left")
                .show(left, |left| {
                    self.unsafe_left(left);
                });

            let right = &mut cols[1];
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .id_salt("unsafe_right")
                .show(right, |right| {
                    self.unsafe_right(right);
                });
        });

        collapsing_open(ui, "Smokes", |ui| {
            if ui
                .checkbox(&mut self.config.misc.no_smoke, "No Smoke")
                .changed()
            {
                self.send_config();
            }

            if ui
                .checkbox(
                    &mut self.config.misc.change_smoke_color,
                    "Change Smoke Color",
                )
                .changed()
            {
                self.send_config();
            }

            if color_picker(ui, "Smoke Color", &mut self.config.misc.smoke_color) {
                self.send_config();
            }
        });
    }

    fn unsafe_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Movement", |ui| {
            if ui
                .checkbox(&mut self.config.misc.bunnyhop, "Bunnyhop")
                .changed()
            {
                self.send_config();
            }
            if keybind(
                ui,
                "bhop_hotkey",
                "Bunnyhop Hotkey",
                &mut self.config.misc.bhop_hotkey,
            ) {
                self.send_config();
            }
            if ui
                .checkbox(&mut self.config.misc.autostrafe, "Auto Strafe")
                .changed()
            {
                self.send_config();
            }
            if ui
                .checkbox(&mut self.config.misc.counter_strafe, "Counter Strafe")
                .changed()
            {
                self.send_config();
            }
            if ui
                .checkbox(&mut self.config.misc.edge_jump, "Edge Jump")
                .changed()
            {
                self.send_config();
            }
            if keybind(
                ui,
                "edge_jump_hotkey",
                "Edge Jump Hotkey",
                &mut self.config.misc.edge_jump_hotkey,
            ) {
                self.send_config();
            }
            if keybind(
                ui,
                "grenade_move_hotkey",
                "Grenade Assist Hotkey",
                &mut self.config.misc.grenade_move_hotkey,
            ) {
                self.send_config();
            }
        });

        collapsing_open(ui, "Mic Noise / Ear-Rape", |ui| {
            if ui
                .checkbox(&mut self.config.misc.mic_tone, "Enable Mic Feature")
                .changed()
            {
                self.send_config();
            }
            if keybind(
                ui,
                "mic_tone_hotkey",
                "Mic Hotkey",
                &mut self.config.misc.mic_tone_hotkey,
            ) {
                self.send_config();
            }
            egui::ComboBox::from_label("Mode")
                .selected_text(match self.config.misc.mic_tone_mode {
                    1 => "Replay Desktop Audio (Ear-Rape)",
                    _ => "Pure Sine Tone",
                })
                .show_ui(ui, |ui| {
                    if ui.selectable_value(&mut self.config.misc.mic_tone_mode, 1, "Replay Desktop Audio (Ear-Rape)").changed() {
                        self.send_config();
                    }
                    if ui.selectable_value(&mut self.config.misc.mic_tone_mode, 0, "Pure Sine Tone").changed() {
                        self.send_config();
                    }
                });

            if self.config.misc.mic_tone_mode == 0 {
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            DragValue::new(&mut self.config.misc.mic_tone_frequency)
                                .range(100.0..=12000.0)
                                .speed(50.0)
                                .suffix(" Hz"),
                        )
                        .changed()
                    {
                        self.send_config();
                    }
                    ui.label("Frequency");
                });
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.config.misc.mic_tone_volume)
                            .range(1.0..=30.0)
                            .speed(0.5)
                            .max_decimals(1)
                            .suffix("x"),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Software Gain / Amplification");
            });
            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.config.misc.mic_hw_boost)
                            .range(100.0..=1000.0)
                            .speed(25.0)
                            .suffix("%"),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("PulseAudio Hardware Boost");
            });
        });

        collapsing_open(ui, "No Flash", |ui| {
            if ui
                .checkbox(&mut self.config.misc.no_flash, "No Flash")
                .changed()
            {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.config.misc.max_flash_alpha)
                            .range(0.0..=255.0)
                            .speed(0.5)
                            .max_decimals(0),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Max Flash Alpha");
            });
        });
    }

    fn unsafe_right(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "FOV Changer", |ui| {
            if ui
                .checkbox(&mut self.config.misc.fov_changer, "FOV Changer")
                .changed()
            {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.config.misc.desired_fov)
                            .speed(0.1)
                            .range(1..=179),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Desired FOV");

                if ui.button("Reset").clicked() {
                    self.config.misc.desired_fov = crate::constants::cs2::DEFAULT_FOV;
                    self.send_config();
                }
            });
        });

        collapsing_open(ui, "Radar", |ui| {
            if ui
                .checkbox(&mut self.config.misc.radar, "Enable Web Radar")
                .changed()
            {
                self.send_config();
            }

            if self.config.misc.radar {
                ui.separator();
                let local_link = crate::radar::RADAR_LOCAL_LINK
                    .lock()
                    .map(|l| l.clone())
                    .unwrap_or_default();
                let public_link = crate::radar::RADAR_PUBLIC_LINK
                    .lock()
                    .map(|p| p.clone())
                    .unwrap_or_default();

                if local_link.is_empty() || local_link == "Starting server..." {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Starting radar server...");
                    });
                } else if local_link.starts_with("http") {
                    ui.horizontal(|ui| {
                        ui.label("Local:");
                        let mut val = local_link.clone();
                        ui.text_edit_singleline(&mut val);
                        if ui.button("Copy").clicked() {
                            ui.ctx().copy_text(local_link.clone());
                            copy_to_system_clipboard(&local_link);
                        }
                    });
                } else {
                    ui.label(format!("Local: {}", local_link));
                }

                if public_link == "Fetching..." {
                    ui.horizontal(|ui| {
                        ui.label("Public:");
                        ui.spinner();
                        ui.label("Fetching Cloudflare URL...");
                    });
                } else if public_link.starts_with("http") {
                    ui.horizontal(|ui| {
                        ui.label("Public:");
                        let mut val = public_link.clone();
                        ui.text_edit_singleline(&mut val);
                        if ui.button("Copy").clicked() {
                            ui.ctx().copy_text(public_link.clone());
                            copy_to_system_clipboard(&public_link);
                        }
                    });
                } else if !public_link.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label("Public:");
                        ui.label(&public_link);
                    });
                }

                ui.horizontal(|ui| {
                    if ui.button("Restart Cloudflare Tunnel").clicked() {
                        crate::radar::restart_cloudflared();
                    }
                });

                ui.weak("Links are also saved to radar_link.txt");
            }
        });
    }
}

fn copy_to_system_clipboard(link: &str) {
    let link = link.to_owned();
    std::thread::spawn(move || {
        let _ = std::process::Command::new("wl-copy")
            .arg(&link)
            .stderr(std::process::Stdio::null())
            .spawn();

        if let Ok(mut child) = std::process::Command::new("xclip")
            .args(&["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(link.as_bytes());
            }
            let _ = child.wait();
        }
    });
}
