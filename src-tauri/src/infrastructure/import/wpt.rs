//! OziExplorer Waypoint File (.wpt) reader.
//!
//! We have written this format since ADR-0022 and never read it. That is the
//! wrong way round for a field tool: a neighbouring headquarters running the
//! original OziExplorer hands over a `.wpt`, and until now the only way to
//! take it was to find something that converts it to GPX first.
//!
//! Format (OziExplorer manual, "Waypoint File", and
//! `docs/reference/oziexplorer.md`):
//!
//! ```text
//! OziExplorer Waypoint File Version 1.1
//! WGS 84
//! Reserved 2
//! Reserved 3
//! 1,ШТАБ         , 53.900000,  27.566700,0,18,1,0,0,65535,Базовый лагерь,0,0,0,-777,6,0,17,0,0,0,,,
//! ```
//!
//! Four header lines, then one waypoint per line across 24 comma-separated
//! fields. The ones that mean something to this application are 2 (name),
//! 3 (latitude), 4 (longitude) and 6 (symbol). Everything else is either a
//! GPS-upload detail, a label style, or a proximity alarm — none of which this
//! product models.
//!
//! The encoding is whatever the machine that wrote the file used, which for
//! the crews this tool serves means Windows-1251 more often than not. The
//! decoding chain is the PLT one, for the same reason.
//!
//! **The `chr(209)` escape is deliberately not honoured.** The original
//! reserves that byte for a comma inside a text field and turns it back on
//! reading. In Windows-1251 byte 209 is `С`, a letter in every other Russian
//! word, so honouring it would put a comma in the middle of «СТАРТ» and
//! «ЛИСА15 Северная». In a Western-European locale the byte is `Ñ` and the
//! escape is harmless; for these files it is not. The writer does not emit it
//! either — see `export::wpt::sanitise_text` for the same reasoning from the
//! other side.
//!
//! Trailing fields may be omitted entirely, so a short row is valid as long as
//! it carries the four that matter.

use std::path::Path;

use crate::domain::{Waypoint, WaypointId};

use super::plt::{PltImportError, decode_plt_bytes};

/// What a `.wpt` file yielded.
#[derive(Debug, Clone, PartialEq)]
pub struct WptImport {
    source_path: String,
    waypoints: Vec<Waypoint>,
}

impl WptImport {
    pub fn source_path(&self) -> &str {
        &self.source_path
    }

    pub fn waypoints(&self) -> &[Waypoint] {
        &self.waypoints
    }
}

#[derive(Debug)]
pub enum WptImportError {
    Io(std::io::Error),
    /// The first line is not the signature. Reading on would turn a PLT or a
    /// CSV into a scatter of waypoints in the wrong place.
    NotAWaypointFile,
    Decode(PltImportError),
}

