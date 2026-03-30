mod sqlite_tiles;
mod theme;

use theme::CatppuccinFlavor;

use crate::application::{ActiveMapKind, AppState, DiagnosticLevel};
use crate::domain::TrackLayer;
use crate::infrastructure::import::{
    OziGeoreference, OziRasterLevelMetadata, OziRasterTileSource, open_ozi_raster_tile_source,
    parse_ozi_georeference, parse_ozi_map_metadata, read_ozi_map_text,
};
use eframe::egui;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use walkers::{
    HttpTiles, Map, MapMemory, Plugin, Position, Projector, lon_lat, sources::OpenStreetMap,
};

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
    track_name_edits:
        std::collections::HashMap<(crate::domain::LayerId, crate::domain::TrackId), String>,
    console_open: bool,
    theme: CatppuccinFlavor,
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
    georeference: Option<OziGeoreference>,
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

const STORAGE_KEY_LAST_PROJECT: &str = "last_project_path";
const STORAGE_KEY_ACTIVE_MAP: &str = "active_map";
const STORAGE_KEY_BUNDLES_ROOT: &str = "bundles_root";
const STORAGE_KEY_THEME: &str = "theme";

impl OziApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut state = AppState::new();

        state.load_projects();

        if let Some(storage) = cc.storage {
            let bundles_root: Option<PathBuf> =
                eframe::get_value(storage, STORAGE_KEY_BUNDLES_ROOT);
            if let Some(root) = bundles_root {
                state.set_bundles_root(root);
            }

            let last_project_path: Option<PathBuf> =
                eframe::get_value(storage, STORAGE_KEY_LAST_PROJECT);
            if let Some(path) = last_project_path {
                if path.exists() {
                    state.load_project_from(path);
                }
            }

            let active_map: Option<crate::application::ActiveMapSelection> =
                eframe::get_value(storage, STORAGE_KEY_ACTIVE_MAP);
            if let Some(selection) = active_map {
                state.restore_active_map(selection);
            }
        }

        let theme: CatppuccinFlavor = cc
            .storage
            .and_then(|s| eframe::get_value(s, STORAGE_KEY_THEME))
            .unwrap_or_else(|| match cc.egui_ctx.system_theme() {
                Some(egui::Theme::Light) => CatppuccinFlavor::Latte,
                _ => CatppuccinFlavor::Mocha,
            });
        theme.apply(&cc.egui_ctx);

        Self {
            project_search: String::new(),
            state,
            fps_counter: FpsCounter::new(),
            loaded_map_path: None,
            map_center: lon_lat(37.6176, 55.7558),
            map_memory: MapMemory::default(),
            offline_tiles: None,
            ozi_renderer: None,
            osm_tiles: HttpTiles::new(OpenStreetMap, cc.egui_ctx.clone()),
            track_name_edits: std::collections::HashMap::new(),
            console_open: false,
            theme: CatppuccinFlavor::Mocha,
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
                ui.horizontal(|ui| {
                    ui.heading("Map Picker");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let prev = self.theme;
                        egui::ComboBox::from_id_salt("theme_picker")
                            .selected_text(self.theme.name())
                            .show_ui(ui, |ui| {
                                for flavor in CatppuccinFlavor::ALL {
                                    ui.selectable_value(&mut self.theme, flavor, flavor.name());
                                }
                            });
                        if self.theme != prev {
                            self.theme.apply(ui.ctx());
                        }
                    });
                });

                // Bundles storage directory settings
                egui::CollapsingHeader::new("Map Bundles Storage")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(self.state.bundles_root().display().to_string())
                                .small()
                                .color(egui::Color32::GRAY),
                        );
                        ui.horizontal(|ui| {
                            if ui.button("Change…").clicked() {
                                if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                                    self.state.set_bundles_root(dir);
                                }
                            }
                            if ui.button("Open local bundle…").clicked() {
                                if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                                    self.state.open_local_bundle(dir);
                                }
                            }
                        });
                    });

                ui.label("Source: maps.lizaalert.ru");

                if ui.button("Refresh projects").clicked() {
                    self.state.load_projects();
                }

                if self.state.lizaalert_busy() {
                    ui.spinner();
                }

                ui.label(self.state.lizaalert_status());
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

    fn show_tracks_panel(&mut self, ctx: &egui::Context) {
        egui::Window::new("Tracks")
            .id(egui::Id::new("tracks_window"))
            .resizable(true)
            .default_size([280.0, 500.0])
            .show(ctx, |ui| {
                ui.heading("Tracks");

                ui.horizontal(|ui| {
                    if ui.button("Import GPX…").clicked() {
                        if let Some(path) = pick_gpx_file() {
                            match self.state.import_gpx_file(path) {
                                Ok(report) => self.state.report_runtime_info(format!(
                                    "Imported {} track(s) from GPX",
                                    report.imported_tracks()
                                )),
                                Err(e) => self
                                    .state
                                    .report_runtime_error(format!("GPX import failed: {e}")),
                            }
                        }
                    }
                    if ui.button("Import PLT…").clicked() {
                        if let Some(path) = pick_plt_file() {
                            match self.state.import_plt_file(path) {
                                Ok(report) => self.state.report_runtime_info(format!(
                                    "Imported {} track(s) from PLT",
                                    report.imported_tracks()
                                )),
                                Err(e) => self
                                    .state
                                    .report_runtime_error(format!("PLT import failed: {e}")),
                            }
                        }
                    }
                });

                ui.separator();

                // Collect deferred mutations to avoid borrow conflicts.
                let mut visibility_toggles: Vec<(crate::domain::LayerId, crate::domain::TrackId)> =
                    Vec::new();
                let mut color_changes: Vec<(
                    crate::domain::LayerId,
                    crate::domain::TrackId,
                    [u8; 4],
                )> = Vec::new();
                let mut renames: Vec<(crate::domain::LayerId, crate::domain::TrackId, String)> =
                    Vec::new();
                let mut export_layer: Option<(crate::domain::LayerId, String)> = None;

                egui::ScrollArea::vertical()
                    .id_salt("tracks_scroll")
                    .show(ui, |ui| {
                        for layer in self.state.track_layers() {
                            let layer_header =
                                egui::CollapsingHeader::new(layer.name()).default_open(true);
                            layer_header.show(ui, |ui| {
                                if ui.small_button("Export GPX…").clicked() {
                                    export_layer = Some((layer.id(), layer.name().to_owned()));
                                }
                                for track in layer.tracks() {
                                    ui.horizontal(|ui| {
                                        // Visibility checkbox
                                        let mut visible = track.style().visible;
                                        if ui.checkbox(&mut visible, "").changed() {
                                            visibility_toggles.push((layer.id(), track.id()));
                                        }

                                        // Color picker button
                                        let [r, g, b, a] = track.style().color;
                                        let mut egui_color =
                                            egui::Color32::from_rgba_unmultiplied(r, g, b, a);
                                        if egui::color_picker::color_edit_button_srgba(
                                            ui,
                                            &mut egui_color,
                                            egui::color_picker::Alpha::Opaque,
                                        )
                                        .changed()
                                        {
                                            color_changes.push((
                                                layer.id(),
                                                track.id(),
                                                egui_color.to_array(),
                                            ));
                                        }
                                    });

                                    // Editable track name with OK standard validation hint
                                    let edit_key = (layer.id(), track.id());
                                    let edit_buf = self
                                        .track_name_edits
                                        .entry(edit_key)
                                        .or_insert_with(|| track.name().to_owned());

                                    let name_valid = is_ok_track_name(edit_buf);
                                    let name_color = if name_valid {
                                        ui.visuals().text_color()
                                    } else {
                                        egui::Color32::from_rgb(220, 160, 60)
                                    };
                                    let response = ui.add(
                                        egui::TextEdit::singleline(edit_buf)
                                            .desired_width(f32::INFINITY)
                                            .text_color(name_color)
                                            .hint_text("YYYYMMDD_Callsign"),
                                    );
                                    if response.lost_focus()
                                        && !edit_buf.is_empty()
                                        && edit_buf.as_str() != track.name()
                                    {
                                        renames.push((layer.id(), track.id(), edit_buf.clone()));
                                    }
                                    if !name_valid {
                                        ui.label(
                                            egui::RichText::new("⚠ name: YYYYMMDD_Callsign")
                                                .small()
                                                .color(egui::Color32::from_rgb(220, 160, 60)),
                                        );
                                    }

                                    // Stats line
                                    let dist = track.total_distance_km();
                                    let pts = track.point_count();
                                    let stats = if let Some(dur) = track.total_duration() {
                                        let total_secs = dur.num_seconds().abs();
                                        let h = total_secs / 3600;
                                        let m = (total_secs % 3600) / 60;
                                        format!("{dist:.1} km  {h}h{m:02}m  {pts} pts")
                                    } else {
                                        format!("{dist:.1} km  {pts} pts")
                                    };
                                    ui.add(
                                        egui::Label::new(
                                            egui::RichText::new(stats)
                                                .small()
                                                .color(egui::Color32::GRAY),
                                        )
                                        .wrap(),
                                    );
                                }
                            });
                        }
                    });

                for (layer_id, track_id) in visibility_toggles {
                    self.state.toggle_track_visible(layer_id, track_id);
                }
                for (layer_id, track_id, color) in color_changes {
                    self.state.set_track_color(layer_id, track_id, color);
                }
                for (layer_id, track_id, new_name) in renames {
                    // Keep buffer in sync after commit
                    self.track_name_edits
                        .insert((layer_id, track_id), new_name.clone());
                    self.state.rename_track(layer_id, track_id, new_name);
                }
                if let Some((layer_id, layer_name)) = export_layer {
                    let suggested_dir = self.state.active_bundle_dir().map(|d| d.join("10-Tracks"));
                    let suggested_name = format!("{}.gpx", ok_safe_file_name(&layer_name));
                    if let Some(path) =
                        save_gpx_file_dialog(suggested_dir.as_deref(), &suggested_name)
                    {
                        if let Some(parent) = path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        self.state.export_layer_to_gpx(layer_id, path);
                    }
                }
            });
    }

    fn show_console(&mut self, ctx: &egui::Context) {
        // Toggle on tilde/backtick
        if ctx.input(|i| i.key_pressed(egui::Key::Backtick)) {
            self.console_open = !self.console_open;
        }

        if !self.console_open {
            return;
        }

        let screen = ctx.content_rect();
        let console_height = (screen.height() * 0.45).min(400.0);

        egui::Window::new("Console")
            .id(egui::Id::new("dev_console"))
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .fixed_rect(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(screen.width(), console_height),
            ))
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgba_premultiplied(20, 20, 20, 230))
                    .inner_margin(egui::Margin::same(6)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.monospace(
                        egui::RichText::new("Console")
                            .color(egui::Color32::from_gray(180))
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button(
                                egui::RichText::new("✕").color(egui::Color32::from_gray(160)),
                            )
                            .clicked()
                        {
                            self.console_open = false;
                        }
                        ui.monospace(
                            egui::RichText::new("~ to toggle")
                                .color(egui::Color32::from_gray(100))
                                .small(),
                        );
                    });
                });

                ui.separator();

                let available = ui.available_height();
                egui::ScrollArea::vertical()
                    .id_salt("console_scroll")
                    .max_height(available)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for entry in self.state.recent_diagnostics() {
                            let (color, prefix) = match entry.level() {
                                DiagnosticLevel::Info => (egui::Color32::from_gray(200), ""),
                                DiagnosticLevel::Error => {
                                    (egui::Color32::from_rgb(255, 100, 80), "[ERROR] ")
                                }
                            };
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(format!("{}{}", prefix, entry.message()))
                                        .monospace()
                                        .size(11.0)
                                        .color(color),
                                )
                                .wrap(),
                            );
                        }
                    });
            });
    }
}

