//! Settings panel — File menu + status bar.

use std::path::PathBuf;

use egui::containers::panel::Panel;
use mva_core::PlayerCommand;
use mva_core::state::EngineSnapshot;

/// Top menu bar with File (native open dialogs), Help, and a text path input.
pub fn menu_bar(ui: &mut egui::Ui, commands: &mut Vec<PlayerCommand>, path_buf: &mut String) {
    Panel::top("menu_bar").show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open File…").clicked() {
                    ui.close();
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("MVA Project", &["mva"])
                        .add_filter("Audio Files", &["mp3", "flac", "wav"])
                        .pick_file()
                    {
                        commands.push(PlayerCommand::OpenFile(path));
                    }
                }
                if ui.button("Open Folder…").clicked() {
                    ui.close();
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        commands.push(PlayerCommand::OpenFile(path));
                    }
                }
            });
            ui.menu_button("Help", |ui| {
                ui.label("MVA Player v0.1.0");
                ui.separator();
                ui.label("Place an mp3 + .lrc in the same folder");
                ui.label("and open the folder.");
            });
        });

        // Simple path input (no native file dialog dependency).
        ui.horizontal(|ui| {
            ui.label("Path:");
            let resp = ui.text_edit_singleline(path_buf);
            let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
            if resp.lost_focus() && enter {
                let p = PathBuf::from(path_buf.trim());
                if !p.as_os_str().is_empty() {
                    commands.push(PlayerCommand::OpenFile(p));
                }
            }
            if ui.button("Open").clicked() {
                let p = PathBuf::from(path_buf.trim());
                if !p.as_os_str().is_empty() {
                    commands.push(PlayerCommand::OpenFile(p));
                }
            }
        });
    });
}

/// Bottom status bar — state hint and config warnings (display only, no commands).
pub fn status_bar(ui: &mut egui::Ui, snap: &EngineSnapshot, config_warnings: &[String]) {
    Panel::bottom("status_bar").show(ui, |ui| {
        use mva_core::state::PlaybackState;
        match snap.state {
            PlaybackState::Stopped => {
                ui.label("Drop a file or enter a path above.");
            }
            PlaybackState::Loading => {
                ui.label("Loading…");
            }
            PlaybackState::Error => {
                if let Some(ref err) = snap.error {
                    ui.colored_label(egui::Color32::RED, format!("Error: {err:?}"));
                }
                ui.label("Press Stop to resume or open another file.");
            }
            _ => {}
        }

        if !config_warnings.is_empty() {
            ui.separator();
            ui.colored_label(
                egui::Color32::from_rgb(255, 191, 0),
                "Configuration Warnings (using defaults where needed):",
            );
            for w in config_warnings {
                ui.label(w.as_str());
            }
        }
    });
}
