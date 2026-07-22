//! Layer-building types: id, style, blend, and the evaluated layer
//! enum.

use serde::{Deserialize, Serialize};

use mva_types::AssetRef;

use super::colour::Rgba;

/// Stable identifier of a layer, unique within a project.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LayerId(pub String);

/// Compositing blend mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlendMode {
    /// Standard source-over alpha compositing.
    #[default]
    Normal,
    /// Additive blending.
    Add,
    /// Multiplicative darkening.
    Multiply,
    /// Screen blending (lighten).
    Screen,
}

/// Text styling information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    /// Font family name; `None` = renderer default.
    #[serde(default)]
    pub font_family: Option<String>,
    /// Font size in points.
    pub font_size: f32,
    /// Text colour.
    #[serde(
        deserialize_with = "deser_rgba",
        serialize_with = "ser_rgba",
        default = "default_rgba"
    )]
    pub color: Rgba,
}

// -- serde helpers for the `[u8; 4]` legacy format --------------------

fn deser_rgba<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Rgba, D::Error> {
    let arr = <[u8; 4]>::deserialize(d)?;
    Ok(Rgba::from(arr))
}

fn ser_rgba<S: serde::Serializer>(c: &Rgba, s: S) -> Result<S::Ok, S::Error> {
    let arr: [u8; 4] = (*c).into();
    arr.serialize(s)
}

fn default_rgba() -> Rgba {
    Rgba::WHITE
}

// ---------------------------------------------------------------------

/// Evaluated layer content — the resolved kind at time `t`.
#[derive(Debug, Clone, PartialEq)]
pub enum EvaluatedLayerKind {
    /// Text layer with the bound string.
    Text {
        /// The evaluated text.
        text: String,
        /// Static styling.
        style: TextStyle,
    },
    /// Image layer with a resolved asset reference.
    Image {
        /// Asset to render (renderer resolves at paint time).
        asset: AssetRef,
    },
}

/// One layer after evaluation: resolved content + computed transform
/// + visibility at time `t` (§5).
#[derive(Debug, Clone, PartialEq)]
pub struct EvaluatedLayer {
    /// Original layer id (stable).
    pub id: LayerId,
    /// Layer name (debug / overlay).
    pub name: String,
    /// Position in the timeline's layer list (z-order).
    pub layer_index: usize,
    /// Resolved content.
    pub kind: EvaluatedLayerKind,
    /// Computed transform at time `t`.
    pub transform: super::ComputedTransform,
    /// Whether inside `visible_range`.
    pub visible: bool,
    /// Compositing mode.
    pub blend_mode: BlendMode,
}
