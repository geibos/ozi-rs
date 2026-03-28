use std::path::{Path, PathBuf};

const OZI_MAP_HEADER_PREFIX: &str = "OziExplorer Map Data File";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OziMapMetadata {
    title: String,
    source_path: PathBuf,
    raster_reference: String,
    resolved_raster_path: PathBuf,
    raster_kind: OziRasterKind,
    projection_name: String,
    datum_name: String,
    calibration_points: Vec<String>,
}

impl OziMapMetadata {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn raster_reference(&self) -> &str {
        &self.raster_reference
    }

    pub fn resolved_raster_path(&self) -> &Path {
        &self.resolved_raster_path
    }

    pub const fn raster_kind(&self) -> &OziRasterKind {
        &self.raster_kind
    }

    pub fn projection_name(&self) -> &str {
        &self.projection_name
    }

    pub fn datum_name(&self) -> &str {
        &self.datum_name
    }

    pub fn calibration_points(&self) -> &[String] {
        &self.calibration_points
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OziRasterKind {
    DirectImage(DirectImageFormat),
    DeferredOzf,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectImageFormat {
    Jpeg,
    Png,
    Bmp,
    Gif,
    Tiff,
    WebP,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OziMapParseError {
    MissingHeader,
    MissingTitle,
    MissingRasterReference,
    MissingProjectionName,
    MissingDatumName,
}

impl std::fmt::Display for OziMapParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingHeader => write!(f, "missing OziExplorer map header"),
            Self::MissingTitle => write!(f, "missing OZI map title"),
            Self::MissingRasterReference => write!(f, "missing OZI raster reference"),
            Self::MissingProjectionName => write!(f, "missing OZI projection name"),
            Self::MissingDatumName => write!(f, "missing OZI datum name"),
        }
    }
}

impl std::error::Error for OziMapParseError {}

pub fn parse_ozi_map_metadata(
    source_path: impl Into<PathBuf>,
    contents: &str,
) -> Result<OziMapMetadata, OziMapParseError> {
    let source_path = source_path.into();
    let lines = normalized_lines(contents);

    validate_header(&lines)?;
    let title = required_line(&lines, 1).ok_or(OziMapParseError::MissingTitle)?;
    let raster_reference =
        required_line(&lines, 2).ok_or(OziMapParseError::MissingRasterReference)?;
    let projection_name =
        required_line(&lines, 4).ok_or(OziMapParseError::MissingProjectionName)?;
    let datum_name = required_line(&lines, 6).ok_or(OziMapParseError::MissingDatumName)?;

    Ok(OziMapMetadata {
        title: title.to_owned(),
        source_path: source_path.clone(),
        raster_reference: raster_reference.to_owned(),
        resolved_raster_path: resolve_raster_path(&source_path, raster_reference),
        raster_kind: classify_raster_reference(raster_reference),
        projection_name: projection_name.to_owned(),
        datum_name: datum_name.to_owned(),
        calibration_points: calibration_points(&lines),
    })
}

fn normalized_lines(contents: &str) -> Vec<&str> {
    contents
        .lines()
        .map(|line| line.trim_end_matches('\r'))
        .collect()
}

fn validate_header(lines: &[&str]) -> Result<(), OziMapParseError> {
    let Some(header) = required_line(lines, 0) else {
        return Err(OziMapParseError::MissingHeader);
    };

    if header.starts_with(OZI_MAP_HEADER_PREFIX) {
        return Ok(());
    }

    Err(OziMapParseError::MissingHeader)
}

fn required_line<'a>(lines: &'a [&str], index: usize) -> Option<&'a str> {
    lines
        .get(index)
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
}

fn calibration_points(lines: &[&str]) -> Vec<String> {
    lines
        .iter()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed
                .starts_with("Point")
                .then_some(trimmed)
                .filter(|line| is_populated_calibration_point(line))
                .map(ToOwned::to_owned)
        })
        .collect()
}

fn is_populated_calibration_point(line: &str) -> bool {
    let mut fields = line.split(',').map(str::trim);
    let _point_name = fields.next();
    let _point_kind = fields.next();
    let x = fields.next().unwrap_or_default();
    let y = fields.next().unwrap_or_default();

    !x.is_empty() && !y.is_empty()
}

fn resolve_raster_path(source_path: &Path, raster_reference: &str) -> PathBuf {
    let reference = Path::new(raster_reference.trim());
    let Some(map_dir) = source_path.parent() else {
        return safe_file_name_path(reference);
    };

    if reference.is_absolute() || contains_parent_dir(reference) {
        return map_dir.join(safe_file_name_path(reference));
    }

    map_dir.join(reference)
}

