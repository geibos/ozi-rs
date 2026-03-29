mod sqlite_tiles;

use crate::application::{ActiveMapKind, AppState, DiagnosticEntry, DiagnosticLevel};
use crate::infrastructure::import::{
    OziRasterLevelMetadata, OziRasterTileSource, open_ozi_raster_tile_source,
    parse_ozi_map_metadata, read_ozi_map_text,
};
use eframe::egui;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use walkers::{HttpTiles, Map, MapMemory, Position, lon_lat, sources::OpenStreetMap};

use self::sqlite_tiles::SqliteTiles;

const OZI_TILE_TEXTURE_CACHE_SIZE: usize = 256;
const OZI_MIN_ZOOM_FACTOR: f32 = 0.25;
const OZI_MAX_OVERZOOM_FACTOR: f32 = 8.0;

pub struct OziApp {
    project_search: String,
    state: AppState,
    fps_counter: FpsCounter,
    loaded_map_path: Option<PathBuf>,
    map_center: Position,
    map_memory: MapMemory,
    offline_tiles: Option<SqliteTiles>,
    ozi_renderer: Option<OziRasterRenderer>,
    osm_tiles: HttpTiles,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct OziTileCacheKey {
    level_index: usize,
    tile_x: u32,
    tile_y: u32,
}

struct OziRasterRenderer {
    tile_source: OziRasterTileSource,
    textures: LruCache<OziTileCacheKey, egui::TextureHandle>,
    viewport: OziViewportState,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct OziViewportState {
    zoom: f32,
    top_left_base_pixels: egui::Vec2,
    initialized: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct BaseImageRect {
    min: egui::Vec2,
    size: egui::Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct VisibleTileRange {
    start_x: u32,
    end_x: u32,
    start_y: u32,
    end_y: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct OziNativeLevelSelection {
    level_index: usize,
    level_scale_x: f32,
    level_scale_y: f32,
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
            ozi_renderer: None,
            osm_tiles: HttpTiles::new(OpenStreetMap, cc.egui_ctx.clone()),
        }
    }

    fn sync_active_map(&mut self, ctx: &egui::Context) {
        let Some(active_map) = self.state.active_map() else {
            if self.loaded_map_path.is_some() {
                self.loaded_map_path = None;
                self.offline_tiles = None;
                self.ozi_renderer = None;
            }
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
                        self.ozi_renderer = None;
                    }
                    Err(error) => {
                        self.loaded_map_path = None;
                        self.offline_tiles = None;
                        self.ozi_renderer = None;
                        self.state.report_runtime_error(format!(
                            "Failed to open downloaded map: {error}"
                        ));
                    }
                }
            }
            ActiveMapKind::OziRaster => match OziRasterRenderer::open(&active_map.local_path) {
                Ok(renderer) => {
                    self.loaded_map_path = Some(active_map.local_path.clone());
                    self.offline_tiles = None;
                    self.ozi_renderer = Some(renderer);
                }
                Err(error) => {
                    self.loaded_map_path = None;
                    self.offline_tiles = None;
                    self.ozi_renderer = None;
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
                if ui.button("Open…").clicked() {
                    if let Some(path) = pick_project_file() {
                        self.state.load_project_from(path);
                    }
                }
                let save_label = if self.state.project_file_path().is_some() {
                    "Save"
                } else {
                    "Save As…"
                };
                if ui.button(save_label).clicked() {
                    let path = self
                        .state
                        .project_file_path()
                        .map(PathBuf::from)
                        .or_else(save_project_file_dialog);
                    if let Some(path) = path {
                        self.state.save_project_to(path);
                    }
                }
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

            if let Some(renderer) = self.ozi_renderer.as_mut() {
                if let Err(error) = renderer.render(ui) {
                    ui.colored_label(egui::Color32::LIGHT_RED, error);
                }
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

fn pick_project_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("ozi-rs project", &["ozp"])
        .pick_file()
}

fn save_project_file_dialog() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("ozi-rs project", &["ozp"])
        .set_file_name("project.ozp")
        .save_file()
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

impl OziRasterRenderer {
    fn open(map_path: &std::path::Path) -> Result<Self, String> {
        let map_contents = read_ozi_map_text(map_path).map_err(|error| error.to_string())?;
        let metadata =
            parse_ozi_map_metadata(map_path, &map_contents).map_err(|error| error.to_string())?;
        let tile_source =
            open_ozi_raster_tile_source(&metadata).map_err(|error| error.to_string())?;

        Ok(Self {
            tile_source,
            textures: LruCache::new(
                NonZeroUsize::new(OZI_TILE_TEXTURE_CACHE_SIZE).expect("cache size"),
            ),
            viewport: OziViewportState::default(),
        })
    }

    fn render(&mut self, ui: &mut egui::Ui) -> Result<(), String> {
        let available_size = ui.available_size();
        let desired_size = egui::vec2(available_size.x.max(1.0), available_size.y.max(1.0));
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::drag());
        let painter = ui.painter_at(rect);

        painter.rect_filled(rect, 0.0, egui::Color32::BLACK);

        let Some(base_level) = self.tile_source.levels().first() else {
            return Err("OZI raster has no native levels".to_owned());
        };

        self.viewport.ensure_initialized(base_level, rect.size());
        self.viewport
            .apply_interaction(&response, rect, base_level, ui);

        let base_image_size = egui::vec2(base_level.width() as f32, base_level.height() as f32);
        let base_view_size = rect.size() / self.viewport.zoom;
        let image_rect = BaseImageRect {
            min: self.viewport.top_left_base_pixels,
            size: base_view_size,
        };
        let level_selection =
            select_native_ozi_level(self.tile_source.levels(), self.viewport.zoom)
                .ok_or_else(|| "OZI raster has no native levels".to_owned())?;
        let level = self
            .tile_source
            .level(level_selection.level_index)
            .cloned()
            .ok_or_else(|| "selected OZI level is out of bounds".to_owned())?;
        let visible_tiles = visible_ozi_tiles(&level, image_rect, level_selection)?;

        for tile_y in visible_tiles.start_y..=visible_tiles.end_y {
            for tile_x in visible_tiles.start_x..=visible_tiles.end_x {
                let Ok((texture, tile_size_level_pixels)) =
                    self.load_tile_texture(ui.ctx(), level_selection.level_index, tile_x, tile_y)
                else {
                    continue;
                };
                let tile_rect = ozi_tile_screen_rect(
                    rect,
                    image_rect.min,
                    self.viewport.zoom,
                    &level,
                    level_selection,
                    tile_x,
                    tile_y,
                    tile_size_level_pixels,
                );
                painter.image(
                    texture.id(),
                    tile_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
        }

        let drawn_image_rect =
            ozi_image_screen_rect(rect, image_rect.min, self.viewport.zoom, base_image_size);
        painter.rect_stroke(
            drawn_image_rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
            egui::StrokeKind::Outside,
        );

        Ok(())
    }

    fn load_tile_texture(
        &mut self,
        ctx: &egui::Context,
        level_index: usize,
        tile_x: u32,
        tile_y: u32,
    ) -> Result<(egui::TextureHandle, egui::Vec2), String> {
        let key = OziTileCacheKey {
            level_index,
            tile_x,
            tile_y,
        };

        if let Some(texture) = self.textures.get(&key) {
            let level = self
                .tile_source
                .level(level_index)
                .ok_or_else(|| "selected OZI level is out of bounds".to_owned())?;
            let (width, height) = level
                .tile_pixel_size(tile_x, tile_y)
                .ok_or_else(|| "selected OZI tile is out of bounds".to_owned())?;

            return Ok((texture.clone(), egui::vec2(width as f32, height as f32)));
        }

        let tile = self
            .tile_source
            .decode_rgba_tile(level_index, tile_x, tile_y)
            .map_err(|error| error.to_string())?;
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [tile.width() as usize, tile.height() as usize],
            tile.rgba_pixels(),
        );
        let texture = ctx.load_texture(
            format!(
                "ozi-raster:{}:{level_index}:{tile_x}:{tile_y}",
                self.tile_source.source_path().display()
            ),
            image,
            egui::TextureOptions::LINEAR,
        );

        self.textures.put(key, texture.clone());
        Ok((
            texture,
            egui::vec2(tile.width() as f32, tile.height() as f32),
        ))
    }
}

impl Default for OziViewportState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            top_left_base_pixels: egui::Vec2::ZERO,
            initialized: false,
        }
    }
}

impl OziViewportState {
    fn ensure_initialized(&mut self, level: &OziRasterLevelMetadata, viewport_size: egui::Vec2) {
        if self.initialized {
            return;
        }

        let image_size = egui::vec2(level.width() as f32, level.height() as f32);
        self.zoom = fit_ozi_zoom(image_size, viewport_size).clamp(OZI_MIN_ZOOM_FACTOR, 1.0);
        self.top_left_base_pixels = centered_ozi_top_left(image_size, viewport_size, self.zoom);
        self.initialized = true;
    }

