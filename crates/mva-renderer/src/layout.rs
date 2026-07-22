//! Simple layout: convert an evaluated text layer into a screen‑space
//! [`DrawCommand::Text`].

use mva_scene::{ComputedTransform, TextStyle};

use crate::draw::DrawCommand;
use crate::viewport::Viewport;

/// Convert a text layer into a single draw command.
///
/// Phase 1.6: text is centred in the viewport, offset by the layer's
/// position, scaled by the layer's scale.
pub(super) fn text_to_draw_command(
    text: &str,
    style: &TextStyle,
    xf: &ComputedTransform,
    vp: &Viewport,
) -> DrawCommand {
    let cx = vp.width / 2.0;
    let cy = vp.height / 2.0;
    let scaled_size = style.font_size * xf.scale.x.max(0.01);

    let color_arr: [u8; 4] = style.color.into();

    DrawCommand::Text {
        text: text.to_owned(),
        font_size: scaled_size,
        color: color_arr,
        x: cx + xf.position.x,
        y: cy + xf.position.y,
        opacity: xf.opacity.clamp(0.0, 1.0),
    }
}
