use std::{
    fs,
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use rmcp::schemars;
use serde::Serialize;

pub const EVIDENCE_ROOT: &str = ".sisyphus/evidence/native-qa";

#[derive(Debug, Clone, Copy, Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStatus {
    Passed,
    Failed,
    Error,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema, PartialEq, Eq)]
pub struct EvidenceMetadata {
    pub tool: String,
    pub started_at: String,
    pub duration_ms: u128,
    pub command: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout_path: String,
    pub stderr_path: String,
    pub artifact_paths: Vec<String>,
    pub status: EvidenceStatus,
    pub error_kind: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EvidencePaths {
    project_root: PathBuf,
    evidence_root: PathBuf,
}

impl EvidencePaths {
    pub fn new(project_root: impl AsRef<Path>) -> Self {
        let project_root = project_root.as_ref().to_path_buf();
        let evidence_root = project_root.join(EVIDENCE_ROOT);

        Self {
            project_root,
            evidence_root,
        }
    }

    pub fn evidence_root(&self) -> &Path {
        &self.evidence_root
    }

    pub fn path_for(&self, tool: &str, file_name: &str) -> anyhow::Result<PathBuf> {
        self.confined_path([tool, file_name])
    }

    pub fn root_file(&self, file_name: &str) -> anyhow::Result<PathBuf> {
        self.confined_path([file_name])
    }

    pub fn prepare_file(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let path = path.as_ref();
        if !path.starts_with(&self.evidence_root) {
            anyhow::bail!("evidence file escaped native QA evidence root");
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
            self.reject_symlink_components(parent)?;
        }

        if fs::symlink_metadata(path)
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false)
        {
            anyhow::bail!("evidence file is a symlink");
        }

        Ok(())
    }

    pub fn relative_display(&self, path: impl AsRef<Path>) -> anyhow::Result<String> {
        Ok(path
            .as_ref()
            .strip_prefix(&self.project_root)?
            .to_string_lossy()
            .to_string())
    }

    fn confined_path<const N: usize>(&self, parts: [&str; N]) -> anyhow::Result<PathBuf> {
        let mut path = self.evidence_root.clone();

        for part in parts {
            reject_unsafe_relative_path(part)?;
            path.push(part);
        }

        if !path.starts_with(&self.evidence_root) {
            anyhow::bail!("evidence path escaped native QA evidence root");
        }

        Ok(path)
    }

    fn reject_symlink_components(&self, path: &Path) -> anyhow::Result<()> {
        let relative = path.strip_prefix(&self.project_root)?;
        let mut current = self.project_root.clone();

        for component in relative.components() {
            current.push(component.as_os_str());
            if fs::symlink_metadata(&current)?.file_type().is_symlink() {
                anyhow::bail!(
                    "evidence path component is a symlink: {}",
                    current.display()
                );
            }
        }

        Ok(())
    }
}

pub fn started_at_now() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("unix_ms:{millis}")
}

fn reject_unsafe_relative_path(value: &str) -> anyhow::Result<()> {
    let path = Path::new(value);

    if value.is_empty() || path.is_absolute() {
        anyhow::bail!("evidence path component must be non-empty and relative");
    }

    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir
            | Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => {
                anyhow::bail!("evidence path component contains traversal");
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_absolute_evidence_components() {
        assert!(reject_unsafe_relative_path("/tmp/outside").is_err());
    }
}
