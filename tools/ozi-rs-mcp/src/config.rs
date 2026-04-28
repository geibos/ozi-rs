use std::{env, path::Path};

pub const PROJECT_ROOT_ENV: &str = "OZI_RS_PROJECT_ROOT";

pub fn repo_root() -> anyhow::Result<std::path::PathBuf> {
    let env_root = env::var_os(PROJECT_ROOT_ENV).map(std::path::PathBuf::from);
    find_repo_root_from(env::current_dir()?, env_root.as_deref())
}

pub fn find_repo_root_from(
    start: impl AsRef<Path>,
    env_root: Option<&Path>,
) -> anyhow::Result<std::path::PathBuf> {
    if let Some(root) = env_root {
        if is_repo_root(root) {
            return Ok(root.to_path_buf());
        }
        anyhow::bail!(
            "{} does not point to an ozi-rs repo root: {}",
            PROJECT_ROOT_ENV,
            root.display()
        );
    }

    for candidate in start.as_ref().ancestors() {
        if is_repo_root(candidate) {
            return Ok(candidate.to_path_buf());
        }
    }

    anyhow::bail!(
        "could not find ozi-rs repo root from {}; expected ancestor containing justfile and src-tauri/tauri.conf.json",
        start.as_ref().display()
    )
}

fn is_repo_root(path: &Path) -> bool {
    path.join("justfile").is_file() && path.join("src-tauri/tauri.conf.json").is_file()
}
