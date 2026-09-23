//! A folder holding one moment: what was on screen, and what the application
//! thought at the time.
//!
//! Between a coordinator noticing something wrong and describing it later,
//! almost everything useful is lost — what the screen looked like, which map
//! was open, what the last error said, whether the project had unsaved work.
//! By the time it is written down it is "the import did something odd", which
//! nobody can act on.
//!
//! So the moment is captured where it happens, into a dated folder, and the
//! folders are handed over together. The owner asked for exactly this on
//! 2026-09-23.
//!
//! Deliberately **not** behind a debug flag. The strangeness happens in the
//! field, on a release build, at four in the morning — a report that only
//! exists in builds somebody runs specially is a report that never exists.

use crate::commands::{AppStateDto, SharedState, lock_app_state};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, State};

/// Where the folders go. Findable without being told twice.
const REPORTS_DIR: &str = "ozi-rs-отчёты";

#[derive(serde::Serialize, specta::Type)]
pub struct ReportDto {
    /// The folder just written, so the interface can offer to show it.
    pub path: String,
    /// True when the screen could not be photographed — see below.
    pub screenshot_missing: bool,
}

/// Capture the moment.
///
/// Answers the folder. A screenshot that could not be taken does not fail the
/// report: the diagnostics and the state are most of the value, and a report
/// that refuses to exist because of a missing permission is the worst of both.
#[tauri::command]
#[specta::specta]
pub fn save_report(
    state: State<SharedState>,
    app: AppHandle,
    note: Option<String>,
) -> Result<ReportDto, String> {
    let root = reports_root(&app);
    let stamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let folder = root.join(&stamp);
    std::fs::create_dir_all(&folder)
        .map_err(|e| format!("не удалось создать папку отчёта {}: {e}", folder.display()))?;

    // The state first: it is the part that cannot be taken later, and if
    // anything below fails the folder still says something.
    let snapshot = {
        let app_state = lock_app_state(state.inner())?;
        crate::commands::app_state_dto(&app_state)
    };
    write(&folder, "state.json", &to_json(&snapshot))?;
    write(&folder, "diagnostics.txt", &diagnostics_text(&snapshot))?;
    write(&folder, "about.txt", &about_text())?;
    if let Some(note) = note.as_deref().map(str::trim).filter(|n| !n.is_empty()) {
        write(&folder, "note.txt", &format!("{note}\n"))?;
    }

    let screenshot_missing = capture_window(&app, &folder.join("screenshot.png")).is_err();

    Ok(ReportDto {
        path: folder.display().to_string(),
        screenshot_missing,
    })
}

/// Add a description to a report already written.
///
/// Two steps on purpose. The screen is captured the moment the operator asks,
/// before a dialog can cover it or the state move on; the sentence about what
/// happened comes after, when there is a second to write it. A report with no
/// note is still worth having, so this is allowed to be skipped.
#[tauri::command]
#[specta::specta]
pub fn add_report_note(path: String, note: String) -> Result<(), String> {
    let folder = PathBuf::from(&path);
    if !folder.is_dir() {
        return Err(format!("нет такой папки отчёта: {path}"));
    }
    let trimmed = note.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    write(&folder, "note.txt", &format!("{trimmed}\n"))
}

/// Show the reports folder, so a day's worth can be handed over at once.
#[tauri::command]
#[specta::specta]
pub fn reveal_reports(app: AppHandle) -> Result<String, String> {
    let root = reports_root(&app);
    std::fs::create_dir_all(&root)
        .map_err(|e| format!("не удалось создать папку отчётов {}: {e}", root.display()))?;
    crate::application::reveal_in_file_manager(&root);
    Ok(root.display().to_string())
}

fn reports_root(app: &AppHandle) -> PathBuf {
    match app.path().document_dir() {
        Ok(documents) => documents.join(REPORTS_DIR),
        Err(_) => PathBuf::from(REPORTS_DIR),
    }
}

fn write(folder: &Path, name: &str, contents: &str) -> Result<(), String> {
    let path = folder.join(name);
    std::fs::write(&path, contents)
        .map_err(|e| format!("не удалось записать {}: {e}", path.display()))
}

fn to_json(snapshot: &AppStateDto) -> String {
    serde_json::to_string_pretty(snapshot)
        .unwrap_or_else(|e| format!("{{\"error\":\"состояние не сериализуется: {e}\"}}"))
}

/// The diagnostics as a person reads them, newest last.
///
/// `state.json` has them too, but nobody opens a JSON file to find out what
/// went wrong. This is the file a reader looks at first.
fn diagnostics_text(snapshot: &AppStateDto) -> String {
    let mut out = String::new();
    out.push_str("Последние сообщения приложения, старые сверху.\n");
    out.push_str(
        "Уровни: info — что произошло, warning — на что посмотреть, error — что не вышло.\n\n",
    );
    for entry in &snapshot.diagnostics {
        // The line a session opens with has no moment; a row of spaces keeps
        // the columns without claiming a time it does not have.
        let at = if entry.at.is_empty() {
            "        "
        } else {
            &entry.at
        };
        out.push_str(&format!("{at}  {:<7}  {}\n", entry.level, entry.message));
    }
    if snapshot.diagnostics.is_empty() {
        out.push_str("(пусто — приложению нечего было сказать за эту сессию)\n");
    }
    out.push_str(&format!(
        "\nСтрока состояния на момент отчёта:\n{}\n",
        snapshot.status
    ));
    out
}

