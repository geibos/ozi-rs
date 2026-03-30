use crate::application::{
    ActiveMapKind, ActiveMapSelection, LizaMapPackage, LizaProject, LizaProjectSummary, MapCenter,
};
use crate::infrastructure::import::{
    ArchiveEntryKind, SupportedArchiveEntryKind, extract_zip_entries_to_directory,
    inventory_zip_entries, parse_ozi_map_metadata, read_ozi_map_text,
};
use regex::Regex;
use reqwest::blocking::Client;
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};

const ROOT_URL: &str = "https://maps.lizaalert.ru/maps/";
const MOBILE_MAPS_DIR_URL: &str = "8-Android%26iOS/";
const MOBILE_MAPS_DIR_NAME: &str = "8-Android&iOS";
const PROJECT_EXTRACTED_DIR: &str = "extracted";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
}

pub struct ProjectOpenProgress {
    pub message: String,
}

pub fn fetch_project_summaries() -> Result<Vec<LizaProjectSummary>, String> {
    let html = fetch_text(ROOT_URL)?;
    let link_regex =
        Regex::new(r#"href="([^"]+)"[^>]*>([^<]+)</a>"#).map_err(|err| err.to_string())?;
    let project_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}_.+/$").map_err(|err| err.to_string())?;

    let mut projects = link_regex
        .captures_iter(&html)
        .filter_map(|caps| {
            let href = caps.get(1)?.as_str();
            let label = caps.get(2)?.as_str();
            project_regex.is_match(href).then(|| LizaProjectSummary {
                slug: label.trim_end_matches('/').to_owned(),
                name: label.trim_end_matches('/').replace('_', " "),
                url: format!("{ROOT_URL}{href}"),
            })
        })
        .collect::<Vec<_>>();

    projects.sort_by(|left, right| right.slug.cmp(&left.slug));
    Ok(projects)
}

pub fn fetch_project(summary: LizaProjectSummary) -> Result<LizaProject, String> {
    let coordinates_url = format!("{}{}/2-Coordinates.txt", ROOT_URL, summary.slug);
    let maps_url = format!("{}{}/{MOBILE_MAPS_DIR_URL}", ROOT_URL, summary.slug);

    let coordinates_text = fetch_text(&coordinates_url)?;
    let center = parse_center(&coordinates_text)?;
    let maps_html = fetch_text(&maps_url)?;
    let maps = parse_map_packages(&maps_html, &maps_url)?;

    Ok(LizaProject {
        summary,
        center,
        maps,
    })
}

pub fn open_project<F>(
    summary: LizaProjectSummary,
    root: &Path,
    mut on_progress: F,
) -> Result<LizaProject, String>
where
    F: FnMut(ProjectOpenProgress),
{
    if !is_project_cached(&summary.slug, root) {
        on_progress(ProjectOpenProgress {
            message: format!("Downloading project bundle: {}", summary.name),
        });
        cache_project_at_root(&summary, root, &mut on_progress)?;
    } else {
        on_progress(ProjectOpenProgress {
            message: format!("Opening cached project bundle: {}", summary.name),
        });
    }

    on_progress(ProjectOpenProgress {
        message: format!("Extracting cached OZI bundles: {}", summary.name),
    });
    materialize_cached_ozi_archives(root, &summary.slug, &mut on_progress)?;

    on_progress(ProjectOpenProgress {
        message: format!("Indexing cached project maps: {}", summary.name),
    });
    let project = load_cached_project_from_root(summary, root)?;
    ensure_tracks_dir(root, &project.summary.slug);
    Ok(project)
}

pub fn is_project_cached(project_slug: &str, root: &Path) -> bool {
    project_coordinates_path(root, project_slug).exists()
}