impl eframe::App for OziApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(
            storage,
            STORAGE_KEY_BUNDLES_ROOT,
            &self.state.bundles_root(),
        );
        if let Some(path) = self.state.project_file_path() {
            eframe::set_value(storage, STORAGE_KEY_LAST_PROJECT, &path);
        }
        if let Some(map) = self.state.active_map() {
            eframe::set_value(storage, STORAGE_KEY_ACTIVE_MAP, map);
        }
        eframe::set_value(storage, STORAGE_KEY_THEME, &self.theme);
    }

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
        self.show_tracks_panel(ui.ctx());

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
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Loaded: {} / {}",
                        active_map.project_name, active_map.package_name
                    ));
                    if self.state.active_bundle_dir().is_some()
                        && ui.small_button("Show in Finder").clicked()
                    {
                        self.state.reveal_active_bundle();
                    }
                });
                if active_map.kind == ActiveMapKind::OziRaster {
                    ui.label(
                        egui::RichText::new(active_map.local_path.display().to_string())
                            .small()
                            .color(egui::Color32::GRAY),
                    );
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

            let track_layers = self.state.track_layers();
            if let Some(renderer) = self.ozi_renderer.as_mut() {
                if let Err(error) = renderer.render(ui, track_layers) {
                    ui.colored_label(egui::Color32::LIGHT_RED, error);
                }
            } else {
                let track_plugin = TrackPlugin {
                    layers: track_layers,
                };
                let map = if let Some(offline_tiles) = self.offline_tiles.as_mut() {
                    Map::new(Some(offline_tiles), &mut self.map_memory, self.map_center)
                        .with_plugin(track_plugin)
                } else {
                    Map::new(
                        Some(&mut self.osm_tiles),
                        &mut self.map_memory,
                        self.map_center,
                    )
                    .with_plugin(track_plugin)
                };

                ui.add(map);
            }

            self.show_console(ui.ctx());
        });
    }
}

