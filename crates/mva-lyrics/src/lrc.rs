//! LRC file parser.
//!
//! Parses standard line-timed LRC files into a
//! [`LyricTimeline`](mva_timeline::model::LyricTimeline).  Time tags
//! without trailing text (e.g. `[01:23.45]`) are treated as blank
//! lines.  Only **the last timestamp** on a repeated-tag line is used.

use mva_timeline::model::{LyricLine, LyricTimeline, LyricTrack};
use regex::Regex;

use crate::error::LyricError;

/// Internal line data scraped from the LRC file before sorting.
struct ScrapedLine {
    /// Start timestamp of the last `[mm:ss.xx]` tag on the line.
    start: f64,
    /// The text portion of the line (after the last timestamp).
    text: String,
}

/// Parse an LRC string into a [`LyricTimeline`].
///
/// Lines without a recognised timestamp tag are ignored.  Blank text
/// carries an empty string.  The track is marked as [`LyricRole::Original`].
pub fn parse_lrc(content: &str, offset_adjust: f64) -> Result<LyricTimeline, LyricError> {
    let offset_tag = extract_offset(content);
    let combined_offset = offset_tag + offset_adjust;

    let raw_lines = extract_lines(content)?;

    let lines = to_lyric_lines(raw_lines, combined_offset);

    Ok(LyricTimeline {
        tracks: vec![LyricTrack {
            offset: combined_offset,
            lines,
            ..Default::default()
        }],
    })
}

/// Regex: `[mm:ss.xx]` or `[mm:ss.xxx]` — the standard LRC timestamp.
fn timestamp_regex() -> Regex {
    Regex::new(r"\[(\d{1,3}):(\d{2})(?:[.,](\d{2,3}))?\]").expect("timestamp regex is valid")
}

/// Scan for the `[offset:…]` tag.  Returns the offset in seconds, or
/// `0.0` if the tag is missing or unparseable.
fn extract_offset(content: &str) -> f64 {
    let re = Regex::new(r"\[offset:\s*([+-]?\d+)\]").expect("offset regex is valid");
    if let Some(caps) = re.captures(content) {
        if let Ok(ms) = caps[1].parse::<i64>() {
            return ms as f64 / 1000.0;
        }
    }
    0.0
}

/// Collect every line that carries at least one timestamp into a
/// [`ScrapedLine`], extracting the timestamp and text.
fn extract_lines(content: &str) -> Result<Vec<ScrapedLine>, LyricError> {
    let re = timestamp_regex();
    let mut result = Vec::new();

    for (line_no, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        let mut last_ts: Option<f64> = None;
        let mut last_end: usize = 0;

        // Scan all `[mm:ss.xx]` matches; last one wins.
        for caps in re.captures_iter(line) {
            let minutes: f64 = caps[1]
                .parse()
                .map_err(|_| LyricError::Parse {
                    line: line_no + 1,
                    message: "invalid minutes in timestamp".into(),
                })?;
            let seconds: f64 = caps[2]
                .parse()
                .map_err(|_| LyricError::Parse {
                    line: line_no + 1,
                    message: "invalid seconds in timestamp".into(),
                })?;
            let centis = if let Some(cs) = caps.get(3) {
                let raw = cs.as_str();
                // 2 digits = centiseconds; 3 digits = milliseconds
                if raw.len() == 2 {
                    raw.parse::<f64>().map_err(|_| LyricError::Parse {
                        line: line_no + 1,
                        message: "invalid centiseconds in timestamp".into(),
                    })? / 100.0
                } else {
                    raw.parse::<f64>().map_err(|_| LyricError::Parse {
                        line: line_no + 1,
                        message: "invalid milliseconds in timestamp".into(),
                    })? / 1000.0
                }
            } else {
                0.0
            };
            let ts = minutes * 60.0 + seconds + centis;
            last_ts = Some(ts);
            last_end = caps.get(0).unwrap().end();
        }

        if let Some(start) = last_ts {
            let text = line[last_end..].trim().to_owned();
            result.push(ScrapedLine { start, text });
        }
    }

    Ok(result)
}

/// Sort scraped lines, compute `end` timestamps, and produce
/// [`LyricLine`]s with the combined offset applied.
fn to_lyric_lines(mut scraped: Vec<ScrapedLine>, offset: f64) -> Vec<LyricLine> {
    scraped.sort_unstable_by(|a, b| a.start.total_cmp(&b.start));

    let mut lines = Vec::with_capacity(scraped.len());
    for i in 0..scraped.len() {
        let adjusted = scraped[i].start + offset;
        let end = scraped.get(i + 1).map(|n| n.start + offset);
        lines.push(LyricLine {
            start: adjusted,
            end,
            text: std::mem::take(&mut scraped[i].text),
            ..Default::default()
        });
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_single_line() {
        let lrc = "[00:01.50] hello world\n";
        let timeline = parse_lrc(lrc, 0.0).unwrap();
        let track = &timeline.tracks[0];
        assert_eq!(track.lines.len(), 1);
        assert_eq!(track.lines[0].start, 1.5);
        assert_eq!(track.lines[0].text, "hello world");
    }

    #[test]
    fn offset_tag() {
        let lrc = "[offset:+1000]\n[00:05.00] later\n";
        let timeline = parse_lrc(lrc, 0.0).unwrap();
        // offset +1000ms = +1.0s; line at 5.0 → 6.0
        assert!((timeline.tracks[0].lines[0].start - 6.0).abs() < 0.01);
    }

    #[test]
    fn three_digit_millis() {
        let lrc = "[00:02.100] three digits\n";
        let timeline = parse_lrc(lrc, 0.0).unwrap();
        assert!((timeline.tracks[0].lines[0].start - 2.1).abs() < 0.001);
    }

    #[test]
    fn end_timestamps() {
        let lrc = "[00:01.00] first\n[00:03.00] second\n[00:05.00] third\n";
        let timeline = parse_lrc(lrc, 0.0).unwrap();
        let lines = &timeline.tracks[0].lines;
        assert_eq!(lines.len(), 3);
        assert!((lines[0].end.unwrap() - 3.0).abs() < 0.001);
        assert!((lines[1].end.unwrap() - 5.0).abs() < 0.001);
        assert!(lines[2].end.is_none());
    }
}
