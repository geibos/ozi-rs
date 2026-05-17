use crate::domain::{
    Track, TrackId, TrackPoint, TrackPointId, TrackSegment, TrackSegmentId, TrackStyle,
};
use std::fmt;
use std::path::Path;

const PLT_HEADER: &str = "OziExplorer Track Point File Version";
const PLT_FIXED_HEADER_LINES: usize = 6;

const BOM_UTF8: &[u8] = &[0xEF, 0xBB, 0xBF];
const BOM_UTF16_LE: &[u8] = &[0xFF, 0xFE];
const BOM_UTF16_BE: &[u8] = &[0xFE, 0xFF];

#[derive(Debug, Clone, PartialEq)]
pub struct PltImport {
    pub source_path: String,
    pub track: Track,
}

#[derive(Debug)]
pub enum PltImportError {
    Io(std::io::Error),
    MissingHeader,
    MissingTrackProperties,
    Decode(&'static str),
}

impl fmt::Display for PltImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "failed to read PLT file: {e}"),
            Self::MissingHeader => write!(f, "not an OziExplorer track file"),
            Self::MissingTrackProperties => write!(f, "missing track properties line"),
            Self::Decode(encoding) => {
                write!(f, "failed to decode PLT bytes as {encoding}")
            }
        }
    }
}

impl std::error::Error for PltImportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

/// Decode raw PLT file bytes into a UTF-8 `String` using a prioritized
/// detection chain.
///
/// Order of attempts:
/// 1. **BOM** — UTF-8 (`EF BB BF`), UTF-16 LE (`FF FE`), or UTF-16 BE
///    (`FE FF`). The BOM is stripped and the body decoded with the matching
///    `encoding_rs::Encoding`.
/// 2. **Strict UTF-8** — if the bytes are valid UTF-8 (no malformed
///    sequences), reuse them verbatim. This also handles pure ASCII.
/// 3. **`chardetng` statistical detection** — feed bytes to the detector with
///    `allow_utf8 = false`; if it returns a legacy single-byte encoding,
///    decode with it.
/// 4. **Windows-1251 fallback** — the historically correct default for
///    OziExplorer PLT files exported on Russian Windows.
///
/// The decoder must not introduce `U+FFFD` replacement characters when the
/// bytes are a valid sequence in any supported encoding; in the fallback
/// branch we still rely on `encoding_rs` to map cp1251 single-byte values,
/// which never fails (every byte has a defined codepoint).
pub fn decode_plt_bytes(bytes: &[u8]) -> Result<String, PltImportError> {
    // Step 1: BOM check. Strip the BOM and decode the body strictly so that
    // genuinely-corrupt files surface as `Decode` errors instead of silently
    // gaining `U+FFFD`.
    if bytes.starts_with(BOM_UTF8) {
        return decode_with(encoding_rs::UTF_8, &bytes[BOM_UTF8.len()..], "UTF-8 (with BOM)");
    }
    if bytes.starts_with(BOM_UTF16_LE) {
        return decode_with(encoding_rs::UTF_16LE, &bytes[BOM_UTF16_LE.len()..], "UTF-16 LE");
    }
    if bytes.starts_with(BOM_UTF16_BE) {
        return decode_with(encoding_rs::UTF_16BE, &bytes[BOM_UTF16_BE.len()..], "UTF-16 BE");
    }

    // Step 2: strict UTF-8 (also covers pure ASCII).
    if let Ok(s) = std::str::from_utf8(bytes) {
        return Ok(s.to_owned());
    }

    // Step 3: chardetng for legacy single-byte encodings.
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    let detected = detector.guess(None, false);
    // `guess` with `allow_utf8 = false` may still return UTF-8 if nothing
    // else fits; in that case its `decode_without_bom_handling_and_without_replacement`
    // would fail, so fall through to cp1251.
    if detected != encoding_rs::UTF_8
        && let Some(cow) = detected.decode_without_bom_handling_and_without_replacement(bytes)
    {
        return Ok(cow.into_owned());
    }

    // Step 4: Windows-1251 fallback.
    // cp1251 is a single-byte encoding with a mapping for every byte value,
    // so decoding is infallible.
    let (cow, _) = encoding_rs::WINDOWS_1251.decode_without_bom_handling(bytes);
    Ok(cow.into_owned())
}

fn decode_with(
    encoding: &'static encoding_rs::Encoding,
    bytes: &[u8],
    label: &'static str,
) -> Result<String, PltImportError> {
    encoding
        .decode_without_bom_handling_and_without_replacement(bytes)
        .map(|cow| cow.into_owned())
        .ok_or(PltImportError::Decode(label))
}

