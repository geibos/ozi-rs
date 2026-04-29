use ozi_rs_mcp::appium::{
    AppiumCommandOutput, AppiumDoctorState, AppiumProbe, DEFAULT_APPIUM_SERVER_URL,
    appium_click_with_session, appium_click_with_session_id, appium_doctor_for_state,
    appium_doctor_with_availability, appium_doctor_with_probe,
    appium_launch_session_with_availability, appium_launch_session_with_options,
    appium_launch_session_with_server, appium_screenshot_with_fake_image,
    appium_screenshot_with_session, appium_screenshot_with_session_id,
    appium_stop_session_with_session, appium_stop_session_with_session_id,
    appium_type_text_with_session, appium_type_text_with_session_id,
};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
};

#[test]
fn appium_absent_is_graceful() {
    let result = appium_doctor_with_availability(false);

    assert!(!result.ok);
    assert_eq!(result.tool, "appium_doctor");
    assert!(!result.available);
    assert_eq!(result.error_kind.as_deref(), Some("dependency_missing"));
    assert!(result.missing.iter().any(|missing| missing == "appium"));
    assert!(!result.install_hints.is_empty());
}

#[test]
fn appium_click_requires_session() {
    let result = appium_click_with_session(false);

    assert!(!result.ok);
    assert_eq!(result.tool, "appium_click");
    assert!(!result.available);
    assert_eq!(result.error_kind.as_deref(), Some("session_missing"));
}

#[test]
fn appium_launch_session_absent_is_graceful() {
    let result = appium_launch_session_with_availability(false);

    assert!(!result.ok);
    assert_eq!(result.tool, "appium_launch_session");
    assert!(!result.available);
    assert_eq!(result.error_kind.as_deref(), Some("dependency_missing"));
    assert!(result.missing.iter().any(|missing| missing == "appium"));
    assert!(
        result
            .message
            .as_deref()
            .is_some_and(|message| message.contains(DEFAULT_APPIUM_SERVER_URL))
    );
}

#[test]
fn appium_launch_session_available_attempts_webdriver_and_reports_server_unavailable() {
    // Use a known-unreachable URL so the test is independent of whether a real
    // Appium server happens to be running on this developer machine.
    let result = appium_launch_session_with_server(true, "http://127.0.0.1:9");

    assert!(!result.ok);
    assert_eq!(result.tool, "appium_launch_session");
    assert!(result.available);
    assert_eq!(result.error_kind.as_deref(), Some("server_unavailable"));
    assert!(
        result
            .message
            .as_deref()
            .is_some_and(|message| message.contains("127.0.0.1:9"))
    );
    assert!(
        !result
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("minimal adapter")
    );
    // Reference to keep DEFAULT_APPIUM_SERVER_URL imported by the test module.
    assert!(DEFAULT_APPIUM_SERVER_URL.starts_with("http://"));
}

#[test]
fn appium_fake_doctor_states_are_structured() {
    let driver_missing = appium_doctor_for_state(AppiumDoctorState::Mac2DriverMissing);
    assert!(!driver_missing.ok);
    assert!(driver_missing.available);
    assert_eq!(
        driver_missing.error_kind.as_deref(),
        Some("mac2_driver_missing")
    );
    assert!(
        driver_missing
            .missing
            .iter()
            .any(|missing| missing == "appium-mac2-driver")
    );

    let permissions_missing = appium_doctor_for_state(AppiumDoctorState::PermissionsMissing);
    assert!(!permissions_missing.ok);
    assert!(permissions_missing.available);
    assert_eq!(
        permissions_missing.error_kind.as_deref(),
        Some("permissions_missing")
    );
    assert!(permissions_missing.message.is_some());

    let success = appium_doctor_for_state(AppiumDoctorState::Ready);
    assert!(success.ok);
    assert!(success.available);
    assert_eq!(success.error_kind, None);
    assert!(success.missing.is_empty());
}

