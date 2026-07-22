//! Contract tests for the serialized data model
//! (architecture §3.4, §4, §6).
//!
//! These tests pin the **JSON shape** of the model, because that shape
//! is the contract future `.mva` artifacts, `*.anim.json` files and
//! external tools rely on.
//!
//! `#![allow(clippy::float_cmp)]` — exact equality is intentional here:
//! key values (0.5, 42.0, etc.) are exactly representable and come
//! through identical serde JSON parse paths on both sides of every
//! comparison.

#![allow(clippy::float_cmp)]

use mva_timeline::model::*;

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn sample_project() -> Project {
    let mut custom = std::collections::BTreeMap::new();
    custom.insert("genre".to_owned(), "test".to_owned());

    Project {
        metadata: ProjectMetadata {
            title: "Test Song".to_owned(),
            artist: Some("Test Artist".to_owned()),
            album: None,
            duration: Some(213.5),
            cover_image: None,
            languages: vec!["en".to_owned()],
            author: None,
            created_with: Some("mva-test 0.1.0".to_owned()),
            format_version: "1.0".to_owned(),
            id: "00000000-0000-0000-0000-000000000001".to_owned(),
            custom,
        },
        audio: AudioTimeline {
            source: AudioSource::ExternalFile {
                path: "song.mp3".to_owned(),
            },
            duration: 213.5,
            sample_rate: 44_100,
            channels: 2,
            volume_envelope: None,
        },
        lyrics: LyricTimeline {
            tracks: vec![LyricTrack {
                role: LyricRole::Original,
                language: Some("en".to_owned()),
                offset: -0.25,
                lines: vec![
                    LyricLine {
                        start: 10.0,
                        end: Some(14.0),
                        text: "Hello".to_owned(),
                        words: None,
                    },
                    LyricLine {
                        start: 14.0,
                        end: None,
                        text: "World".to_owned(),
                        words: None,
                    },
                ],
            }],
        },
        animation: AnimationTimeline {
            layers: vec![Layer {
                id: LayerId("lyric-main".to_owned()),
                name: "Main lyric".to_owned(),
                kind: LayerKind::Text {
                    source: TextSource::LyricLine,
                    style: TextStyle {
                        font_family: None,
                        font_size: 42.0,
                        color: [255, 255, 255, 255].into(),
                    },
                },
                transform: Transform {
                    opacity: Track {
                        keyframes: vec![
                            Keyframe {
                                time: 0.0,
                                value: 0.0,
                                easing: Easing::Named(NamedEase::EaseOutCubic),
                            },
                            Keyframe {
                                time: 0.4,
                                value: 1.0,
                                easing: Easing::Hold,
                            },
                        ],
                    },
                    ..Transform::default()
                },
                visible_range: (0.0, 213.5),
                parent: None,
                blend_mode: BlendMode::Normal,
            }],
        },
        effect_timeline: EffectTimeline::default(),
    }
}

// ---------------------------------------------------------------------------
// round-trip
// ---------------------------------------------------------------------------

#[test]
fn project_json_round_trip() {
    let project = sample_project();
    let json = serde_json::to_string_pretty(&project).expect("serialize");
    let back: Project = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(project, back);
}

// ---------------------------------------------------------------------------
// authored anim.json (§5 data-driven rule)
// ---------------------------------------------------------------------------

