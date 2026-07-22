//! # mva-renderer — Animation Renderer
//!
//! Converts a renderer‑independent [`Scene`] into a backend‑neutral
//! [`DrawList`] (architecture §7.3).

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod config;
mod cull;
mod draw;
mod layout;
mod viewport;

pub use config::RendererConfig;
pub use draw::{DrawCommand, DrawList, EffectDraw, ImageDraw};
pub use viewport::Viewport;

use mva_scene::Scene;

/// The renderer — stateless beyond its startup [`RendererConfig`].
///
/// Created once; [`render`](Self::render) is called every frame with
/// the current [`Scene`] and runtime [`Viewport`].
pub struct Renderer {
    _config: RendererConfig,
}

impl Renderer {
    /// Create a new renderer with the given static configuration.
    pub fn new(config: RendererConfig) -> Self {
        Self { _config: config }
    }

    /// Produce a backend‑neutral [`DrawList`] from an evaluated
    /// scene at the given viewport size.
    pub fn render(&self, scene: &Scene, viewport: &Viewport) -> DrawList {
        let mut commands = Vec::new();

        // 1. layers → draw commands
        for layer in &scene.layers {
            if !layer.visible {
                continue;
            }
            if layer.transform.opacity <= 0.0 {
                continue;
            }
            if cull::off_viewport(&layer.transform, viewport) {
                continue;
            }
            match &layer.kind {
                mva_scene::EvaluatedLayerKind::Text { text, style } => {
                    let cmd = layout::text_to_draw_command(text, style, &layer.transform, viewport);
                    commands.push(cmd);
                }
                mva_scene::EvaluatedLayerKind::Image { asset } => {
                    commands.push(DrawCommand::Image(ImageDraw {
                        asset: asset.clone(),
                        x: layer.transform.position.x,
                        y: layer.transform.position.y,
                        width: 100.0,
                        height: 100.0,
                        opacity: layer.transform.opacity,
                    }));
                }
            }
        }

        // 2. effects → draw commands
        for effect in &scene.effects {
            let target_rect = match &effect.target {
                mva_types::EffectTarget::WholeScene => (0.0, 0.0, viewport.width, viewport.height),
                mva_types::EffectTarget::Background => (0.0, 0.0, viewport.width, viewport.height),
                mva_types::EffectTarget::Layer { .. } => {
                    // Layer lookup deferred to Phase 3+.
                    (0.0, 0.0, viewport.width, viewport.height)
                }
            };
            commands.push(DrawCommand::Effect(EffectDraw {
                effect_id: effect.effect_id.clone(),
                params: effect.params.clone(),
                target_rect,
            }));
        }

        DrawList { commands }
    }
}
