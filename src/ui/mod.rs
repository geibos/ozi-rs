mod sqlite_tiles;

use crate::application::{ActiveMapKind, AppState, DiagnosticEntry, DiagnosticLevel};
use crate::infrastructure::import::{
    decode_ozi_raster_image, parse_ozi_map_metadata, read_ozi_map_text,
};
use eframe::egui;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use walkers::{HttpTiles, Map, MapMemory, Position, lon_lat, sources::OpenStreetMap};

use self::sqlite_tiles::SqliteTiles;

const MAX_OZI_TEXTURE_SIDE: usize = 8192;

pub struct OziApp {
    project_search: String,
    state: AppState,
    fps_counter: FpsCounter,
    loaded_map_path: Option<PathBuf>,
    map_center: Position,
    map_memory: MapMemory,
    offline_tiles: Option<SqliteTiles>,
    ozi_texture: Option<egui::TextureHandle>,
    osm_tiles: HttpTiles,
}

struct FpsCounter {
    frame_count: u32,
    fps: f32,
    last_sample_at: Instant,
}

impl FpsCounter {
    fn new() -> Self {
        Self {
            frame_count: 0,
            fps: 0.0,
            last_sample_at: Instant::now(),
        }
    }

    fn tick(&mut self) {
        self.frame_count += 1;

        let elapsed = self.last_sample_at.elapsed();
        if elapsed >= Duration::from_millis(500) {
            self.fps = self.frame_count as f32 / elapsed.as_secs_f32();
            self.frame_count = 0;
            self.last_sample_at = Instant::now();
        }
    }

    fn label(&self) -> String {
        format!("FPS: {:.1}", self.fps)
    }
}

impl OziApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            project_search: String::new(),
            state: AppState::new(),
            fps_counter: FpsCounter::new(),
            loaded_map_path: None,
            map_center: lon_lat(37.6176, 55.7558),
            map_memory: MapMemory::default(),
            offline_tiles: None,
            ozi_texture: None,
            osm_tiles: HttpTiles::new(OpenStreetMap, cc.egui_ctx.clone()),
        }
    }

    fn sync_active_map(&mut self, ctx: &egui::Context) {
        let Some(active_map) = self.state.active_map() else {
            return;
        };

        if self
            .loaded_map_path
            .as_ref()
            .is_some_and(|path| path == &active_map.local_path)
        {
            return;
        }

        match active_map.kind {
            ActiveMapKind::SqliteTiles => {
                match SqliteTiles::open(&active_map.local_path, ctx.clone(), active_map.base_zoom) {
                    Ok(offline_tiles) => {
                        self.map_center = lon_lat(active_map.center.lon, active_map.center.lat);
                        self.map_memory.center_at(self.map_center);
                        let _ = self.map_memory.set_zoom(active_map.base_zoom.into());
                        self.loaded_map_path = Some(active_map.local_path.clone());
                        self.offline_tiles = Some(offline_tiles);
                        self.ozi_texture = None;
                    }
                    Err(error) => {
                        self.loaded_map_path = None;
                        self.offline_tiles = None;
                        self.ozi_texture = None;
                        self.state.report_runtime_error(format!(
                            "Failed to open downloaded map: {error}"
                        ));
                    }
                }
            }
            ActiveMapKind::OziRaster => match load_ozi_texture(ctx, &active_map.local_path) {
                Ok(texture) => {
                    self.loaded_map_path = Some(active_map.local_path.clone());
                    self.offline_tiles = None;
                    self.ozi_texture = Some(texture);
                }
                Err(error) => {
                    self.loaded_map_path = None;
                    self.offline_tiles = None;
                    self.ozi_texture = None;
                    self.state
                        .report_runtime_error(format!("Failed to open OZI map: {error}"));
                }
            },
        }
    }

    fn show_project_sidebar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("lizaalert_projects")
            .resizable(true)
            .default_size(280.0)
            .show_inside(ui, |ui| {
                ui.heading("Map Picker");
                ui.label("Source: maps.lizaalert.ru");

                if ui.button("Refresh projects").clicked() {
                    self.state.load_projects();
                }

                if self.state.lizaalert_busy() {
                    ui.spinner();
                }

                ui.label(self.state.lizaalert_status());
                self.show_diagnostics_console(ui);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Projects");
                    ui.label(format!("({})", self.state.lizaalert_projects().len()));
                });
                ui.add(
                    egui::TextEdit::singleline(&mut self.project_search)
                        .hint_text("Search projects"),
                );

                let project_query = self.project_search.trim();
                let mut visible_projects = 0usize;
                let mut selected_project_slug = None;

                egui::ScrollArea::vertical()
                    .max_height(240.0)
                    .show(ui, |ui| {
                        for project in self.state.lizaalert_projects() {
                            if !project_matches_query(project, project_query) {
                                continue;
                            }

                            visible_projects += 1;
                            if ui.button(&project.name).clicked() {
                                selected_project_slug = Some(project.slug.clone());
                            }
                        }
                    });

                if let Some(project_slug) = selected_project_slug {
                    self.state.load_project(&project_slug);
                }

                if visible_projects == 0 {
                    ui.label("No projects match the current search.");
                }

                let mut selected_map_name = None;

                if let Some(project) = self.state.current_project() {
                    ui.separator();
                    ui.label(format!("Selected project: {}", project.summary.name));
                    ui.label(format!(
                        "Center: {:.5}, {:.5}",
                        project.center.lat, project.center.lon
                    ));
                    ui.separator();
                    ui.label("Available cached project maps");

                    for map in &project.maps {
                        if ui.button(&map.name).clicked() {
                            selected_map_name = Some(map.name.clone());
                        }
                    }
                }

                if let Some(map_name) = selected_map_name {
                    self.state.open_selected_map(&map_name);
                }
            });
    }

    fn show_diagnostics_console(&self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Diagnostics")
            .default_open(true)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("diagnostics_console")
                    .max_height(160.0)
                    .show(ui, |ui| {
                        for entry in self.state.recent_diagnostics().rev() {
                            render_diagnostic_entry(ui, entry);
                        }
                    });
            });
    }
}

