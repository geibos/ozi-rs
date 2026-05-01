use std::{
    env,
    error::Error,
    fmt, fs,
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use rmcp::schemars;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{config, evidence::EvidencePaths};

pub const DEFAULT_APPIUM_SERVER_URL: &str = "http://127.0.0.1:4723";
pub const DEFAULT_APPIUM_BUNDLE_ID: &str = "ru.lizaalert.ozi-rs";

#[derive(Debug, Clone, Serialize, schemars::JsonSchema, PartialEq, Eq)]
pub struct AppiumToolResult {
    pub ok: bool,
    pub tool: String,
    pub available: bool,
    pub error_kind: Option<String>,
    pub missing: Vec<String>,
    pub message: Option<String>,
    pub session_id: Option<String>,
    pub install_hints: Vec<String>,
    pub artifact_paths: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppiumDoctorState {
    AppiumMissing,
    Mac2DriverMissing,
    PermissionsMissing,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppiumCommandOutput {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl AppiumCommandOutput {
    pub fn success(stdout: &str) -> Self {
        Self {
            exit_code: Some(0),
            stdout: stdout.to_owned(),
            stderr: String::new(),
        }
    }

    pub fn failure(exit_code: i32, stdout: &str, stderr: &str) -> Self {
        Self {
            exit_code: Some(exit_code),
            stdout: stdout.to_owned(),
            stderr: stderr.to_owned(),
        }
    }

    fn ok(&self) -> bool {
        self.exit_code == Some(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppiumProbe {
    pub appium_available: bool,
    pub driver_list: Option<AppiumCommandOutput>,
    pub doctor: Option<AppiumCommandOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct AppiumSessionState {
    server_url: String,
    session_id: String,
}

pub fn appium_doctor() -> AppiumToolResult {
    if !command_available("appium") {
        return appium_doctor_for_state(AppiumDoctorState::AppiumMissing);
    }

    let driver_list = run_appium_command(["driver", "list", "--installed"]);
    let doctor = run_appium_command(["driver", "doctor", "mac2"]);
    appium_doctor_with_probe(AppiumProbe {
        appium_available: true,
        driver_list: Some(driver_list),
        doctor: Some(doctor),
    })
}

pub fn appium_doctor_with_availability(appium_available: bool) -> AppiumToolResult {
    appium_doctor_with_probe(AppiumProbe {
        appium_available,
        driver_list: None,
        doctor: None,
    })
}

pub fn appium_doctor_with_probe(probe: AppiumProbe) -> AppiumToolResult {
    if !probe.appium_available {
        return appium_doctor_for_state(AppiumDoctorState::AppiumMissing);
    }

    let Some(driver_list) = probe.driver_list else {
        return appium_doctor_for_state(AppiumDoctorState::Mac2DriverMissing);
    };
    let driver_output = format!("{}\n{}", driver_list.stdout, driver_list.stderr).to_lowercase();
    if !driver_list.ok() || !driver_output.contains("mac2") {
        return appium_doctor_for_state(AppiumDoctorState::Mac2DriverMissing);
    }

    let Some(doctor) = probe.doctor else {
        return appium_doctor_for_state(AppiumDoctorState::PermissionsMissing);
    };
    if !doctor.ok() {
        return appium_doctor_for_state(AppiumDoctorState::PermissionsMissing);
    }

    appium_doctor_for_state(AppiumDoctorState::Ready)
}

pub fn appium_doctor_for_state(state: AppiumDoctorState) -> AppiumToolResult {
    match state {
        AppiumDoctorState::AppiumMissing => AppiumToolResult {
            ok: false,
            tool: "appium_doctor".to_owned(),
            available: false,
            error_kind: Some("dependency_missing".to_owned()),
            missing: vec!["appium".to_owned()],
            message: Some(
                "Appium is not available on PATH; Appium Mac2 tools are disabled".to_owned(),
            ),
            session_id: None,
            install_hints: vec![
                "Install Appium 2 and ensure the appium executable is on PATH".to_owned(),
                "Install the Appium Mac2 driver before enabling native UI actions".to_owned(),
            ],
            artifact_paths: Vec::new(),
        },
        AppiumDoctorState::Mac2DriverMissing => AppiumToolResult {
            ok: false,
            tool: "appium_doctor".to_owned(),
            available: true,
            error_kind: Some("mac2_driver_missing".to_owned()),
            missing: vec!["appium-mac2-driver".to_owned()],
            message: Some("Appium is present, but the Mac2 driver is not confirmed".to_owned()),
            session_id: None,
            install_hints: vec![
                "Install the Appium Mac2 driver with `appium driver install mac2`".to_owned(),
            ],
            artifact_paths: Vec::new(),
        },
        AppiumDoctorState::PermissionsMissing => AppiumToolResult {
            ok: false,
            tool: "appium_doctor".to_owned(),
            available: true,
            error_kind: Some("permissions_missing".to_owned()),
            missing: vec!["macos-accessibility-permissions".to_owned()],
            message: Some(
                "Appium is present, but macOS Accessibility permissions are not confirmed"
                    .to_owned(),
            ),
            session_id: None,
            install_hints: vec![
                "Grant Accessibility permissions before enabling Appium Mac2 UI actions".to_owned(),
            ],
            artifact_paths: Vec::new(),
        },
        AppiumDoctorState::Ready => AppiumToolResult {
            ok: true,
            tool: "appium_doctor".to_owned(),
            available: true,
            error_kind: None,
            missing: Vec::new(),
            message: Some(format!(
                "Appium Mac2 prerequisites are available; default server URL is {DEFAULT_APPIUM_SERVER_URL}"
            )),
            session_id: None,
            install_hints: Vec::new(),
            artifact_paths: Vec::new(),
        },
    }
}

pub fn appium_launch_session() -> AppiumToolResult {
    appium_launch_session_with_options(
        command_available("appium"),
        &appium_server_url(),
        bundle_id_override(),
    )
}

pub fn appium_launch_session_with_availability(appium_available: bool) -> AppiumToolResult {
    appium_launch_session_with_options(appium_available, DEFAULT_APPIUM_SERVER_URL, None)
}

pub fn appium_launch_session_with_server(
    appium_available: bool,
    server_url: &str,
) -> AppiumToolResult {
    appium_launch_session_with_options(appium_available, server_url, None)
}

pub fn appium_launch_session_with_caller_options(
    caller_bundle_id: Option<&str>,
) -> AppiumToolResult {
    let env_bundle = bundle_id_override();
    let resolved = caller_bundle_id.or(env_bundle);
    appium_launch_session_with_options(command_available("appium"), &appium_server_url(), resolved)
}

pub fn appium_launch_session_with_options(
    appium_available: bool,
    server_url: &str,
    bundle_id: Option<&str>,
) -> AppiumToolResult {
    if !appium_available {
        return appium_missing_result(
            "appium_launch_session",
            format!("Appium is not available on PATH; no session was started for {server_url}"),
        );
    }

    let resolved_bundle = bundle_id.unwrap_or(DEFAULT_APPIUM_BUNDLE_ID);
    let body = json!({
        "capabilities": {
            "alwaysMatch": {
                "platformName": "Mac",
                "appium:automationName": "Mac2",
                "appium:bundleId": resolved_bundle
            }
        }
    });

    match webdriver_request("POST", server_url, "/session", Some(&body)) {
        Ok(response) if response.status_code < 400 => {
            let session_id = extract_session_id(&response.body);
            if let Some(session_id) = session_id {
                let _ = persist_session(server_url, &session_id);
                AppiumToolResult {
                    ok: true,
                    tool: "appium_launch_session".to_owned(),
                    available: true,
                    error_kind: None,
                    missing: Vec::new(),
                    message: Some(format!(
                        "Started Appium Mac2 WebDriver session at {server_url} for {resolved_bundle}"
                    )),
                    session_id: Some(session_id),
                    install_hints: Vec::new(),
                    artifact_paths: Vec::new(),
                }
            } else {
                appium_failure_result(
                    "appium_launch_session",
                    "session_error",
                    format!("Appium server at {server_url} did not return a sessionId"),
                )
            }
        }
        Ok(response) => appium_failure_result(
            "appium_launch_session",
            "session_error",
            format!(
                "Appium server at {server_url} rejected session creation with HTTP {} (body: {})",
                response.status_code,
                truncate(&response.body, 400),
            ),
        ),
        Err(error) => webdriver_request_error_result("appium_launch_session", server_url, &error),
    }
}

fn bundle_id_override() -> Option<&'static str> {
    static OVERRIDE: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    OVERRIDE
        .get_or_init(|| env::var("OZI_RS_APPIUM_BUNDLE_ID").ok())
        .as_deref()
}

fn truncate(text: &str, max: usize) -> String {
    if text.len() <= max {
        text.to_owned()
    } else {
        let mut trimmed = text[..max].to_owned();
        trimmed.push_str("...");
        trimmed
    }
}

fn appium_missing_result(tool: &str, message: String) -> AppiumToolResult {
    AppiumToolResult {
        ok: false,
        tool: tool.to_owned(),
        available: false,
        error_kind: Some("dependency_missing".to_owned()),
        missing: vec!["appium".to_owned()],
        message: Some(message),
        session_id: None,
        install_hints: vec![
            "Install Appium 2 and ensure the appium executable is on PATH".to_owned(),
            "Install the Appium Mac2 driver before enabling native UI actions".to_owned(),
        ],
        artifact_paths: Vec::new(),
    }
}

pub fn appium_click() -> AppiumToolResult {
    appium_click_for_params(None)
}

pub fn appium_click_for_params(selector: Option<&str>) -> AppiumToolResult {
    match load_session() {
        Some(session) => {
            appium_click_with_session_id(&session.server_url, &session.session_id, selector)
        }
        None => appium_click_with_session(false),
    }
}

pub fn appium_click_with_session(session_running: bool) -> AppiumToolResult {
    appium_action_with_session("appium_click", "click actions", session_running)
}

pub fn appium_type_text() -> AppiumToolResult {
    appium_type_text_for_params(None, "")
}

pub fn appium_type_text_for_params(selector: Option<&str>, text: &str) -> AppiumToolResult {
    match load_session() {
        Some(session) => appium_type_text_with_session_id(
            &session.server_url,
            &session.session_id,
            selector,
            text,
        ),
        None => appium_type_text_with_session(false),
    }
}

pub fn appium_type_text_with_session(session_running: bool) -> AppiumToolResult {
    appium_action_with_session("appium_type_text", "text input actions", session_running)
}

pub fn appium_screenshot() -> AppiumToolResult {
    match load_session() {
        Some(session) => {
            appium_screenshot_with_session_id(&session.server_url, &session.session_id)
        }
        None => appium_screenshot_with_session(false),
    }
}

pub fn appium_screenshot_with_session(session_running: bool) -> AppiumToolResult {
    appium_action_with_session("appium_screenshot", "screenshot capture", session_running)
}

pub fn appium_screenshot_with_fake_image(
    workspace_root: &Path,
    image_bytes: &[u8],
) -> anyhow::Result<AppiumToolResult> {
    let paths = EvidencePaths::new(workspace_root);
    let path = paths.path_for("appium_screenshot", "screenshot.png")?;
    paths.prepare_file(&path)?;
    fs::write(&path, image_bytes)?;
    let relative_path = paths.relative_display(&path)?;

    Ok(AppiumToolResult {
        ok: true,
        tool: "appium_screenshot".to_owned(),
        available: true,
        error_kind: None,
        missing: Vec::new(),
        message: Some("Fake Appium screenshot evidence written".to_owned()),
        session_id: None,
        install_hints: Vec::new(),
        artifact_paths: vec![relative_path],
    })
}

pub fn appium_stop_session() -> AppiumToolResult {
    match load_session() {
        Some(session) => {
            appium_stop_session_with_session_id(&session.server_url, &session.session_id)
        }
        None => appium_stop_session_with_session(false),
    }
}

pub fn appium_stop_session_with_session(session_running: bool) -> AppiumToolResult {
    appium_action_with_session("appium_stop_session", "session shutdown", session_running)
}

fn appium_action_with_session(
    tool: &'static str,
    action_description: &'static str,
    session_running: bool,
) -> AppiumToolResult {
    if session_running {
        return AppiumToolResult {
            ok: true,
            tool: tool.to_owned(),
            available: true,
            error_kind: None,
            missing: Vec::new(),
            message: Some(format!(
                "Appium session is available for {action_description}"
            )),
            session_id: None,
            install_hints: Vec::new(),
            artifact_paths: Vec::new(),
        };
    }

    AppiumToolResult {
        ok: false,
        tool: tool.to_owned(),
        available: false,
        error_kind: Some("session_missing".to_owned()),
        missing: Vec::new(),
        message: Some(format!(
            "Appium {action_description} requires an active Appium session"
        )),
        session_id: None,
        install_hints: Vec::new(),
        artifact_paths: Vec::new(),
    }
}

/// Parse a selector string into a WebDriver (using, value) pair.
///
/// Precedence (first match wins):
/// - Starts with `~`      → `accessibility id` / rest of string
/// - Starts with `//` or `(/` → `xpath` / full string
/// - Starts with `**/`   → `-ios class chain` / full string
/// - Starts with `name=` → `name` / rest of string after `name=`
/// - Otherwise            → `name` / full string  (convenience default)
fn parse_selector(s: &str) -> (&'static str, String) {
    if let Some(rest) = s.strip_prefix('~') {
        return ("accessibility id", rest.to_owned());
    }
    if s.starts_with("//") || s.starts_with("(/") {
        return ("xpath", s.to_owned());
    }
    if s.starts_with("**/") {
        return ("-ios class chain", s.to_owned());
    }
    if let Some(rest) = s.strip_prefix("name=") {
        return ("name", rest.to_owned());
    }
    ("name", s.to_owned())
}

/// POST `/session/{sid}/element` and return the element id string on success.
///
/// Returns `Err(AppiumToolResult)` on any failure (not-found or HTTP error).
/// `tool` is used to brand the error result (e.g. `"appium_click"` or `"appium_type_text"`).
#[allow(clippy::result_large_err)]
fn find_wd_element(
    tool: &str,
    server_url: &str,
    session_id: &str,
    using: &str,
    value: &str,
    original_selector: &str,
) -> Result<String, AppiumToolResult> {
    let body = json!({ "using": using, "value": value });
    match webdriver_request(
        "POST",
        server_url,
        &format!("/session/{session_id}/element"),
        Some(&body),
    ) {
        Ok(response) if response.status_code < 400 => {
            // Try legacy ELEMENT key first, then W3C key.
            let eid = serde_json::from_str::<serde_json::Value>(&response.body)
                .ok()
                .and_then(|v| {
                    let obj = v.get("value")?;
                    obj.get("ELEMENT")
                        .or_else(|| obj.get("element-6066-11e4-a52e-4f735466cecf"))
                        .and_then(|id| id.as_str())
                        .map(str::to_owned)
                });
            eid.ok_or_else(|| {
                appium_failure_result(
                    tool,
                    "webdriver_error",
                    format!(
                        "find_element response missing element id: {}",
                        truncate(&response.body, 400)
                    ),
                )
            })
        }
        Ok(response)
            if response.status_code == 404
                && (response.body.contains("NoSuchElement")
                    || response.body.contains("no such element")) =>
        {
            Err(appium_failure_result(
                tool,
                "element_not_found",
                format!(
                    "No element found matching selector \"{original_selector}\": {}",
                    truncate(&response.body, 400)
                ),
            ))
        }
        Ok(response) => Err(appium_failure_result(
            tool,
            "webdriver_error",
            format!(
                "find_element failed with HTTP {}: {}",
                response.status_code,
                truncate(&response.body, 400)
            ),
        )),
        Err(error) => Err(webdriver_request_error_result(tool, server_url, &error)),
    }
}

pub fn appium_click_with_session_id(
    server_url: &str,
    session_id: &str,
    selector: Option<&str>,
) -> AppiumToolResult {
    let sel = match selector.filter(|s| !s.is_empty()) {
        Some(s) => s,
        None => {
            return appium_failure_result(
                "appium_click",
                "selector_required",
                "appium_click requires a selector".to_owned(),
            )
        }
    };

    let (using, value) = parse_selector(sel);

    let eid = match find_wd_element("appium_click", server_url, session_id, using, &value, sel) {
        Ok(id) => id,
        Err(result) => return result,
    };

    match webdriver_request(
        "POST",
        server_url,
        &format!("/session/{session_id}/element/{eid}/click"),
        Some(&json!({})),
    ) {
        Ok(response) if response.status_code < 400 => AppiumToolResult {
            ok: true,
            tool: "appium_click".to_owned(),
            available: true,
            error_kind: None,
            missing: Vec::new(),
            message: Some(format!(
                "Clicked element matching \"{sel}\" in session {session_id}"
            )),
            session_id: Some(session_id.to_owned()),
            install_hints: Vec::new(),
            artifact_paths: Vec::new(),
        },
        Ok(response) => appium_failure_result(
            "appium_click",
            "webdriver_error",
            format!(
                "Element click failed with HTTP {}: {}",
                response.status_code,
                truncate(&response.body, 400)
            ),
        ),
        Err(error) => webdriver_request_error_result("appium_click", server_url, &error),
    }
}

pub fn appium_type_text_with_session_id(
    server_url: &str,
    session_id: &str,
    selector: Option<&str>,
    text: &str,
) -> AppiumToolResult {
    let sel = match selector.filter(|s| !s.is_empty()) {
        Some(s) => s,
        None => {
            return appium_failure_result(
                "appium_type_text",
                "selector_required",
                "appium_type_text requires a selector".to_owned(),
            )
        }
    };

    let (using, value) = parse_selector(sel);

    let eid =
        match find_wd_element("appium_type_text", server_url, session_id, using, &value, sel) {
            Ok(id) => id,
            Err(result) => return result,
        };

    let body = json!({ "text": text });
    match webdriver_request(
        "POST",
        server_url,
        &format!("/session/{session_id}/element/{eid}/value"),
        Some(&body),
    ) {
        Ok(response) if response.status_code < 400 => AppiumToolResult {
            ok: true,
            tool: "appium_type_text".to_owned(),
            available: true,
            error_kind: None,
            missing: Vec::new(),
            message: Some(format!(
                "Typed text into element matching \"{sel}\" in session {session_id}"
            )),
            session_id: Some(session_id.to_owned()),
            install_hints: Vec::new(),
            artifact_paths: Vec::new(),
        },
        Ok(response) => appium_failure_result(
            "appium_type_text",
            "webdriver_error",
            format!(
                "Element value failed with HTTP {}: {}",
                response.status_code,
                truncate(&response.body, 400)
            ),
        ),
        Err(error) => webdriver_request_error_result("appium_type_text", server_url, &error),
    }
}

pub fn appium_screenshot_with_session_id(server_url: &str, session_id: &str) -> AppiumToolResult {
    match webdriver_request(
        "GET",
        server_url,
        &format!("/session/{session_id}/screenshot"),
        None,
    ) {
        Ok(response) if response.status_code < 400 => {
            let bytes = serde_json::from_str::<serde_json::Value>(&response.body)
                .ok()
                .and_then(|value| {
                    value
                        .get("value")
                        .and_then(|value| value.as_str())
                        .map(str::to_owned)
                })
                .unwrap_or(response.body)
                .into_bytes();
            match config::repo_root()
                .and_then(|root| appium_screenshot_with_fake_image(&root, &bytes))
            {
                Ok(mut result) => {
                    result.message =
                        Some("Captured Appium WebDriver screenshot evidence".to_owned());
                    result.session_id = Some(session_id.to_owned());
                    result
                }
                Err(error) => appium_failure_result(
                    "appium_screenshot",
                    "evidence_error",
                    format!("Appium screenshot succeeded but evidence write failed: {error}"),
                ),
            }
        }
        Ok(response) => appium_failure_result(
            "appium_screenshot",
            "webdriver_error",
            format!(
                "Appium screenshot failed with HTTP {}",
                response.status_code
            ),
        ),
        Err(error) => webdriver_request_error_result("appium_screenshot", server_url, &error),
    }
}

pub fn appium_stop_session_with_session_id(server_url: &str, session_id: &str) -> AppiumToolResult {
    let result = webdriver_action(
        "appium_stop_session",
        server_url,
        session_id,
        "DELETE",
        &format!("/session/{session_id}"),
        None,
    );
    if result.ok {
        let _ = remove_persisted_session();
    }
    result
}

fn webdriver_action(
    tool: &str,
    server_url: &str,
    session_id: &str,
    method: &str,
    path: &str,
    body: Option<&serde_json::Value>,
) -> AppiumToolResult {
    match webdriver_request(method, server_url, path, body) {
        Ok(response) if response.status_code < 400 => AppiumToolResult {
            ok: true,
            tool: tool.to_owned(),
            available: true,
            error_kind: None,
            missing: Vec::new(),
            message: Some(format!(
                "Appium WebDriver action completed for session {session_id}"
            )),
            session_id: Some(session_id.to_owned()),
            install_hints: Vec::new(),
            artifact_paths: Vec::new(),
        },
        Ok(response) => appium_failure_result(
            tool,
            "webdriver_error",
            format!(
                "Appium WebDriver action failed with HTTP {}",
                response.status_code
            ),
        ),
        Err(error) => webdriver_request_error_result(tool, server_url, &error),
    }
}

fn webdriver_request_error_result(
    tool: &str,
    server_url: &str,
    error: &WebDriverRequestError,
) -> AppiumToolResult {
    if error.is_unresponsive() {
        return appium_failure_result(
            tool,
            "webdriver_unresponsive",
            format!(
                "Appium server at {server_url} accepted the connection but did not return a valid WebDriver response: {error}. Restart Appium and clear stale Mac2 sessions before retrying."
            ),
        );
    }

    appium_failure_result(
        tool,
        "server_unavailable",
        format!("Appium server at {server_url} is unreachable: {error}"),
    )
}

#[derive(Debug)]
enum WebDriverRequestError {
    Unavailable(anyhow::Error),
    Unresponsive(anyhow::Error),
}

impl WebDriverRequestError {
    fn unavailable(error: impl Into<anyhow::Error>) -> Self {
        Self::Unavailable(error.into())
    }

    fn unresponsive(error: impl Into<anyhow::Error>) -> Self {
        Self::Unresponsive(error.into())
    }

    fn is_unresponsive(&self) -> bool {
        matches!(self, Self::Unresponsive(_))
    }
}

impl fmt::Display for WebDriverRequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(error) | Self::Unresponsive(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for WebDriverRequestError {}

fn appium_failure_result(tool: &str, error_kind: &str, message: String) -> AppiumToolResult {
    AppiumToolResult {
        ok: false,
        tool: tool.to_owned(),
        available: true,
        error_kind: Some(error_kind.to_owned()),
        missing: Vec::new(),
        message: Some(message),
        session_id: None,
        install_hints: Vec::new(),
        artifact_paths: Vec::new(),
    }
}

struct HttpResponse {
    status_code: u16,
    body: String,
}

fn webdriver_request(
    method: &str,
    server_url: &str,
    path: &str,
    body: Option<&serde_json::Value>,
) -> Result<HttpResponse, WebDriverRequestError> {
    let (host, port) = parse_http_url(server_url).map_err(WebDriverRequestError::unavailable)?;
    let mut stream =
        TcpStream::connect((host.as_str(), port)).map_err(WebDriverRequestError::unavailable)?;
    // Mac2 session creation can take 15-30s while the driver attaches to the
    // app and probes Accessibility; quick endpoints return instantly, so a
    // generous read budget is safe.
    stream
        .set_read_timeout(Some(Duration::from_secs(60)))
        .map_err(WebDriverRequestError::unresponsive)?;
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(WebDriverRequestError::unresponsive)?;
    let body_text = body.map(serde_json::Value::to_string).unwrap_or_default();
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body_text}",
        body_text.len()
    );
    stream
        .write_all(request.as_bytes())
        .map_err(WebDriverRequestError::unresponsive)?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(WebDriverRequestError::unresponsive)?;
    let (head, body) = response.split_once("\r\n\r\n").ok_or_else(|| {
        WebDriverRequestError::unresponsive(anyhow::anyhow!("malformed HTTP response"))
    })?;
    let status_code = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| {
            WebDriverRequestError::unresponsive(anyhow::anyhow!("missing HTTP status"))
        })?;

    Ok(HttpResponse {
        status_code,
        body: body.to_owned(),
    })
}

fn parse_http_url(url: &str) -> anyhow::Result<(String, u16)> {
    let without_scheme = url
        .strip_prefix("http://")
        .ok_or_else(|| anyhow::anyhow!("only http:// Appium server URLs are supported"))?;
    let authority = without_scheme.split('/').next().unwrap_or(without_scheme);
    let (host, port) = authority.rsplit_once(':').unwrap_or((authority, "4723"));
    Ok((host.to_owned(), port.parse()?))
}

fn extract_session_id(body: &str) -> Option<String> {
    let value = serde_json::from_str::<serde_json::Value>(body).ok()?;
    value
        .get("value")
        .and_then(|value| value.get("sessionId"))
        .or_else(|| value.get("sessionId"))
        .and_then(|value| value.as_str())
        .map(str::to_owned)
}

fn appium_server_url() -> String {
    env::var("OZI_RS_APPIUM_SERVER_URL").unwrap_or_else(|_| DEFAULT_APPIUM_SERVER_URL.to_owned())
}

fn session_path() -> anyhow::Result<PathBuf> {
    let root = config::repo_root()?;
    let paths = EvidencePaths::new(root);
    paths.path_for("appium", "session.json")
}

fn persist_session(server_url: &str, session_id: &str) -> anyhow::Result<()> {
    let root = config::repo_root()?;
    let paths = EvidencePaths::new(root);
    let path = paths.path_for("appium", "session.json")?;
    paths.prepare_file(&path)?;
    let tmp_path = path.with_extension("json.tmp");
    fs::write(
        &tmp_path,
        serde_json::to_vec_pretty(&AppiumSessionState {
            server_url: server_url.to_owned(),
            session_id: session_id.to_owned(),
        })?,
    )?;
    fs::rename(tmp_path, path)?;
    Ok(())
}

fn load_session() -> Option<AppiumSessionState> {
    fs::read(session_path().ok()?)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
}

fn remove_persisted_session() -> anyhow::Result<()> {
    let path = session_path()?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn run_appium_command<const N: usize>(args: [&str; N]) -> AppiumCommandOutput {
    match Command::new("appium").args(args).output() {
        Ok(output) => AppiumCommandOutput {
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        },
        Err(error) => AppiumCommandOutput {
            exit_code: None,
            stdout: String::new(),
            stderr: error.to_string(),
        },
    }
}

fn command_available(program: &str) -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&path).any(|dir| dir.join(program).is_file())
}