impl std::fmt::Display for WptImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "failed to read WPT file: {e}"),
            Self::NotAWaypointFile => write!(f, "not an OziExplorer waypoint file"),
            Self::Decode(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for WptImportError {}

impl From<std::io::Error> for WptImportError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

/// The signature OziExplorer writes on line 1, matched loosely: the version
/// has moved between releases and the rows have not.
const SIGNATURE: &str = "oziexplorer waypoint file";

pub fn import_wpt_file(path: &Path) -> Result<WptImport, WptImportError> {
    let bytes = std::fs::read(path)?;
    let text = decode_plt_bytes(&bytes).map_err(WptImportError::Decode)?;
    import_wpt_text(path.to_string_lossy().into_owned(), &text)
}

pub fn import_wpt_text(source_path: String, text: &str) -> Result<WptImport, WptImportError> {
    let mut lines = text.lines();
    let signature = lines.next().unwrap_or_default();
    if !signature.trim().to_ascii_lowercase().starts_with(SIGNATURE) {
        return Err(WptImportError::NotAWaypointFile);
    }

    // Lines 2–4 are the datum and two reserved lines. The datum is read and
    // discarded on purpose: the owner fixed the working datum to WGS 84 on
    // 2026-07-16, and transforming between datums is an explicit non-goal.
    for _ in 0..3 {
        lines.next();
    }

    let mut waypoints = Vec::new();
    for line in lines {
        let Some(waypoint) = parse_row(line, waypoints.len()) else {
            continue;
        };
        waypoints.push(waypoint);
    }

    Ok(WptImport {
        source_path,
        waypoints,
    })
}

/// One row, or `None` when the line carries no usable waypoint.
///
/// The original writes placeholder rows for empty GPS slots, and a file that
/// has been through a text editor picks up blank lines. Neither is a waypoint,
/// and neither is an error worth stopping a day's import for.
fn parse_row(line: &str, index: usize) -> Option<Waypoint> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let fields: Vec<&str> = trimmed.split(',').collect();
    if fields.len() < 4 {
        return None;
    }

    let latitude: f64 = fields[2].trim().parse().ok()?;
    let longitude: f64 = fields[3].trim().parse().ok()?;
    if !latitude.is_finite() || !longitude.is_finite() {
        return None;
    }
    // A slot with no coordinates is written as a placeholder, not omitted.
    if latitude == 0.0 && longitude == 0.0 && fields[1].trim().is_empty() {
        return None;
    }

    let name = fields[1].trim();
    let name = if name.is_empty() {
        format!("Waypoint {}", index + 1)
    } else {
        name.to_owned()
    };

    let mut waypoint = Waypoint::new(
        WaypointId::new((index + 1) as u64),
        name,
        latitude,
        longitude,
    );

    // Field 6 is the symbol index. The export maps names to these codes; the
    // import keeps the code as the symbol string, which is what the export
    // reads back unchanged (`map_symbol_to_code` passes a number through), so
    // a waypoint survives the trip out and back.
    if let Some(symbol) = fields.get(5).map(|s| s.trim()).filter(|s| !s.is_empty())
        && symbol.parse::<u16>().is_ok()
        && symbol != "0"
    {
        let _ = waypoint.set_symbol(Some(symbol.to_owned()));
    }

    Some(waypoint)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str =
        "OziExplorer Waypoint File Version 1.1\r\nWGS 84\r\nReserved 2\r\nReserved 3\r\n";

    #[test]
    fn reads_the_fields_that_mean_something() {
        let text = format!(
            "{HEADER}1,ШТАБ,53.900000,27.566700,0,18,1,0,0,65535,Базовый лагерь,0,0,0,-777,6,0,17,0,0,0,,,\r\n"
        );
        let import = import_wpt_text("/tmp/day.wpt".to_owned(), &text).expect("import");

        assert_eq!(import.waypoints().len(), 1);
        let wp = &import.waypoints()[0];
        assert_eq!(wp.name(), "ШТАБ");
        assert!((wp.latitude() - 53.9).abs() < 1e-9);
        assert!((wp.longitude() - 27.5667).abs() < 1e-9);
        assert_eq!(wp.symbol(), Some("18"));
    }

    /// A file that is not a waypoint file must be refused rather than read as
    /// a scatter of marks somewhere off the coast of Africa.
    #[test]
    fn refuses_a_file_that_is_not_one() {
        let plt = "OziExplorer Track Point File Version 2.0\r\nWGS 84\r\n";
        assert!(matches!(
            import_wpt_text("/tmp/day.plt".to_owned(), plt),
            Err(WptImportError::NotAWaypointFile)
        ));
    }

    /// The `chr(209)` escape is not honoured, on purpose. In Windows-1251
    /// that byte is `С`, so turning it into a comma would put one inside
    /// «СТАРТ» — which is exactly the file a Russian crew hands over.
    #[test]
    fn a_cyrillic_es_is_a_letter_and_not_a_comma() {
        let text = format!("{HEADER}1,СТАРТ,53.9,27.5,0,0,1\r\n");
        let import = import_wpt_text("/tmp/x.wpt".to_owned(), &text).expect("import");
        assert_eq!(import.waypoints()[0].name(), "СТАРТ");
    }

    /// Trailing fields may be omitted entirely.
    #[test]
    fn a_short_row_is_still_a_waypoint() {
        let text = format!("{HEADER}1,Отметка,53.1,27.2\r\n");
        let import = import_wpt_text("/tmp/x.wpt".to_owned(), &text).expect("import");
        assert_eq!(import.waypoints().len(), 1);
        assert_eq!(import.waypoints()[0].name(), "Отметка");
        assert_eq!(import.waypoints()[0].symbol(), None);
    }

    /// Empty GPS slots are written as placeholder rows, and a file that has
    /// been through an editor picks up blank lines. Neither is a mark, and
    /// neither should stop a day's import.
    #[test]
    fn placeholders_and_blank_lines_are_skipped() {
        let text = format!(
            "{HEADER}1,,0.000000,0.000000,0,0,1\r\n\r\n2,ШТАБ,53.9,27.5,0,0,1\r\nnot a row\r\n"
        );
        let import = import_wpt_text("/tmp/x.wpt".to_owned(), &text).expect("import");
        assert_eq!(import.waypoints().len(), 1);
        assert_eq!(import.waypoints()[0].name(), "ШТАБ");
    }

    /// The trip out and back, which is what CJ-6 actually promises: the
    /// neighbouring headquarters is running the original, and a file that
    /// leaves here has to come back as the same marks. Writing a format we
    /// could not read meant this had never been checked end to end.
    #[test]
    fn a_waypoint_survives_the_trip_out_and_back_through_wpt() {
        use crate::infrastructure::export::wpt::write_wpt;

        let mut camp = Waypoint::new(WaypointId::new(1), "ШТАБ", 53.9, 27.5667);
        let _ = camp.set_symbol(Some("camp".to_owned()));
        let drop_off = Waypoint::new(WaypointId::new(2), "ЛИСА15, вечер", 59.9524, 31.5960);

        let mut written = Vec::new();
        write_wpt([camp.clone(), drop_off.clone()], &mut written).expect("write");

        let decoded = decode_plt_bytes(&written).expect("decode what we wrote");
        let back = import_wpt_text("/tmp/round.wpt".to_owned(), &decoded).expect("read it back");

        assert_eq!(back.waypoints().len(), 2);
        assert_eq!(back.waypoints()[0].name(), "ШТАБ");
        assert!((back.waypoints()[0].latitude() - 53.9).abs() < 1e-6);
        assert!((back.waypoints()[0].longitude() - 27.5667).abs() < 1e-6);
        // `camp` is symbol 18 in the original's set; it comes back as the code,
        // which the writer passes through unchanged on the next trip out.
        assert_eq!(back.waypoints()[0].symbol(), Some("18"));

        // The comma is spent on the way out, deliberately: the escape the
        // format reserves is a byte that Windows-1251 spells `С`. One
        // character of a name is the price of not corrupting every name that
        // has a `С` in it. What matters is that the trip is now *checked*.
        assert_eq!(back.waypoints()[1].name(), "ЛИСА15  вечер");
        assert_eq!(back.waypoints()[1].symbol(), None);
    }

    /// Windows-1251 is what the crews' files carry, and the decoding chain is
    /// the PLT one for exactly that reason.
    #[test]
    fn a_cp1251_file_reads_its_cyrillic() {
        let text = format!("{HEADER}1,Заброс,53.9,27.5,0,0,1\r\n");
        let (bytes, _, had_errors) = encoding_rs::WINDOWS_1251.encode(&text);
        assert!(!had_errors);
        let decoded = decode_plt_bytes(&bytes).expect("decode");
        let import = import_wpt_text("/tmp/x.wpt".to_owned(), &decoded).expect("import");
        assert_eq!(import.waypoints()[0].name(), "Заброс");
    }
}