/// Open an arbitrary local LizaAlert bundle directory.
///
/// The directory must be named like `YYYY-MM-DD-Place` and contain `2-Coordinates.txt`,
/// `8-Android&iOS/`, and OZI map files/archives directly inside (no `source/` subdir).
/// A `10-Tracks/` subfolder is created if absent.
pub fn open_bundle_directory<F>(dir: &Path, mut on_progress: F) -> Result<LizaProject, String>
where
    F: FnMut(ProjectOpenProgress),
{
    let slug = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("bundle")
        .to_owned();
    let root = dir.parent().unwrap_or(dir);
    let name = slug.replace('_', " ");
    let summary = LizaProjectSummary {
        slug: slug.clone(),
        name: name.clone(),
        url: String::new(),
    };

    on_progress(ProjectOpenProgress {
        message: format!("Extracting OZI archives in: {name}"),
    });
    materialize_cached_ozi_archives(root, &slug, &mut on_progress)?;

    on_progress(ProjectOpenProgress {
        message: format!("Indexing maps in: {name}"),
    });
    let project = load_cached_project_from_root(summary, root)?;
    ensure_tracks_dir(root, &slug);
    Ok(project)
}

/// Return the root directory of the bundle for the given slug.
pub fn bundle_directory(bundles_root: &Path, project_slug: &str) -> PathBuf {
    bundles_root.join(project_slug)
}

fn ensure_tracks_dir(root: &Path, project_slug: &str) {
    let tracks_dir = project_source_root(root, project_slug).join("10-Tracks");
    let _ = fs::create_dir_all(tracks_dir);
}

pub fn download_map<F>(
    selection: ActiveMapSelection,
    mut on_progress: F,
) -> Result<ActiveMapSelection, String>
where
    F: FnMut(DownloadProgress),
{
    let client = client()?;
    let mut response = client
        .get(&selection.remote_url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|err| err.to_string())?;

    if let Some(parent) = selection.local_path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let mut file = File::create(&selection.local_path).map_err(|err| err.to_string())?;
    let total_bytes = response.content_length();
    let mut downloaded_bytes = 0u64;
    let mut buffer = [0u8; 16 * 1024];

    loop {
        let read_bytes = response.read(&mut buffer).map_err(|err| err.to_string())?;
        if read_bytes == 0 {
            break;
        }

        file.write_all(&buffer[..read_bytes])
            .map_err(|err| err.to_string())?;
        downloaded_bytes += read_bytes as u64;
        on_progress(DownloadProgress {
            downloaded_bytes,
            total_bytes,
        });
    }

    Ok(selection)
}

pub fn build_active_map_selection(
    project: &LizaProject,
    map: &LizaMapPackage,
    bundles_root: &Path,
) -> ActiveMapSelection {
    let local_path = map.local_path.clone().unwrap_or_else(|| {
        project_mobile_maps_dir(bundles_root, &project.summary.slug).join(&map.file_name)
    });

    ActiveMapSelection {
        kind: map_kind_from_local_path(&local_path),
        project_name: project.summary.name.clone(),
        package_name: map.name.clone(),
        remote_url: map.url.clone(),
        local_path,
        center: project.center,
        base_zoom: map.base_zoom,
    }
}

fn parse_center(text: &str) -> Result<MapCenter, String> {
    let regex = Regex::new(r"([0-9]{1,3}\.[0-9]+)").map_err(|err| err.to_string())?;
    let numbers = regex
        .captures_iter(text)
        .filter_map(|captures| captures.get(1))
        .filter_map(|value| value.as_str().parse::<f64>().ok())
        .collect::<Vec<_>>();

    if numbers.len() < 2 {
        return Err("Could not parse project coordinates".to_owned());
    }

    Ok(MapCenter {
        lat: numbers[0],
        lon: numbers[1],
    })
}