#[test]
fn authored_anim_json_parses_into_layers() {
    let json = r#"{
        "layers": [
            {
                "id": "lyric-main",
                "name": "Main lyric",
                "kind": {
                    "type": "text",
                    "source": { "type": "lyric_line" },
                    "style": { "font_size": 42.0, "color": [255, 255, 255, 255] }
                },
                "transform": {
                    "opacity": {
                        "keyframes": [
                            {
                                "time": 0.0,
                                "value": 0.0,
                                "easing": { "named": "ease_out_cubic" }
                            },
                            {
                                "time": 0.4,
                                "value": 1.0,
                                "easing": "hold"
                            }
                        ]
                    },
                    "scale": {
                        "keyframes": [
                            {
                                "time": 0.0,
                                "value": { "x": 0.9, "y": 0.9 }
                            },
                            {
                                "time": 0.4,
                                "value": { "x": 1.0, "y": 1.0 }
                            }
                        ]
                    }
                },
                "visible_range": [0.0, 213.5]
            }
        ]
    }"#;

    let timeline: AnimationTimeline = serde_json::from_str(json).unwrap();
    assert_eq!(timeline.layers.len(), 1);

    let layer = &timeline.layers[0];
    assert_eq!(layer.id, LayerId("lyric-main".to_owned()));
    assert_eq!(layer.name, "Main lyric");

    let LayerKind::Text { source, style } = &layer.kind else {
        panic!("expected Text layer");
    };
    assert_eq!(*source, TextSource::LyricLine);
    assert_eq!(style.font_size, 42.0);
    assert!(style.font_family.is_none());
    assert_eq!(style.color, [255, 255, 255, 255].into());

    // opacity track
    let opacity = &layer.transform.opacity.keyframes;
    assert_eq!(opacity.len(), 2);
    assert_eq!(opacity[0].time, 0.0);
    assert_eq!(opacity[0].value, 0.0);
    assert_eq!(opacity[0].easing, Easing::Named(NamedEase::EaseOutCubic));
    assert_eq!(opacity[1].time, 0.4);
    assert_eq!(opacity[1].value, 1.0);
    assert_eq!(opacity[1].easing, Easing::Hold);

    // scale track: missing easing defaults to Linear (§4.6)
    let scale = &layer.transform.scale.keyframes;
    assert_eq!(scale[0].easing, Easing::Linear);
    assert_eq!(scale[0].value, Vec2 { x: 0.9, y: 0.9 });
    assert_eq!(scale[1].value, Vec2 { x: 1.0, y: 1.0 });

    // defaults for fields absent from JSON
    assert_eq!(layer.parent, None);
    assert_eq!(layer.blend_mode, BlendMode::Normal);
    assert!(layer.transform.position.keyframes.is_empty());
    assert!(layer.transform.rotation.keyframes.is_empty());
    assert!(layer.transform.anchor.keyframes.is_empty());
}

// ---------------------------------------------------------------------------
// easing serialization shapes (§4.6)
// ---------------------------------------------------------------------------

#[test]
fn easing_serializes_as_specified() {
    assert_eq!(serde_json::to_string(&Easing::Hold).unwrap(), "\"hold\"");
    assert_eq!(
        serde_json::to_string(&Easing::Linear).unwrap(),
        "\"linear\""
    );
    let named_json = serde_json::to_string(&Easing::Named(NamedEase::EaseInQuad)).unwrap();
    assert_eq!(named_json, "{\"named\":\"ease_in_quad\"}");
}

#[test]
fn easing_deserializes_from_author_friendly_shapes() {
    // Phase 1 accepted shapes (§4.6).  Missing & unrecognized shapes
    // fail with a serde error — forward tolerance (§6.3) is about
    // *unknown fields*, not unknown enum variants.
    let e: Easing = serde_json::from_str("\"hold\"").unwrap();
    assert_eq!(e, Easing::Hold);

    let e: Easing = serde_json::from_str("\"linear\"").unwrap();
    assert_eq!(e, Easing::Linear);

    let e: Easing = serde_json::from_str("{\"named\":\"ease_out_cubic\"}").unwrap();
    assert_eq!(e, Easing::Named(NamedEase::EaseOutCubic));
}

// ---------------------------------------------------------------------------
// LRC-shaped lyric data (§4.3)
// ---------------------------------------------------------------------------

#[test]
fn lyric_track_matches_lrc_model() {
    // What the Phase 1 LRC parser (mva-lyrics) will produce: line-level
    // timings, the LRC `offset` tag, no word timings (§4.3).
    let json = r#"{
        "role": "original",
        "language": "en",
        "offset": 0.5,
        "lines": [
            { "start": 12.5, "text": "first line" },
            { "start": 16.0, "end": 20.0, "text": "second line", "words": null }
        ]
    }"#;

    let track: LyricTrack = serde_json::from_str(json).unwrap();
    assert_eq!(track.offset, 0.5);
    assert_eq!(track.role, LyricRole::Original);
    assert_eq!(track.language.as_deref(), Some("en"));
    assert_eq!(track.lines.len(), 2);
    assert_eq!(track.lines[0].start, 12.5);
    assert_eq!(track.lines[0].text, "first line");
    assert_eq!(track.lines[0].end, None);
    assert_eq!(track.lines[0].words, None);
    assert_eq!(track.lines[1].end, Some(20.0));
    assert_eq!(track.lines[1].words, None);
}

// ---------------------------------------------------------------------------
// serde defaults: minimal JSON → sensible defaults (§4.6, §5)
// ---------------------------------------------------------------------------

