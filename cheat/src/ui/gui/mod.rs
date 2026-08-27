use std::time::Duration;

use egui::{Align, Ui};

use crate::{
    config::{aim::WeaponConfig, write_config},
    message::{GameMessage, GameStatus},
    ui::{app::App, color::Colors, gui::aimbot::AimbotTab},
};

mod about;
pub mod aimbot;
mod application;
mod config;
mod grenade;
mod helpers;
mod hud;
mod player;
mod r#unsafe;
mod telemetry;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum Tab {
    #[default]
    Aimbot,
    Player,
    Hud,
    Grenades,
    Unsafe,
    Config,
    Application,
    Telemetry,
}

impl Tab {
    pub const ALL: &'static [Tab] = &[
        Tab::Aimbot,
        Tab::Player,
        Tab::Hud,
        Tab::Grenades,
        Tab::Unsafe,
        Tab::Config,
        Tab::Application,
        Tab::Telemetry,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Tab::Aimbot => "aimbot",
            Tab::Player => "player",
            Tab::Hud => "hud",
            Tab::Grenades => "grenades",
            Tab::Unsafe => "unsafe",
            Tab::Config => "config",
            Tab::Application => "application",
            Tab::Telemetry => "telemetry",
        }
    }
}

impl App {
    pub fn send_config(&mut self) {
        self.send_message(GameMessage(Box::new(self.config.clone())));
        self.save();
    }

    pub fn send_message(&self, message: GameMessage) {
        if self.channel.send(message).is_err() {
            std::process::exit(1);
        }
    }

    fn save(&self) {
        write_config(&self.config, &self.current_config);
    }

    fn gui(&mut self, ui: &mut Ui) {
        ui.ctx().set_pixels_per_point(self.display_scale);
        
        // Ensure image loaders are installed so egui can decode the compiled PNG
        egui_extras::install_image_loaders(ui.ctx());

        egui::Panel::left("sidebar")
            .resizable(false)
            .show_inside(ui, |ui| {
                // Display the Deadlocked Logo
                ui.add_space(8.0);
                ui.vertical_centered(|ui| {
                    ui.add(
                        egui::Image::new(egui::include_image!("logo.svg"))
                            .max_width(120.0)
                            .max_height(48.0)
                    );
                });
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                ui.selectable_value(&mut self.current_tab, Tab::Aimbot, "\u{f04fe} Aimbot");
                ui.selectable_value(&mut self.current_tab, Tab::Player, "\u{f0013} Player");
                ui.selectable_value(&mut self.current_tab, Tab::Hud, "\u{f0379} Hud");
                ui.selectable_value(&mut self.current_tab, Tab::Grenades, "\u{f0691} Grenades");
                ui.selectable_value(&mut self.current_tab, Tab::Unsafe, "\u{f0ce6} Unsafe");
                ui.selectable_value(&mut self.current_tab, Tab::Config, "\u{f168b} Config");
                ui.selectable_value(
                    &mut self.current_tab,
                    Tab::Application,
                    "\u{f1577} Application",
                );
                ui.selectable_value(
                    &mut self.current_tab,
                    Tab::Telemetry,
                    "\u{f012} Telemetry",
                );

                ui.with_layout(egui::Layout::bottom_up(Align::Min), |ui| {
                    if ui.button("Report Issue").clicked() {
                        let _ = std::process::Command::new("xdg-open")
                            .arg("https://github.com/avitran0/deadlocked/issues")
                            .status();
                    }

                    if ui.button("About").clicked() {
                        self.show_about = true;
                    }

                    ui.label(egui::RichText::new(format!("{}", self.game_status)).color(
                        match self.game_status {
                            GameStatus::Working => Colors::GREEN,
                            GameStatus::NotStarted => Colors::YELLOW,
                        },
                    ));

                    let frame_avg = if self.frame_times.is_empty() {
                        0.0f32
                    } else {
                        let frame_sum =
                            self.frame_times.iter().sum::<Duration>().as_secs_f32() * 1000.0;
                        frame_sum / self.frame_times.len() as f32
                    };
                    ui.label(format!("{frame_avg:.1} ms",));
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| match self.current_tab {
            Tab::Aimbot => self.aimbot_settings(ui),
            Tab::Player => self.player_settings(ui),
            Tab::Hud => self.hud_settings(ui),
            Tab::Grenades => self.grenade_settings(ui),
            Tab::Unsafe => self.unsafe_settings(ui),
            Tab::Config => self.config_settings(ui),
            Tab::Application => self.application_settings(ui),
            Tab::Telemetry => self.telemetry_settings(ui),
        });

        if self.show_about {
            self.about(ui.ctx());
        }
    }

    fn weapon_config(&mut self) -> &mut WeaponConfig {
        if self.aimbot_tab == AimbotTab::Weapon {
            self.config
                .aim
                .weapons
                .get_mut(&self.aimbot_weapon)
                .unwrap()
        } else {
            &mut self.config.aim.global
        }
    }

    pub fn render(&mut self) {
        let data = self.data.lock();
        if data.total_damage > self.last_total_damage {
            self.hit_marker_time = std::time::Instant::now();
        }
        self.last_total_damage = data.total_damage;
        drop(data);

        let self_ptr = self as *mut Self;

        let gui = self.gui.as_mut().unwrap();

        if let Err(err) = gui.make_current() {
            utils::error!("could not make gui window current: {err}");
            return;
        }
        gui.run(|ui| (unsafe { &mut *self_ptr }).gui(ui));
        gui.clear();
        gui.paint();

        if let Err(err) = gui.swap_buffers() {
            utils::error!("could not swap gui window buffers: {err}");
            return;
        }

        let overlay = self.overlay.as_mut().unwrap();

        overlay.window().set_cursor_hittest(false).unwrap();
        if let Err(err) = overlay.make_current() {
            utils::error!("could not make overlay window current: {err}");
            return;
        }

        overlay.run(move |ui| {
            (unsafe { &mut *self_ptr }).overlay(ui);
        });
        overlay.clear();
        overlay.paint();

        if let Err(err) = overlay.swap_buffers() {
            utils::error!("could not swap overlay window buffers: {err}");
        }

        if self.demo_mode {
            if self.demo_tab_idx < Tab::ALL.len() {
                self.current_tab = Tab::ALL[self.demo_tab_idx];
                if self.demo_last_step.elapsed() >= Duration::from_millis(600) {
                    let tab_name = Tab::ALL[self.demo_tab_idx].name();
                    let file_name = format!("media/previews/{:02}_{}.png", self.demo_tab_idx + 1, tab_name);
                    let _ = std::fs::create_dir_all("media/previews");
                    
                    utils::info!("Capturing full-system GUI screenshot for tab '{}' -> {}", tab_name, file_name);
                    let _ = std::process::Command::new("sh")
                        .arg("-c")
                        .arg(format!("grim '{file_name}' || spectacle -b -n -o '{file_name}' || import -window root '{file_name}'"))
                        .status();

                    self.demo_tab_idx += 1;
                    self.demo_last_step = std::time::Instant::now();
                }
            } else if self.demo_last_step.elapsed() >= Duration::from_millis(500) {
                utils::info!("All GUI tab screenshots captured successfully! Exiting.");
                std::process::exit(0);
            }
        }
    }
}
