use std::{fs, path::Path};

use ozi_rs_mcp::{
    config::find_repo_root_from,
    evidence::{EVIDENCE_ROOT, EvidenceStatus},
    native::{
        NativeSessionState, build_app_with_command, build_command, capture_screenshot_with_command,
        launch_app_from_root, launch_command, qa_environment_for_root, stop_app_at_root,
        stop_command_for_session,
    },
    process::{EvidenceCommand, FakeCommand},
};

fn create_repo_fixture(root: &Path) {
    fs::write(root.join("justfile"), "build:\n    cargo check\n").expect("write justfile");
    let tauri_dir = root.join("src-tauri");
    fs::create_dir_all(&tauri_dir).expect("create tauri dir");
    fs::write(tauri_dir.join("tauri.conf.json"), "{}").expect("write tauri config");
}

#[test]
fn repo_root_detects_repo_root_directory() {
    let repo = tempfile::tempdir().expect("repo fixture");
    create_repo_fixture(repo.path());

    let detected = find_repo_root_from(repo.path(), None).expect("repo root detected");

    assert_eq!(detected, repo.path());
}

#[test]
fn repo_root_detects_ancestor_from_subdirectory() {
    let repo = tempfile::tempdir().expect("repo fixture");
    create_repo_fixture(repo.path());
    let nested = repo.path().join("tools/ozi-rs-mcp/tests");
    fs::create_dir_all(&nested).expect("create nested dir");

    let detected = find_repo_root_from(&nested, None).expect("ancestor repo root detected");

    assert_eq!(detected, repo.path());
}

#[test]
fn repo_root_prefers_env_var_over_current_directory() {
    let cwd_repo = tempfile::tempdir().expect("cwd repo fixture");
    create_repo_fixture(cwd_repo.path());
    let env_repo = tempfile::tempdir().expect("env repo fixture");
    create_repo_fixture(env_repo.path());

    let detected = find_repo_root_from(cwd_repo.path(), Some(env_repo.path()))
        .expect("env repo root detected");

    assert_eq!(detected, env_repo.path());
}

#[test]
fn command_construction_preserves_paths_with_spaces_as_args() {
    let repo = tempfile::tempdir().expect("repo fixture");
    let app_path = repo.path().join("target dir/Ozi RS.app");

    assert_eq!(build_command(repo.path()).command_line(), ["just", "build"]);
    assert_eq!(
        launch_command(&app_path).command_line(),
        ["open", "-n", app_path.to_str().expect("utf8 app path")]
    );
}

#[test]
fn launch_app_missing_artifact_returns_structured_error() {
    let repo = tempfile::tempdir().expect("repo fixture");
    create_repo_fixture(repo.path());

    let result = launch_app_from_root(repo.path()).expect("launch result is structured");

    assert!(!result.ok);
    assert_eq!(result.error_kind.as_deref(), Some("artifact_missing"));
    assert!(result.evidence.is_some());
}

#[test]
fn build_failure_returns_command_exit_duration_and_stream_paths() {
    let repo = tempfile::tempdir().expect("repo fixture");
    create_repo_fixture(repo.path());
    let command = FakeCommand::new("just")
        .arg("build")
        .stdout("build stdout\n")
        .stderr("build stderr\n")
        .exit_code(42);

    let result = build_app_with_command(repo.path(), &command).expect("build result");
    let evidence = result.evidence.expect("build evidence");

    assert!(!result.ok);
    assert_eq!(result.error_kind.as_deref(), Some("exit_code"));
    assert_eq!(evidence.command, ["just", "build"]);
    assert_eq!(evidence.exit_code, Some(42));
    assert!(evidence.duration_ms < u128::MAX);
    assert!(evidence.stdout_path.ends_with("stdout.txt"));
    assert!(evidence.stderr_path.ends_with("stderr.txt"));
}

#[test]
fn stop_app_is_idempotent() {
    let repo = tempfile::tempdir().expect("repo fixture");
    create_repo_fixture(repo.path());

    let first = stop_app_at_root(repo.path()).expect("first stop result");
    let second = stop_app_at_root(repo.path()).expect("second stop result");

    assert!(first.ok);
    assert_eq!(first.error_kind.as_deref(), Some("already_stopped"));
    assert!(second.ok);
    assert_eq!(second.error_kind.as_deref(), Some("already_stopped"));
}

