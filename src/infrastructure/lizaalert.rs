use crate::application::{
    ActiveMapSelection, LizaMapPackage, LizaProject, LizaProjectSummary, MapCenter,
};
use regex::Regex;
use reqwest::blocking::Client;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const ROOT_URL: &str = "https://maps.lizaalert.ru/maps/";
const MOBILE_MAPS_DIR: &str = "8-Android%26iOS/";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
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
    let maps_url = format!("{}{}/{MOBILE_MAPS_DIR}", ROOT_URL, summary.slug);

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
) -> ActiveMapSelection {
    let local_path = cache_path(&project.summary.slug, &map.file_name);

    ActiveMapSelection {
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
            })
        })
        .collect::<Vec<_>>();

    Ok(maps)
}

fn fetch_text(url: &str) -> Result<String, String> {
    client()?
        .get(url)
        .send()
        .and_then(|response| response.error_for_status())
        .map_err(|err| err.to_string())?
        .text()
        .map_err(|err| err.to_string())
}

fn client() -> Result<Client, String> {
    Client::builder().build().map_err(|err| err.to_string())
}

fn cache_path(project_slug: &str, file_name: &str) -> PathBuf {
    cache_root().join(project_slug).join(file_name)
}

fn cache_root() -> PathBuf {
    Path::new(".tmp").join("lizaalert-maps")
}

#[cfg(test)]
mod tests {
    use super::{parse_center, parse_map_packages};

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
    }
}
