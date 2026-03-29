use ozi_rs::infrastructure::import::{
    OziRasterDecodeError, OziRasterKind, decode_ozi_raster_image, open_ozi_raster_tile_source,
    parse_ozi_map_metadata,
};
use std::path::PathBuf;

#[test]
#[ignore = "requires local example_data OZF fixture"]
fn decode_ozi_raster_image_decodes_real_lizaalert_ozf2_sample() {
    let map_path = PathBuf::from(
        "example_data/2021-07-30_Murino/5-Ozi(Win&Android)_Topo_EEKO/Maps/2021-07-30_Murino_Topo_EEKO_z16_ozf.map",
    );
    let map_contents = std::fs::read_to_string(&map_path).expect("example .map contents");
    let metadata = parse_ozi_map_metadata(&map_path, &map_contents).expect("metadata");

    let image = decode_ozi_raster_image(&metadata).expect("decoded OZF image");

    assert_eq!(metadata.raster_kind(), &OziRasterKind::Ozf2);
    assert_eq!(image.width(), 2241);
    assert_eq!(image.height(), 2817);
    assert_eq!(image.rgba_pixels().len(), 2241 * 2817 * 4);
}

#[test]
#[ignore = "requires local example_data OZF fixture"]
fn open_ozi_raster_tile_source_exposes_native_levels_and_decodes_tiles() {
    let map_path = PathBuf::from(
        "example_data/2021-07-30_Murino/5-Ozi(Win&Android)_Topo_EEKO/Maps/2021-07-30_Murino_Topo_EEKO_z16_ozf.map",
    );
    let map_contents = std::fs::read_to_string(&map_path).expect("example .map contents");
    let metadata = parse_ozi_map_metadata(&map_path, &map_contents).expect("metadata");

    let source = open_ozi_raster_tile_source(&metadata).expect("tile source");

    assert_eq!(source.levels().len(), 5);
    assert_eq!(source.levels()[0].width(), 2241);
    assert_eq!(source.levels()[0].height(), 2817);
    assert_eq!(source.levels()[0].tile_width(), 64);
    assert_eq!(source.levels()[0].tile_height(), 64);
    assert_eq!(source.levels()[0].tile_columns(), 36);
    assert_eq!(source.levels()[0].tile_rows(), 45);

    let edge_tile = source
        .decode_rgba_tile(0, 35, 44)
        .expect("edge tile should decode");

    assert_eq!(edge_tile.width(), 1);
    assert_eq!(edge_tile.height(), 1);
    assert_eq!(edge_tile.rgba_pixels().len(), 4);
}

#[test]
fn decode_ozi_raster_image_rejects_non_ozf_references() {
    let metadata = parse_ozi_map_metadata(
        PathBuf::from("bundle/5-OZI/calibration.map"),
        &sample_map("image.png"),
    )
    .expect("metadata");

    let error = decode_ozi_raster_image(&metadata).expect_err("unsupported raster kind");

    assert!(matches!(
        error,
        OziRasterDecodeError::UnsupportedRasterKind(OziRasterKind::DirectImage(_))
    ));
}

#[test]
fn open_ozi_raster_tile_source_rejects_non_ozf_references() {
    let metadata = parse_ozi_map_metadata(
        PathBuf::from("bundle/5-OZI/calibration.map"),
        &sample_map("image.png"),
    )
    .expect("metadata");

    let error = open_ozi_raster_tile_source(&metadata).expect_err("unsupported raster kind");

    assert!(matches!(
        error,
        OziRasterDecodeError::UnsupportedRasterKind(OziRasterKind::DirectImage(_))
    ));
}

fn sample_map(raster_reference: &str) -> String {
    format!(
        "OziExplorer Map Data File Version 2.2\nForest map\n{raster_reference}\n1 ,Map Code,\nMap Projection,Latitude/Longitude,PolyCal,No,AutoCalOnly,No,BSBUseWPX,No\nProjection Setup,,,,,,,,,,\nPulkovo 1942\nReserved 1\nReserved 2\nMagnetic Variation,,,E\nPoint01,xy,10,20,in, deg,54,30.000,N,48,24.000,E, grid, , , ,N\n"
    )
}