/// The `-R` rectangle for a window, in points.
///
/// `screencapture -R` takes **points**; Tauri reports the window in physical
/// pixels. On a Retina screen that is a factor of two, and getting it wrong
/// photographs a quarter of the window.
///
/// Checked rather than assumed, 2026-09-23: `screencapture -x -o
/// -R100,100,400,300` writes an 800×600 PNG on a display whose scale is 2.
fn capture_rect(x: f64, y: f64, width: f64, height: f64, scale: f64) -> String {
    let scale = if scale > 0.0 { scale } else { 1.0 };
    format!(
        "{},{},{},{}",
        (x / scale).round() as i64,
        (y / scale).round() as i64,
        (width / scale).round() as i64,
        (height / scale).round() as i64,
    )
}

fn about_text() -> String {
    format!(
        "ozi-rs {}\n{} {}\n{}\n",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S %:z"),
    )
}

/// Photograph the application's own window.
///
/// macOS only for now, through `screencapture` against the window's rectangle
/// — the same tool the QA harness uses, and the one that captures what is
/// actually composited rather than what the DOM thinks. The map is a WebGL
/// canvas, so anything reading the page instead would come back with a hole
/// where the map is, which is the part of a report most worth looking at.
///
/// The first use asks for Screen Recording, once, for this application.
/// Refusing it costs the screenshot and nothing else.
fn capture_window(app: &AppHandle, target: &Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let window = app
            .get_webview_window("main")
            .ok_or_else(|| "нет окна".to_owned())?;
        let position = window.outer_position().map_err(|e| e.to_string())?;
        let size = window.outer_size().map_err(|e| e.to_string())?;
        let scale = window.scale_factor().unwrap_or(1.0);

        let rect = capture_rect(
            f64::from(position.x),
            f64::from(position.y),
            f64::from(size.width),
            f64::from(size.height),
            scale,
        );

        let status = std::process::Command::new("screencapture")
            .args(["-x", "-o", "-R", &rect])
            .arg(target)
            .status()
            .map_err(|e| format!("screencapture не запустился: {e}"))?;
        if !status.success() {
            return Err(format!("screencapture вернул {status}"));
        }
        // A refused Screen Recording grant still exits zero and writes
        // nothing, so the file is what says whether it worked.
        if !target.exists() {
            return Err("снимок не записан — вероятно, нет разрешения на запись экрана".to_owned());
        }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, target);
        Err("снимок экрана пока только на macOS".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::{about_text, diagnostics_text};
    use crate::commands::{AppStateDto, DiagnosticDto};

    fn snapshot(diagnostics: Vec<DiagnosticDto>) -> AppStateDto {
        AppStateDto {
            project_name: "Сагра".to_owned(),
            project_saved: true,
            project_dirty: false,
            project_path: Some("/searches/Сагра.ozp".to_owned()),
            status: "Opened: /searches/Сагра.ozp".to_owned(),
            listing_busy: false,
            bundle_busy: false,
            downloading_maps: vec![],
            current_project: None,
            active_map: None,
            diagnostics,
            track_layers: vec![],
            waypoint_layers: vec![],
            track_layer_count: 0,
            waypoint_layer_count: 0,
            tracks: vec![],
        }
    }

    #[test]
    fn diagnostics_text_carries_the_time_and_the_level() {
        // Without the time the list is an order and nothing else: a reader
        // cannot tell whether the error was before the screenshot or hours
        // earlier.
        let text = diagnostics_text(&snapshot(vec![
            DiagnosticDto {
                level: "info",
                message: "Opened: /searches/Сагра.ozp".to_owned(),
                at: "03:41:02".to_owned(),
            },
            DiagnosticDto {
                level: "error",
                message: "Import failed: неожиданный конец данных".to_owned(),
                at: "04:12:55".to_owned(),
            },
        ]));

        assert!(text.contains("03:41:02"), "{text}");
        assert!(text.contains("04:12:55"), "{text}");
        assert!(text.contains("error"), "{text}");
        assert!(text.contains("неожиданный конец данных"), "{text}");
        // Newest last, as a log reads.
        assert!(
            text.find("03:41:02") < text.find("04:12:55"),
            "oldest first: {text}"
        );
    }

    #[test]
    fn diagnostics_text_says_so_when_there_is_nothing() {
        // An empty file reads as a broken capture; a sentence reads as a quiet
        // session.
        let text = diagnostics_text(&snapshot(vec![]));
        assert!(text.contains("пусто"), "{text}");
        assert!(
            text.contains("Opened: /searches/Сагра.ozp"),
            "status: {text}"
        );
    }

    #[test]
    fn capture_rect_is_in_points_not_pixels() {
        // A 1400×900-point window on a Retina screen: Tauri says 2800×1800.
        assert_eq!(
            super::capture_rect(200.0, 100.0, 2800.0, 1800.0, 2.0),
            "100,50,1400,900"
        );
        // And on a plain display the numbers pass through.
        assert_eq!(
            super::capture_rect(200.0, 100.0, 1400.0, 900.0, 1.0),
            "200,100,1400,900"
        );
    }

    #[test]
    fn capture_rect_survives_a_scale_of_zero() {
        // A scale factor that cannot be read must not divide the window into
        // nothing; it is the whole screen or a quarter of it, not a crash.
        assert_eq!(
            super::capture_rect(0.0, 0.0, 800.0, 600.0, 0.0),
            "0,0,800,600"
        );
    }

    #[test]
    fn about_text_names_the_build_and_the_machine() {
        let text = about_text();
        assert!(text.contains(env!("CARGO_PKG_VERSION")), "{text}");
        assert!(text.contains(std::env::consts::OS), "{text}");
    }
}