impl eframe::App for OziApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.state.poll_background_tasks();
        self.sync_active_map(ctx);
        self.fps_counter.tick();
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.state.window_title()));

        if self.state.lizaalert_busy() {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show_project_sidebar(ui);

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("ozi-rs");
                ui.label(format!("Project: {}", self.state.project_name()));
                ui.separator();
                ui.monospace(self.fps_counter.label());
            });

            if let Some(active_map) = self.state.active_map() {
                ui.label(format!(
                    "Loaded project map: {} / {}",
                    active_map.project_name, active_map.package_name
                ));
                if active_map.kind == ActiveMapKind::OziRaster {
                    ui.label(format!("OZI source: {}", active_map.local_path.display()));
                }
            } else {
                ui.label(
                    "Fallback map: OpenStreetMap. Select a LizaAlert project map on the left.",
                );
            }
            ui.label(format!(
                "Registered project maps: {}",
                self.state.map_layer_count()
            ));

            ui.separator();

            if let Some(texture) = self.ozi_texture.as_ref() {
                render_ozi_texture(ui, texture);
            } else {
                let map = if let Some(offline_tiles) = self.offline_tiles.as_mut() {
                    Map::new(Some(offline_tiles), &mut self.map_memory, self.map_center)
                } else {
                    Map::new(
                        Some(&mut self.osm_tiles),
                        &mut self.map_memory,
                        self.map_center,
                    )
                };

                ui.add(map);
            }
        });
    }
}

fn project_matches_query(project: &crate::application::LizaProjectSummary, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let query = query.to_ascii_lowercase();
    project.name.to_ascii_lowercase().contains(&query)
        || project.slug.to_ascii_lowercase().contains(&query)
}

fn render_diagnostic_entry(ui: &mut egui::Ui, entry: &DiagnosticEntry) {
    let (prefix, color) = match entry.level() {
        DiagnosticLevel::Info => ("INFO", egui::Color32::LIGHT_GRAY),
        DiagnosticLevel::Error => ("ERROR", egui::Color32::LIGHT_RED),
    };

    ui.colored_label(color, format!("[{prefix}] {}", entry.message()));
}

fn load_ozi_texture(
    ctx: &egui::Context,
    map_path: &std::path::Path,
) -> Result<egui::TextureHandle, String> {
    let map_contents = read_ozi_map_text(map_path).map_err(|error| error.to_string())?;
    let metadata =
        parse_ozi_map_metadata(map_path, &map_contents).map_err(|error| error.to_string())?;
    let image = decode_ozi_raster_image(&metadata).map_err(|error| error.to_string())?;
    let prepared_image = prepare_ozi_color_image(
        image.width() as usize,
        image.height() as usize,
        image.rgba_pixels(),
        MAX_OZI_TEXTURE_SIDE,
    );

    Ok(ctx.load_texture(
        format!("ozi-raster:{}", map_path.display()),
        prepared_image,
        egui::TextureOptions::LINEAR,
    ))
}

