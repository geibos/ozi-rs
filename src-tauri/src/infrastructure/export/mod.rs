pub mod gpx;
pub mod plt;

pub use gpx::export_layer_to_gpx_file;

pub fn export_plt(
    track: &crate::domain::Track,
    color: u32,
    width: f64,
    writer: &mut impl std::io::Write,
) -> Result<(), ExportError> {
    plt::export_plt(track, color, width, writer)
}

#[derive(Debug)]
pub enum ExportError {
    Io(std::io::Error),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "export write failed: {err}"),
        }
    }
}

impl std::error::Error for ExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for ExportError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
