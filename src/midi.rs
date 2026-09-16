//! Real Standard MIDI File (`.mid`) ingestion — no dependencies.
//!
//! The core crate works on hand-built [`NoteEvent`]s. This module is the seam
//! that lets a persona digest *actual* MIDI files, so a playtester can point it
//! at real music instead of typing note tuples. It is a small, self-contained
//! SMF parser (formats 0 and 1) — pure Rust, no external crates, and it never
//! panics on malformed input (every path returns a [`Result`]).
//!
//! It reduces polyphony to a single time-ordered stream (chords arpeggiate by
//! start order; simultaneous notes get a zero gap), which is exactly the
//! monophonic "phrase" shape the embedding model expects. Timing is rescaled to
//! the crate's convention of **480 ticks per quarter note**, so `Duration`'s
//! helpers stay meaningful regardless of the file's own division.
//!
//! ```no_run
//! use musician_soul::midi;
//! use std::fs;
//!
//! let bytes = fs::read("solo.mid").expect("read file");
//! let phrases = midi::phrases_from_smf(&bytes, "trumpet", "solo.mid")
//!     .expect("parse MIDI");
//! println!("{} phrases", phrases.len());
//! ```
//!
//! For the fleet's tensor-domain representation of MIDI (timing as tensors for
//! agent-dialogue cadence), see the sibling crate
//! [tensor-midi](https://github.com/SuperInstance/tensor-midi); this module is
//! the universal file on-ramp that needs no such dependency.

use crate::{split_phrases, Duration, NoteEvent, Phrase, Pitch, Velocity};

/// The crate's tick convention: 480 ticks per quarter note.
const TARGET_TPQN: u32 = 480;

/// Why a MIDI parse failed. Malformed files produce these instead of panicking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MidiError {
    /// The file did not start with a valid `MThd` header chunk.
    NotSmf,
    /// The byte stream ended in the middle of a structure.
    Truncated,
    /// A construct this minimal parser doesn't support (e.g. SMPTE timing).
    Unsupported(String),
}

impl std::fmt::Display for MidiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MidiError::NotSmf => write!(f, "not a Standard MIDI File (missing MThd header)"),
            MidiError::Truncated => write!(f, "MIDI data ended unexpectedly"),
            MidiError::Unsupported(s) => write!(f, "unsupported MIDI construct: {s}"),
        }
    }
}

impl std::error::Error for MidiError {}

/// A note collected during parsing, in the file's own tick units.
struct RawNote {
    start: u64,
    end: u64,
    pitch: u8,
    velocity: u8,
}

