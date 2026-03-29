mod sqlite_tiles;

use crate::application::{ActiveMapKind, AppState, DiagnosticEntry, DiagnosticLevel};
use crate::infrastructure::import::{decode_ozi_raster_image, parse_ozi_map_metadata};
use eframe::egui;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use walkers::{HttpTiles, Map, MapMemory, Position, lon_lat, sources::OpenStreetMap};

use self::sqlite_tiles::SqliteTiles;

pub struct OziApp {
    local_ozi_map_path: String,
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
            local_ozi_map_path:
                "example_data/2021-07-30_Murino/5-Ozi(Win&Android)_Topo_EEKO/Maps/2021-07-30_Murino_Topo_EEKO_z16_ozf.map"
                    .to_owned(),
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
        egui::SidePanel::left("lizaalert_projects")
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
                ui.heading("Local OZI map");
                ui.add(
                    egui::TextEdit::singleline(&mut self.local_ozi_map_path)
                        .hint_text("Path to .map file"),
                );
                if ui.button("Open OZI map").clicked() {
                    let path = self.local_ozi_map_path.trim();
                    if path.is_empty() {
                        self.state
                            .report_runtime_error("Local OZI map path is empty".to_owned());
                    } else if let Err(error) = self.state.open_local_ozi_map(path) {
                        self.state.report_runtime_error(error.to_string());
                    }
                }
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
    let map_contents = std::fs::read_to_string(map_path).map_err(|error| error.to_string())?;
    let metadata =
        parse_ozi_map_metadata(map_path, &map_contents).map_err(|error| error.to_string())?;
    let image = decode_ozi_raster_image(&metadata).map_err(|error| error.to_string())?;
    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize],
        image.rgba_pixels(),
    );

    Ok(ctx.load_texture(
        format!("ozi-raster:{}", map_path.display()),
        color_image,
        egui::TextureOptions::LINEAR,
    ))
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
    use super::project_matches_query;
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
}