fn prepare_ozi_color_image(
    width: usize,
    height: usize,
    rgba_pixels: &[u8],
    texture_limit: usize,
) -> egui::ColorImage {
    let (prepared_width, prepared_height) = fit_size_within_limit(width, height, texture_limit);

    if prepared_width == width && prepared_height == height {
        return egui::ColorImage::from_rgba_unmultiplied([width, height], rgba_pixels);
    }

    let resized_pixels =
        resize_rgba_nearest(rgba_pixels, width, height, prepared_width, prepared_height);
    egui::ColorImage::from_rgba_unmultiplied([prepared_width, prepared_height], &resized_pixels)
}

fn fit_size_within_limit(width: usize, height: usize, limit: usize) -> (usize, usize) {
    if width <= limit && height <= limit {
        return (width, height);
    }

    let scale = (limit as f64 / width as f64).min(limit as f64 / height as f64);
    let fitted_width = ((width as f64 * scale).floor() as usize).max(1);
    let fitted_height = ((height as f64 * scale).floor() as usize).max(1);
    (fitted_width, fitted_height)
}

fn resize_rgba_nearest(
    rgba_pixels: &[u8],
    source_width: usize,
    source_height: usize,
    target_width: usize,
    target_height: usize,
) -> Vec<u8> {
    let mut resized = vec![0; target_width * target_height * 4];

    for target_y in 0..target_height {
        let source_y = target_y * source_height / target_height;

        for target_x in 0..target_width {
            let source_x = target_x * source_width / target_width;
            let source_offset = (source_y * source_width + source_x) * 4;
            let target_offset = (target_y * target_width + target_x) * 4;
            resized[target_offset..target_offset + 4]
                .copy_from_slice(&rgba_pixels[source_offset..source_offset + 4]);
        }
    }

    resized
}

fn render_ozi_texture(ui: &mut egui::Ui, texture: &egui::TextureHandle) {
    let image_size = texture.size_vec2();
    let available_size = ui.available_size();
    let scale = (available_size.x / image_size.x)
        .min(available_size.y / image_size.y)
        .max(0.1);
    let desired_size = image_size * scale.min(1.0);

    egui::ScrollArea::both().show(ui, |ui| {
        ui.add(egui::Image::new(texture).fit_to_exact_size(desired_size));
    });
}

#[cfg(test)]
mod tests {
    use super::{fit_size_within_limit, prepare_ozi_color_image, project_matches_query};
    use crate::application::LizaProjectSummary;

    fn sample_project() -> LizaProjectSummary {
        LizaProjectSummary {
            slug: "2026-03-28_demo-project".to_owned(),
            name: "2026-03-28 demo project".to_owned(),
            url: "https://example.test/project".to_owned(),
        }
    }

    #[test]
    fn project_search_matches_empty_query() {
        assert!(project_matches_query(&sample_project(), ""));
    }

    #[test]
    fn project_search_matches_name_case_insensitively() {
        assert!(project_matches_query(&sample_project(), "DEMO"));
    }

    #[test]
    fn project_search_matches_slug_case_insensitively() {
        assert!(project_matches_query(&sample_project(), "PROJECT"));
    }

    #[test]
    fn project_search_rejects_non_matching_query() {
        assert!(!project_matches_query(&sample_project(), "missing"));
    }

    #[test]
    fn fit_size_within_limit_keeps_smaller_images_unchanged() {
        assert_eq!(fit_size_within_limit(2048, 1024, 8192), (2048, 1024));
    }

    #[test]
    fn fit_size_within_limit_scales_large_images_proportionally() {
        assert_eq!(fit_size_within_limit(8961, 2817, 8192), (8192, 2575));
    }

    #[test]
    fn prepare_ozi_color_image_downscales_oversized_rasters() {
        let pixels = vec![255; 8961 * 2 * 4];

        let image = prepare_ozi_color_image(8961, 2, &pixels, 8192);

        assert_eq!(image.size, [8192, 1]);
    }
}