/// Import an OziExplorer `.plt` track file.
pub fn import_plt_file(path: &Path) -> Result<PltImport, PltImportError> {
    let bytes = std::fs::read(path).map_err(PltImportError::Io)?;
    let text = decode_plt_bytes(&bytes)?;
    import_plt_text(path.display().to_string(), &text)
}

pub fn import_plt_text(source_path: String, text: &str) -> Result<PltImport, PltImportError> {
    let lines: Vec<&str> = text.lines().map(|l| l.trim_end_matches('\r')).collect();

    // Validate header
    if lines
        .first()
        .map(|l| l.trim())
        .is_none_or(|l| !l.starts_with(PLT_HEADER))
    {
        return Err(PltImportError::MissingHeader);
    }

    // Line 5 (index 4) = track properties; line 6 = point count or 0
    let properties_line = lines.get(4).ok_or(PltImportError::MissingTrackProperties)?;
    let style = parse_track_style(properties_line);

    let track_name = parse_track_name(properties_line)
        .unwrap_or_else(|| source_path_stem(&source_path).to_owned());

    let mut track = Track::new(TrackId::new(1), track_name);
    *track.style_mut() = style;

    let mut current_segment: Option<TrackSegment> = None;
    let mut segment_id = 1u64;
    let mut point_id = 1u64;

    for line in lines.iter().skip(PLT_FIXED_HEADER_LINES) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Skip the "number of points" line if it's just a single integer
        if current_segment.is_none() && trimmed.parse::<u64>().is_ok() {
            continue;
        }

        if let Some(point) = parse_plt_point(trimmed, point_id) {
            let in_new_segment = is_new_segment(trimmed);

            if in_new_segment || current_segment.is_none() {
                if let Some(seg) = current_segment.take() {
                    track.add_segment(seg);
                }
                current_segment = Some(TrackSegment::new(TrackSegmentId::new(segment_id)));
                segment_id += 1;
            }

            current_segment.as_mut().unwrap().add_point(point);
            point_id += 1;
        }
    }

    if let Some(seg) = current_segment {
        track.add_segment(seg);
    }

    // If no segments were created the file had no valid points; add one empty segment
    // so callers can still distinguish "loaded but empty" from "not loaded".

    Ok(PltImport { source_path, track })
}

/// Parse the track style from the PLT properties line (line index 4).
///
/// Format (comma-separated):
/// `visible, line_width, colorref, name, skip_n, type, line_style, fill_color, closed, reserved`
fn parse_track_style(line: &str) -> TrackStyle {
    let fields: Vec<&str> = line.split(',').map(str::trim).collect();
    let mut style = TrackStyle::default();

    // visible: 0 = shown, 1 = hidden (OziExplorer convention)
    if let Some(v) = fields.first().and_then(|s| s.parse::<u8>().ok()) {
        style.visible = v == 0;
    }

    // line_width
    if let Some(w) = fields.get(1).and_then(|s| s.parse::<f32>().ok()) {
        style.line_width = w.max(1.0);
    }

    // color as Windows COLORREF (0x00BBGGRR)
    if let Some(colorref) = fields.get(2).and_then(|s| s.parse::<u32>().ok()) {
        style.color = colorref_to_rgba(colorref);
    }

    style
}

fn parse_track_name(line: &str) -> Option<String> {
    let name = line.split(',').nth(3)?.trim();
    if name.is_empty() {
        None
    } else {
        Some(name.to_owned())
    }
}

/// Convert a Windows COLORREF (`0x00BBGGRR`) to `[R, G, B, A]`.
fn colorref_to_rgba(colorref: u32) -> [u8; 4] {
    let r = (colorref & 0xFF) as u8;
    let g = ((colorref >> 8) & 0xFF) as u8;
    let b = ((colorref >> 16) & 0xFF) as u8;
    [r, g, b, 255]
}

fn is_new_segment(line: &str) -> bool {
    // field index 2 = segment break flag (1 = new segment)
    line.split(',')
        .nth(2)
        .and_then(|s| s.trim().parse::<u8>().ok())
        .map(|v| v == 1)
        .unwrap_or(false)
}

/// Parse a single track point line.
///
/// Standard OziExplorer format:
/// `lat, lon, code, altitude_ft, ole_date, date_text, time_text`
fn parse_plt_point(line: &str, point_id: u64) -> Option<TrackPoint> {
    let fields: Vec<&str> = line.split(',').map(str::trim).collect();
    if fields.len() < 2 {
        return None;
    }

    let lat: f64 = fields[0].parse().ok()?;
    let lon: f64 = fields[1].parse().ok()?;

    // Basic range check to reject header/property lines that sneak through
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
        return None;
    }

    let mut point = TrackPoint::new(TrackPointId::new(point_id), lat, lon);

    // Altitude in feet (field index 3)
    if let Some(alt_ft) = fields.get(3).and_then(|s| s.parse::<f64>().ok())
        && alt_ft > -777.0
    {
        point = point.with_elevation(alt_ft * 0.3048);
    }

    // Timestamp from OLE Automation date (field index 4), if non-zero
    if let Some(ole) = fields.get(4).and_then(|s| s.parse::<f64>().ok())
        && ole > 0.0
        && let Some(ts) = ole_date_to_chrono(ole)
    {
        point = point.with_timestamp(ts);
    }

    Some(point)
}