fn parse_map_packages(html: &str, base_url: &str) -> Result<Vec<LizaMapPackage>, String> {
    let link_regex = Regex::new(r#"href="([^"]+\.sqlitedb)"[^>]*>([^<]+)</a>"#)
        .map_err(|err| err.to_string())?;
    let zoom_regex = Regex::new(r"_z(\d+)\.sqlitedb$").map_err(|err| err.to_string())?;

    let maps = link_regex
        .captures_iter(html)
        .filter_map(|caps| {
            let href = caps.get(1)?.as_str();
            let label = caps.get(2)?.as_str();
            let zoom = zoom_regex
                .captures(label)
                .and_then(|captures| captures.get(1))?
                .as_str()
                .parse::<u8>()
                .ok()?;

            Some(LizaMapPackage {
                name: label.to_owned(),
                file_name: label.to_owned(),
                url: format!("{base_url}{href}"),
                base_zoom: zoom,
                local_path: None,
            })
        })
        .collect::<Vec<_>>();

    Ok(maps)
}

fn fetch_text(url: &str) -> Result<String, String> {
    let bytes = client()?
        .get(url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|err| err.to_string())?
        .bytes()
        .map_err(|err| err.to_string())?;

    Ok(decode_text_bytes(bytes.as_ref()))
}

fn client() -> Result<Client, String> {
    Client::builder().build().map_err(|err| err.to_string())
}

fn cache_project_at_root<F>(
    summary: &LizaProjectSummary,
    root: &Path,
    on_progress: &mut F,
) -> Result<(), String>
where
    F: FnMut(ProjectOpenProgress),
{
    let source_root = project_source_root(root, &summary.slug);
    fs::create_dir_all(&source_root).map_err(|err| err.to_string())?;
    mirror_remote_directory(&summary.url, &source_root, on_progress)
}

fn mirror_remote_directory<F>(
    url: &str,
    local_dir: &Path,
    on_progress: &mut F,
) -> Result<(), String>
where
    F: FnMut(ProjectOpenProgress),
{
    fs::create_dir_all(local_dir).map_err(|err| err.to_string())?;
    on_progress(ProjectOpenProgress {
        message: format!("Scanning {}", local_dir.display()),
    });
    let html = fetch_text(url)?;

    for entry in parse_directory_entries(&html)? {
        let child_url = format!("{url}{}", entry.href);
        let child_path = local_dir.join(&entry.name);

        if entry.is_dir {
            mirror_remote_directory(&child_url, &child_path, on_progress)?;
        } else {
            on_progress(ProjectOpenProgress {
                message: format!("Downloading {}", child_path.display()),
            });
            download_to_path(&child_url, &child_path)?;
        }
    }

    Ok(())
}

fn download_to_path(url: &str, path: &Path) -> Result<(), String> {
    let client = client()?;
    let mut response = client
        .get(url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|err| err.to_string())?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let mut file = File::create(path).map_err(|err| err.to_string())?;
    let mut buffer = [0u8; 16 * 1024];

    loop {
        let read_bytes = response.read(&mut buffer).map_err(|err| err.to_string())?;
        if read_bytes == 0 {
            break;
        }

        file.write_all(&buffer[..read_bytes])
            .map_err(|err| err.to_string())?;
    }

    Ok(())
}

fn parse_directory_entries(html: &str) -> Result<Vec<DirectoryEntry>, String> {
    let link_regex =
        Regex::new(r#"href="([^"]+)"[^>]*>([^<]+)</a>"#).map_err(|err| err.to_string())?;
    let mut entries = link_regex
        .captures_iter(html)
        .filter_map(|captures| {
            let href = captures.get(1)?.as_str().trim();
            let label = captures.get(2)?.as_str().trim();
            let fallback_name = label.trim_end_matches('/').trim();
            let name = decode_entry_name(href).unwrap_or_else(|| fallback_name.to_owned());

            if href.is_empty()
                || label.is_empty()
                || href == "../"
                || href == "./"
                || href.starts_with('?')
                || href.starts_with('#')
                || name.trim().is_empty()
            {
                return None;
            }

            Some(DirectoryEntry {
                href: href.to_owned(),
                name,
                is_dir: href.ends_with('/'),
            })
        })
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(entries)
}

fn decode_entry_name(href: &str) -> Option<String> {
    let raw_name = href.trim_end_matches('/').rsplit('/').next()?.trim();
    if raw_name.is_empty() {
        return None;
    }

    let mut decoded = String::new();
    let bytes = raw_name.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok()?;
            let value = u8::from_str_radix(hex, 16).ok()?;
            decoded.push(value as char);
            index += 3;
        } else {
            decoded.push(bytes[index] as char);
            index += 1;
        }
    }

    Some(decoded)
}

