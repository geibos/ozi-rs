use std::{
    env, fs,
    path::{Path, PathBuf},
};

use rmcp::schemars;
use serde::{Deserialize, Serialize};

use crate::{
    config,
    evidence::{EvidenceMetadata, EvidencePaths, EvidenceStatus, started_at_now},
    process::{EvidenceCommand, RealCommand, run_command_with_evidence},
};

#[derive(Debug, Clone, Serialize, schemars::JsonSchema, PartialEq, Eq)]
pub struct QaEnvironment {
    pub platform: String,
    pub repo_root: String,
    pub evidence_root: String,
    pub just_available: bool,
    pub open_available: bool,
    pub log_available: bool,
    pub screencapture_available: bool,
    pub appium_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq)]
pub struct NativeSessionState {
    pub running: bool,
    pub app_path: Option<String>,
    pub launched_at: Option<String>,
    pub stopped_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct NativeToolResult {
    pub ok: bool,
    pub tool: String,
    pub error_kind: Option<String>,
    pub message: Option<String>,
    pub environment: Option<QaEnvironment>,
    pub evidence: Option<EvidenceMetadata>,
    pub session: Option<NativeSessionState>,
    pub artifact_paths: Vec<String>,
}

pub fn qa_environment() -> NativeToolResult {
    match config::repo_root().and_then(|root| qa_environment_for_root(&root)) {
        Ok(result) => result,
        Err(error) => error_result("qa_environment", "repo_root", error.to_string()),
    }
}

pub fn build_app() -> NativeToolResult {
    match config::repo_root().and_then(|root| {
        let command = build_command(&root);
        build_app_with_command(&root, &command)
    }) {
        Ok(result) => result,
        Err(error) => error_result("build_app", "command_failed", error.to_string()),
    }
}

pub fn launch_app() -> NativeToolResult {
    match config::repo_root().and_then(|root| launch_app_from_root(&root)) {
        Ok(result) => result,
        Err(error) => error_result("launch_app", "command_failed", error.to_string()),
    }
}

pub fn stop_app() -> NativeToolResult {
    match config::repo_root().and_then(|root| stop_app_at_root(&root)) {
        Ok(result) => result,
        Err(error) => error_result("stop_app", "command_failed", error.to_string()),
    }
}

pub fn capture_logs() -> NativeToolResult {
    match config::repo_root().and_then(|root| {
        let command = RealCommand::new("log")
            .arg("show")
            .arg("--style")
            .arg("compact")
            .arg("--last")
            .arg("5m")
            .arg("--predicate")
            .arg("process == \"ozi-rs\"");
        run_native_command("capture_logs", &root, &command, Vec::new())
    }) {
        Ok(result) => result,
        Err(error) => error_result("capture_logs", "command_failed", error.to_string()),
    }
}

pub fn capture_screenshot() -> NativeToolResult {
    match config::repo_root().and_then(|root| {
        let paths = EvidencePaths::new(&root);
        let screenshot_path = paths.path_for("capture_screenshot", "screenshot.png")?;
        let command = screenshot_command(&screenshot_path);
        capture_screenshot_with_command(&root, &command)
    }) {
        Ok(result) => result,
        Err(error) => error_result("capture_screenshot", "command_failed", error.to_string()),
    }
}

pub fn qa_observe() -> NativeToolResult {
    match config::repo_root().and_then(|root| {
        let session = read_session(&root)?.unwrap_or_else(stopped_session);
        Ok(NativeToolResult {
            ok: true,
            tool: "qa_observe".to_owned(),
            error_kind: None,
            message: None,
            environment: Some(environment_for_root(&root)),
            evidence: None,
            session: Some(session),
            artifact_paths: Vec::new(),
        })
    }) {
        Ok(result) => result,
        Err(error) => error_result("qa_observe", "command_failed", error.to_string()),
    }
}

pub fn build_command(repo_root: &Path) -> RealCommand {
    RealCommand::new("just").arg("build").current_dir(repo_root)
}

pub fn launch_command(app_path: &Path) -> RealCommand {
    RealCommand::new("open")
        .arg("-n")
        .arg(app_path.to_string_lossy().to_string())
}

pub fn screenshot_command(screenshot_path: &Path) -> RealCommand {
    RealCommand::new("screencapture")
        .arg("-x")
        .arg(screenshot_path.to_string_lossy().to_string())
}

pub fn stop_command_for_session(session: &NativeSessionState) -> anyhow::Result<RealCommand> {
    let app_path = session
        .app_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("running session has no app path"))?;
    let app_name = Path::new(app_path)
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| anyhow::anyhow!("running session app path has no valid .app name"))?;
    let escaped_name = app_name.replace('\\', "\\\\").replace('"', "\\\"");

    Ok(RealCommand::new("osascript")
        .arg("-e")
        .arg(format!("quit app \"{escaped_name}\"")))
}