/// Convert an OLE Automation date (days since December 30, 1899) to `chrono::DateTime<Utc>`.
fn ole_date_to_chrono(ole_date: f64) -> Option<chrono::DateTime<chrono::Utc>> {
    // Unix epoch (January 1, 1970) = OLE date 25569.0
    let unix_seconds = (ole_date - 25569.0) * 86400.0;
    let secs = unix_seconds.floor() as i64;
    let nanos = ((unix_seconds - unix_seconds.floor()) * 1_000_000_000.0) as u32;
    chrono::TimeZone::timestamp_opt(&chrono::Utc, secs, nanos).single()
}

fn source_path_stem(path: &str) -> &str {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    name.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(name)
}

#[cfg(test)]
mod tests {
    use super::{
        colorref_to_rgba, decode_plt_bytes, import_plt_file, import_plt_text, ole_date_to_chrono,
    };

    fn sample_plt(color: u32, points: &str) -> String {
        format!(
            "OziExplorer Track Point File Version 2.1\nWGS 84\nAltitude is in Feet\nReserved 3\n0,2,{color},Field track,,0,0,8421376,-1,0\n0\n{points}"
        )
    }

    #[test]
    fn import_plt_text_parses_basic_track() {
        let text = sample_plt(
            255,
            "60.0,30.0,0,0,44000.5,1,0.0\n60.001,30.001,0,100,44000.51,1,30.48\n",
        );

        let import = import_plt_text("field.plt".to_owned(), &text).expect("import");

        assert_eq!(import.track.segments().len(), 1);
        assert_eq!(import.track.segments()[0].points().len(), 2);
        assert!((import.track.segments()[0].points()[0].latitude() - 60.0).abs() < 1e-9);
    }

    #[test]
    fn import_plt_text_splits_on_segment_break_flag() {
        let text = sample_plt(
            255,
            "60.0,30.0,0,0,44000.0,1,0.0\n60.001,30.001,1,0,44001.0,1,0.0\n60.002,30.002,0,0,44001.1,1,0.0\n",
        );

        let import = import_plt_text("field.plt".to_owned(), &text).expect("import");

        assert_eq!(import.track.segments().len(), 2);
        assert_eq!(import.track.segments()[0].points().len(), 1);
        assert_eq!(import.track.segments()[1].points().len(), 2);
    }

    #[test]
    fn import_plt_text_rejects_missing_header() {
        let err = import_plt_text("x.plt".to_owned(), "not a plt\nfoo\n").unwrap_err();
        assert!(matches!(err, super::PltImportError::MissingHeader));
    }

    #[test]
    fn import_plt_text_reads_track_color_from_colorref() {
        // 255 = 0x000000FF → R=255, G=0, B=0
        let text = sample_plt(255, "60.0,30.0,0,0,44000.0,1,0.0\n");
        let import = import_plt_text("t.plt".to_owned(), &text).expect("import");
        assert_eq!(import.track.style().color, [255, 0, 0, 255]);
    }

    #[test]
    fn import_plt_text_reads_track_name() {
        let text = "OziExplorer Track Point File Version 2.1\nWGS 84\nAltitude is in Feet\nReserved 3\n0,2,255,My Route,,0,0,8421376,-1,0\n0\n60.0,30.0,0,0,44000.0,1,0.0\n";
        let import = import_plt_text("t.plt".to_owned(), text).expect("import");
        assert_eq!(import.track.name(), "My Route");
    }

    #[test]
    fn import_plt_text_captures_elevation_from_altitude_feet_field() {
        // 494 feet ≈ 150.57 meters
        let text = sample_plt(255, "60.0,30.0,0,494,44000.0,01-01-2020,12:00:00\n");
        let import = import_plt_text("t.plt".to_owned(), &text).expect("import");
        let elev = import.track.segments()[0].points()[0].elevation();
        assert!((elev.unwrap() - 150.57).abs() < 0.1);
    }

    #[test]
    fn colorref_to_rgba_maps_low_byte_to_red() {
        // 0x000000FF → R=255, G=0, B=0
        assert_eq!(colorref_to_rgba(0x000000FF), [255, 0, 0, 255]);
        // 0x0000FF00 → R=0, G=255, B=0
        assert_eq!(colorref_to_rgba(0x0000FF00), [0, 255, 0, 255]);
        // 0x00FF0000 → R=0, G=0, B=255
        assert_eq!(colorref_to_rgba(0x00FF0000), [0, 0, 255, 255]);
    }