fn read_text_file_lossy(path: &Path) -> Result<String, std::io::Error> {
    let bytes = fs::read(path)?;
    Ok(decode_text_bytes(&bytes))
}

fn decode_text_bytes(bytes: &[u8]) -> String {
    match String::from_utf8(bytes.to_vec()) {
        Ok(text) => text,
        Err(error) => String::from_utf8_lossy(&error.into_bytes()).into_owned(),
    }
}

fn load_cached_project_from_root(
    summary: LizaProjectSummary,
    root: &Path,
) -> Result<LizaProject, String> {
    let coordinates_text = read_text_file_lossy(&project_coordinates_path(root, &summary.slug))
        .map_err(|err| err.to_string())?;
    let center = parse_center(&coordinates_text)?;
    let maps = read_cached_map_packages(root, &summary.slug)?;

    Ok(LizaProject {
        summary,
        center,
        maps,
    })
}

fn read_cached_map_packages(
    root: &Path,
    project_slug: &str,
) -> Result<Vec<LizaMapPackage>, String> {
    let zoom_regex = Regex::new(r"_z(\d+)\.sqlitedb$").map_err(|err| err.to_string())?;
    let bundle_dir = project_source_root(root, project_slug);
    let mut maps = read_cached_sqlite_map_packages(root, project_slug, &zoom_regex)?;
    // OZI maps: recursive scan of the whole bundle dir (includes extracted/ subdir)
    maps.extend(read_cached_ozi_map_packages(&bundle_dir)?);

    maps.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(maps)
}

fn read_cached_sqlite_map_packages(
    root: &Path,
    project_slug: &str,
    zoom_regex: &Regex,
) -> Result<Vec<LizaMapPackage>, String> {
    let maps_dir = project_mobile_maps_dir(root, project_slug);
    if !maps_dir.exists() {
        return Ok(Vec::new());
    }

    let maps = fs::read_dir(&maps_dir)
        .map_err(|err| err.to_string())?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            let file_name = path.file_name()?.to_str()?.to_owned();
            if !file_name.ends_with(".sqlitedb") {
                return None;
            }

            let base_zoom = zoom_regex
                .captures(&file_name)
                .and_then(|captures| captures.get(1))?
                .as_str()
                .parse::<u8>()
                .ok()?;

            Some(LizaMapPackage {
                name: file_name.clone(),
                file_name,
                url: String::new(),
                base_zoom,
                local_path: Some(path),
            })
        })
        .collect();

    Ok(maps)
}

fn read_cached_ozi_map_packages(source_root: &Path) -> Result<Vec<LizaMapPackage>, String> {
    let mut map_files = Vec::new();
    collect_cached_ozi_map_files(source_root, &mut map_files)?;

    let mut packages = Vec::new();

    for map_path in map_files {
        let contents = read_ozi_map_text(&map_path).map_err(|err| err.to_string())?;
        let metadata =
            parse_ozi_map_metadata(&map_path, &contents).map_err(|err| err.to_string())?;
        let relative_name = map_path
            .strip_prefix(source_root)
            .ok()
            .and_then(|path| path.to_str())
            .map(|path| path.replace(std::path::MAIN_SEPARATOR, "/"))
            .unwrap_or_else(|| {
                map_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown.map")
                    .to_owned()
            });

        packages.push(LizaMapPackage {
            name: format!("OZI: {}", metadata.title()),
            file_name: relative_name,
            url: String::new(),
            base_zoom: 0,
            local_path: Some(map_path),
        });
    }

    Ok(packages)
}

