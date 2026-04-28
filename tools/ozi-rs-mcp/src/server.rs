use rmcp::{
    Json, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{Implementation, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
};

use crate::{appium, native, types::ToolMetadata};
use serde::Deserialize;

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AppiumClickParams {
    selector: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AppiumTypeTextParams {
    selector: Option<String>,
    text: Option<String>,
}

const REQUIRED_TOOL_NAMES: [&str; 13] = [
    "qa_environment",
    "build_app",
    "launch_app",
    "stop_app",
    "capture_logs",
    "capture_screenshot",
    "qa_observe",
    "appium_doctor",
    "appium_launch_session",
    "appium_click",
    "appium_type_text",
    "appium_screenshot",
    "appium_stop_session",
];

#[derive(Debug, Clone)]
pub struct OziRsMcpServer {
    tool_router: ToolRouter<Self>,
}

impl OziRsMcpServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router(router = tool_router)]
impl OziRsMcpServer {
    #[tool(description = "Report QA environment details")]
    fn qa_environment(&self) -> Json<crate::native::NativeToolResult> {
        Json(native::qa_environment())
    }

    #[tool(description = "Build the ozi-rs desktop app")]
    fn build_app(&self) -> Json<crate::native::NativeToolResult> {
        Json(native::build_app())
    }

    #[tool(description = "Launch the ozi-rs desktop app")]
    fn launch_app(&self) -> Json<crate::native::NativeToolResult> {
        Json(native::launch_app())
    }

    #[tool(description = "Stop the launched ozi-rs desktop app")]
    fn stop_app(&self) -> Json<crate::native::NativeToolResult> {
        Json(native::stop_app())
    }

    #[tool(description = "Capture ozi-rs application logs")]
    fn capture_logs(&self) -> Json<crate::native::NativeToolResult> {
        Json(native::capture_logs())
    }

    #[tool(description = "Capture an ozi-rs application screenshot")]
    fn capture_screenshot(&self) -> Json<crate::native::NativeToolResult> {
        Json(native::capture_screenshot())
    }

    #[tool(description = "Observe the current QA target state")]
    fn qa_observe(&self) -> Json<crate::native::NativeToolResult> {
        Json(native::qa_observe())
    }

    #[tool(description = "Run Appium environment diagnostics")]
    fn appium_doctor(&self) -> Json<crate::appium::AppiumToolResult> {
        Json(appium::appium_doctor())
    }

    #[tool(description = "Launch an Appium automation session")]
    fn appium_launch_session(&self) -> Json<crate::appium::AppiumToolResult> {
        Json(appium::appium_launch_session())
    }

    #[tool(description = "Click through an Appium automation session")]
    fn appium_click(
        &self,
        params: Parameters<AppiumClickParams>,
    ) -> Json<crate::appium::AppiumToolResult> {
        let Parameters(params) = params;

        Json(appium::appium_click_for_params(params.selector.as_deref()))
    }

    #[tool(description = "Type text through an Appium automation session")]
    fn appium_type_text(
        &self,
        params: Parameters<AppiumTypeTextParams>,
    ) -> Json<crate::appium::AppiumToolResult> {
        let Parameters(params) = params;

        Json(appium::appium_type_text_for_params(
            params.selector.as_deref(),
            params.text.as_deref().unwrap_or_default(),
        ))
    }

    #[tool(description = "Capture an Appium session screenshot")]
    fn appium_screenshot(&self) -> Json<crate::appium::AppiumToolResult> {
        Json(appium::appium_screenshot())
    }

    #[tool(description = "Stop the active Appium automation session")]
    fn appium_stop_session(&self) -> Json<crate::appium::AppiumToolResult> {
        Json(appium::appium_stop_session())
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for OziRsMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("ozi-rs-mcp", env!("CARGO_PKG_VERSION")))
    }
}

pub async fn run_stdio_server() -> anyhow::Result<()> {
    OziRsMcpServer::new()
        .serve(stdio())
        .await?
        .waiting()
        .await?;
    Ok(())
}

pub fn tool_inventory() -> Vec<ToolMetadata> {
    let router = OziRsMcpServer::tool_router();
    let registered = router.list_all();

    assert_eq!(
        registered.len(),
        REQUIRED_TOOL_NAMES.len(),
        "MCP tool registration count drifted"
    );

    REQUIRED_TOOL_NAMES
        .iter()
        .map(|name| {
            assert!(
                registered.iter().any(|tool| tool.name.as_ref() == *name),
                "MCP tool registration missing {name}"
            );
            ToolMetadata { name }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_input_schemas_are_objects_for_opencode_discovery() {
        let router = OziRsMcpServer::tool_router();

        for tool in router.list_all() {
            let schema = tool.input_schema.as_ref();
            assert_eq!(
                schema.get("type").and_then(|value| value.as_str()),
                Some("object"),
                "{} input schema must be a JSON object schema for OpenCode discovery: {schema:?}",
                tool.name
            );
        }
    }
}
