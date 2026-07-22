//! Lyric timeline lookup: find the active line, word, or line index
//! at time `t` (§4.3).

use crate::model::{LyricLine, LyricRole, LyricTimeline, LyricTrack};

/// The active lyric text at time `t` in the timeline, or `None` if no
/// line is currently active (gap, instrumental, or no lyrics).
///
/// **Phase 1 track selection:** picks the first `Original` track; if
/// none exists, picks the first track.  Multi-track explicit binding
/// will be added when the layer gains a `lyric_track` field (out of
/// scope for 1.x).
pub fn active_lyric_text(lyrics: &LyricTimeline, t: f64) -> Option<&str> {
    let track = choose_lyric_track(lyrics)?;
    let effective = t - track.offset;
    let (_, line) = resolve_active_line(&track.lines, effective)?;
    Some(line.text.as_str())
}

/// Index of the currently active lyric line (0-based within the
/// selected track), or `None` when no line is active.
///
/// Used by [`EngineSnapshot`](crate::eval) so the UI can highlight the
/// current line without recomputing the lookup.
pub fn active_lyric_index(lyrics: &LyricTimeline, t: f64) -> Option<usize> {
    let track = choose_lyric_track(lyrics)?;
    let effective = t - track.offset;
    resolve_active_line(&track.lines, effective).map(|(idx, _)| idx)
}

/// Find the active karaoke word within the active line (Phase 2).
pub fn active_lyric_word(lyrics: &LyricTimeline, t: f64) -> Option<&str> {
    let track = choose_lyric_track(lyrics)?;
    let effective = t - track.offset;
    let (_line_idx, line) = resolve_active_line(&track.lines, effective)?;
    let words = line.words.as_ref()?;

    let wi = words.partition_point(|w| w.start <= effective);
    if wi == 0 {
        return None;
    }
    Some(words[wi - 1].text.as_str())
}

// -------------------------------------------------------------------
// internal helpers
// -------------------------------------------------------------------

fn choose_lyric_track(lyrics: &LyricTimeline) -> Option<&LyricTrack> {
    lyrics
        .tracks
        .iter()
        .find(|tr| matches!(tr.role, LyricRole::Original))
        .or_else(|| lyrics.tracks.first())
}

/// Binary search the line list and return `(index, &line)` if the
/// line at that index is active at `effective_t`.
fn resolve_active_line(lines: &[LyricLine], effective_t: f64) -> Option<(usize, &LyricLine)> {
    let idx = lines.partition_point(|l| l.start <= effective_t);
    if idx == 0 {
        return None;
    }
    let line = &lines[idx - 1];
    let active = match line.end {
        Some(end) => effective_t < end,
        None => idx == lines.len() || effective_t < lines[idx].start,
    };
    if active { Some((idx - 1, line)) } else { None }
}
