//! Backend‑neutral draw list (architecture §7.3).

use mva_types::AssetRef;

/// The output of the renderer — a flat, ordered list of drawing
/// commands ready to be consumed by a painter adapter (egui, wgpu,
/// etc.).
#[derive(Debug, Clone, PartialEq)]
pub struct DrawList {
    /// Drawing commands in paint order (back → front).
    pub commands: Vec<DrawCommand>,
}

impl DrawList {
    /// An empty draw list (no layers to render).
    pub fn empty() -> Self {
        Self {
            commands: Vec::new(),
        }
    }
}

/// A single drawable primitive.
#[derive(Debug, Clone, PartialEq)]
pub enum DrawCommand {
    /// A run of text.
    Text {
        /// The text to render.
        text: String,
        /// Font size in points.
        font_size: f32,
        /// RGBA colour.
        color: [u8; 4],
        /// Screen‑space x (left edge, pixels).
        x: f32,
        /// Screen‑space y (baseline or top, renderer‑dependent).
        y: f32,
        /// Layer opacity `0.0–1.0`.
        ///
        /// The painter backend multiplies this with the colour alpha
        /// channel to produce the final per‑pixel alpha:
        /// `final_alpha = color[3] / 255.0 * opacity`.
        opacity: f32,
    },
    /// An image draw command.
    Image(ImageDraw),
    /// An effect draw command.
    Effect(EffectDraw),
}

/// Instructions to draw a single image.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageDraw {
    /// The image asset to render.
    pub asset: AssetRef,
    /// Screen‑space x (pixels).
    pub x: f32,
    /// Screen‑space y (pixels).
    pub y: f32,
    /// Display width (pixels; may differ from source size).
    pub width: f32,
    /// Display height (pixels).
    pub height: f32,
    /// Layer opacity `0.0–1.0`.
    pub opacity: f32,
}

/// Instructions to apply a visual effect.
#[derive(Debug, Clone, PartialEq)]
pub struct EffectDraw {
    /// Effect identifier (e.g. `"bloom"`, `"spectrum"`).
    pub effect_id: String,
    /// Resolved parameters (name → value).
    pub params: Vec<(String, mva_types::ParamValue)>,
    /// Target rectangle in screen space.
    pub target_rect: (f32, f32, f32, f32),
}