struct TrackPlugin<'a> {
    layers: &'a [TrackLayer],
}

impl Plugin for TrackPlugin<'_> {
    fn run(
        self: Box<Self>,
        ui: &mut egui::Ui,
        _response: &egui::Response,
        projector: &Projector,
        _map_memory: &MapMemory,
    ) {
        let painter = ui.painter();
        for layer in self.layers {
            for track in layer.tracks() {
                let style = track.style();
                if !style.visible {
                    continue;
                }
                let alpha = (style.color[3] as f32 * style.opacity).clamp(0.0, 255.0) as u8;
                let color = egui::Color32::from_rgba_unmultiplied(
                    style.color[0],
                    style.color[1],
                    style.color[2],
                    alpha,
                );
                let stroke = egui::Stroke::new(style.line_width, color);

                for segment in track.segments() {
                    let screen_points: Vec<egui::Pos2> = segment
                        .points()
                        .iter()
                        .map(|p| {
                            let v = projector.project(lon_lat(p.longitude(), p.latitude()));
                            egui::pos2(v.x, v.y)
                        })
                        .collect();

                    for pair in screen_points.windows(2) {
                        painter.line_segment([pair[0], pair[1]], stroke);
                    }
                }
            }
        }
    }
}

fn pick_project_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("ozi-rs project", &["ozp"])
        .pick_file()
}