pub fn qa_environment_for_root(repo_root: &Path) -> anyhow::Result<NativeToolResult> {
    Ok(NativeToolResult {
        ok: true,
        tool: "qa_environment".to_owned(),
        error_kind: None,
        message: None,
        environment: Some(environment_for_root(repo_root)),
        evidence: None,
        session: read_session(repo_root)?,
        artifact_paths: Vec::new(),
    })
}

pub fn build_app_with_command(
    repo_root: &Path,
    command: &impl EvidenceCommand,
) -> anyhow::Result<NativeToolResult> {
    let result = run_command_with_evidence("build_app", &EvidencePaths::new(repo_root), command)?;
    let ok = result.metadata.status == EvidenceStatus::Passed;
    let error_kind = result.metadata.error_kind.clone();

    Ok(NativeToolResult {
        ok,
        tool: "build_app".to_owned(),
        error_kind,
        message: None,
        environment: None,
        evidence: Some(result.metadata),
        session: None,
        artifact_paths: app_artifacts(repo_root)
            .into_iter()
            .map(|path| display_path(repo_root, &path))
            .collect(),
    })
}

pub fn capture_screenshot_with_command(
    repo_root: &Path,
    command: &impl EvidenceCommand,
) -> anyhow::Result<NativeToolResult> {
    let paths = EvidencePaths::new(repo_root);
    let screenshot_path = paths.path_for("capture_screenshot", "screenshot.png")?;
    paths.prepare_file(&screenshot_path)?;
    let mut result = run_native_command(
        "capture_screenshot",
        repo_root,
        command,
        vec![paths.relative_display(&screenshot_path)?],
    )?;

    if !result.ok
        && result.error_kind.as_deref() == Some("exit_code")
        && let Some(evidence) = result.evidence.as_ref()
    {
        let stderr_text = fs::read_to_string(repo_root.join(&evidence.stderr_path))
            .unwrap_or_default()
            .to_lowercase();
        if stderr_text.contains("could not create image from display") {
            result.error_kind = Some("screen_recording_denied".to_owned());
            result.message = Some(
                "screencapture failed: macOS denied Screen Recording for the MCP host. \
                 Grant access in System Settings → Privacy & Security → Screen Recording, \
                 then restart the MCP client."
                    .to_owned(),
            );
        }
    }

    Ok(result)
}

pub fn launch_app_from_root(repo_root: &Path) -> anyhow::Result<NativeToolResult> {
    let Some(app_path) = find_app_artifact(repo_root) else {
        let metadata = synthetic_metadata(
            "launch_app",
            Vec::new(),
            None,
            EvidenceStatus::Error,
            Some("artifact_missing"),
            repo_root,
            vec![format!("{}/build_app", crate::evidence::EVIDENCE_ROOT)],
        )?;
        return Ok(NativeToolResult {
            ok: false,
            tool: "launch_app".to_owned(),
            error_kind: Some("artifact_missing".to_owned()),
            message: Some(
                "no packaged or debug .app artifact found; run build_app first".to_owned(),
            ),
            environment: None,
            evidence: Some(metadata),
            session: read_session(repo_root)?,
            artifact_paths: Vec::new(),
        });
    };

    let command = launch_command(&app_path);
    let result = run_command_with_evidence("launch_app", &EvidencePaths::new(repo_root), &command)?;
    let ok = result.metadata.status == EvidenceStatus::Passed;
    let error_kind = result.metadata.error_kind.clone();
    let artifact_path = display_path(repo_root, &app_path);
    let session = NativeSessionState {
        running: ok,
        app_path: Some(artifact_path.clone()),
        launched_at: ok.then(started_at_now),
        stopped_at: None,
    };
    write_session(repo_root, &session)?;

    Ok(NativeToolResult {
        ok,
        tool: "launch_app".to_owned(),
        error_kind,
        message: None,
        environment: None,
        evidence: Some(result.metadata),
        session: Some(session),
        artifact_paths: vec![artifact_path],
    })
}

pub fn stop_app_at_root(repo_root: &Path) -> anyhow::Result<NativeToolResult> {
    let session = read_session(repo_root)?.unwrap_or_else(stopped_session);
    if !session.running {
        return Ok(NativeToolResult {
            ok: true,
            tool: "stop_app".to_owned(),
            error_kind: Some("already_stopped".to_owned()),
            message: Some("no running native QA app session".to_owned()),
            environment: None,
            evidence: None,
            session: Some(session),
            artifact_paths: Vec::new(),
        });
    }

    let command = stop_command_for_session(&session)?;
    let result = run_native_command("stop_app", repo_root, &command, Vec::new())?;
    let stopped = NativeSessionState {
        running: false,
        app_path: session.app_path,
        launched_at: session.launched_at,
        stopped_at: Some(started_at_now()),
    };
    write_session(repo_root, &stopped)?;

    Ok(NativeToolResult {
        session: Some(stopped),
        ..result
    })
}

