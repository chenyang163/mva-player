//! Simple culling: skip layers that are entirely outside the
//! viewport.

use mva_scene::ComputedTransform;

use crate::viewport::Viewport;

/// Returns `true` when the layer is definitely off‑screen and can
/// be skipped.
pub(super) fn off_viewport(xf: &ComputedTransform, vp: &Viewport) -> bool {
    let margin = 200.0;
    let x = xf.position.x;
    let y = xf.position.y;
    x < -margin || x > vp.width + margin || y < -margin || y > vp.height + margin
}
