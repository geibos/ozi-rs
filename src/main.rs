use eframe::NativeOptions;
use ozi_rs::{app_name, ui::OziApp};

fn main() -> eframe::Result<()> {
    let options = NativeOptions::default();

    eframe::run_native(
        app_name(),
        options,
        Box::new(|cc| Ok(Box::new(OziApp::new(cc)))),
    )
}