fn contains_parent_dir(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
}

fn safe_file_name_path(path: &Path) -> PathBuf {
    path.file_name()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(path.as_os_str()))
}

fn classify_raster_reference(raster_reference: &str) -> OziRasterKind {
    match Path::new(raster_reference)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => OziRasterKind::DirectImage(DirectImageFormat::Jpeg),
        Some("png") => OziRasterKind::DirectImage(DirectImageFormat::Png),
        Some("bmp") => OziRasterKind::DirectImage(DirectImageFormat::Bmp),
        Some("gif") => OziRasterKind::DirectImage(DirectImageFormat::Gif),
        Some("tif") | Some("tiff") => OziRasterKind::DirectImage(DirectImageFormat::Tiff),
        Some("webp") => OziRasterKind::DirectImage(DirectImageFormat::WebP),
        Some("ozf2") | Some("ozfx3") => OziRasterKind::DeferredOzf,
        _ => OziRasterKind::Unsupported,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DirectImageFormat, OziMapParseError, OziRasterKind, parse_ozi_map_metadata,
        resolve_raster_path,
    };
    use std::path::{Path, PathBuf};

    #[test]
    fn parse_ozi_map_metadata_reads_minimal_supported_image_reference() {
        let metadata = parse_ozi_map_metadata(
            PathBuf::from("archives/field/calibration.map"),
            &sample_map("raster.jpg"),
        )
        .expect("metadata");

        assert_eq!(metadata.title(), "Field calibration");
        assert_eq!(metadata.raster_reference(), "raster.jpg");
        assert_eq!(
            metadata.resolved_raster_path(),
            Path::new("archives/field/raster.jpg")
        );
        assert_eq!(
            metadata.projection_name(),
            "Map Projection,Mercator,PolyCal,No,AutoCalOnly,No,BSBUseWPX,No"
        );
        assert_eq!(metadata.datum_name(), "WGS 84");
        assert_eq!(
            metadata.raster_kind(),
            &OziRasterKind::DirectImage(DirectImageFormat::Jpeg)
        );
        assert_eq!(metadata.calibration_points().len(), 2);
    }

    #[test]
    fn parse_ozi_map_metadata_rejects_missing_header() {
        let error = parse_ozi_map_metadata(
            PathBuf::from("archives/field/calibration.map"),
            "Not an ozi map\nTitle\nraster.jpg",
        )
        .expect_err("missing header");

        assert_eq!(error, OziMapParseError::MissingHeader);
    }

    #[test]
    fn parse_ozi_map_metadata_requires_raster_reference() {
        let error = parse_ozi_map_metadata(
            PathBuf::from("archives/field/calibration.map"),
            &sample_map(""),
        )
        .expect_err("missing raster");

        assert_eq!(error, OziMapParseError::MissingRasterReference);
    }

    #[test]
    fn parse_ozi_map_metadata_marks_ozf_payloads_as_deferred() {
        let metadata = parse_ozi_map_metadata(
            PathBuf::from("archives/field/calibration.map"),
            &sample_map("maps/base.ozf2"),
        )
        .expect("metadata");

        assert_eq!(metadata.raster_kind(), &OziRasterKind::DeferredOzf);
    }

    #[test]
    fn resolve_raster_path_falls_back_to_map_directory_file_name() {
        let resolved = resolve_raster_path(
            Path::new("archives/field/calibration.map"),
            "../outside/base.ozf2",
        );

        assert_eq!(resolved, Path::new("archives/field/base.ozf2"));
    }

    fn sample_map(raster_reference: &str) -> String {
        format!(
            "OziExplorer Map Data File Version 2.2\nField calibration\n{raster_reference}\n1 ,Map Code,\nMap Projection,Mercator,PolyCal,No,AutoCalOnly,No,BSBUseWPX,No\nProjection Setup,,,,,,,,,,\nWGS 84\nReserved 1\nReserved 2\nMagnetic Variation,,,E\nMap Projection,Mercator,PolyCal,No,AutoCalOnly,No,BSBUseWPX,No\nPoint01,xy,100,200,in, deg,54,30.000,N,48,24.000,E, grid, , , ,N\nPoint02,xy,300,400,in, deg,54,31.000,N,48,25.000,E, grid, , , ,N\nPoint03,xy, , ,in, deg, , , , , , , grid, , , ,N\n"
        )
    }
}