fn run_native_command(
    tool: &str,
    repo_root: &Path,
    command: &impl EvidenceCommand,
    artifact_paths: Vec<String>,
) -> anyhow::Result<NativeToolResult> {
    let mut result = run_command_with_evidence(tool, &EvidencePaths::new(repo_root), command)?;
    result.metadata.artifact_paths = artifact_paths.clone();
    let ok = result.metadata.status == EvidenceStatus::Passed;
    let error_kind = result.metadata.error_kind.clone();

    Ok(NativeToolResult {
        ok,
        tool: tool.to_owned(),
        error_kind,
        message: None,
        environment: None,
        evidence: Some(result.metadata),
        session: None,
        artifact_paths,
    })
}

fn environment_for_root(repo_root: &Path) -> QaEnvironment {
    let paths = EvidencePaths::new(repo_root);
    QaEnvironment {
        platform: "macos".to_owned(),
        repo_root: repo_root.to_string_lossy().to_string(),
        evidence_root: paths.evidence_root().to_string_lossy().to_string(),
        just_available: command_available("just"),
        open_available: command_available("open"),
        log_available: command_available("log"),
        screencapture_available: command_available("screencapture"),
        appium_available: command_available("appium"),
    }
}

fn command_available(program: &str) -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&path).any(|dir| dir.join(program).is_file())
}

fn find_app_artifact(repo_root: &Path) -> Option<PathBuf> {
    app_artifacts(repo_root).into_iter().next()
}

fn app_artifacts(repo_root: &Path) -> Vec<PathBuf> {
    let mut artifacts = Vec::new();
    for root in [
        repo_root.join("src-tauri/target/debug/bundle/macos"),
        repo_root.join("src-tauri/target/release/bundle/macos"),
        repo_root.join("target/debug/bundle/macos"),
        repo_root.join("target/release/bundle/macos"),
    ] {
        collect_app_artifacts(&root, &mut artifacts);
    }
    artifacts.sort();
    artifacts
}

fn collect_app_artifacts(root: &Path, artifacts: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|extension| extension == "app") {
            artifacts.push(path);
        }
    }
}

fn session_path(repo_root: &Path) -> PathBuf {
    EvidencePaths::new(repo_root)
        .root_file("session.json")
        .expect("static session file name is safe")
}

fn read_session(repo_root: &Path) -> anyhow::Result<Option<NativeSessionState>> {
    let path = session_path(repo_root);
    if !path.is_file() {
        return Ok(None);
    }
    Ok(Some(serde_json::from_slice(&fs::read(path)?)?))
}

fn write_session(repo_root: &Path, session: &NativeSessionState) -> anyhow::Result<()> {
    let path = session_path(repo_root);
    EvidencePaths::new(repo_root).prepare_file(&path)?;
    fs::write(path, serde_json::to_vec_pretty(session)?)?;
    Ok(())
}

pub fn write_session_for_test(
    repo_root: &Path,
    session: &NativeSessionState,
) -> anyhow::Result<()> {
    write_session(repo_root, session)
}

fn stopped_session() -> NativeSessionState {
    NativeSessionState {
        running: false,
        app_path: None,
        launched_at: None,
        stopped_at: None,
    }
}

fn synthetic_metadata(
    tool: &str,
    command: Vec<String>,
    exit_code: Option<i32>,
    status: EvidenceStatus,
    error_kind: Option<&str>,
    repo_root: &Path,
    artifact_paths: Vec<String>,
) -> anyhow::Result<EvidenceMetadata> {
    let paths = EvidencePaths::new(repo_root);
    let stdout_path = paths.path_for(tool, "stdout.txt")?;
    let stderr_path = paths.path_for(tool, "stderr.txt")?;
    paths.prepare_file(&stdout_path)?;
    paths.prepare_file(&stderr_path)?;
    fs::write(&stdout_path, [])?;
    fs::write(&stderr_path, [])?;

    Ok(EvidenceMetadata {
        tool: tool.to_owned(),
        started_at: started_at_now(),
        duration_ms: 0,
        command,
        exit_code,
        stdout_path: paths.relative_display(stdout_path)?,
        stderr_path: paths.relative_display(stderr_path)?,
        artifact_paths,
        status,
        error_kind: error_kind.map(str::to_owned),
    })
}

fn error_result(tool: &str, error_kind: &str, message: String) -> NativeToolResult {
    NativeToolResult {
        ok: false,
        tool: tool.to_owned(),
        error_kind: Some(error_kind.to_owned()),
        message: Some(message),
        environment: None,
        evidence: None,
        session: None,
        artifact_paths: Vec::new(),
    }
}

fn display_path(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}