/// Parse a Standard MIDI File into a single time-ordered stream of note events.
///
/// Note-ons are matched to their note-offs (a note-on with velocity 0 counts as
/// a note-off, per the spec). Timing is rescaled to 480 ticks/quarter. The
/// resulting `tick_offset` on each event is the **rest before it** — the gap
/// since the previous note ended, clamped at zero — which is what the phrase
/// model's rest/space features expect.
pub fn parse_smf(bytes: &[u8]) -> Result<Vec<NoteEvent>, MidiError> {
    let mut r = Reader::new(bytes);

    // ── Header chunk: "MThd" len(=6) format ntracks division ──
    // Too-short or wrong-magic input is "not an SMF"; only a valid magic
    // followed by a truncated body counts as Truncated.
    match r.take(4) {
        Ok(magic) if magic == b"MThd" => {}
        _ => return Err(MidiError::NotSmf),
    }
    let header_len = r.u32()?;
    if header_len < 6 {
        return Err(MidiError::Truncated);
    }
    let _format = r.u16()?;
    let _ntracks = r.u16()?;
    let division = r.u16()?;
    // Skip any extra header bytes some writers add.
    r.skip((header_len - 6) as usize)?;

    if division & 0x8000 != 0 {
        return Err(MidiError::Unsupported("SMPTE timecode division".into()));
    }
    let tpqn = (division & 0x7fff) as u32;
    if tpqn == 0 {
        return Err(MidiError::Unsupported("zero ticks-per-quarter".into()));
    }

    // ── Collect notes across all tracks (absolute ticks) ──
    let mut notes: Vec<RawNote> = Vec::new();
    // Open note-ons keyed by (channel, pitch) → (start_tick, velocity).
    // A Vec is fine: MIDI has at most 16*128 possible keys and phrases are small.
    let mut open: Vec<(u8, u8, u64, u8)> = Vec::new();

    while !r.at_end() {
        // Each chunk: 4-byte id + u32 length. Non-track chunks are skipped.
        let id = match r.take(4) {
            Ok(id) => id.to_vec(),
            Err(_) => break, // trailing padding: stop cleanly
        };
        let len = r.u32()? as usize;
        if id != b"MTrk" {
            r.skip(len)?;
            continue;
        }
        let track_end = r.pos + len;
        if track_end > r.data.len() {
            return Err(MidiError::Truncated);
        }

        let mut abs: u64 = 0;
        let mut running_status: u8 = 0;
        while r.pos < track_end {
            let delta = r.vlq()?;
            abs += delta as u64;

            let mut status = r.peek()?;
            if status & 0x80 == 0 {
                // Running status: reuse the previous status byte.
                if running_status == 0 {
                    return Err(MidiError::Truncated);
                }
                status = running_status;
            } else {
                r.pos += 1; // consume the status byte
                if status < 0xf0 {
                    running_status = status;
                }
            }

            match status {
                0xf0 | 0xf7 => {
                    // SysEx: a VLQ length then that many bytes.
                    let n = r.vlq()? as usize;
                    r.skip(n)?;
                }
                0xff => {
                    // Meta event: type byte + VLQ length + data.
                    let _meta_type = r.u8()?;
                    let n = r.vlq()? as usize;
                    r.skip(n)?;
                }
                _ => {
                    let kind = status & 0xf0;
                    let channel = status & 0x0f;
                    match kind {
                        0x80 => {
                            // Note off.
                            let pitch = r.u8()?;
                            let _vel = r.u8()?;
                            close_note(&mut open, &mut notes, channel, pitch, abs);
                        }
                        0x90 => {
                            let pitch = r.u8()?;
                            let vel = r.u8()?;
                            if vel == 0 {
                                close_note(&mut open, &mut notes, channel, pitch, abs);
                            } else {
                                open.push((channel, pitch, abs, vel));
                            }
                        }
                        // Two-data-byte messages we don't use.
                        0xa0 | 0xb0 | 0xe0 => {
                            r.u8()?;
                            r.u8()?;
                        }
                        // One-data-byte messages.
                        0xc0 | 0xd0 => {
                            r.u8()?;
                        }
                        _ => return Err(MidiError::Unsupported(format!("status 0x{status:02x}"))),
                    }
                }
            }
        }
        // Any notes still open at end-of-track end there.
        for (_c, pitch, start, vel) in open.drain(..) {
            notes.push(RawNote {
                start,
                end: abs,
                pitch,
                velocity: vel,
            });
        }
        r.pos = track_end;
    }

    // ── Reduce to a monophonic, time-ordered NoteEvent stream ──
    notes.sort_by(|a, b| a.start.cmp(&b.start).then(b.pitch.cmp(&a.pitch)));
    let scale = |t: u64| -> u32 { ((t * TARGET_TPQN as u64) / tpqn as u64) as u32 };

    let mut events = Vec::with_capacity(notes.len());
    let mut prev_end: u64 = 0;
    for (i, n) in notes.iter().enumerate() {
        let gap = if i == 0 {
            0
        } else {
            scale(n.start.saturating_sub(prev_end))
        };
        let dur = scale(n.end.saturating_sub(n.start)).max(1); // at least one tick
        events.push(NoteEvent {
            pitch: Pitch(n.pitch),
            velocity: Velocity(n.velocity),
            duration: Duration(dur),
            tick_offset: gap,
        });
        prev_end = prev_end.max(n.end);
    }
    Ok(events)
}

/// Parse an SMF and split it into phrases at rest boundaries.
///
/// Convenience wrapper: [`parse_smf`] followed by [`split_phrases`].
pub fn phrases_from_smf(
    bytes: &[u8],
    instrument: &str,
    source: &str,
) -> Result<Vec<Phrase>, MidiError> {
    let events = parse_smf(bytes)?;
    Ok(split_phrases(&events, instrument, source))
}

fn close_note(
    open: &mut Vec<(u8, u8, u64, u8)>,
    notes: &mut Vec<RawNote>,
    channel: u8,
    pitch: u8,
    end: u64,
) {
    // Match the most recent still-open note-on for this (channel, pitch).
    if let Some(idx) = open
        .iter()
        .rposition(|&(c, p, _, _)| c == channel && p == pitch)
    {
        let (_c, p, start, vel) = open.remove(idx);
        notes.push(RawNote {
            start,
            end,
            pitch: p,
            velocity: vel,
        });
    }
    // A note-off with no matching note-on is ignored (some files do this).
}

