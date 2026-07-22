//! Settings panel — "Open File" + version info.

use egui::containers::panel::Panel;
use mva_core::PlayerCommand;
use mva_core::state::EngineSnapshot;

pub fn show(
    ui: &mut egui::Ui,
    commands: &mut Vec<PlayerCommand>,
    snap: &EngineSnapshot,
    path_buf: &mut String,
) {
    Panel::top("menu_bar").show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open…").clicked() {
                    ui.close();
                }
            });
            ui.menu_button("Help", |ui| {
                ui.label("MVA Player v0.1.0");
                ui.separator();
                ui.label("Phase 2 Step 4 — real file loading.");
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
                let p = std::path::PathBuf::from(path_buf.trim());
                if !p.as_os_str().is_empty() {
                    commands.push(PlayerCommand::OpenFile(p));
                }
            }
            if ui.button("Open").clicked() {
                let p = std::path::PathBuf::from(path_buf.trim());
                if !p.as_os_str().is_empty() {
                    commands.push(PlayerCommand::OpenFile(p));
                }
            }
        });

        // Show current state hint.
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
            }
            _ => {}
        }
    });
}
