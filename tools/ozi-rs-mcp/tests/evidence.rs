use std::fs;

use ozi_rs_mcp::{
    evidence::{EvidenceMetadata, EvidencePaths, EvidenceStatus},
    process::{FakeCommand, run_command_with_evidence},
};

#[test]
fn evidence_metadata_schema_contains_required_fields() {
    let metadata = EvidenceMetadata {
        tool: "build_app".to_owned(),
        started_at: "2026-04-27T11:45:00Z".to_owned(),
        duration_ms: 42,
        command: vec!["just".to_owned(), "build".to_owned()],
        exit_code: Some(0),
        stdout_path: ".sisyphus/evidence/native-qa/build/stdout.txt".to_owned(),
        stderr_path: ".sisyphus/evidence/native-qa/build/stderr.txt".to_owned(),
        artifact_paths: vec![".sisyphus/evidence/native-qa/build/result.json".to_owned()],
        status: EvidenceStatus::Passed,
        error_kind: None,
    };

    let encoded = serde_json::to_value(&metadata).expect("metadata serializes");
    let object = encoded.as_object().expect("metadata serializes as object");

    for field in [
        "tool",
        "started_at",
        "duration_ms",
        "command",
        "exit_code",
        "stdout_path",
        "stderr_path",
        "artifact_paths",
        "status",
        "error_kind",
    ] {
        assert!(object.contains_key(field), "missing evidence field {field}");
    }
}

#[test]
fn evidence_path_confinement_rejects_traversal() {
    let project_root = tempfile::tempdir().expect("temp project root");
    let paths = EvidencePaths::new(project_root.path());

    let stdout_path = paths
        .path_for("build_app", "stdout.txt")
        .expect("stdout path is confined");
    let stderr_path = paths
        .path_for("build_app", "stderr.txt")
        .expect("stderr path is confined");

    assert!(stdout_path.starts_with(project_root.path().join(".sisyphus/evidence/native-qa")));
    assert!(stderr_path.starts_with(project_root.path().join(".sisyphus/evidence/native-qa")));
    assert_ne!(stdout_path, stderr_path);
    assert!(paths.path_for("../escape", "stdout.txt").is_err());
    assert!(paths.path_for("build_app", "../escape.txt").is_err());

    println!(
        "evidence_path={}",
        stdout_path
            .strip_prefix(project_root.path())
            .expect("path is under project root")
            .display()
    );
}

#[test]
fn fake_command_captures_streams_to_distinct_evidence_files() {
    let project_root = tempfile::tempdir().expect("temp project root");
    let paths = EvidencePaths::new(project_root.path());
    let command = FakeCommand::new("fake-build")
        .arg("--deterministic")
        .stdout("deterministic stdout\n")
        .stderr("deterministic stderr\n")
        .exit_code(7);

    let result =
        run_command_with_evidence("build_app", &paths, &command).expect("fake command runs");

    assert_eq!(result.metadata.tool, "build_app");
    assert_eq!(result.metadata.command, ["fake-build", "--deterministic"]);
    assert_eq!(result.metadata.exit_code, Some(7));
    assert_eq!(result.metadata.status, EvidenceStatus::Failed);
    assert_ne!(result.stdout_path, result.stderr_path);
    assert_eq!(
        fs::read_to_string(&result.stdout_path).expect("stdout captured"),
        "deterministic stdout\n"
    );
    assert_eq!(
        fs::read_to_string(&result.stderr_path).expect("stderr captured"),
        "deterministic stderr\n"
    );

    println!(
        "stdout_path={} stderr_path={}",
        result
            .stdout_path
            .strip_prefix(project_root.path())
            .expect("stdout path is under project root")
            .display(),
        result
            .stderr_path
            .strip_prefix(project_root.path())
            .expect("stderr path is under project root")
            .display()
    );
}

#[cfg(unix)]
#[test]
fn evidence_writes_reject_symlink_redirects() {
    use std::os::unix::fs::symlink;

    let project_root = tempfile::tempdir().expect("temp project root");
    let outside = tempfile::tempdir().expect("outside dir");
    let paths = EvidencePaths::new(project_root.path());
    let tool_dir = paths.evidence_root().join("build_app");
    fs::create_dir_all(tool_dir.parent().expect("evidence root parent"))
        .expect("create evidence root");
    symlink(outside.path(), &tool_dir).expect("create symlink redirect");

    let command = FakeCommand::new("fake-build").stdout("secret");

    assert!(run_command_with_evidence("build_app", &paths, &command).is_err());
    assert!(!outside.path().join("stdout.txt").exists());
}
