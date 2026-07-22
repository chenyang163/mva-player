//! Animation timeline model (§4.4, §5): layers, transforms, text.
//!
//! Building‑block types ([`LayerId`], [`BlendMode`], [`TextStyle`])
//! live in `mva-scene` and are re‑exported here for backwards
//! compatibility.

use serde::{Deserialize, Serialize};

pub use mva_scene::{BlendMode, LayerId, TextStyle};

use mva_types::AssetRef;

use super::track::Track;
use super::units::Vec2;

/// The motion-graphics timeline: a z-ordered stack of layers (bottom
/// first), analogous to an After Effects composition whose duration
/// equals the audio duration (§5).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AnimationTimeline {
    /// Layers in z-order, bottom first.
    #[serde(default)]
    pub layers: Vec<Layer>,
}

/// A visual object with its own transform and local timing (§5).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    /// Unique layer id.
    pub id: LayerId,
    /// Human-readable name (editor display).
    #[serde(default)]
    pub name: String,
    /// What the layer renders.
    pub kind: LayerKind,
    /// Animated transform (AE-style P/S/R/O/A, §5).
    #[serde(default)]
    pub transform: Transform,
    /// Visibility window in seconds `(start, end)`; the layer only
    /// exists in the scene inside this range.
    pub visible_range: (f64, f64),
    /// Optional parent layer for transform inheritance (§5 parenting).
    #[serde(default)]
    pub parent: Option<LayerId>,
    /// Compositing mode.
    #[serde(default)]
    pub blend_mode: BlendMode,
}

/// Layer content kinds.  `Text` (Phase 1), `Image` (Phase 3),
/// `Shape` (Phase 2), `ParticleEmitter` (Phase 3+).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LayerKind {
    /// A text layer.
    Text {
        /// Where the displayed string comes from.
        source: TextSource,
        /// How it is styled.
        style: TextStyle,
    },
    /// An image layer referencing an asset.
    Image {
        /// Asset to display.
        asset: AssetRef,
    },
}

/// Binding for a text layer's content (§5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TextSource {
    /// A fixed string.
    Static {
        /// The text.
        text: String,
    },
    /// Binds to the currently active lyric line at time `t`.
    LyricLine,
    /// Binds to the currently active karaoke word at time `t` (Phase 2
    /// evaluation).
    LyricWord,
}

/// Animated transform of a layer (AE P/S/R/O/A mapping, §5).
///
/// # Identity defaults (applied by the evaluator, NOT stored here)
///
/// - position / anchor: `(0, 0)`
/// - scale: `(1, 1)`
/// - rotation: `0` (degrees, clockwise, matching screen-space y-down)
/// - opacity: `1` (fully opaque)
///
/// An empty track means "use the identity value" — the evaluator
/// applies defaults; the model does not bake them in.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Transform {
    /// Position track.
    #[serde(default)]
    pub position: Track<Vec2>,
    /// Scale track (1.0 = 100%).
    #[serde(default)]
    pub scale: Track<Vec2>,
    /// Rotation track in degrees (clockwise; screen-space y-down).
    #[serde(default)]
    pub rotation: Track<f32>,
    /// Opacity track (0.0–1.0).
    #[serde(default)]
    pub opacity: Track<f32>,
    /// Anchor point track (pivot for scale/rotation).
    #[serde(default)]
    pub anchor: Track<Vec2>,
}