#[test]
fn appium_doctor_probe_distinguishes_driver_and_permissions() {
    let driver_missing = appium_doctor_with_probe(AppiumProbe {
        appium_available: true,
        driver_list: Some(AppiumCommandOutput::success("uiautomator2\n")),
        doctor: None,
    });
    assert!(!driver_missing.ok);
    assert_eq!(
        driver_missing.error_kind.as_deref(),
        Some("mac2_driver_missing")
    );

    let permissions_missing = appium_doctor_with_probe(AppiumProbe {
        appium_available: true,
        driver_list: Some(AppiumCommandOutput::success("mac2@1.0.0\n")),
        doctor: Some(AppiumCommandOutput::failure(
            1,
            "",
            "Accessibility permissions were not granted",
        )),
    });
    assert!(!permissions_missing.ok);
    assert_eq!(
        permissions_missing.error_kind.as_deref(),
        Some("permissions_missing")
    );

    let ready = appium_doctor_with_probe(AppiumProbe {
        appium_available: true,
        driver_list: Some(AppiumCommandOutput::success("mac2@1.0.0\n")),
        doctor: Some(AppiumCommandOutput::success("Doctor check passed\n")),
    });
    assert!(ready.ok);
    assert_eq!(ready.error_kind, None);
}

#[test]
fn appium_launch_session_includes_bundle_id_capability() {
    let server = FakeWebDriverServer::with_bodies(vec![FakeResponse::json(
        200,
        r#"{"value":{"sessionId":"session-bundle","capabilities":{}}}"#,
    )]);

    let result =
        appium_launch_session_with_options(true, &server.url(), Some("ru.lizaalert.ozi-rs"));

    assert!(result.ok, "{result:?}");
    let bodies = server.bodies();
    let posted = bodies.first().expect("at least one POST body");
    assert!(
        posted.contains("\"appium:bundleId\":\"ru.lizaalert.ozi-rs\""),
        "POST body missing bundleId capability: {posted}",
    );
}

#[test]
fn appium_launch_session_posts_to_webdriver_server() {
    let server = FakeWebDriverServer::new(vec![FakeResponse::json(
        200,
        r#"{"value":{"sessionId":"session-123","capabilities":{}}}"#,
    )]);

    let result = appium_launch_session_with_server(true, &server.url());

    assert!(result.ok, "{result:?}");
    assert_eq!(result.session_id.as_deref(), Some("session-123"));
    assert_eq!(server.requests(), vec!["POST /session"]);
}

#[test]
fn appium_launch_session_reports_unreachable_server() {
    let result = appium_launch_session_with_server(true, "http://127.0.0.1:9");

    assert!(!result.ok);
    assert!(result.available);
    assert_eq!(result.error_kind.as_deref(), Some("server_unavailable"));
    assert!(
        !result
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("minimal adapter")
    );
}

#[test]
fn appium_launch_session_reports_unresponsive_webdriver() {
    let server = FakeWebDriverServer::new(vec![FakeResponse::empty()]);

    let result = appium_launch_session_with_server(true, &server.url());

    assert!(!result.ok);
    assert!(result.available);
    assert_eq!(result.error_kind.as_deref(), Some("webdriver_unresponsive"));
    assert!(
        result
            .message
            .as_deref()
            .is_some_and(|message| message.contains("accepted the connection but did not return")),
        "unexpected message: {result:?}",
    );
    assert_eq!(server.requests(), vec!["POST /session"]);
}

#[test]
fn appium_screenshot_reports_unresponsive_webdriver() {
    let server = FakeWebDriverServer::new(vec![FakeResponse::empty()]);

    let result = appium_screenshot_with_session_id(&server.url(), "session-stale");

    assert!(!result.ok);
    assert!(result.available);
    assert_eq!(result.error_kind.as_deref(), Some("webdriver_unresponsive"));
    assert_eq!(
        server.requests(),
        vec!["GET /session/session-stale/screenshot"]
    );
}

