use ozi_rs::infrastructure::import::{
    DirectImageFormat, OziMapParseError, OziRasterKind, parse_ozi_map_metadata,
};
use std::path::{Path, PathBuf};

#[test]
fn parse_ozi_map_metadata_supports_relative_png_reference() {
    let metadata = parse_ozi_map_metadata(
        PathBuf::from("bundle/5-OZI/calibration.map"),
        &sample_map("images/raster.PNG"),
    )
    .expect("metadata");

    assert_eq!(metadata.title(), "Forest map");
    assert_eq!(
        metadata.resolved_raster_path(),
        Path::new("bundle/5-OZI/images/raster.PNG")
    );
    assert_eq!(
        metadata.raster_kind(),
        &OziRasterKind::DirectImage(DirectImageFormat::Png)
    );
}

#[test]
fn parse_ozi_map_metadata_reports_missing_title() {
    let error = parse_ozi_map_metadata(
        PathBuf::from("bundle/5-OZI/calibration.map"),
        "OziExplorer Map Data File Version 2.2\n\nraster.png\n1 ,Map Code,\nProjection\nProjection Setup,,,,,,,,,,\nWGS 84\n",
    )
    .expect_err("missing title");

    assert_eq!(error, OziMapParseError::MissingTitle);
}

#[test]
fn parse_ozi_map_metadata_treats_ozfx3_as_deferred() {
    let metadata = parse_ozi_map_metadata(
        PathBuf::from("bundle/7-OZI/calibration.map"),
        &sample_map("legacy/base.ozfx3"),
    )
    .expect("metadata");

    assert_eq!(metadata.raster_kind(), &OziRasterKind::DeferredOzf);
}

fn sample_map(raster_reference: &str) -> String {
    format!(
        "OziExplorer Map Data File Version 2.2\nForest map\n{raster_reference}\n1 ,Map Code,\nMap Projection,Latitude/Longitude,PolyCal,No,AutoCalOnly,No,BSBUseWPX,No\nProjection Setup,,,,,,,,,,\nPulkovo 1942\nReserved 1\nReserved 2\nMagnetic Variation,,,E\nPoint01,xy,10,20,in, deg,54,30.000,N,48,24.000,E, grid, , , ,N\n"
    )
}
