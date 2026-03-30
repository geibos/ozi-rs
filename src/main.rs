use eframe::NativeOptions;
use ozi_rs::{app_name, ui::OziApp};

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let options = NativeOptions::default();

    eframe::run_native(
        app_name(),
        options,
        Box::new(|cc| Ok(Box::new(OziApp::new(cc)))),
    )
}