/// A tiny, bounds-checked byte cursor. Every read is fallible; nothing panics.
struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
    fn at_end(&self) -> bool {
        self.pos >= self.data.len()
    }
    fn peek(&self) -> Result<u8, MidiError> {
        self.data.get(self.pos).copied().ok_or(MidiError::Truncated)
    }
    fn u8(&mut self) -> Result<u8, MidiError> {
        let b = self.peek()?;
        self.pos += 1;
        Ok(b)
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], MidiError> {
        let end = self.pos.checked_add(n).ok_or(MidiError::Truncated)?;
        let slice = self.data.get(self.pos..end).ok_or(MidiError::Truncated)?;
        self.pos = end;
        Ok(slice)
    }
    fn skip(&mut self, n: usize) -> Result<(), MidiError> {
        self.take(n).map(|_| ())
    }
    fn u16(&mut self) -> Result<u16, MidiError> {
        let s = self.take(2)?;
        Ok(u16::from_be_bytes([s[0], s[1]]))
    }
    fn u32(&mut self) -> Result<u32, MidiError> {
        let s = self.take(4)?;
        Ok(u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
    }
    /// MIDI variable-length quantity: 7 bits per byte, high bit = continue.
    fn vlq(&mut self) -> Result<u32, MidiError> {
        let mut value: u32 = 0;
        for _ in 0..4 {
            let b = self.u8()?;
            value = (value << 7) | (b & 0x7f) as u32;
            if b & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err(MidiError::Unsupported(
            "variable-length quantity too long".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal format-0 SMF in memory so tests need no fixture files.
    /// `div` = ticks per quarter. `events` = (delta, status, d1, d2) triples;
    /// pass d2 = 0 for one-data-byte messages (unused here).
    fn build_smf(div: u16, track: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"MThd");
        out.extend_from_slice(&6u32.to_be_bytes());
        out.extend_from_slice(&0u16.to_be_bytes()); // format 0
        out.extend_from_slice(&1u16.to_be_bytes()); // 1 track
        out.extend_from_slice(&div.to_be_bytes());
        out.extend_from_slice(b"MTrk");
        out.extend_from_slice(&(track.len() as u32).to_be_bytes());
        out.extend_from_slice(track);
        out
    }

    /// Encode a delta-time as a MIDI variable-length quantity.
    fn vlq(mut n: u32) -> Vec<u8> {
        let mut buf = vec![(n & 0x7f) as u8];
        n >>= 7;
        while n > 0 {
            buf.push(((n & 0x7f) as u8) | 0x80);
            n >>= 7;
        }
        buf.reverse();
        buf
    }
    fn note_on(delta: u32, pitch: u8, vel: u8) -> Vec<u8> {
        let mut v = vlq(delta);
        v.extend_from_slice(&[0x90, pitch, vel]);
        v
    }
    fn note_off(delta: u32, pitch: u8) -> Vec<u8> {
        let mut v = vlq(delta);
        v.extend_from_slice(&[0x80, pitch, 0]);
        v
    }
    const END_OF_TRACK: [u8; 4] = [0x00, 0xff, 0x2f, 0x00];

    #[test]
    fn rejects_non_smf() {
        assert_eq!(parse_smf(b"not midi at all"), Err(MidiError::NotSmf));
    }

    #[test]
    fn empty_input_is_not_smf() {
        assert_eq!(parse_smf(&[]), Err(MidiError::NotSmf));
    }

    #[test]
    fn parses_two_sequential_notes() {
        // 480 TPQN so no rescaling. C4 for a quarter, then E4 a quarter later.
        let mut track = Vec::new();
        track.extend(note_on(0, 60, 100)); // C4 at t=0
        track.extend(note_off(480, 60)); // off at t=480 (quarter note long)
        track.extend(note_on(0, 64, 90)); // E4 immediately after
        track.extend(note_off(240, 64)); // off at t=720 (eighth note)
        track.extend_from_slice(&END_OF_TRACK);
        let smf = build_smf(480, &track);

        let events = parse_smf(&smf).expect("parse");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].pitch.0, 60);
        assert_eq!(events[0].velocity.0, 100);
        assert_eq!(events[0].duration.0, 480);
        assert_eq!(events[0].tick_offset, 0); // first note, no leading gap
        assert_eq!(events[1].pitch.0, 64);
        assert_eq!(events[1].duration.0, 240);
        assert_eq!(events[1].tick_offset, 0); // no rest between them
    }

    #[test]
    fn computes_rest_gap_between_notes() {
        // C4 quarter, then a quarter-note rest, then E4.
        let mut track = Vec::new();
        track.extend(note_on(0, 60, 100));
        track.extend(note_off(480, 60)); // ends at 480
        track.extend(note_on(480, 64, 90)); // starts at 960 → 480-tick rest
        track.extend(note_off(240, 64));
        track.extend_from_slice(&END_OF_TRACK);
        let events = parse_smf(&build_smf(480, &track)).expect("parse");
        assert_eq!(events.len(), 2);
        assert_eq!(events[1].tick_offset, 480); // the rest is captured
    }

    #[test]
    fn rescales_division_to_480() {
        // 96 TPQN: a "quarter note" is 96 ticks and must scale to 480.
        let mut track = Vec::new();
        track.extend(note_on(0, 60, 100));
        track.extend(note_off(96, 60));
        track.extend_from_slice(&END_OF_TRACK);
        let events = parse_smf(&build_smf(96, &track)).expect("parse");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].duration.0, 480); // 96 * 480 / 96
        assert!(events[0].duration.is_long());
    }

    #[test]
    fn note_on_velocity_zero_is_note_off() {
        // Running-status-free note-off via note-on vel 0.
        let mut track = Vec::new();
        track.extend(note_on(0, 62, 80));
        track.extend(note_on(240, 62, 0)); // vel 0 = off at t=240
        track.extend_from_slice(&END_OF_TRACK);
        let events = parse_smf(&build_smf(480, &track)).expect("parse");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].duration.0, 240);
    }

    #[test]
    fn handles_running_status() {
        // Two note-ons sharing one 0x90 status byte (running status), then offs.
        let mut track = Vec::new();
        track.extend_from_slice(&[0x00, 0x90, 60, 100]); // note-on C4
        track.extend_from_slice(&[0x00, 64, 100]); // running status: note-on E4
        track.extend_from_slice(&[0x81, 0x70, 0x80, 60, 0]); // delta VLQ 0xF0=240, off C4
        track.extend_from_slice(&[0x00, 0x80, 64, 0]); // off E4
        track.extend_from_slice(&END_OF_TRACK);
        let events = parse_smf(&build_smf(480, &track)).expect("parse");
        // Two notes started at t=0 (a chord); both captured.
        assert_eq!(events.len(), 2);
        assert!(events.iter().any(|e| e.pitch.0 == 60));
        assert!(events.iter().any(|e| e.pitch.0 == 64));
        // Simultaneous starts → the second has zero gap.
        assert_eq!(events[1].tick_offset, 0);
    }

    #[test]
    fn skips_meta_and_program_change() {
        let mut track = Vec::new();
        // Meta: track name "Hi" (0xff 0x03 len data).
        track.extend_from_slice(&[0x00, 0xff, 0x03, 0x02, b'H', b'i']);
        // Program change (one data byte) — must be consumed correctly.
        track.extend_from_slice(&[0x00, 0xc0, 0x38]);
        track.extend(note_on(0, 67, 90));
        track.extend(note_off(480, 67));
        track.extend_from_slice(&END_OF_TRACK);
        let events = parse_smf(&build_smf(480, &track)).expect("parse");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].pitch.0, 67);
    }

    #[test]
    fn phrases_from_smf_splits_on_long_rest() {
        // Two notes, a long rest, then two more → two phrases.
        let mut track = Vec::new();
        track.extend(note_on(0, 60, 80));
        track.extend(note_off(240, 60));
        track.extend(note_on(0, 62, 80));
        track.extend(note_off(240, 62));
        track.extend(note_on(960, 64, 80)); // big rest → new phrase
        track.extend(note_off(240, 64));
        track.extend(note_on(0, 65, 80));
        track.extend(note_off(240, 65));
        track.extend_from_slice(&END_OF_TRACK);
        let phrases =
            phrases_from_smf(&build_smf(480, &track), "piano", "test.mid").expect("parse");
        assert_eq!(phrases.len(), 2);
        assert_eq!(phrases[0].instrument, "piano");
        assert_eq!(phrases[0].source, "test.mid");
    }

    #[test]
    fn truncated_header_errors_gracefully() {
        assert_eq!(parse_smf(b"MThd\x00\x00\x00"), Err(MidiError::Truncated));
    }

    #[test]
    fn truncated_track_body_errors_gracefully() {
        // Declares an MTrk of 100 bytes but provides only a few.
        let mut smf = Vec::new();
        smf.extend_from_slice(b"MThd");
        smf.extend_from_slice(&6u32.to_be_bytes());
        smf.extend_from_slice(&0u16.to_be_bytes());
        smf.extend_from_slice(&1u16.to_be_bytes());
        smf.extend_from_slice(&480u16.to_be_bytes());
        smf.extend_from_slice(b"MTrk");
        smf.extend_from_slice(&100u32.to_be_bytes());
        smf.extend_from_slice(&[0x00, 0x90, 60]); // truncated
        assert_eq!(parse_smf(&smf), Err(MidiError::Truncated));
    }
}
