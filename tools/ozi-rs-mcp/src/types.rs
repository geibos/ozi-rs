use rmcp::schemars;
use serde::Serialize;

use crate::server::tool_inventory;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ToolMetadata {
    pub name: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SelfCheckReport {
    pub server_name: &'static str,
    pub stdio_safe: bool,
    pub tool_count: usize,
    pub tools: Vec<ToolMetadata>,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct StubToolResult {
    pub not_implemented: bool,
    pub tool: &'static str,
}

pub fn self_check() -> SelfCheckReport {
    let tools = tool_inventory();

    SelfCheckReport {
        server_name: "ozi-rs-mcp",
        stdio_safe: true,
        tool_count: tools.len(),
        tools,
    }
}

pub fn not_implemented(tool: &'static str) -> StubToolResult {
    StubToolResult {
        not_implemented: true,
        tool,
    }
}
