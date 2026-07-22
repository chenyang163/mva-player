//! Viewport panel — paints the [`DrawList`] onto the central area.

use egui::containers::panel::CentralPanel;
use mva_renderer::DrawList;

use crate::painter::{TextureCache, paint_draw_list};

pub fn show(ui: &mut egui::Ui, cache: &mut TextureCache, draw_list: &DrawList) {
    CentralPanel::default().show(ui, |ui| {
        let painter = ui.painter().clone();
        let ctx = ui.ctx().clone();
        paint_draw_list(cache, &ctx, &painter, draw_list);
    });
}