#[test]
fn stop_command_targets_launched_app_name_not_broad_repo_pattern() {
    let session = NativeSessionState {
        running: true,
        app_path: Some("src-tauri/target/debug/bundle/macos/Ozi RS.app".to_owned()),
        launched_at: Some("unix_ms:1".to_owned()),
        stopped_at: None,
    };

    let command = stop_command_for_session(&session).expect("stop command");
    let argv = command.command_line();

    assert_eq!(argv[0], "osascript");
    assert!(argv.iter().any(|arg| arg.contains("Ozi RS")));
    assert!(
        !argv
            .iter()
            .any(|arg| arg == "pkill" || arg.contains("ozi-rs"))
    );
}

#[test]
fn capture_screenshot_creates_artifact_parent_before_command_runs() {
    let repo = tempfile::tempdir().expect("repo fixture");
    create_repo_fixture(repo.path());
    let command = FakeCommand::new("screencapture").exit_code(0);

    let result = capture_screenshot_with_command(repo.path(), &command).expect("screenshot result");
    let artifact = result
        .artifact_paths
        .first()
        .expect("screenshot artifact path");
    let artifact_parent = repo
        .path()
        .join(artifact)
        .parent()
        .expect("artifact parent")
        .to_path_buf();

    assert!(result.ok);
    assert!(artifact_parent.is_dir());
}

#[test]
fn task3_qa_environment_evidence_is_parseable_json() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root");

    let result = qa_environment_for_root(repo).expect("qa environment result");
    let output_path = repo.join(".sisyphus/evidence/task-3-qa-environment.json");
    fs::create_dir_all(output_path.parent().expect("evidence parent"))
        .expect("create evidence dir");
    fs::write(
        &output_path,
        serde_json::to_vec_pretty(&result).expect("serialize environment evidence"),
    )
    .expect("write environment evidence");

    let decoded: serde_json::Value =
        serde_json::from_slice(&fs::read(&output_path).expect("read environment evidence"))
            .expect("environment evidence parses");
    assert_eq!(decoded["environment"]["platform"], "macos");
    assert!(
        decoded["environment"]["repo_root"]
            .as_str()
            .expect("repo root string")
            .ends_with("ozi-rs")
    );
    assert!(decoded["environment"].get("just_available").is_some());
    assert!(decoded["environment"].get("open_available").is_some());
    assert!(decoded["environment"].get("log_available").is_some());
    assert!(
        decoded["environment"]
            .get("screencapture_available")
            .is_some()
    );
    assert!(decoded["environment"].get("appium_available").is_some());
}

#[test]
fn task3_missing_artifact_evidence_is_parseable_json() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root");
    let repo = tempfile::tempdir().expect("repo fixture");
    create_repo_fixture(repo.path());

    let result = launch_app_from_root(repo.path()).expect("missing artifact result");
    let output_path = workspace.join(".sisyphus/evidence/task-3-missing-artifact.json");
    fs::create_dir_all(output_path.parent().expect("evidence parent"))
        .expect("create evidence dir");
    fs::write(
        &output_path,
        serde_json::to_vec_pretty(&result).expect("serialize missing artifact evidence"),
    )
    .expect("write missing artifact evidence");

    let decoded: serde_json::Value =
        serde_json::from_slice(&fs::read(&output_path).expect("read missing artifact evidence"))
            .expect("missing artifact evidence parses");
    assert_eq!(decoded["ok"], false);
    assert_eq!(decoded["error_kind"], "artifact_missing");
    assert_eq!(
        decoded["evidence"]["status"],
        serde_json::json!(EvidenceStatus::Error)
    );
    assert!(
        decoded["evidence"]["artifact_paths"]
            .as_array()
            .expect("artifact paths")
            .iter()
            .any(|path| path.as_str().expect("path string").contains(EVIDENCE_ROOT))
    );
}
