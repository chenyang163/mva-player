//! Painter adapter — converts a backend‑neutral [`DrawList`] into
//! egui shapes (architecture §7.3, `docs/ui-architecture.md`).

use std::collections::HashMap;

use mva_renderer::{DrawCommand, DrawList, EffectDraw, ImageDraw};
use mva_types::AssetRef;

/// Cache keyed by asset path to avoid re‑decoding every frame.
pub type TextureCache = HashMap<String, egui::TextureHandle>;

/// Paint every command in the draw list onto an egui [`Painter`].
///
/// The `cache` persists across frames inside [`MvaUiApp`]; `ctx` is
/// needed to upload new textures via [`egui::Context::load_texture`].
pub fn paint_draw_list(
    cache: &mut TextureCache,
    ctx: &egui::Context,
    painter: &egui::Painter,
    draw_list: &DrawList,
) {
    for cmd in &draw_list.commands {
        match cmd {
            DrawCommand::Text {
                text,
                font_size,
                color,
                x,
                y,
                opacity,
            } => {
                let [r, g, b, a] = *color;
                let alpha = (a as f32 / 255.0 * opacity).clamp(0.0, 1.0);
                let egui_color = egui::Color32::from_rgba_premultiplied(
                    (r as f32 * alpha) as u8,
                    (g as f32 * alpha) as u8,
                    (b as f32 * alpha) as u8,
                    (alpha * 255.0) as u8,
                );
                let font_id = egui::FontId::new(*font_size, egui::FontFamily::Proportional);
                painter.text(
                    egui::pos2(*x, *y),
                    egui::Align2::LEFT_TOP,
                    text,
                    font_id,
                    egui_color,
                );
            }
            DrawCommand::Image(img) => paint_image(cache, ctx, painter, img),
            DrawCommand::Effect(eff) => paint_effect_debug(painter, eff),
        }
    }
}

// -------------------------------------------------------------------
// effect debug visualization
// -------------------------------------------------------------------

fn paint_effect_debug(painter: &egui::Painter, eff: &EffectDraw) {
    let rect = egui::Rect::from_min_max(
        egui::pos2(eff.target_rect.0, eff.target_rect.1),
        egui::pos2(eff.target_rect.2, eff.target_rect.3),
    );

    // Semi‑transparent fill so effects are visible.
    let fill = egui::Color32::from_rgba_premultiplied(0, 100, 200, 60);
    painter.rect_filled(rect, 0.0, fill);

    // Outline.
    let stroke = egui::Color32::from_rgba_premultiplied(0, 150, 255, 180);
    painter.rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(1.5, stroke),
        egui::StrokeKind::Middle,
    );

    // Label with effect id + param count.
    let label = format!("Effect: {} ({} params)", eff.effect_id, eff.params.len());
    let font_id = egui::FontId::new(14.0, egui::FontFamily::Proportional);
    painter.text(
        rect.left_top() + egui::vec2(4.0, 2.0),
        egui::Align2::LEFT_TOP,
        label,
        font_id,
        egui::Color32::WHITE,
    );
}

// -------------------------------------------------------------------
// image helper
// -------------------------------------------------------------------

fn paint_image(
    cache: &mut TextureCache,
    ctx: &egui::Context,
    painter: &egui::Painter,
    img: &ImageDraw,
) {
    let handle = match &img.asset {
        AssetRef::File { path } => {
            let entry = cache.entry(path.clone());
            let handle = entry.or_insert_with(|| load_texture_from_file(ctx, path));
            handle.clone()
        }
        AssetRef::Pkg { .. } => {
            // Pkg assets require the mva-assets subsystem (Phase 4+).
            // Draw a placeholder rect to indicate the missing image.
            let rect = egui::Rect::from_min_size(
                egui::pos2(img.x, img.y),
                egui::vec2(img.width.max(1.0), img.height.max(1.0)),
            );
            painter.rect_filled(
                rect,
                0.0,
                egui::Color32::from_rgba_premultiplied(100, 100, 100, 80),
            );
            return;
        }
    };

    let alpha = img.opacity.clamp(0.0, 1.0);
    let tint = egui::Color32::from_rgba_premultiplied(255, 255, 255, (alpha * 255.0) as u8);

    let rect = egui::Rect::from_min_size(
        egui::pos2(img.x, img.y),
        egui::vec2(img.width.max(1.0), img.height.max(1.0)),
    );

    painter.image(
        handle.id(),
        rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        tint,
    );
}

fn load_texture_from_file(ctx: &egui::Context, path: &str) -> egui::TextureHandle {
    match std::fs::read(path) {
        Ok(bytes) => match image::load_from_memory(&bytes) {
            Ok(dyn_img) => {
                let rgba = dyn_img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels: Vec<egui::Color32> = rgba
                    .pixels()
                    .map(|p| egui::Color32::from_rgba_premultiplied(p[0], p[1], p[2], p[3]))
                    .collect();
                let color_image = egui::ColorImage {
                    size,
                    source_size: [size[0] as f32, size[1] as f32].into(),
                    pixels,
                };
                ctx.load_texture(path, color_image, egui::TextureOptions::LINEAR)
            }
            Err(_) => fallback_texture(ctx, path),
        },
        Err(_) => fallback_texture(ctx, path),
    }
}

fn fallback_texture(ctx: &egui::Context, _path: &str) -> egui::TextureHandle {
    // A 2×2 magenta/black checkerboard — indicates a missing image
    // without panicking.
    let pixels = vec![
        egui::Color32::from_rgba_premultiplied(255, 0, 255, 255),
        egui::Color32::BLACK,
        egui::Color32::BLACK,
        egui::Color32::from_rgba_premultiplied(255, 0, 255, 255),
    ];
    let color_image = egui::ColorImage {
        size: [2, 2],
        source_size: [2.0, 2.0].into(),
        pixels,
    };
    ctx.load_texture("_fallback", color_image, egui::TextureOptions::NEAREST)
}
