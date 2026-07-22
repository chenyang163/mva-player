//! Info panel — top bar with playback status.

use egui::containers::panel::Panel;
use mva_core::state::EngineSnapshot;

pub fn show(ui: &mut egui::Ui, snap: &EngineSnapshot) {
    Panel::top("info").show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(format!("State: {}", snap.state));
            ui.separator();
            ui.label(format!("{:.1} / {:.1} s", snap.position, snap.duration));
            ui.separator();
            match snap.active_lyric_index {
                Some(i) => {
                    ui.label(format!("Lyric: #{i}"));
                }
                None => {
                    ui.label("Lyric: —");
                }
            }
            if let Some(ref err) = snap.error {
                ui.separator();
                ui.colored_label(egui::Color32::RED, format!("Error: {err:?}"));
            }
        });
    });
}
