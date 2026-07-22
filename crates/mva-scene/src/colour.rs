//! RGBA colour.

use serde::{Deserialize, Serialize};

/// 8‑bit RGBA colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rgba {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
    /// Alpha channel (0 = fully transparent, 255 = fully opaque).
    pub a: u8,
}

impl Rgba {
    /// Opaque white — common default.
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
}

impl From<[u8; 4]> for Rgba {
    fn from(v: [u8; 4]) -> Self {
        Self {
            r: v[0],
            g: v[1],
            b: v[2],
            a: v[3],
        }
    }
}

impl From<Rgba> for [u8; 4] {
    fn from(c: Rgba) -> Self {
        [c.r, c.g, c.b, c.a]
    }
}