#[test]
fn appium_actions_call_session_scoped_webdriver_endpoints() {
    let server = FakeWebDriverServer::new(vec![
        FakeResponse::json(200, r#"{"value":null}"#),
        FakeResponse::json(200, r#"{"value":null}"#),
        FakeResponse::json(200, r#"{"value":null}"#),
    ]);

    let click = appium_click_with_session_id(&server.url(), "session-123", Some("OK"));
    let typed =
        appium_type_text_with_session_id(&server.url(), "session-123", Some("Name"), "hello");
    let stop = appium_stop_session_with_session_id(&server.url(), "session-123");

    assert!(click.ok, "{click:?}");
    assert!(typed.ok, "{typed:?}");
    assert!(stop.ok, "{stop:?}");
    assert_eq!(
        server.requests(),
        vec![
            "POST /session/session-123/appium/mac2/click",
            "POST /session/session-123/appium/mac2/keys",
            "DELETE /session/session-123",
        ]
    );
}

#[test]
fn appium_action_helpers_require_session() {
    let actions = [
        appium_type_text_with_session(false),
        appium_screenshot_with_session(false),
        appium_stop_session_with_session(false),
    ];

    for result in actions {
        assert!(!result.ok);
        assert!(!result.available);
        assert_eq!(result.error_kind.as_deref(), Some("session_missing"));
    }
}

#[test]
fn appium_action_helpers_accept_existing_session() {
    let actions = [
        ("appium_click", appium_click_with_session(true)),
        ("appium_type_text", appium_type_text_with_session(true)),
        ("appium_screenshot", appium_screenshot_with_session(true)),
        (
            "appium_stop_session",
            appium_stop_session_with_session(true),
        ),
    ];

    for (tool, result) in actions {
        assert!(result.ok);
        assert!(result.available);
        assert_eq!(result.tool, tool);
        assert_eq!(result.error_kind, None);
    }
}

#[test]
fn appium_screenshot_writes_fake_evidence() {
    let temp = tempfile::tempdir().expect("tempdir");
    let result = appium_screenshot_with_fake_image(temp.path(), b"fake png bytes")
        .expect("fake screenshot evidence");

    assert!(result.ok);
    assert_eq!(result.tool, "appium_screenshot");
    assert_eq!(result.error_kind, None);
    assert_eq!(
        result.artifact_paths,
        vec![".sisyphus/evidence/native-qa/appium_screenshot/screenshot.png"]
    );
    assert_eq!(
        std::fs::read(temp.path().join(&result.artifact_paths[0])).expect("screenshot bytes"),
        b"fake png bytes"
    );
}

struct FakeResponse {
    status: u16,
    body: Option<String>,
}

impl FakeResponse {
    fn json(status: u16, body: &str) -> Self {
        Self {
            status,
            body: Some(body.to_owned()),
        }
    }

    fn empty() -> Self {
        Self {
            status: 200,
            body: None,
        }
    }
}

struct FakeWebDriverServer {
    url: String,
    request_rx: mpsc::Receiver<String>,
    body_rx: mpsc::Receiver<String>,
    handle: Option<thread::JoinHandle<()>>,
}

impl FakeWebDriverServer {
    fn new(responses: Vec<FakeResponse>) -> Self {
        Self::with_bodies(responses)
    }

    fn with_bodies(responses: Vec<FakeResponse>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("fake webdriver listener");
        let url = format!("http://{}", listener.local_addr().expect("local addr"));
        let (request_tx, request_rx) = mpsc::channel();
        let (body_tx, body_rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().expect("webdriver connection");
                let mut buffer = [0_u8; 4096];
                let read = stream.read(&mut buffer).expect("read request");
                let request = String::from_utf8_lossy(&buffer[..read]);
                let request_line = request.lines().next().expect("request line");
                let mut parts = request_line.split_whitespace();
                let method = parts.next().expect("method");
                let path = parts.next().expect("path");
                request_tx
                    .send(format!("{method} {path}"))
                    .expect("record request");
                let body = request
                    .split_once("\r\n\r\n")
                    .map(|(_, body)| body.to_owned())
                    .unwrap_or_default();
                body_tx.send(body).expect("record body");
                let status_text = if response.status == 200 {
                    "OK"
                } else {
                    "ERROR"
                };
                if let Some(body) = response.body {
                    let response_text = format!(
                        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        response.status,
                        status_text,
                        body.len(),
                        body
                    );
                    stream
                        .write_all(response_text.as_bytes())
                        .expect("write response");
                }
            }
        });

        Self {
            url,
            request_rx,
            body_rx,
            handle: Some(handle),
        }
    }

    fn url(&self) -> String {
        self.url.clone()
    }

    fn requests(mut self) -> Vec<String> {
        if let Some(handle) = self.handle.take() {
            handle.join().expect("fake webdriver finished");
        }
        self.request_rx.try_iter().collect()
    }

    fn bodies(&self) -> Vec<String> {
        self.body_rx.try_iter().collect()
    }
}
