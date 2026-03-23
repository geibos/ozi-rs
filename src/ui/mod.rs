mod sqlite_tiles;

use crate::application::AppState;
use eframe::egui;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use walkers::{lon_lat, sources::OpenStreetMap, HttpTiles, Map, MapMemory, Position};

use self::sqlite_tiles::SqliteTiles;

pub struct OziApp {
    state: AppState,
    fps_counter: FpsCounter,
    loaded_map_path: Option<PathBuf>,
    map_center: Position,
    map_memory: MapMemory,
    offline_tiles: Option<SqliteTiles>,
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
            state: AppState::new(),
            fps_counter: FpsCounter::new(),
            loaded_map_path: None,
            map_center: lon_lat(37.6176, 55.7558),
            map_memory: MapMemory::default(),
            offline_tiles: None,
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

        match SqliteTiles::open(
            &active_map.local_path,
            ctx.clone(),
            format!("LizaAlert {}", active_map.package_name),
            active_map.base_zoom,
        ) {
            Ok(offline_tiles) => {
                self.map_center = lon_lat(active_map.center.lon, active_map.center.lat);
                self.map_memory.center_at(self.map_center);
                let _ = self.map_memory.set_zoom(active_map.base_zoom.into());
                self.loaded_map_path = Some(active_map.local_path.clone());
                self.offline_tiles = Some(offline_tiles);
            }
            Err(error) => {
                self.loaded_map_path = None;
                self.offline_tiles = None;
                eprintln!("Failed to open downloaded map: {error}");
            }
        }
    }

    fn show_project_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("lizaalert_projects")
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.heading("Map Picker");
                ui.label("Source: maps.lizaalert.ru");

                if ui.button("Refresh projects").clicked() {
                    self.state.load_projects();
                }

                if self.state.lizaalert_busy() {
                    ui.spinner();
                }

                ui.label(self.state.lizaalert_status());
                ui.separator();
                ui.label("Projects");

                egui::ScrollArea::vertical()
                    .max_height(240.0)
                    .show(ui, |ui| {
                        let projects = self.state.lizaalert_projects().to_vec();
                        for project in projects {
                            if ui.button(&project.name).clicked() {
                                self.state.load_project(&project.slug);
                            }
                        }
                    });

                if let Some(project) = self.state.current_project().cloned() {
                    ui.separator();
                    ui.label(format!("Selected project: {}", project.summary.name));
                    ui.label(format!(
                        "Center: {:.5}, {:.5}",
                        project.center.lat, project.center.lon
                    ));
                    ui.separator();
                    ui.label("Available mobile map packages");

                    for map in project.maps {
                        if ui.button(&map.name).clicked() {
                            self.state.open_selected_map(&map.name);
                        }
                    }
                }
            });
    }
}

impl eframe::App for OziApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.state.poll_background_tasks();
        self.sync_active_map(ctx);
        self.fps_counter.tick();
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.state.window_title()));
        ctx.request_repaint();

        self.show_project_sidebar(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
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
        });
    }
}
