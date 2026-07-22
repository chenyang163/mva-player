//! Controls panel — play / pause / stop / seek / volume.

use egui::containers::panel::Panel;
use mva_core::PlayerCommand;
use mva_core::state::EngineSnapshot;

pub fn show(
    ui: &mut egui::Ui,
    snap: &EngineSnapshot,
    seek_pos: &mut f64,
    commands: &mut Vec<PlayerCommand>,
) {
    Panel::bottom("controls").show(ui, |ui| {
        ui.horizontal(|ui| {
            use mva_core::state::PlaybackState;

            match snap.state {
                PlaybackState::Stopped | PlaybackState::Ready | PlaybackState::Paused => {
                    if ui.button("Play").clicked() {
                        commands.push(PlayerCommand::Play);
                    }
                }
                PlaybackState::Playing => {
                    if ui.button("Pause").clicked() {
                        commands.push(PlayerCommand::Pause);
                    }
                }
                PlaybackState::Finished => {
                    if ui.button("Replay").clicked() {
                        commands.push(PlayerCommand::Play);
                    }
                }
                PlaybackState::Loading => {
                    ui.label("Loading…");
                }
                PlaybackState::Error => {
                    ui.label("Error");
                }
            }

            if snap.state != PlaybackState::Loading
                && snap.state != PlaybackState::Error
                && ui.button("Stop").clicked()
            {
                commands.push(PlayerCommand::Stop);
            }

            let dur = snap.duration;
            if dur > 0.0 {
                let mut s = *seek_pos;
                ui.add(egui::Slider::new(&mut s, 0.0..=dur).text("Seek"));
                if (s - *seek_pos).abs() > 0.01 {
                    *seek_pos = s;
                    commands.push(PlayerCommand::Seek(s));
                }
            }

            let mut vol = snap.volume;
            ui.add(egui::Slider::new(&mut vol, 0.0..=1.0).text("Vol"));
            if (vol - snap.volume).abs() > 0.001 {
                commands.push(PlayerCommand::SetVolume(vol));
            }
        });
    });
}
