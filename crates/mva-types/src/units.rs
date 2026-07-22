/// 2D vector for spatial values (position, scale, anchor).
///
/// Defined **in** `mva-types` to keep this crate a leaf (§6.2).
/// `mva-timeline::eval` converts `mva_types::Vec2` →
/// `mva_scene::Vec2` at the scene boundary.
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