    fn apply_interaction(
        &mut self,
        response: &egui::Response,
        rect: egui::Rect,
        level: &OziRasterLevelMetadata,
        ui: &egui::Ui,
    ) {
        let image_size = egui::vec2(level.width() as f32, level.height() as f32);

        if response.dragged() {
            let delta = ui.input(|input| input.pointer.delta());
            self.top_left_base_pixels -= delta / self.zoom;
        }

        if response.hovered() {
            // Two-finger scroll pan (matches walkers panning behavior)
            let scroll_delta = ui.input(|input| input.smooth_scroll_delta);
            if scroll_delta != egui::Vec2::ZERO {
                self.top_left_base_pixels -= scroll_delta / self.zoom;
            }

            // Pinch or Ctrl+scroll zoom (matches walkers zoom_delta behavior)
            let zoom_delta = ui.input(|input| input.zoom_delta());
            if (zoom_delta - 1.0).abs() > f32::EPSILON {
                let min_zoom =
                    fit_ozi_zoom(image_size, rect.size()).max(0.01) * OZI_MIN_ZOOM_FACTOR;
                let max_zoom = OZI_MAX_OVERZOOM_FACTOR;
                let pointer_position = ui
                    .input(|input| input.pointer.hover_pos())
                    .unwrap_or(rect.center());
                let pointer_offset = pointer_position - rect.min;
                let anchor = self.top_left_base_pixels + pointer_offset / self.zoom;
                let new_zoom = (self.zoom * zoom_delta).clamp(min_zoom, max_zoom);

                self.top_left_base_pixels = anchor - pointer_offset / new_zoom;
                self.zoom = new_zoom;
            }
        }

        self.top_left_base_pixels = clamp_ozi_top_left(
            self.top_left_base_pixels,
            image_size,
            rect.size(),
            self.zoom,
        );
    }
}

fn fit_ozi_zoom(image_size: egui::Vec2, viewport_size: egui::Vec2) -> f32 {
    (viewport_size.x / image_size.x)
        .min(viewport_size.y / image_size.y)
        .max(0.01)
}

fn centered_ozi_top_left(
    image_size: egui::Vec2,
    viewport_size: egui::Vec2,
    zoom: f32,
) -> egui::Vec2 {
    let view_size = viewport_size / zoom;
    egui::vec2(
        center_ozi_axis(image_size.x, view_size.x),
        center_ozi_axis(image_size.y, view_size.y),
    )
}

fn center_ozi_axis(image_extent: f32, view_extent: f32) -> f32 {
    if view_extent >= image_extent {
        -((view_extent - image_extent) * 0.5)
    } else {
        0.0
    }
}

fn clamp_ozi_top_left(
    top_left: egui::Vec2,
    image_size: egui::Vec2,
    viewport_size: egui::Vec2,
    zoom: f32,
) -> egui::Vec2 {
    let view_size = viewport_size / zoom;
    egui::vec2(
        clamp_ozi_axis(top_left.x, image_size.x, view_size.x),
        clamp_ozi_axis(top_left.y, image_size.y, view_size.y),
    )
}

fn clamp_ozi_axis(offset: f32, image_extent: f32, view_extent: f32) -> f32 {
    if view_extent >= image_extent {
        center_ozi_axis(image_extent, view_extent)
    } else {
        offset.clamp(0.0, image_extent - view_extent)
    }
}

fn select_native_ozi_level(
    levels: &[OziRasterLevelMetadata],
    zoom: f32,
) -> Option<OziNativeLevelSelection> {
    let base_level = levels.first()?;
    let desired_downsample = (1.0 / zoom.max(f32::EPSILON)).max(1.0);
    let desired_log = desired_downsample.log2();

    levels
        .iter()
        .enumerate()
        .map(|(index, level)| {
            let scale_x = base_level.width() as f32 / level.width() as f32;
            let scale_y = base_level.height() as f32 / level.height() as f32;
            let distance = (scale_x.log2() - desired_log).abs();

            (
                distance,
                index,
                OziNativeLevelSelection {
                    level_index: level.level_index(),
                    level_scale_x: scale_x,
                    level_scale_y: scale_y,
                },
            )
        })
        .min_by(|left, right| {
            left.0
                .partial_cmp(&right.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.1.cmp(&right.1))
        })
        .map(|(_, _, selection)| selection)
}

fn visible_ozi_tiles(
    level: &OziRasterLevelMetadata,
    image_rect: BaseImageRect,
    selection: OziNativeLevelSelection,
) -> Result<VisibleTileRange, String> {
    if level.tile_columns() == 0 || level.tile_rows() == 0 {
        return Err("selected OZI level has no tiles".to_owned());
    }

    let level_tile_width = level.tile_width() as f32 * selection.level_scale_x;
    let level_tile_height = level.tile_height() as f32 * selection.level_scale_y;
    let max_x = level.tile_columns() - 1;
    let max_y = level.tile_rows() - 1;
    let start_x = tile_index_for_position(image_rect.min.x, level_tile_width).min(max_x);
    let end_x =
        tile_index_for_position(image_rect.min.x + image_rect.size.x, level_tile_width).min(max_x);
    let start_y = tile_index_for_position(image_rect.min.y, level_tile_height).min(max_y);
    let end_y =
        tile_index_for_position(image_rect.min.y + image_rect.size.y, level_tile_height).min(max_y);

    Ok(VisibleTileRange {
        start_x,
        end_x,
        start_y,
        end_y,
    })
}

fn tile_index_for_position(position: f32, tile_extent: f32) -> u32 {
    if position <= 0.0 {
        0
    } else {
        (position / tile_extent).floor() as u32
    }
}

fn ozi_tile_screen_rect(
    viewport_rect: egui::Rect,
    base_top_left: egui::Vec2,
    zoom: f32,
    level: &OziRasterLevelMetadata,
    selection: OziNativeLevelSelection,
    tile_x: u32,
    tile_y: u32,
    tile_size_level_pixels: egui::Vec2,
) -> egui::Rect {
    let base_min = egui::vec2(
        tile_x as f32 * level.tile_width() as f32 * selection.level_scale_x,
        tile_y as f32 * level.tile_height() as f32 * selection.level_scale_y,
    );
    let base_size = egui::vec2(
        tile_size_level_pixels.x * selection.level_scale_x,
        tile_size_level_pixels.y * selection.level_scale_y,
    );

    egui::Rect::from_min_size(
        viewport_rect.min + (base_min - base_top_left) * zoom,
        base_size * zoom,
    )
}

fn ozi_image_screen_rect(
    viewport_rect: egui::Rect,
    base_top_left: egui::Vec2,
    zoom: f32,
    base_size: egui::Vec2,
) -> egui::Rect {
    egui::Rect::from_min_size(
        viewport_rect.min + (-base_top_left * zoom),
        base_size * zoom,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        BaseImageRect, OziNativeLevelSelection, clamp_ozi_axis, fit_ozi_zoom,
        ozi_image_screen_rect, project_matches_query, select_native_ozi_level, visible_ozi_tiles,
    };
    use crate::application::LizaProjectSummary;
    use crate::infrastructure::import::OziRasterLevelMetadata;
    use eframe::egui;

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
    fn fit_ozi_zoom_returns_viewport_fit_ratio() {
        let zoom = fit_ozi_zoom(egui::vec2(2000.0, 1000.0), egui::vec2(1000.0, 750.0));

        assert_eq!(zoom, 0.5);
    }

    #[test]
    fn select_native_ozi_level_prefers_finest_level_for_overzoom() {
        let levels = sample_levels();

        let selection = select_native_ozi_level(&levels, 2.5).expect("selection");

        assert_eq!(selection.level_index, 0);
        assert_eq!(selection.level_scale_x, 1.0);
    }

    #[test]
    fn select_native_ozi_level_prefers_nearest_coarser_native_resolution() {
        let levels = sample_levels();

        let selection = select_native_ozi_level(&levels, 0.3).expect("selection");

        assert_eq!(selection.level_index, 2);
        assert_eq!(selection.level_scale_x, 4.0);
    }

    #[test]
    fn visible_ozi_tiles_clamps_to_image_bounds() {
        let level = OziRasterLevelMetadata::new(1, 512, 256, 64, 64, 8, 4);
        let selection = OziNativeLevelSelection {
            level_index: 1,
            level_scale_x: 2.0,
            level_scale_y: 2.0,
        };

        let visible = visible_ozi_tiles(
            &level,
            BaseImageRect {
                min: egui::vec2(900.0, 100.0),
                size: egui::vec2(400.0, 300.0),
            },
            selection,
        )
        .expect("visible tiles");

        assert_eq!(visible.start_x, 7);
        assert_eq!(visible.end_x, 7);
        assert_eq!(visible.start_y, 0);
        assert_eq!(visible.end_y, 3);
    }

    #[test]
    fn clamp_ozi_axis_centers_image_when_view_is_larger_than_image() {
        let clamped = clamp_ozi_axis(25.0, 800.0, 1200.0);

        assert_eq!(clamped, -200.0);
    }

    #[test]
    fn ozi_image_screen_rect_positions_image_from_base_offset() {
        let rect = ozi_image_screen_rect(
            egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(500.0, 400.0)),
            egui::vec2(100.0, 50.0),
            2.0,
            egui::vec2(300.0, 200.0),
        );

        assert_eq!(rect.min, egui::pos2(-190.0, -80.0));
        assert_eq!(rect.size(), egui::vec2(600.0, 400.0));
    }

    fn sample_levels() -> Vec<OziRasterLevelMetadata> {
        vec![
            OziRasterLevelMetadata::new(0, 2048, 1024, 64, 64, 32, 16),
            OziRasterLevelMetadata::new(1, 1024, 512, 64, 64, 16, 8),
            OziRasterLevelMetadata::new(2, 512, 256, 64, 64, 8, 4),
            OziRasterLevelMetadata::new(3, 256, 128, 64, 64, 4, 2),
        ]
    }
}