#[test]
fn missing_layer_fields_use_serde_defaults() {
    let json = r#"{
        "id": "l1",
        "kind": {
            "type": "text",
            "source": { "type": "static", "text": "hi" },
            "style": { "font_size": 20.0, "color": [0, 0, 0, 255] }
        },
        "visible_range": [1.0, 2.0]
    }"#;

    let layer: Layer = serde_json::from_str(json).unwrap();
    assert_eq!(layer.id, LayerId("l1".to_owned()));
    assert_eq!(layer.name, "");
    assert_eq!(layer.parent, None);
    assert_eq!(layer.blend_mode, BlendMode::Normal);
    assert!(layer.transform.opacity.keyframes.is_empty());
    assert!(layer.transform.position.keyframes.is_empty());
    assert!(layer.transform.scale.keyframes.is_empty());
    assert!(layer.transform.rotation.keyframes.is_empty());
    assert!(layer.transform.anchor.keyframes.is_empty());
    assert_eq!(layer.visible_range, (1.0, 2.0));
}

// ---------------------------------------------------------------------------
// forward tolerance (§6.3): unknown fields at every level are ignored
// ---------------------------------------------------------------------------

#[test]
fn unknown_project_fields_are_ignored() {
    let json = r#"{
        "metadata": { "title": "t", "future_meta": 123 },
        "audio": {
            "source": { "type": "embedded", "entry_path": "audio/main.mp3", "extra": true },
            "unused": null
        },
        "lyrics": { "tracks": [], "future": [1,2,3] },
        "animation": { "layers": [] },
        "top_level_future": { "x": 1 }
    }"#;

    let project: Project = serde_json::from_str(json).unwrap();
    assert_eq!(project.metadata.title, "t");
    assert!(
        matches!(project.audio.source, AudioSource::Embedded { .. }),
        "should parse embedded source"
    );
    assert!(project.lyrics.tracks.is_empty());
    assert!(project.animation.layers.is_empty());
}

// ---------------------------------------------------------------------------
// TextSource static + LyricWord model shapes (§5)
// ---------------------------------------------------------------------------

#[test]
fn text_source_static_round_trip() {
    let src = TextSource::Static {
        text: "Hello, World!".to_owned(),
    };
    let json = serde_json::to_string_pretty(&src).unwrap();
    assert!(json.contains("\"type\": \"static\""));
    assert!(json.contains("\"text\": \"Hello, World!\""));
    let back: TextSource = serde_json::from_str(&json).unwrap();
    assert_eq!(src, back);
}

#[test]
fn text_source_unit_variants_round_trip() {
    // Internally-tagged enums (serde(tag = "type")) emit `{"type":…}`
    // even for unit variants — this is the consistent JSON shape for
    // all TextSource variants (§5).
    for (src, expected_json) in [
        (TextSource::LyricLine, r#"{"type":"lyric_line"}"#),
        (TextSource::LyricWord, r#"{"type":"lyric_word"}"#),
    ] {
        let json = serde_json::to_string(&src).unwrap();
        assert_eq!(json, expected_json);
        let back: TextSource = serde_json::from_str(&json).unwrap();
        assert_eq!(src, back);
    }
}

// ---------------------------------------------------------------------------
// deterministic serialization: BTreeMap keys
// ---------------------------------------------------------------------------

#[test]
fn custom_metadata_serializes_with_sorted_keys() {
    let mut meta = ProjectMetadata::default();
    meta.custom.insert("b".into(), "2".into());
    meta.custom.insert("a".into(), "1".into());
    meta.custom.insert("c".into(), "3".into());

    let json = serde_json::to_string(&meta).unwrap();

    let pos_a = json.find("\"a\"").unwrap();
    let pos_b = json.find("\"b\"").unwrap();
    let pos_c = json.find("\"c\"").unwrap();
    assert!(pos_a < pos_b, "BTreeMap should serialize sorted");
    assert!(pos_b < pos_c, "BTreeMap should serialize sorted");
}

// ---------------------------------------------------------------------------
// Audio timeline / source variants (§4.2)
// ---------------------------------------------------------------------------

#[test]
fn audio_source_embedded_round_trip() {
    let json = r#"{"type":"embedded","entry_path":"audio/main.flac"}"#;
    let src: AudioSource = serde_json::from_str(json).unwrap();
    assert!(
        matches!(src, AudioSource::Embedded { ref entry_path } if entry_path == "audio/main.flac")
    );
    let back = serde_json::to_string(&src).unwrap();
    let src2: AudioSource = serde_json::from_str(&back).unwrap();
    assert_eq!(src, src2);
}

#[test]
fn audio_source_external_file_round_trip() {
    let src = AudioSource::ExternalFile {
        path: "C:/music/song.mp3".to_owned(),
    };
    let json = serde_json::to_string(&src).unwrap();
    assert!(json.contains("\"external_file\""));
    let back: AudioSource = serde_json::from_str(&json).unwrap();
    assert_eq!(src, back);
}
