//! OziExplorer Waypoint File (.wpt) v1.1 writer.
//!
//! Format reference (OziExplorer manual, "Waypoint File"):
//! - 4-line header
//! - one comma-separated row per waypoint, 24 fields
//! - Windows-1251 encoding (Cyrillic locale matches existing OziExplorer assets)
//! - CRLF line endings
//!
//! Waypoints in this domain only carry `(name, symbol, latitude, longitude)`;
//! elevation/timestamp are not modelled. Missing values use OziExplorer defaults
//! (date `0`, altitude `-777` ft).

use crate::domain::Waypoint;
use encoding_rs::WINDOWS_1251;
use std::io::Write;

use super::ExportError;

/// Default symbol code when a waypoint has no symbol assigned ("dot").
const DEFAULT_SYMBOL_CODE: u16 = 0;
/// Foreground colour for waypoint text (black, 0x000000 in BGR int).
const FOREGROUND_COLOR: u32 = 0;
/// Background colour for waypoint text (yellow, OziExplorer default).
const BACKGROUND_COLOR: u32 = 65_535;
/// "Altitude unknown" sentinel used throughout OziExplorer files.
const ALTITUDE_UNKNOWN: i32 = -777;
/// Default font size used by OziExplorer.
const FONT_SIZE: u16 = 6;
/// Default font style (regular).
const FONT_STYLE: u16 = 0;
/// Default symbol size in pixels.
const SYMBOL_SIZE: u16 = 17;
/// Waypoint status (always 1 in the v1.1 spec).
const STATUS: u16 = 1;

/// Write the given waypoints to `writer` in OziExplorer Waypoint File v1.1
/// format, encoding non-ASCII text as Windows-1251.
pub fn write_wpt<I>(waypoints: I, writer: &mut impl Write) -> Result<(), ExportError>
where
    I: IntoIterator<Item = Waypoint>,
{
    write_line(writer, "OziExplorer Waypoint File Version 1.1")?;
    write_line(writer, "WGS 84")?;
    write_line(writer, "Reserved 2")?;
    write_line(writer, "Reserved 3")?;

    for (index, waypoint) in waypoints.into_iter().enumerate() {
        let number = (index as u64) + 1;
        let symbol_code = map_symbol_to_code(waypoint.symbol());
        let line = format!(
            "{number},{name},{lat:.6},{lon:.6},{date},{symbol},{status},0,{fg},{bg},{desc},0,0,0,{alt},{font_size},{font_style},{symbol_size},0,0,0,,,",
            number = number,
            name = sanitise_text(waypoint.name(), 14),
            lat = waypoint.latitude(),
            lon = waypoint.longitude(),
            date = "0",
            symbol = symbol_code,
            status = STATUS,
            fg = FOREGROUND_COLOR,
            bg = BACKGROUND_COLOR,
            desc = "",
            alt = ALTITUDE_UNKNOWN,
            font_size = FONT_SIZE,
            font_style = FONT_STYLE,
            symbol_size = SYMBOL_SIZE,
        );
        write_line(writer, &line)?;
    }

    Ok(())
}

/// Map an internal OziExplorer-style symbol identifier to the integer code
/// emitted in the WPT row. Unknown / unset symbols fall back to the default
/// "dot" code (0), matching the empty-symbol behaviour of OziExplorer itself.
pub fn map_symbol_to_code(symbol: Option<&str>) -> u16 {
    let Some(symbol) = symbol else {
        return DEFAULT_SYMBOL_CODE;
    };

    if let Ok(parsed) = symbol.parse::<u16>() {
        return parsed;
    }

    match symbol.to_ascii_lowercase().as_str() {
        "dot" => 0,
        "flag" => 9,
        "house" => 10,
        "fuel" | "gas" => 11,
        "car" => 12,
        "fish" => 13,
        "boat" => 14,
        "anchor" => 15,
        "camp" | "camping" | "tent" => 18,
        "skull" | "danger" => 19,
        "first_aid" | "firstaid" | "medical" | "cross" => 21,
        "info" => 22,
        "lodging" | "hotel" => 24,
        "phone" => 25,
        "restaurant" => 26,
        "scenic" | "view" => 27,
        "shopping" => 28,
        "swim" | "swimming" => 29,
        "water" => 30,
        "ice" => 31,
        "park" => 32,
        "trailhead" => 33,
        "truck" => 34,
        "circle" => 41,
        "square" => 42,
        "diamond" => 43,
        "triangle" => 44,
        "star" => 45,
        "cross_hair" | "crosshair" | "x" => 46,
        _ => DEFAULT_SYMBOL_CODE,
    }
}

