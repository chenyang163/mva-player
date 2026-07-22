//! Runtime viewport — the drawable area that may change on window
//! resize.  Passed to [`Renderer::render`](crate::Renderer::render)
//! every frame.

/// Runtime window geometry — separate from static [`RendererConfig`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    /// Width in pixels.
    pub width: f32,
    /// Height in pixels.
    pub height: f32,
}
