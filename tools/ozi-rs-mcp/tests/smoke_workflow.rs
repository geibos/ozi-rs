use std::{fs, path::Path, sync::Mutex};

use ozi_rs_mcp::{
    appium::{AppiumToolResult, appium_doctor_with_availability},
    evidence::{EvidenceMetadata, EvidencePaths, EvidenceStatus},
    native::{build_app_with_command, capture_screenshot_with_command, qa_environment_for_root},
    process::{FakeCommand, run_command_with_evidence},
};
use serde::Serialize;

const PHASES: [&str; 7] = [
    "qa_environment",
    "build_app",
    "launch_app",
    "capture_screenshot",
    "capture_logs",
    "stop_app",
    "appium_doctor",
];

static SMOKE_WORKFLOW_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Serialize)]
struct SmokeWorkflowReport {
    tier1_ok: bool,
    appium_available: bool,
    appium_blocked: bool,
    failed_phase: Option<&'static str>,
    error_kind: Option<String>,
    exit_code: Option<i32>,
    phase_order: Vec<&'static str>,
    evidence_files: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct FakeSmokeScenario {
    build_exit_code: i32,
    launch_exit_code: i32,
    launch_exited_early: bool,
}

impl Default for FakeSmokeScenario {
    fn default() -> Self {
        Self {
            build_exit_code: 0,
            launch_exit_code: 0,
            launch_exited_early: false,
        }
    }
}

#[derive(Debug, Serialize)]
struct AppiumGateEvidence {
    #[serde(flatten)]
    doctor: AppiumToolResult,
    blocked: bool,
    reason: String,
}

#[test]
fn native_smoke_workflow_writes_tier1_and_appium_gate_evidence() {
    let _guard = SMOKE_WORKFLOW_LOCK.lock().expect("smoke workflow lock");
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root");

    let report = run_native_smoke_with_fake_backend(workspace, FakeSmokeScenario::default())
        .expect("smoke workflow report");

    assert!(report.tier1_ok);
    assert!(!report.appium_available);
    assert!(!report.appium_blocked);
    assert_eq!(report.failed_phase, None);
    assert_eq!(report.error_kind, None);
    assert_eq!(report.exit_code, None);
    assert_eq!(report.phase_order, PHASES,);

    for evidence_file in &report.evidence_files {
        assert!(
            workspace.join(evidence_file).is_file(),
            "missing smoke evidence file {evidence_file}"
        );
    }
}

#[test]
fn native_smoke_workflow_reports_build_failure_evidence() {
    let _guard = SMOKE_WORKFLOW_LOCK.lock().expect("smoke workflow lock");
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root");

    run_native_smoke_with_fake_backend(workspace, FakeSmokeScenario::default())
        .expect("seed happy-path shared smoke report");

    let report = run_native_smoke_with_fake_backend(
        workspace,
        FakeSmokeScenario {
            build_exit_code: 42,
            ..FakeSmokeScenario::default()
        },
    )
    .expect("build failure smoke workflow report");

    assert!(!report.tier1_ok);
    assert!(!report.appium_available);
    assert!(!report.appium_blocked);
    assert_eq!(report.failed_phase, Some("build_app"));
    assert_eq!(report.error_kind.as_deref(), Some("exit_code"));
    assert_eq!(report.exit_code, Some(42));
    assert_eq!(report.phase_order, PHASES);
    assert!(
        report
            .evidence_files
            .contains(&".sisyphus/evidence/native-qa/build_app/stdout.txt".to_owned())
    );
    assert!(
        report
            .evidence_files
            .contains(&".sisyphus/evidence/native-qa/build_app/stderr.txt".to_owned())
    );

    let shared_report_path =
        workspace.join(".sisyphus/evidence/native-qa/smoke_workflow/report.json");
    let shared_report_bytes =
        fs::read(&shared_report_path).expect("shared smoke report remains readable");
    let shared_report: serde_json::Value = serde_json::from_slice(&shared_report_bytes)
        .expect("shared smoke report remains one JSON object");
    assert!(
        shared_report["tier1_ok"].as_bool().unwrap_or_default(),
        "failure scenarios must not overwrite final happy-path smoke evidence"
    );
}

#[test]
fn native_smoke_workflow_reports_launch_exited_early_and_logs() {
    let _guard = SMOKE_WORKFLOW_LOCK.lock().expect("smoke workflow lock");
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root");

    run_native_smoke_with_fake_backend(workspace, FakeSmokeScenario::default())
        .expect("seed happy-path shared smoke report");

    let report = run_native_smoke_with_fake_backend(
        workspace,
        FakeSmokeScenario {
            launch_exited_early: true,
            ..FakeSmokeScenario::default()
        },
    )
    .expect("launch exited early smoke workflow report");

    assert!(!report.tier1_ok);
    assert!(!report.appium_available);
    assert!(!report.appium_blocked);
    assert_eq!(report.failed_phase, Some("launch_app"));
    assert_eq!(report.error_kind.as_deref(), Some("exited_early"));
    assert_eq!(report.exit_code, Some(0));
    assert_eq!(report.phase_order, PHASES);
    assert!(
        report
            .evidence_files
            .contains(&".sisyphus/evidence/native-qa/capture_logs/stdout.txt".to_owned())
    );
    assert!(
        report
            .evidence_files
            .contains(&".sisyphus/evidence/native-qa/capture_logs/stderr.txt".to_owned())
    );

    let shared_report_path =
        workspace.join(".sisyphus/evidence/native-qa/smoke_workflow/report.json");
    let shared_report_bytes =
        fs::read(&shared_report_path).expect("shared smoke report remains readable");
    let shared_report: serde_json::Value = serde_json::from_slice(&shared_report_bytes)
        .expect("shared smoke report remains one JSON object");
    assert!(
        shared_report["tier1_ok"].as_bool().unwrap_or_default(),
        "failure scenarios must not overwrite final happy-path smoke evidence"
    );
}

fn run_native_smoke_with_fake_backend(
    workspace: &Path,
    scenario: FakeSmokeScenario,
) -> anyhow::Result<SmokeWorkflowReport> {
    let paths = EvidencePaths::new(workspace);
    let mut evidence_files = Vec::new();
    let mut failed_phase = None;
    let mut error_kind = None;
    let mut exit_code = None;

    let environment = qa_environment_for_root(workspace)?;
    let environment_path = paths.path_for("qa_environment", "result.json")?;
    paths.prepare_file(&environment_path)?;
    fs::write(&environment_path, serde_json::to_vec_pretty(&environment)?)?;
    evidence_files.push(paths.relative_display(&environment_path)?);

    let build = build_app_with_command(
        workspace,
        &FakeCommand::new("just")
            .arg("build")
            .stdout("fake native smoke build succeeded\n")
            .stderr("fake native smoke build failed deterministically\n")
            .exit_code(scenario.build_exit_code),
    )?;
    let build_evidence = build.evidence.expect("build evidence");
    evidence_files.extend(evidence_files_for_metadata(&build_evidence));
    if !build.ok {
        failed_phase = Some("build_app");
        error_kind = build_evidence.error_kind.clone();
        exit_code = build_evidence.exit_code;
    }

    let launch = run_command_with_evidence(
        "launch_app",
        &paths,
        &FakeCommand::new("open")
            .arg("-n")
            .arg("target/debug/bundle/macos/Ozi RS.app")
            .stdout("fake native smoke launch succeeded\n")
            .exit_code(scenario.launch_exit_code),
    )?;
    evidence_files.extend(evidence_files_for_metadata(&launch.metadata));
    if failed_phase.is_none() && launch.metadata.status != EvidenceStatus::Passed {
        failed_phase = Some("launch_app");
        error_kind = launch.metadata.error_kind.clone();
        exit_code = launch.metadata.exit_code;
    }
    if failed_phase.is_none() && scenario.launch_exited_early {
        failed_phase = Some("launch_app");
        error_kind = Some("exited_early".to_owned());
        exit_code = launch.metadata.exit_code;
    }

    let screenshot = capture_screenshot_with_command(
        workspace,
        &FakeCommand::new("screencapture")
            .arg("-x")
            .arg(".sisyphus/evidence/native-qa/capture_screenshot/screenshot.png")
            .stdout("fake native smoke screenshot succeeded\n"),
    )?;
    let screenshot_path = paths.path_for("capture_screenshot", "screenshot.png")?;
    paths.prepare_file(&screenshot_path)?;
    fs::write(&screenshot_path, b"fake screenshot placeholder\n")?;
    let screenshot_evidence = screenshot.evidence.expect("screenshot evidence");
    evidence_files.extend(evidence_files_for_metadata(&screenshot_evidence));

    let logs = run_command_with_evidence(
        "capture_logs",
        &paths,
        &FakeCommand::new("log")
            .arg("show")
            .arg("--last")
            .arg("5m")
            .stdout("fake native smoke logs captured\n"),
    )?;
    evidence_files.extend(evidence_files_for_metadata(&logs.metadata));

    let stop = run_command_with_evidence(
        "stop_app",
        &paths,
        &FakeCommand::new("osascript")
            .arg("-e")
            .arg("quit app \"Ozi RS\"")
            .stdout("fake native smoke stop succeeded\n"),
    )?;
    evidence_files.extend(evidence_files_for_metadata(&stop.metadata));

    let appium_doctor = appium_doctor_with_availability(false);
    let appium_gate = AppiumGateEvidence {
        blocked: false,
        reason: appium_doctor
            .message
            .clone()
            .unwrap_or_else(|| "Appium Mac2 adapter is unavailable but gated safely".to_owned()),
        doctor: appium_doctor,
    };
    let appium_native_path = paths.path_for("appium_doctor", "result.json")?;
    paths.prepare_file(&appium_native_path)?;
    fs::write(
        &appium_native_path,
        serde_json::to_vec_pretty(&appium_gate)?,
    )?;
    evidence_files.push(paths.relative_display(&appium_native_path)?);

    let task_appium_path = workspace.join(".sisyphus/evidence/task-8-appium-gated.json");
    fs::create_dir_all(task_appium_path.parent().expect("task evidence parent"))?;
    fs::write(&task_appium_path, serde_json::to_vec_pretty(&appium_gate)?)?;
    evidence_files.push(
        task_appium_path
            .strip_prefix(workspace)?
            .to_string_lossy()
            .to_string(),
    );

    let tier1_ok = failed_phase.is_none()
        && build.ok
        && launch.metadata.status == EvidenceStatus::Passed
        && screenshot.ok
        && logs.metadata.status == EvidenceStatus::Passed
        && stop.metadata.status == EvidenceStatus::Passed;

    let report = SmokeWorkflowReport {
        tier1_ok,
        appium_available: appium_gate.doctor.available,
        appium_blocked: appium_gate.blocked,
        failed_phase,
        error_kind,
        exit_code,
        phase_order: PHASES.to_vec(),
        evidence_files,
    };
    let report_file_name = if report.tier1_ok {
        "report.json".to_owned()
    } else {
        format!(
            "report-{}.json",
            report.failed_phase.unwrap_or("unknown_failure")
        )
    };
    let report_path = paths.path_for("smoke_workflow", &report_file_name)?;
    write_json_atomically(&paths, &report_path, &report)?;

    Ok(report)
}

fn write_json_atomically<T: Serialize>(
    paths: &EvidencePaths,
    path: &Path,
    value: &T,
) -> anyhow::Result<()> {
    paths.prepare_file(path)?;
    let tmp_path = path.with_extension("json.tmp");
    paths.prepare_file(&tmp_path)?;
    fs::write(&tmp_path, serde_json::to_vec_pretty(value)?)?;
    fs::rename(tmp_path, path)?;
    Ok(())
}

fn evidence_files_for_metadata(metadata: &EvidenceMetadata) -> Vec<String> {
    let mut files = vec![metadata.stdout_path.clone(), metadata.stderr_path.clone()];
    files.extend(metadata.artifact_paths.clone());
    files
}
