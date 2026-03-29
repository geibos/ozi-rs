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

    assert_eq!(metadata.raster_kind(), &OziRasterKind::Ozfx3);
}

#[test]
fn parse_real_lizaalert_ozi_map_extracts_ozf2_reference() {
    let source_path = PathBuf::from(
        "example_data/2021-07-30_Murino/5-Ozi(Win&Android)_Topo_EEKO/Maps/2021-07-30_Murino_Topo_EEKO_z16_ozf.map",
    );
    let metadata = parse_ozi_map_metadata(&source_path, REAL_LIZAALERT_MAP).expect("metadata");

    assert_eq!(metadata.title(), "2021-07-30_Murino_Topo_EEKO_z16.png");
    assert_eq!(metadata.datum_name(), "WGS 84");
    assert_eq!(metadata.raster_kind(), &OziRasterKind::Ozf2);
    assert_eq!(
        metadata.resolved_raster_path(),
        Path::new(
            "example_data/2021-07-30_Murino/5-Ozi(Win&Android)_Topo_EEKO/Maps/2021-07-30_Murino_Topo_EEKO_z16.ozf2"
        )
    );
    assert_eq!(metadata.calibration_points().len(), 9);
}

const REAL_LIZAALERT_MAP: &str = r#"OziExplorer Map Data File Version 2.2
2021-07-30_Murino_Topo_EEKO_z16.png
C:\LA\map\OZF\2021-07-30_Murino_Topo_EEKO_z16.ozf2
1 ,Map Code,
WGS 84,,   0.0000,   0.0000,WGS 84
Reserved 1
Reserved 2
Magnetic Variation,,,E
Map Projection,Mercator,PolyCal,No,AutoCalOnly,No,BSBUseWPX,No
Point01,xy,    0, 0,in, deg, 60, 5.6385,N, 30, 24.2875,E, grid,   ,           ,           ,N
Point02,xy,    1120, 0,in, deg, 60, 5.6385,N, 30, 27.1740,E, grid,   ,           ,           ,N
Point03,xy,    2241, 0,in, deg, 60, 5.6385,N, 30, 30.0579,E, grid,   ,           ,           ,N
Point04,xy,    0, 1408,in, deg, 60, 3.8287,N, 30, 24.2875,E, grid,   ,           ,           ,N
Point05,xy,    1120, 1408,in, deg, 60, 3.8287,N, 30, 27.1740,E, grid,   ,           ,           ,N
Point06,xy,    2241, 1408,in, deg, 60, 3.8287,N, 30, 30.0579,E, grid,   ,           ,           ,N
Point07,xy,    0, 2817,in, deg, 60, 2.0187,N, 30, 24.2875,E, grid,   ,           ,           ,N
Point08,xy,    1120, 2817,in, deg, 60, 2.0187,N, 30, 27.1740,E, grid,   ,           ,           ,N
Point09,xy,    2241, 2817,in, deg, 60, 2.0187,N, 30, 30.0579,E, grid,   ,           ,           ,N
Point10,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Point11,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Point12,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Point13,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Point14,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Point15,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Point16,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Point17,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Point18,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Point19,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Point20,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Point21,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Point22,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Point23,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Point24,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Point25,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Point26,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Point27,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Point28,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Point29,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Point30,xy,     ,     ,in, deg,    ,        ,N,    ,        ,W, grid,   ,           ,           ,N
Projection Setup,,,,,,,,,,
Map Feature = MF ; Map Comment = MC     These follow if they exist
Track File = TF      These follow if they exist
Moving Map Parameters = MM?    These follow if they exist
MM0,Yes
MMPNUM,4
MMPXY,1,0,0
MMPXY,2,2241,0
MMPXY,3,2241,2817
MMPXY,4,0,2817
MMPLL,1, 30.404792, 60.093974
MMPLL,2, 30.500965, 60.093974
MMPLL,3, 30.500965, 60.033645
MMPLL,4, 30.404792, 60.033645
MM1B,2.384048
Other Grid Setup
GRGRID,Yes,500 m,No,255,255,500 m,0,16777215,8,1,Yes,No,No,x
MOP,Map Open Position,0,0
IWH,Map Image Width/Height,2241,2817
MLP,Map Last Position,60.0584295,30.4699516,50
"#;

fn sample_map(raster_reference: &str) -> String {
    format!(
        "OziExplorer Map Data File Version 2.2\nForest map\n{raster_reference}\n1 ,Map Code,\nMap Projection,Latitude/Longitude,PolyCal,No,AutoCalOnly,No,BSBUseWPX,No\nProjection Setup,,,,,,,,,,\nPulkovo 1942\nReserved 1\nReserved 2\nMagnetic Variation,,,E\nPoint01,xy,10,20,in, deg,54,30.000,N,48,24.000,E, grid, , , ,N\n"
    )
}
