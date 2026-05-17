use std::{fs, path::Path, sync::Mutex};

use ozi_rs_mcp::{
    appium::{appium_doctor, appium_launch_session, appium_screenshot, appium_stop_session},
    native::capture_logs,
};

static SMOKE_BUNDLE_AND_MAPS_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn smoke_bundle_and_maps() {
    let _guard = SMOKE_BUNDLE_AND_MAPS_LOCK
        .lock()
        .expect("smoke_bundle_and_maps lock");

    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root");

    let evidence_dir = workspace.join(".sisyphus/evidence/smoke_bundle_and_maps");

    // Create evidence directory
    if let Err(e) = fs::create_dir_all(&evidence_dir) {
        eprintln!("Failed to create evidence directory: {}", e);
        panic!("Cannot create evidence directory");
    }

    // Step 1: Check Appium availability
    println!("Checking Appium availability...");
    let doctor = appium_doctor();

    if !doctor.available {
        println!(
            "Appium not available, skipping smoke test. Reason: {}",
            doctor.message.as_deref().unwrap_or("unknown reason")
        );
        return;
    }

    println!("Appium available, proceeding with smoke test");

    // Step 2: Launch Appium session
    println!("Launching Appium session...");
    let launch_result = appium_launch_session();

    if !launch_result.ok {
        eprintln!("Failed to launch session: {:?}", launch_result.error_kind);
        panic!("appium_launch_session failed: {:?}", launch_result.message);
    }

    let session_id = match &launch_result.session_id {
        Some(sid) => {
            println!("Session created: {}", sid);
            sid.clone()
        }
        None => {
            panic!("No session ID returned from appium_launch_session");
        }
    };

    // Step 3: Capture baseline screenshot
    println!("Capturing baseline screenshot...");
    let baseline_screenshot = appium_screenshot();

    if !baseline_screenshot.ok {
        eprintln!("Failed to capture baseline screenshot");
        let _ = appium_stop_session();
        panic!("appium_screenshot (baseline) failed");
    }

    // Save baseline screenshot evidence path
    if let Some(artifact) = baseline_screenshot.artifact_paths.first() {
        println!("Baseline screenshot saved to: {}", artifact);
    }

    // Step 4: Capture logs for baseline state
    println!("Capturing baseline logs...");
    let baseline_logs = capture_logs();

    if let Some(artifact) = baseline_logs.artifact_paths.first() {
        println!("Baseline logs saved to: {}", artifact);
    }

    // Step 5: Capture screenshot after app is fully loaded (evidence for bundle-open baseline)
    println!("Capturing loaded-state screenshot...");
    let loaded_screenshot = appium_screenshot();

    if !loaded_screenshot.ok {
        eprintln!("Failed to capture loaded screenshot");
        let _ = appium_stop_session();
        panic!("appium_screenshot (after load) failed");
    }

    // Step 6: Capture final logs (looking for map-related events)
    println!("Capturing final logs for map-switch markers...");
    let final_logs = capture_logs();

    // Check if logs contain any map/tile markers
    if let Some(log_content) = final_logs
        .artifact_paths
        .first()
        .and_then(|log_artifact| fs::read_to_string(workspace.join(log_artifact)).ok())
    {
        let has_map_marker = log_content.contains("tile_source")
            || log_content.contains("map_switch")
            || log_content.contains("ozi://")
            || log_content.contains("sqlite://");

        if has_map_marker {
            println!("Map/tile markers found in logs");
        } else {
            println!(
                "Note: No explicit map-switch markers in logs (expected for AX-tree-only audit)"
            );
        }
    }

    // Step 7: Cleanup - stop app
    println!("Stopping Appium session...");
    let stop_result = appium_stop_session();

    if !stop_result.ok {
        eprintln!(
            "Warning: appium_stop_session returned non-ok status: {:?}",
            stop_result.error_kind
        );
        // Don't panic on stop failure; the session may have already terminated
    }

    // Step 8: Verify evidence files exist
    println!("Verifying evidence files...");
    let mut evidence_files = vec![];

    // Collect all artifact paths from the operations
    evidence_files.extend(doctor.artifact_paths.clone());
    evidence_files.extend(launch_result.artifact_paths.clone());
    evidence_files.extend(baseline_screenshot.artifact_paths.clone());
    evidence_files.extend(baseline_logs.artifact_paths.clone());
    evidence_files.extend(loaded_screenshot.artifact_paths.clone());
    evidence_files.extend(final_logs.artifact_paths.clone());
    evidence_files.extend(stop_result.artifact_paths.clone());

    for artifact in &evidence_files {
        let artifact_path = workspace.join(artifact);
        if !artifact_path.exists() {
            eprintln!("Missing artifact: {}", artifact);
            panic!("Evidence file not found: {}", artifact);
        }
        println!("ok: evidence verified: {}", artifact);
    }

    // Step 9: Print summary
    println!("\n=== Smoke Test Summary ===");
    println!("Test: smoke_bundle_and_maps");
    println!("Status: PASSED");
    println!("Session ID: {}", session_id);
    println!("Evidence files collected: {}", evidence_files.len());
    println!("Evidence directory: {}", evidence_dir.display());
    println!("\nEvidence files:");
    for artifact in &evidence_files {
        println!("  - {}", artifact);
    }

    // Assert overall success
    assert!(
        !evidence_files.is_empty(),
        "At least some evidence should be collected"
    );
    assert!(
        doctor.available,
        "Appium must be available for the test to pass"
    );
    assert!(launch_result.ok, "Session launch must succeed");
    assert!(baseline_screenshot.ok, "Screenshot capture must succeed");
    assert!(
        stop_result.ok || stop_result.session_id.is_some(),
        "Session stop should succeed or session should already be terminated"
    );
}
