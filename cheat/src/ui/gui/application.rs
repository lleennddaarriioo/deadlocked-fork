use egui::Ui;

use crate::{
    ui::{app::AppState, gui::helpers::open_url},
    update::UpdateStatus,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

impl AppState {
    pub fn application_settings(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("deadlocked");
            ui.label("author: avitrano");
            ui.label(format!("Version: v{VERSION}"));

            ui.separator();

            match &self.update_status {
                UpdateStatus::UpToDate => {
                    ui.colored_label(crate::ui::color::Colors::GREEN, "Up to date");
                }
                UpdateStatus::Available { version, url } => {
                    ui.colored_label(
                        crate::ui::color::Colors::YELLOW,
                        format!("Update available: {version}"),
                    );
                    if ui.link("Download").clicked() {
                        open_url(url);
                    }
                }
                UpdateStatus::Error(err) => {
                    ui.colored_label(
                        crate::ui::color::Colors::RED,
                        format!("Update check failed: {err}"),
                    );
                }
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            ui.heading("Flag Bypass Mode");
            ui.label("Disables all memory writes, aimbot, triggerbot, bhop, & input simulation.");
            ui.label("Only renders ESP overlay, 3D sound ESP, radar, & visual indicators.");
            ui.add_space(4.0);
            if ui
                .checkbox(
                    &mut self.config.visuals_only_mode,
                    "Enable Flag Bypass Mode (ESP & Visuals Only)",
                )
                .changed()
            {
                self.send_config();
            }
        });
    }
}
