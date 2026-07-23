//! Built-in `--demo` showcase project (Phase 4).
//!
//! Synthetic project exercising all Phase 3 data types (Text layer +
//! Image layer + Effect).  Accessed via `mva-player --demo`.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use mva_timeline::model::*;
use mva_types::{AssetRef, EffectInstance, EffectParam, EffectTarget, EffectTimeline, ParamValue};

/// Path to the generated test image (written at startup).
const DEMO_IMG: &str = "assets/phase3_demo.ppm";

/// Generate a small 128×64 blue‑gradient PPM image for the demo
/// Image layer.  Returns the absolute path to the file.
pub fn ensure_demo_image() -> PathBuf {
    let path = PathBuf::from(DEMO_IMG);
    if path.exists() {
        return path;
    }
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let w: u32 = 128;
    let h: u32 = 64;
    let mut buf = Vec::new();
    // PPM P6 header (binary RGB).
    write!(buf, "P6\n{w} {h}\n255\n").unwrap();
    for y in 0..h {
        for x in 0..w {
            let r = (x as u8).wrapping_mul(2);
            let g = (y as u8).wrapping_mul(4);
            let b = 180u8;
            buf.push(r);
            buf.push(g);
            buf.push(b);
        }
    }
    fs::write(&path, &buf).expect("write demo image");
    path
}

/// Build a Phase 3 demo [`Project`] with:
/// - One [`LayerKind::Text`] layer (bound to lyrics).
/// - One [`LayerKind::Image`] layer (AssetRef::File to the demo PNG).
/// - One [`EffectTimeline`] entry (bloom, whole‑scene, animated opacity).
pub fn make_demo_project() -> Project {
    let img_path = ensure_demo_image();

    Project {
        metadata: ProjectMetadata {
            title: "Phase 3 Demo".into(),
            artist: Some("MVA Player".into()),
            duration: Some(30.0),
            languages: vec!["en".into()],
            format_version: "1.0".into(),
            id: "phase3-demo".into(),
            ..Default::default()
        },
        audio: AudioTimeline {
            source: AudioSource::ExternalFile {
                path: "(sine)".into(),
            },
            duration: 30.0,
            sample_rate: 44100,
            channels: 2,
            volume_envelope: None,
        },
        lyrics: LyricTimeline {
            tracks: vec![LyricTrack {
                role: LyricRole::Original,
                language: Some("en".into()),
                offset: 0.0,
                lines: vec![
                    LyricLine {
                        start: 1.0,
                        end: Some(5.0),
                        text: "Hello, Phase 3!".into(),
                        words: None,
                    },
                    LyricLine {
                        start: 5.0,
                        end: Some(9.0),
                        text: "Text + Image + Effects".into(),
                        words: None,
                    },
                    LyricLine {
                        start: 9.0,
                        end: Some(13.0),
                        text: "All data types verified".into(),
                        words: None,
                    },
                    LyricLine {
                        start: 13.0,
                        end: None,
                        text: "End‑to‑end pipeline ok".into(),
                        words: None,
                    },
                ],
            }],
        },
        animation: AnimationTimeline {
            layers: vec![
                // ---- Text layer ----
                Layer {
                    id: LayerId("lyric-text".into()),
                    name: "Lyric".into(),
                    kind: LayerKind::Text {
                        source: TextSource::LyricLine,
                        style: TextStyle {
                            font_family: None,
                            font_size: 36.0,
                            color: [255, 255, 255, 255].into(),
                        },
                    },
                    transform: Transform {
                        opacity: Track {
                            keyframes: vec![
                                Keyframe {
                                    time: 0.5,
                                    value: 0.0,
                                    easing: Easing::Named(NamedEase::EaseOutCubic),
                                },
                                Keyframe {
                                    time: 1.0,
                                    value: 1.0,
                                    easing: Easing::Hold,
                                },
                            ],
                        },
                        scale: Track {
                            keyframes: vec![
                                Keyframe {
                                    time: 0.5,
                                    value: Vec2 { x: 0.95, y: 0.95 },
                                    easing: Easing::Named(NamedEase::EaseOutCubic),
                                },
                                Keyframe {
                                    time: 1.0,
                                    value: Vec2 { x: 1.0, y: 1.0 },
                                    easing: Easing::Hold,
                                },
                            ],
                        },
                        ..Default::default()
                    },
                    visible_range: (0.0, 30.0),
                    parent: None,
                    blend_mode: BlendMode::Normal,
                },
                // ---- Image layer ----
                Layer {
                    id: LayerId("img-layer".into()),
                    name: "Demo Image".into(),
                    kind: LayerKind::Image {
                        asset: AssetRef::File {
                            path: img_path.to_string_lossy().into_owned(),
                        },
                    },
                    transform: Transform {
                        position: Track {
                            keyframes: vec![Keyframe {
                                time: 0.0,
                                value: Vec2 { x: 0.0, y: 80.0 },
                                easing: Easing::Hold,
                            }],
                        },
                        scale: Track {
                            keyframes: vec![Keyframe {
                                time: 0.0,
                                value: Vec2 { x: 1.0, y: 1.0 },
                                easing: Easing::Hold,
                            }],
                        },
                        opacity: Track {
                            keyframes: vec![Keyframe {
                                time: 0.0,
                                value: 0.3,
                                easing: Easing::Hold,
                            }],
                        },
                        ..Default::default()
                    },
                    visible_range: (0.0, 30.0),
                    parent: None,
                    blend_mode: BlendMode::Normal,
                },
            ],
        },
        effect_timeline: EffectTimeline {
            effects: vec![EffectInstance {
                time_range: (0.0, 30.0),
                effect_id: "debug_demo".into(),
                target: EffectTarget::WholeScene,
                parameters: vec![EffectParam {
                    name: "opacity".into(),
                    track: Track {
                        keyframes: vec![
                            Keyframe {
                                time: 0.0,
                                value: ParamValue::Float { value: 0.0 },
                                easing: Easing::Linear,
                            },
                            Keyframe {
                                time: 2.0,
                                value: ParamValue::Float { value: 1.0 },
                                easing: Easing::Hold,
                            },
                        ],
                    },
                }],
            }],
        },
    }
}

/// Build a 30‑second 440 Hz sine wave source.
pub fn make_demo_sine() -> impl rodio::source::Source<Item = f32> + Send + 'static {
    use rodio::source::{SineWave, Source};
    SineWave::new(440.0).take_duration(std::time::Duration::from_secs_f32(30.0))
}
