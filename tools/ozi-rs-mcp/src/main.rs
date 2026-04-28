struct StartupSanity {
    package_name: &'static str,
    binary_name: &'static str,
    stdio_safe_by_default: bool,
}

fn startup_sanity() -> StartupSanity {
    StartupSanity {
        package_name: env!("CARGO_PKG_NAME"),
        binary_name: "ozi-rs-mcp",
        stdio_safe_by_default: true,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sanity = startup_sanity();
    debug_assert_eq!(sanity.package_name, "ozi-rs-mcp");
    debug_assert_eq!(sanity.binary_name, "ozi-rs-mcp");
    debug_assert!(sanity.stdio_safe_by_default);

    if std::env::args().any(|arg| arg == "--self-check") {
        serde_json::to_writer(std::io::stdout(), &ozi_rs_mcp::types::self_check())?;
        return Ok(());
    }

    ozi_rs_mcp::server::run_stdio_server().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use ozi_rs_mcp::server::tool_inventory;
    use ozi_rs_mcp::types::self_check;

    const EXPECTED_TOOLS: [&str; 13] = [
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

    #[test]
    fn startup_sanity_identifies_stdio_safe_binary() {
        let sanity = startup_sanity();

        assert_eq!(sanity.package_name, "ozi-rs-mcp");
        assert_eq!(sanity.binary_name, "ozi-rs-mcp");
        assert!(sanity.stdio_safe_by_default);
    }

    #[test]
    fn tool_inventory_has_exact_required_names_in_order() {
        let names: Vec<_> = tool_inventory().iter().map(|tool| tool.name).collect();

        assert_eq!(names, EXPECTED_TOOLS);
    }

    #[test]
    fn tool_inventory_self_check_reports_stdio_safe_exact_count() {
        let report = self_check();
        let names: Vec<_> = report.tools.iter().map(|tool| tool.name).collect();

        assert_eq!(report.server_name, "ozi-rs-mcp");
        assert!(report.stdio_safe);
        assert_eq!(report.tool_count, 13);
        assert_eq!(names, EXPECTED_TOOLS);
    }
}