    #[test]
    fn ole_date_to_chrono_converts_known_date() {
        // OLE 25569.0 = Unix epoch = 1970-01-01T00:00:00Z
        let ts = ole_date_to_chrono(25569.0).expect("timestamp");
        assert_eq!(ts.timestamp(), 0);
    }

    #[test]
    fn ole_date_to_chrono_handles_fractional_day() {
        // 0.5 days past epoch = noon 1970-01-01
        let ts = ole_date_to_chrono(25569.5).expect("timestamp");
        assert_eq!(ts.timestamp(), 43200); // 12 * 3600
    }

    // ---------------------------------------------------------------------
    // Encoding-detection tests for `decode_plt_bytes`.
    // ---------------------------------------------------------------------

    /// "Поход 2025" encoded as Windows-1251 must decode to the exact Russian
    /// string with no U+FFFD replacement characters.
    #[test]
    fn decode_plt_bytes_cp1251_cyrillic() {
        // "Поход 2025" in cp1251:
        // П=0xCF о=0xEE х=0xF5 о=0xEE д=0xE4 ' '=0x20 '2'=0x32 '0'=0x30 '2'=0x32 '5'=0x35
        let bytes: &[u8] = &[
            0xCF, 0xEE, 0xF5, 0xEE, 0xE4, 0x20, 0x32, 0x30, 0x32, 0x35,
        ];
        let decoded = decode_plt_bytes(bytes).expect("decode cp1251");
        assert_eq!(decoded, "Поход 2025");
        assert!(!decoded.contains('\u{FFFD}'));
    }

    /// Same string as UTF-8 without a BOM must hit the strict-UTF-8 fast path.
    #[test]
    fn decode_plt_bytes_utf8_no_bom() {
        let bytes = "Поход 2025".as_bytes();
        let decoded = decode_plt_bytes(bytes).expect("decode utf-8");
        assert_eq!(decoded, "Поход 2025");
    }

    /// UTF-8 BOM must be stripped and the body decoded as UTF-8.
    #[test]
    fn decode_plt_bytes_utf8_with_bom() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice("Поход 2025".as_bytes());
        let decoded = decode_plt_bytes(&bytes).expect("decode utf-8 with bom");
        assert_eq!(decoded, "Поход 2025");
        assert!(!decoded.starts_with('\u{FEFF}'));
    }

    /// UTF-16 LE with BOM must decode correctly via step 1.
    #[test]
    fn decode_plt_bytes_utf16_le_with_bom() {
        let mut bytes = vec![0xFF, 0xFE];
        for unit in "Поход 2025".encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        let decoded = decode_plt_bytes(&bytes).expect("decode utf-16 le");
        assert_eq!(decoded, "Поход 2025");
    }

    /// Pure ASCII bytes round-trip unchanged (strict-UTF-8 path).
    #[test]
    fn decode_plt_bytes_ascii_unchanged() {
        let bytes = b"OziExplorer Track Point File Version 2.1\n";
        let decoded = decode_plt_bytes(bytes).expect("decode ascii");
        assert_eq!(decoded.as_bytes(), bytes);
    }

    /// Full path: write a cp1251 PLT file to a tempfile, import it via
    /// `import_plt_file`, assert `Track::name()` equals the original Russian
    /// string with no `U+FFFD`.
    #[test]
    fn import_plt_file_cyrillic_track_name_round_trip() {
        // Build the PLT text as a UTF-8 Rust string first, then re-encode to
        // cp1251 for the on-disk file.
        let text = concat!(
            "OziExplorer Track Point File Version 2.1\n",
            "WGS 84\n",
            "Altitude is in Feet\n",
            "Reserved 3\n",
            "0,2,255,Поход 2025,,0,0,8421376,-1,0\n",
            "0\n",
            "60.0,30.0,0,0,44000.0,1,0.0\n",
        );

        let (cp1251_bytes, _, had_errors) = encoding_rs::WINDOWS_1251.encode(text);
        assert!(!had_errors, "all chars must be cp1251-representable");

        let dir = std::env::temp_dir();
        let path = dir.join(format!(
            "ozi-rs-plt-cp1251-{}.plt",
            std::process::id()
        ));
        std::fs::write(&path, &cp1251_bytes).expect("write tempfile");

        let import = import_plt_file(&path).expect("import");
        // Clean up first so a panic still leaves a tidy temp dir.
        let _ = std::fs::remove_file(&path);

        assert_eq!(import.track.name(), "Поход 2025");
        assert!(!import.track.name().contains('\u{FFFD}'));
    }
}