fn collect_cached_ozi_map_files(dir: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();

        if path.is_dir() {
            collect_cached_ozi_map_files(&path, output)?;
            continue;
        }

        if is_ozi_map_path(&path) {
            output.push(path);
        }
    }

    Ok(())
}

fn is_ozi_map_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some(ext) if ext.eq_ignore_ascii_case("map")
    )
}

fn is_zip_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some(ext) if ext.eq_ignore_ascii_case("zip")
    )
}

fn map_kind_from_local_path(path: &Path) -> ActiveMapKind {
    if is_ozi_map_path(path) {
        ActiveMapKind::OziRaster
    } else {
        ActiveMapKind::SqliteTiles
    }
}

fn project_coordinates_path(root: &Path, project_slug: &str) -> PathBuf {
    project_source_root(root, project_slug).join("2-Coordinates.txt")
}

fn project_mobile_maps_dir(root: &Path, project_slug: &str) -> PathBuf {
    project_source_root(root, project_slug).join(MOBILE_MAPS_DIR_NAME)
}

fn project_source_root(root: &Path, project_slug: &str) -> PathBuf {
    root.join(project_slug)
}

fn project_extracted_root(root: &Path, project_slug: &str) -> PathBuf {
    root.join(project_slug).join(PROJECT_EXTRACTED_DIR)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DirectoryEntry {
    href: String,
    name: String,
    is_dir: bool,
}

fn materialize_cached_ozi_archives<F>(
    root: &Path,
    project_slug: &str,
    on_progress: &mut F,
) -> Result<(), String>
where
    F: FnMut(ProjectOpenProgress),
{
    let source_root = project_source_root(root, project_slug);
    let extracted_root = project_extracted_root(root, project_slug);
    let mut zip_files = Vec::new();
    collect_cached_zip_files(&source_root, &mut zip_files)?;

    for zip_path in zip_files {
        if !is_ozi_archive_file(&zip_path)? {
            continue;
        }

        let destination = extraction_destination_for_archive(&extracted_root, &zip_path);
        if destination.exists() {
            continue;
        }

        on_progress(ProjectOpenProgress {
            message: format!("Extracting {}", zip_path.display()),
        });
        extract_cached_archive(&zip_path, &destination)?;
    }

    Ok(())
}

fn collect_cached_zip_files(dir: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();

        if path.is_dir() {
            collect_cached_zip_files(&path, output)?;
            continue;
        }

        if is_zip_path(&path) {
            output.push(path);
        }
    }

    Ok(())
}

fn is_ozi_archive_file(path: &Path) -> Result<bool, String> {
    let bytes = fs::read(path).map_err(|err| err.to_string())?;
    let entries = inventory_zip_entries(Cursor::new(bytes)).map_err(|err| err.to_string())?;

    Ok(entries.iter().any(|entry| {
        matches!(
            entry.kind(),
            ArchiveEntryKind::Supported(SupportedArchiveEntryKind::OziMap)
                | ArchiveEntryKind::Unsupported(_)
        ) && !matches!(
            entry.kind(),
            ArchiveEntryKind::Unsupported(
                crate::infrastructure::import::UnsupportedArchiveEntryKind::SqliteTiles
            ) | ArchiveEntryKind::Unsupported(
                crate::infrastructure::import::UnsupportedArchiveEntryKind::Unknown
            )
        )
    }))
}

fn extract_cached_archive(archive_path: &Path, destination: &Path) -> Result<(), String> {
    let bytes = fs::read(archive_path).map_err(|err| err.to_string())?;
    extract_zip_entries_to_directory(Cursor::new(bytes), destination)
        .map_err(|err| err.to_string())?;
    Ok(())
}

fn extraction_destination_for_archive(extracted_root: &Path, archive_path: &Path) -> PathBuf {
    let stem = archive_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("archive");

    extracted_root.join(stem)
}