/// Encode `line + "\r\n"` as Windows-1251 and append to writer.
///
/// `encoding_rs::WINDOWS_1251.encode()` uses '?' as the unmappable replacement,
/// which matches what legacy OziExplorer expects when it encounters characters
/// outside cp1251.
fn write_line(writer: &mut impl Write, line: &str) -> Result<(), ExportError> {
    let mut buf = String::with_capacity(line.len() + 2);
    buf.push_str(line);
    buf.push_str("\r\n");
    let (encoded, _, _) = WINDOWS_1251.encode(&buf);
    writer.write_all(&encoded)?;
    Ok(())
}

/// Strip characters that would break the comma-separated row (commas and line
/// breaks) and truncate to `max_chars` Unicode scalars. cp1251 encoding happens
/// at write time, so any character invalid in cp1251 becomes '?' on disk
/// rather than corrupting the column layout.
fn sanitise_text(input: &str, max_chars: usize) -> String {
    input
        .chars()
        .take(max_chars)
        .map(|ch| match ch {
            ',' | '\r' | '\n' => ' ',
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{map_symbol_to_code, write_wpt};
    use crate::domain::{Waypoint, WaypointId};
    use encoding_rs::WINDOWS_1251;

    fn waypoint(id: u64, name: &str, lat: f64, lon: f64, symbol: Option<&str>) -> Waypoint {
        let mut wp = Waypoint::new(WaypointId::new(id), name, lat, lon);
        if let Some(sym) = symbol {
            wp.set_symbol(Some(sym.to_owned()));
        }
        wp
    }

    #[test]
    fn write_wpt_emits_exact_v11_header() {
        let mut bytes = Vec::new();
        write_wpt(Vec::<Waypoint>::new(), &mut bytes).expect("write");

        let expected = b"OziExplorer Waypoint File Version 1.1\r\nWGS 84\r\nReserved 2\r\nReserved 3\r\n";
        assert_eq!(bytes, expected);
    }

    #[test]
    fn write_wpt_three_waypoints_with_mixed_symbols_yields_three_rows() {
        let waypoints = vec![
            waypoint(1, "Camp", 55.751244, 37.618423, Some("camp")),
            waypoint(2, "Flag", 55.760000, 37.620000, Some("flag")),
            waypoint(3, "Unnamed", 55.770000, 37.630000, None),
        ];

        let mut bytes = Vec::new();
        write_wpt(waypoints, &mut bytes).expect("write");
        let (decoded, _, had_errors) = WINDOWS_1251.decode(&bytes);
        assert!(!had_errors, "WPT output should be valid cp1251");

        let lines: Vec<&str> = decoded.split("\r\n").collect();
        // 4 header lines + 3 waypoints + trailing empty entry from final CRLF = 8
        assert_eq!(lines.len(), 8, "lines: {lines:#?}");
        assert_eq!(lines[0], "OziExplorer Waypoint File Version 1.1");
        assert_eq!(lines[1], "WGS 84");
        assert_eq!(lines[2], "Reserved 2");
        assert_eq!(lines[3], "Reserved 3");

        let row1: Vec<&str> = lines[4].split(',').collect();
        assert_eq!(row1[0], "1");
        assert_eq!(row1[1], "Camp");
        assert_eq!(row1[2], "55.751244");
        assert_eq!(row1[3], "37.618423");
        assert_eq!(row1[5], "18", "camp symbol should map to code 18");

        let row2: Vec<&str> = lines[5].split(',').collect();
        assert_eq!(row2[5], "9", "flag symbol should map to code 9");

        let row3: Vec<&str> = lines[6].split(',').collect();
        assert_eq!(row3[5], "0", "missing symbol should map to default code 0");

        // Every row must have 24 columns matching OziExplorer 1.1 layout.
        for (i, row) in lines[4..=6].iter().enumerate() {
            let cols: Vec<&str> = row.split(',').collect();
            assert_eq!(cols.len(), 24, "row {i} should have 24 fields: {row}");
        }
    }

    #[test]
    fn write_wpt_encodes_non_ascii_name_as_cp1251() {
        let waypoints = vec![waypoint(1, "Стоянка", 55.0, 37.0, None)];

        let mut bytes = Vec::new();
        write_wpt(waypoints, &mut bytes).expect("write");

        // Find the row line (between the 4th and 5th CRLF).
        let mut splits = bytes.split(|b| *b == b'\n');
        let _ = splits.next(); // header 1
        let _ = splits.next();
        let _ = splits.next();
        let _ = splits.next();
        let row_line = splits.next().expect("row present");
        // After the row name there must be the cp1251 byte sequence for "Стоянка".
        let (expected, _, _) = WINDOWS_1251.encode("Стоянка");
        // Look for the cp1251 bytes within the row.
        let needle: &[u8] = expected.as_ref();
        assert!(
            row_line.windows(needle.len()).any(|w| w == needle),
            "row should contain cp1251 bytes for the Cyrillic name; row={row_line:?}, needle={needle:?}",
        );

        // The bytes must NOT contain the raw UTF-8 sequence (e.g. 0xD0 0xA1 for 'С').
        let utf8_bytes = "Стоянка".as_bytes();
        assert!(
            !bytes.windows(utf8_bytes.len()).any(|w| w == utf8_bytes),
            "output must not contain UTF-8 byte sequence for non-ASCII text"
        );
    }

    #[test]
    fn write_wpt_uses_crlf_line_endings() {
        let waypoints = vec![waypoint(1, "A", 1.0, 2.0, None)];
        let mut bytes = Vec::new();
        write_wpt(waypoints, &mut bytes).expect("write");

        // Every '\n' must be preceded by '\r'.
        for (i, byte) in bytes.iter().enumerate() {
            if *byte == b'\n' {
                assert!(i > 0 && bytes[i - 1] == b'\r', "bare LF at byte {i}");
            }
        }
        // Output must end with CRLF.
        assert!(bytes.ends_with(b"\r\n"));
    }

    #[test]
    fn write_wpt_round_trip_fixture_preserves_coordinates_within_1e_6() {
        // Reference fixture: one waypoint with known cp1251 name and symbol.
        let waypoints = vec![
            waypoint(1, "Camp", 55.751244, 37.618423, Some("camp")),
            waypoint(2, "Метка", 60.123456, 30.654321, None),
        ];

        let mut bytes = Vec::new();
        write_wpt(waypoints.clone(), &mut bytes).expect("write");

        // Parse what we wrote using cp1251 decoding + simple CSV split.
        let (decoded, _, _) = WINDOWS_1251.decode(&bytes);
        let lines: Vec<&str> = decoded
            .split("\r\n")
            .skip(4)
            .filter(|line| !line.is_empty())
            .collect();
        assert_eq!(lines.len(), waypoints.len());

        for (line, original) in lines.iter().zip(waypoints.iter()) {
            let cols: Vec<&str> = line.split(',').collect();
            let lat: f64 = cols[2].trim().parse().expect("lat parses");
            let lon: f64 = cols[3].trim().parse().expect("lon parses");
            assert!(
                (lat - original.latitude()).abs() < 1e-6,
                "lat round-trip drift: {lat} vs {}",
                original.latitude()
            );
            assert!(
                (lon - original.longitude()).abs() < 1e-6,
                "lon round-trip drift: {lon} vs {}",
                original.longitude()
            );
        }
    }

    #[test]
    fn map_symbol_to_code_handles_numeric_passthrough() {
        assert_eq!(map_symbol_to_code(Some("42")), 42);
        assert_eq!(map_symbol_to_code(Some("camp")), 18);
        assert_eq!(map_symbol_to_code(Some("CAMP")), 18);
        assert_eq!(map_symbol_to_code(None), 0);
        assert_eq!(map_symbol_to_code(Some("not-a-known-symbol")), 0);
    }

    #[test]
    fn write_wpt_emits_six_decimal_places_for_coordinates() {
        let waypoints = vec![waypoint(1, "A", 55.0, 37.0, None)];
        let mut bytes = Vec::new();
        write_wpt(waypoints, &mut bytes).expect("write");
        let (decoded, _, _) = WINDOWS_1251.decode(&bytes);
        let row = decoded.split("\r\n").nth(4).expect("row");
        let cols: Vec<&str> = row.split(',').collect();
        assert_eq!(cols[2], "55.000000");
        assert_eq!(cols[3], "37.000000");
    }
}