fn pick_gpx_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("GPX track", &["gpx"])
        .pick_file()
}

fn save_gpx_file_dialog(suggested_dir: Option<&std::path::Path>, name: &str) -> Option<PathBuf> {
    let dialog = rfd::FileDialog::new()
        .add_filter("GPX track", &["gpx"])
        .set_file_name(name);
    let dialog = if let Some(dir) = suggested_dir {
        dialog.set_directory(dir)
    } else {
        dialog
    };
    dialog.save_file()
}

/// Check if a track name follows the LizaAlert OK standard: only Latin letters, digits, `_`, `-`.
fn is_ok_track_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Strip characters not allowed in the LizaAlert OK file naming standard.
fn ok_safe_file_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn pick_plt_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("OziExplorer track", &["plt"])
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

impl OziRasterRenderer {
    fn open(map_path: &std::path::Path) -> Result<Self, String> {
        let map_contents = read_ozi_map_text(map_path).map_err(|error| error.to_string())?;
        let metadata =
            parse_ozi_map_metadata(map_path, &map_contents).map_err(|error| error.to_string())?;
        let tile_source =
            open_ozi_raster_tile_source(&metadata).map_err(|error| error.to_string())?;
        let georeference = parse_ozi_georeference(metadata.calibration_points());

        Ok(Self {
            tile_source,
            textures: LruCache::new(
                NonZeroUsize::new(OZI_TILE_TEXTURE_CACHE_SIZE).expect("cache size"),
            ),
            viewport: OziViewportState::default(),
            georeference,
        })
    }

    fn render(&mut self, ui: &mut egui::Ui, track_layers: &[TrackLayer]) -> Result<(), String> {
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

        self.render_tracks(&painter, rect, track_layers);

        Ok(())
    }

    fn render_tracks(
        &self,
        painter: &egui::Painter,
        viewport_rect: egui::Rect,
        track_layers: &[TrackLayer],
    ) {
        let Some(geo) = &self.georeference else {
            return;
        };

        for layer in track_layers {
            for track in layer.tracks() {
                let style = track.style();
                if !style.visible {
                    continue;
                }
                let alpha = (style.color[3] as f32 * style.opacity).clamp(0.0, 255.0) as u8;
                let color = egui::Color32::from_rgba_unmultiplied(
                    style.color[0],
                    style.color[1],
                    style.color[2],
                    alpha,
                );
                let stroke = egui::Stroke::new(style.line_width, color);

                for segment in track.segments() {
                    let screen_points: Vec<egui::Pos2> = segment
                        .points()
                        .iter()
                        .map(|p| {
                            let (px, py) = geo.lat_lon_to_pixel(p.latitude(), p.longitude());
                            viewport_rect.min
                                + (egui::vec2(px as f32, py as f32)
                                    - self.viewport.top_left_base_pixels)
                                    * self.viewport.zoom
                        })
                        .collect();

                    for pair in screen_points.windows(2) {
                        painter.line_segment([pair[0], pair[1]], stroke);
                    }
                }
            }
        }
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