#[cfg(test)]
mod tests {
    use super::{
        decode_text_bytes, load_cached_project_from_root, materialize_cached_ozi_archives,
        parse_center, parse_directory_entries, parse_map_packages, read_text_file_lossy,
    };
    use crate::application::LizaProjectSummary;
    use std::fs;
    use std::io::{Cursor, Write};
    use std::time::{SystemTime, UNIX_EPOCH};
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    #[test]
    fn parse_center_reads_decimal_coordinates() {
        let center = parse_center("N 54.32821 E 048.40917").expect("center");

        assert_eq!(center.lat, 54.32821);
        assert_eq!(center.lon, 48.40917);
    }

    #[test]
    fn parse_map_packages_reads_sqlite_entries() {
        let html =
            r#"<a href="foo_z16.sqlitedb">foo_z16.sqlitedb</a><a href="bar.txt">bar.txt</a>"#;
        let maps = parse_map_packages(html, "https://example.com/").expect("maps");

        assert_eq!(maps.len(), 1);
        assert_eq!(maps[0].base_zoom, 16);
        assert_eq!(maps[0].url, "https://example.com/foo_z16.sqlitedb");
        assert_eq!(maps[0].local_path, None);
    }

    #[test]
    fn parse_directory_entries_skips_parent_links_and_marks_directories() {
        let html = r#"
            <a href="../">../</a>
            <a href="8-Android%26iOS/">8-Android&amp;iOS/</a>
            <a href="5-Ozi.zip">5-Ozi.zip</a>
        "#;

        let entries = parse_directory_entries(html).expect("entries");

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "5-Ozi.zip");
        assert!(!entries[0].is_dir);
        assert_eq!(entries[1].name, "8-Android&iOS");
        assert!(entries[1].is_dir);
    }

    #[test]
    fn load_cached_project_from_root_reads_local_coordinates_and_ozi_maps() {
        let root = write_cached_project_fixture();
        let summary = LizaProjectSummary {
            slug: "2026-03-29_demo".to_owned(),
            name: "2026-03-29 demo".to_owned(),
            url: "https://example.test/project/".to_owned(),
        };

        let project = load_cached_project_from_root(summary.clone(), &root).expect("project");

        assert_eq!(project.summary, summary);
        assert_eq!(project.center.lat, 54.32821);
        assert_eq!(project.center.lon, 48.40917);
        assert_eq!(project.maps.len(), 2);
        assert_eq!(project.maps[0].base_zoom, 0);
        assert_eq!(project.maps[0].name, "OZI: Demo topo");
        let expected_ozi_path = root.join("2026-03-29_demo/5-Ozi/Maps/demo.map");
        assert_eq!(
            project.maps[0].local_path.as_deref(),
            Some(expected_ozi_path.as_path())
        );
        assert_eq!(project.maps[1].base_zoom, 16);
        let expected_sqlite_path = root.join("2026-03-29_demo/8-Android&iOS/demo_z16.sqlitedb");
        assert_eq!(
            project.maps[1].local_path.as_deref(),
            Some(expected_sqlite_path.as_path())
        );
    }

    #[test]
    fn materialize_cached_ozi_archives_extracts_zip_for_cached_project_indexing() {
        let root = write_cached_project_zip_fixture();

        materialize_cached_ozi_archives(&root, "2026-03-29_demo", &mut |_| {})
            .expect("extract ozi archives");

        let summary = LizaProjectSummary {
            slug: "2026-03-29_demo".to_owned(),
            name: "2026-03-29 demo".to_owned(),
            url: "https://example.test/project/".to_owned(),
        };
        let project = load_cached_project_from_root(summary, &root).expect("project");

        assert_eq!(project.maps.len(), 2);
        assert_eq!(project.maps[0].name, "OZI: Demo topo");
        assert!(
            project.maps[0]
                .local_path
                .as_ref()
                .expect("ozi path")
                .to_string_lossy()
                .contains("extracted/5-Ozi(Win&Android)_Topo/Maps/demo.map")
        );
    }

    #[test]
    fn decode_text_bytes_falls_back_lossy_for_non_utf8() {
        let text = decode_text_bytes(b"demo \xFF bundle");

        assert!(text.contains("demo "));
        assert!(text.contains("bundle"));
    }

    #[test]
    fn read_text_file_lossy_reads_non_utf8_coordinates_file() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "ozi-rs-non-utf8-coordinates-{}-{unique}.txt",
            std::process::id()
        ));
        fs::write(&path, b"N 54.32821 E 048.40917 \xFF").expect("write coordinates bytes");

        let text = read_text_file_lossy(&path).expect("lossy coordinates text");

        assert!(text.contains("54.32821"));
        assert!(text.contains("048.40917"));
    }

    fn write_cached_project_fixture() -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("ozi-rs-lizaalert-cache-{unique}"));
        // Flat structure: files directly in {root}/{slug}/, no source/ subdir
        let bundle_dir = root.join("2026-03-29_demo");
        let mobile_dir = bundle_dir.join("8-Android&iOS");
        let ozi_dir = bundle_dir.join("5-Ozi/Maps");
        fs::create_dir_all(&mobile_dir).expect("create mobile dir");
        fs::create_dir_all(&ozi_dir).expect("create ozi dir");
        fs::write(
            bundle_dir.join("2-Coordinates.txt"),
            "N 54.32821 E 048.40917",
        )
        .expect("write coordinates");
        fs::write(mobile_dir.join("demo_z16.sqlitedb"), []).expect("write sqlite placeholder");
        fs::write(ozi_dir.join("demo.map"), sample_ozi_map()).expect("write ozi map");
        root
    }

    fn write_cached_project_zip_fixture() -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("ozi-rs-lizaalert-cache-zip-{unique}"));
        // Flat structure: files directly in {root}/{slug}/, no source/ subdir
        let bundle_dir = root.join("2026-03-29_demo");
        let mobile_dir = bundle_dir.join("8-Android&iOS");
        fs::create_dir_all(&mobile_dir).expect("create mobile dir");
        fs::write(
            bundle_dir.join("2-Coordinates.txt"),
            "N 54.32821 E 048.40917",
        )
        .expect("write coordinates");
        fs::write(mobile_dir.join("demo_z16.sqlitedb"), []).expect("write sqlite placeholder");
        fs::write(
            bundle_dir.join("5-Ozi(Win&Android)_Topo.zip"),
            build_archive(&[
                ("Maps/demo.map", sample_ozi_map().as_bytes(), false),
                ("Maps/demo.ozf2", b"ozf-placeholder".as_slice(), false),
            ]),
        )
        .expect("write ozi zip");
        root
    }

    fn build_archive(entries: &[(&str, &[u8], bool)]) -> Vec<u8> {
        let mut buffer = Cursor::new(Vec::new());
        let mut writer = ZipWriter::new(&mut buffer);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        for (path, contents, is_directory) in entries {
            if *is_directory {
                writer.add_directory(*path, options).expect("directory");
                continue;
            }

            writer.start_file(*path, options).expect("file");
            writer.write_all(contents).expect("contents");
        }

        writer.finish().expect("finish");
        buffer.into_inner()
    }

    fn sample_ozi_map() -> &'static str {
        "OziExplorer Map Data File Version 2.2\nDemo topo\ndemo.ozf2\n1 ,Map Code,\nWGS 84\nReserved 1\nReserved 2\nMagnetic Variation,,,E\nMap Projection,Latitude/Longitude,PolyCal,No,AutoCalOnly,No,BSBUseWPX,No\nPoint01,xy,10,20,in, deg,54,30.000,N,48,24.000,E, grid, , , ,N\nProjection Setup,,,,,,,,,,\n"
    }
}
