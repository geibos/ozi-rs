//! FTP accounts, as the settings screen sees them.
//!
//! The model, the file and the credential store are in
//! `infrastructure::ftp`; these are the four doors the interface uses. A
//! password crosses the boundary in one direction only — into a save — and
//! no command answers with one.

use crate::infrastructure::ftp::{
    FtpAccount, FtpAccountDraft, FtpAccounts, FtpCheck, FtpRole, check_account,
};
use std::sync::Arc;
use std::time::Duration;
use tauri::State;

pub type SharedFtpAccounts = Arc<FtpAccounts>;

/// How long each step of a check may wait on the server.
const CHECK_LIMIT: Duration = Duration::from_secs(10);

#[derive(serde::Serialize, specta::Type)]
pub struct FtpAccountDto {
    pub id: String,
    pub role: FtpRole,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub login: String,
    pub folder: String,
    /// Whether the credential store holds a password for this account. The
    /// password itself never leaves the backend.
    pub has_password: bool,
}

impl FtpAccountDto {
    fn new(account: FtpAccount, has_password: bool) -> Self {
        Self {
            id: account.id,
            role: account.role,
            name: account.name,
            host: account.host,
            port: account.port,
            login: account.login,
            folder: account.folder,
            has_password,
        }
    }
}

/// An account as the form sends it. `id: null` creates one.
#[derive(serde::Deserialize, specta::Type)]
pub struct FtpAccountInput {
    pub id: Option<String>,
    pub role: FtpRole,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub login: String,
    pub folder: String,
}

/// Which step of a check failed, if one did.
#[derive(serde::Serialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum FtpCheckOutcome {
    Ok,
    Unreachable,
    LoginRefused,
    NoFolder,
    NoPassword,
    Failed,
}

#[derive(serde::Serialize, specta::Type)]
pub struct FtpCheckDto {
    pub outcome: FtpCheckOutcome,
    /// The server's own reply, or the system's reason for not reaching it.
    pub detail: Option<String>,
    /// The folder the server says the account is in, after a success.
    pub folder: Option<String>,
}

#[tauri::command]
#[specta::specta]
pub fn list_ftp_accounts(accounts: State<SharedFtpAccounts>) -> Result<Vec<FtpAccountDto>, String> {
    let listed = accounts.list().map_err(|error| error.to_string())?;
    Ok(listed
        .into_iter()
        .map(|(account, has_password)| FtpAccountDto::new(account, has_password))
        .collect())
}

/// Create or update an account. `password: null` keeps the stored one.
#[tauri::command]
#[specta::specta]
pub fn save_ftp_account(
    account: FtpAccountInput,
    password: Option<String>,
    accounts: State<SharedFtpAccounts>,
) -> Result<FtpAccountDto, String> {
    let draft = FtpAccountDraft {
        id: account.id,
        role: account.role,
        name: account.name,
        host: account.host,
        port: account.port,
        login: account.login,
        folder: account.folder,
    };
    let password = password.filter(|p| !p.is_empty());
    let saved = accounts
        .save(draft, password.as_deref())
        .map_err(|error| error.to_string())?;
    let has_password = accounts
        .list()
        .map_err(|error| error.to_string())?
        .into_iter()
        .any(|(account, has)| account.id == saved.id && has);
    Ok(FtpAccountDto::new(saved, has_password))
}

#[tauri::command]
#[specta::specta]
pub fn delete_ftp_account(
    id: String,
    accounts: State<'_, SharedFtpAccounts>,
) -> Result<(), String> {
    accounts.delete(&id).map_err(|error| error.to_string())
}

/// Connect, log in and change to the account's folder.
///
/// cancel-safe: yes — the only `.await` is on a `spawn_blocking` join handle;
/// dropping it leaves the blocking check to finish on its own thread, which
/// writes nothing and closes its connection when it returns.
#[tauri::command]
#[specta::specta]
pub async fn check_ftp_account(
    id: String,
    accounts: State<'_, SharedFtpAccounts>,
) -> Result<FtpCheckDto, String> {
    let (account, password) = accounts
        .credentials(&id)
        .map_err(|error| error.to_string())?;
    let Some(password) = password else {
        return Ok(FtpCheckDto {
            outcome: FtpCheckOutcome::NoPassword,
            detail: None,
            folder: None,
        });
    };
    let checked = tauri::async_runtime::spawn_blocking(move || {
        check_account(&account, &password, CHECK_LIMIT)
    })
    .await
    .map_err(|error| error.to_string())?;
    Ok(match checked {
        FtpCheck::Ok { folder, greeting } => FtpCheckDto {
            outcome: FtpCheckOutcome::Ok,
            detail: greeting,
            folder: Some(folder),
        },
        FtpCheck::Unreachable(detail) => failure(FtpCheckOutcome::Unreachable, detail),
        FtpCheck::LoginRefused(detail) => failure(FtpCheckOutcome::LoginRefused, detail),
        FtpCheck::NoFolder(detail) => failure(FtpCheckOutcome::NoFolder, detail),
        FtpCheck::Failed(detail) => failure(FtpCheckOutcome::Failed, detail),
    })
}

fn failure(outcome: FtpCheckOutcome, detail: String) -> FtpCheckDto {
    FtpCheckDto {
        outcome,
        detail: Some(detail),
        folder: None,
    }
}
