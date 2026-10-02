//! FTP accounts: where bundles come from and where results go.
//!
//! LizaAlert keeps both on FTP, under different accounts: bundles come from
//! one place under one login, results go back to several "contours", each
//! with a login of its own (owner, 2026-10-01). This module holds the
//! accounts and checks them; transfers stand on it and are not here yet.
//!
//! A password never touches a file. The accounts file in the application's
//! data folder holds the host, port, login, folder, name and role; the
//! password goes to the operating system's credential store under the
//! account's id (owner, 2026-09-21: "the password belongs in the keychain").
//!
//! Crate versions this is written against: `keyring` 4.2 (`v1` feature, the
//! `Entry` wrapper over `keyring-core` 1.0), `suppaftp` 12.1 (sync client, no
//! TLS features).

use crate::infrastructure::persistence::write_atomic;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;
use suppaftp::{FtpError, FtpStream, Status};

/// What an account is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum FtpRole {
    /// Downloading bundles. At most one account has this role.
    Bundles,
    /// Uploading results. Any number, one per contour.
    Results,
}

/// An account as the accounts file holds it — everything but the password.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FtpAccount {
    pub id: String,
    pub role: FtpRole,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub login: String,
    pub folder: String,
}

/// What the operator typed, before it is checked. `id: None` is a new account.
#[derive(Debug, Clone)]
pub struct FtpAccountDraft {
    pub id: Option<String>,
    pub role: FtpRole,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub login: String,
    pub folder: String,
}

/// Why an account could not be saved, read or deleted.
///
/// `Display` writes the key the frontend translates, then the detail after
/// `": "` where there is one — the same shape the `busy:` refusals use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FtpAccountError {
    MissingName,
    MissingHost,
    /// A path, a port, a space or a scheme other than `ftp://` in the host.
    HostNotAName,
    MissingLogin,
    BadPort,
    SecondBundleAccount,
    UnknownAccount,
    /// The accounts file exists and cannot be read. Saving is refused until
    /// it is repaired or removed, so that it is not replaced by an empty list.
    Unreadable(String),
    /// The credential store refused.
    Store(String),
    /// The accounts file could not be written.
    Write(String),
}

impl FtpAccountError {
    pub fn key(&self) -> &'static str {
        match self {
            Self::MissingName => "ftp.error.missingName",
            Self::MissingHost => "ftp.error.missingHost",
            Self::HostNotAName => "ftp.error.hostNotAName",
            Self::MissingLogin => "ftp.error.missingLogin",
            Self::BadPort => "ftp.error.badPort",
            Self::SecondBundleAccount => "ftp.error.secondBundleAccount",
            Self::UnknownAccount => "ftp.error.unknownAccount",
            Self::Unreadable(_) => "ftp.error.unreadable",
            Self::Store(_) => "ftp.error.store",
            Self::Write(_) => "ftp.error.write",
        }
    }
}

impl fmt::Display for FtpAccountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable(detail) | Self::Store(detail) | Self::Write(detail) => {
                write!(f, "{}: {detail}", self.key())
            }
            _ => f.write_str(self.key()),
        }
    }
}

impl std::error::Error for FtpAccountError {}

/// Where FTP passwords live: the operating system's credential store in the
/// application, memory in tests. Keyed by account id.
pub trait SecretStore: Send + Sync {
    fn set(&self, id: &str, password: &str) -> Result<(), String>;
    /// `Ok(None)` when there is no password for this id.
    fn get(&self, id: &str) -> Result<Option<String>, String>;
    /// Deleting a password that is not there is not an error.
    fn delete(&self, id: &str) -> Result<(), String>;
}

/// The operating system's credential store: Keychain on macOS, Credential
/// Manager on Windows, Secret Service elsewhere.
pub struct SystemSecrets;

/// The service name the passwords are filed under in the credential store.
/// Keychain Access lists them under it.
const KEYRING_SERVICE: &str = "ozi-rs FTP";

impl SecretStore for SystemSecrets {
    fn set(&self, id: &str, password: &str) -> Result<(), String> {
        keyring::Entry::new(KEYRING_SERVICE, id)
            .and_then(|entry| entry.set_password(password))
            .map_err(|error| error.to_string())
    }

    fn get(&self, id: &str) -> Result<Option<String>, String> {
        match keyring::Entry::new(KEYRING_SERVICE, id).and_then(|entry| entry.get_password()) {
            Ok(password) => Ok(Some(password)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }

    fn delete(&self, id: &str) -> Result<(), String> {
        match keyring::Entry::new(KEYRING_SERVICE, id).and_then(|entry| entry.delete_credential()) {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }
}

/// The accounts file. Versioned like the project file, so a newer build's
/// file is refused rather than read in part and written back without the rest.
#[derive(Serialize, Deserialize)]
struct AccountsFile {
    version: u32,
    accounts: Vec<FtpAccount>,
}

const ACCOUNTS_FILE_VERSION: u32 = 1;

/// The FTP accounts, the file they live in and the store their passwords
/// live in.
///
/// One lock, held across the whole of a save or a delete — the credential
/// store write and the file write included — so two saves cannot interleave
/// a read of the list with each other's write of it. Nothing else is locked
/// while it is held.
pub struct FtpAccounts {
    path: PathBuf,
    secrets: Box<dyn SecretStore>,
    /// The accounts as last read or written, or why the file could not be read.
    state: Mutex<Result<Vec<FtpAccount>, FtpAccountError>>,
}

impl FtpAccounts {
    pub fn open(path: PathBuf, secrets: Box<dyn SecretStore>) -> Self {
        let state = read_accounts(&path);
        Self {
            path,
            secrets,
            state: Mutex::new(state),
        }
    }

    /// Every account, each with whether a password is stored for it.
    pub fn list(&self) -> Result<Vec<(FtpAccount, bool)>, FtpAccountError> {
        let accounts = self.lock().clone()?;
        accounts
            .into_iter()
            .map(|account| {
                let has_password = self
                    .secrets
                    .get(&account.id)
                    .map_err(FtpAccountError::Store)?
                    .is_some();
                Ok((account, has_password))
            })
            .collect()
    }

    /// Create or update an account. `password: None` leaves a stored password
    /// as it is, which is what an untouched password field means.
    pub fn save(
        &self,
        draft: FtpAccountDraft,
        password: Option<&str>,
    ) -> Result<FtpAccount, FtpAccountError> {
        let mut state = self.lock();
        let current = state.as_ref().map_err(Clone::clone)?;
        let is_new = draft.id.is_none();
        let account = validate(draft, current)?;

        let mut next = current.clone();
        match next.iter_mut().find(|existing| existing.id == account.id) {
            Some(existing) => *existing = account.clone(),
            None => next.push(account.clone()),
        }

        // The password first: a password that cannot be stored must not leave
        // behind an account that looks complete and cannot log in.
        if let Some(password) = password {
            self.secrets
                .set(&account.id, password)
                .map_err(FtpAccountError::Store)?;
        }
        if let Err(error) = write_accounts(&self.path, &next) {
            if is_new && password.is_some() {
                // Best effort: the account was never written, so its password
                // would be filed under an id nothing refers to.
                let _ = self.secrets.delete(&account.id);
            }
            return Err(error);
        }
        *state = Ok(next);
        Ok(account)
    }

    /// Remove an account and its password.
    pub fn delete(&self, id: &str) -> Result<(), FtpAccountError> {
        let mut state = self.lock();
        let current = state.as_ref().map_err(Clone::clone)?;
        if !current.iter().any(|account| account.id == id) {
            return Err(FtpAccountError::UnknownAccount);
        }
        let next: Vec<FtpAccount> = current
            .iter()
            .filter(|account| account.id != id)
            .cloned()
            .collect();
        write_accounts(&self.path, &next)?;
        *state = Ok(next);
        // After the file: an account whose password is gone but which is
        // still listed would fail its check with no explanation, while a
        // password left behind by a failed delete is only clutter.
        self.secrets.delete(id).map_err(FtpAccountError::Store)
    }

    /// An account and its stored password, for a check or a transfer.
    pub fn credentials(&self, id: &str) -> Result<(FtpAccount, Option<String>), FtpAccountError> {
        let account = self
            .lock()
            .as_ref()
            .map_err(Clone::clone)?
            .iter()
            .find(|account| account.id == id)
            .cloned()
            .ok_or(FtpAccountError::UnknownAccount)?;
        let password = self
            .secrets
            .get(&account.id)
            .map_err(FtpAccountError::Store)?;
        Ok((account, password))
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Result<Vec<FtpAccount>, FtpAccountError>> {
        // A panic while saving leaves the last state written or the one before
        // it, both of which are whole lists; neither is worth refusing over.
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

fn read_accounts(path: &Path) -> Result<Vec<FtpAccount>, FtpAccountError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(FtpAccountError::Unreadable(format!(
                "{}: {error}",
                path.display()
            )));
        }
    };
    let file: AccountsFile = serde_json::from_str(&text)
        .map_err(|error| FtpAccountError::Unreadable(format!("{}: {error}", path.display())))?;
    if file.version > ACCOUNTS_FILE_VERSION {
        return Err(FtpAccountError::Unreadable(format!(
            "{}: written by a newer build (format {}, this build reads {ACCOUNTS_FILE_VERSION})",
            path.display(),
            file.version
        )));
    }
    Ok(file.accounts)
}

fn write_accounts(path: &Path, accounts: &[FtpAccount]) -> Result<(), FtpAccountError> {
    let file = AccountsFile {
        version: ACCOUNTS_FILE_VERSION,
        accounts: accounts.to_vec(),
    };
    let text = serde_json::to_string_pretty(&file)
        .map_err(|error| FtpAccountError::Write(error.to_string()))?;
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .map_err(|error| FtpAccountError::Write(format!("{}: {error}", parent.display())))?;
    }
    write_atomic(path, &text)
        .map_err(|error| FtpAccountError::Write(format!("{}: {error}", path.display())))
}

/// Check a draft against the accounts already there and give it an id.
fn validate(
    draft: FtpAccountDraft,
    existing: &[FtpAccount],
) -> Result<FtpAccount, FtpAccountError> {
    let name = draft.name.trim().to_owned();
    if name.is_empty() {
        return Err(FtpAccountError::MissingName);
    }
    let host = normalise_host(&draft.host)?;
    let login = draft.login.trim().to_owned();
    if login.is_empty() {
        return Err(FtpAccountError::MissingLogin);
    }
    if draft.port == 0 {
        return Err(FtpAccountError::BadPort);
    }
    if let Some(id) = &draft.id
        && !existing.iter().any(|account| &account.id == id)
    {
        return Err(FtpAccountError::UnknownAccount);
    }
    let id = draft.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    if draft.role == FtpRole::Bundles
        && existing
            .iter()
            .any(|account| account.role == FtpRole::Bundles && account.id != id)
    {
        return Err(FtpAccountError::SecondBundleAccount);
    }
    let folder = match draft.folder.trim() {
        "" => "/".to_owned(),
        folder => folder.to_owned(),
    };
    Ok(FtpAccount {
        id,
        role: draft.role,
        name,
        host,
        port: draft.port,
        login,
        folder,
    })
}

/// The server's name, from what the operator typed.
///
/// A pasted `ftp://maps.example.org/` is a courtesy worth extending; a path or
/// a port in the host is not guessed at, because the port and the folder have
/// fields of their own and silently moving text between fields is how an
/// account ends up pointing somewhere nobody chose.
fn normalise_host(raw: &str) -> Result<String, FtpAccountError> {
    let trimmed = raw.trim();
    let without_scheme = match trimmed.get(..6) {
        Some(scheme) if scheme.eq_ignore_ascii_case("ftp://") => &trimmed[6..],
        _ => trimmed,
    };
    let host = without_scheme.strip_suffix('/').unwrap_or(without_scheme);
    if host.is_empty() {
        return Err(FtpAccountError::MissingHost);
    }
    if host.contains(['/', ':', '@']) || host.chars().any(char::is_whitespace) {
        return Err(FtpAccountError::HostNotAName);
    }
    Ok(host.to_owned())
}

/// What a check found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FtpCheck {
    /// Logged in and in the folder; `folder` is what the server says it is.
    Ok {
        folder: String,
        greeting: Option<String>,
    },
    /// No connection, no greeting, or a name that does not resolve.
    Unreachable(String),
    /// The server refused the login or the password.
    LoginRefused(String),
    /// Logged in, and the folder is not there.
    NoFolder(String),
    /// Anything else the server said that was not what the step expected.
    Failed(String),
}

/// Connect, log in, change to the account's folder.
///
/// Blocking — every step waits on the network, each for at most `limit` —
/// so it belongs on a blocking thread (`spawn_blocking`), never on an async
/// worker. Only the control connection is used: a listing would need a data
/// connection, and a check that fails on a firewall's passive-mode rules
/// would blame the account for the network.
pub fn check_account(account: &FtpAccount, password: &str, limit: Duration) -> FtpCheck {
    let addresses: Vec<_> = match (account.host.as_str(), account.port).to_socket_addrs() {
        Ok(addresses) => addresses.collect(),
        Err(error) => return FtpCheck::Unreachable(error.to_string()),
    };
    let mut last_error = None;
    let mut connected = None;
    for address in &addresses {
        match TcpStream::connect_timeout(address, limit) {
            Ok(stream) => {
                connected = Some(stream);
                break;
            }
            Err(error) => last_error = Some(error.to_string()),
        }
    }
    let Some(stream) = connected else {
        return FtpCheck::Unreachable(last_error.unwrap_or_else(|| "no address".to_owned()));
    };
    // Without these a server that accepts the connection and never answers
    // holds the check for as long as the operating system lets it.
    if let Err(error) = stream
        .set_read_timeout(Some(limit))
        .and_then(|()| stream.set_write_timeout(Some(limit)))
    {
        return FtpCheck::Unreachable(error.to_string());
    }

    let mut ftp = match FtpStream::connect_with_stream(stream) {
        Ok(ftp) => ftp,
        Err(error) => return FtpCheck::Unreachable(describe(&error)),
    };
    let greeting = ftp.get_welcome_msg().map(|text| text.trim().to_owned());

    if let Err(error) = ftp.login(account.login.as_str(), password) {
        return match error {
            FtpError::UnexpectedResponse(response) if response.status == Status::NotLoggedIn => {
                FtpCheck::LoginRefused(reply_text(&response))
            }
            other => FtpCheck::Failed(describe(&other)),
        };
    }
    if let Err(error) = ftp.cwd(&account.folder) {
        let _ = ftp.quit();
        return match error {
            FtpError::UnexpectedResponse(response) => FtpCheck::NoFolder(reply_text(&response)),
            other => FtpCheck::Failed(describe(&other)),
        };
    }
    let folder = ftp.pwd().unwrap_or_else(|_| account.folder.clone());
    // The answer is in. A server that drops the line on QUIT has nothing more
    // to tell us about the account.
    let _ = ftp.quit();
    FtpCheck::Ok { folder, greeting }
}

fn reply_text(response: &suppaftp::types::Response) -> String {
    match response.as_string() {
        Ok(text) => text.trim().to_owned(),
        Err(_) => format!("{}", response.status.code()),
    }
}

fn describe(error: &FtpError) -> String {
    match error {
        FtpError::UnexpectedResponse(response) => reply_text(response),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::time::Instant;

    #[derive(Default)]
    struct MemorySecrets(Mutex<HashMap<String, String>>);

    impl SecretStore for MemorySecrets {
        fn set(&self, id: &str, password: &str) -> Result<(), String> {
            self.0
                .lock()
                .unwrap()
                .insert(id.to_owned(), password.to_owned());
            Ok(())
        }
        fn get(&self, id: &str) -> Result<Option<String>, String> {
            Ok(self.0.lock().unwrap().get(id).cloned())
        }
        fn delete(&self, id: &str) -> Result<(), String> {
            self.0.lock().unwrap().remove(id);
            Ok(())
        }
    }

    /// A store shared between two `FtpAccounts`, as the keychain is between
    /// two launches.
    #[derive(Clone, Default)]
    struct SharedSecrets(std::sync::Arc<MemorySecrets>);

    impl SecretStore for SharedSecrets {
        fn set(&self, id: &str, password: &str) -> Result<(), String> {
            self.0.set(id, password)
        }
        fn get(&self, id: &str) -> Result<Option<String>, String> {
            self.0.get(id)
        }
        fn delete(&self, id: &str) -> Result<(), String> {
            self.0.delete(id)
        }
    }

    fn draft(role: FtpRole, name: &str, host: &str, login: &str) -> FtpAccountDraft {
        FtpAccountDraft {
            id: None,
            role,
            name: name.to_owned(),
            host: host.to_owned(),
            port: 21,
            login: login.to_owned(),
            folder: String::new(),
        }
    }

    fn accounts_in(dir: &tempfile::TempDir) -> (FtpAccounts, SharedSecrets) {
        let secrets = SharedSecrets::default();
        let accounts = FtpAccounts::open(
            dir.path().join("ftp-accounts.json"),
            Box::new(secrets.clone()),
        );
        (accounts, secrets)
    }

    #[test]
    fn a_saved_password_is_in_the_store_and_not_in_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let (accounts, secrets) = accounts_in(&dir);
        let saved = accounts
            .save(
                draft(FtpRole::Results, "Контур 1", "results.example.org", "crew1"),
                Some("s3cret-пароль"),
            )
            .unwrap();

        let on_disk = std::fs::read_to_string(&accounts.path).unwrap();
        assert!(on_disk.contains("results.example.org"));
        assert!(on_disk.contains("crew1"));
        assert!(!on_disk.contains("s3cret"), "the password reached the file");
        assert_eq!(
            secrets.get(&saved.id).unwrap().as_deref(),
            Some("s3cret-пароль")
        );
        assert_eq!(accounts.list().unwrap(), vec![(saved, true)]);
    }

    #[test]
    fn four_result_endpoints_each_keep_their_own_login() {
        let dir = tempfile::tempdir().unwrap();
        let (accounts, _) = accounts_in(&dir);
        for n in 1..=4 {
            accounts
                .save(
                    draft(
                        FtpRole::Results,
                        &format!("Контур {n}"),
                        &format!("c{n}.example.org"),
                        &format!("crew{n}"),
                    ),
                    Some(&format!("p{n}")),
                )
                .unwrap();
        }
        let listed = accounts.list().unwrap();
        assert_eq!(listed.len(), 4);
        assert!(
            listed
                .iter()
                .all(|(a, has)| a.role == FtpRole::Results && *has)
        );
        let logins: Vec<_> = listed.iter().map(|(a, _)| a.login.as_str()).collect();
        assert_eq!(logins, ["crew1", "crew2", "crew3", "crew4"]);
    }

    #[test]
    fn a_second_bundle_account_is_refused_and_the_first_kept() {
        let dir = tempfile::tempdir().unwrap();
        let (accounts, _) = accounts_in(&dir);
        let first = accounts
            .save(
                draft(FtpRole::Bundles, "Комплекты", "maps.example.org", "maps"),
                None,
            )
            .unwrap();
        let second = accounts.save(
            draft(FtpRole::Bundles, "Ещё", "other.example.org", "x"),
            None,
        );
        assert_eq!(second, Err(FtpAccountError::SecondBundleAccount));
        assert_eq!(accounts.list().unwrap(), vec![(first.clone(), false)]);

        // Editing the one bundle account is not a second one.
        let edited = accounts
            .save(
                FtpAccountDraft {
                    id: Some(first.id.clone()),
                    ..draft(FtpRole::Bundles, "Комплекты", "maps2.example.org", "maps")
                },
                None,
            )
            .unwrap();
        assert_eq!(edited.host, "maps2.example.org");
    }

    #[test]
    fn an_edit_without_a_password_keeps_the_stored_one() {
        let dir = tempfile::tempdir().unwrap();
        let (accounts, secrets) = accounts_in(&dir);
        let saved = accounts
            .save(
                draft(FtpRole::Results, "К", "a.example.org", "u"),
                Some("first"),
            )
            .unwrap();
        accounts
            .save(
                FtpAccountDraft {
                    id: Some(saved.id.clone()),
                    ..draft(FtpRole::Results, "К", "b.example.org", "u")
                },
                None,
            )
            .unwrap();
        assert_eq!(secrets.get(&saved.id).unwrap().as_deref(), Some("first"));
        accounts
            .save(
                FtpAccountDraft {
                    id: Some(saved.id.clone()),
                    ..draft(FtpRole::Results, "К", "b.example.org", "u")
                },
                Some("second"),
            )
            .unwrap();
        assert_eq!(secrets.get(&saved.id).unwrap().as_deref(), Some("second"));
    }

    #[test]
    fn deleting_an_account_deletes_its_password() {
        let dir = tempfile::tempdir().unwrap();
        let (accounts, secrets) = accounts_in(&dir);
        let saved = accounts
            .save(
                draft(FtpRole::Results, "К", "a.example.org", "u"),
                Some("pw"),
            )
            .unwrap();
        accounts.delete(&saved.id).unwrap();
        assert!(accounts.list().unwrap().is_empty());
        assert_eq!(secrets.get(&saved.id).unwrap(), None);
        assert_eq!(
            accounts.delete(&saved.id),
            Err(FtpAccountError::UnknownAccount)
        );
    }

    #[test]
    fn the_accounts_survive_a_restart() {
        let dir = tempfile::tempdir().unwrap();
        let (accounts, secrets) = accounts_in(&dir);
        let saved = accounts
            .save(
                draft(FtpRole::Bundles, "Комплекты", "maps.example.org", "maps"),
                Some("pw"),
            )
            .unwrap();
        drop(accounts);
        let reopened = FtpAccounts::open(dir.path().join("ftp-accounts.json"), Box::new(secrets));
        assert_eq!(reopened.list().unwrap(), vec![(saved, true)]);
    }

    #[test]
    fn a_field_that_cannot_work_is_named() {
        let dir = tempfile::tempdir().unwrap();
        let (accounts, _) = accounts_in(&dir);
        let cases = [
            (
                draft(FtpRole::Results, " ", "a.example.org", "u"),
                FtpAccountError::MissingName,
            ),
            (
                draft(FtpRole::Results, "К", "  ", "u"),
                FtpAccountError::MissingHost,
            ),
            (
                draft(FtpRole::Results, "К", "a.example.org", ""),
                FtpAccountError::MissingLogin,
            ),
            (
                draft(FtpRole::Results, "К", "a.example.org/results", "u"),
                FtpAccountError::HostNotAName,
            ),
            (
                draft(FtpRole::Results, "К", "a.example.org:2121", "u"),
                FtpAccountError::HostNotAName,
            ),
            (
                draft(FtpRole::Results, "К", "a example.org", "u"),
                FtpAccountError::HostNotAName,
            ),
            (
                FtpAccountDraft {
                    port: 0,
                    ..draft(FtpRole::Results, "К", "a.example.org", "u")
                },
                FtpAccountError::BadPort,
            ),
            (
                FtpAccountDraft {
                    id: Some("nobody".to_owned()),
                    ..draft(FtpRole::Results, "К", "a.example.org", "u")
                },
                FtpAccountError::UnknownAccount,
            ),
        ];
        for (input, expected) in cases {
            assert_eq!(accounts.save(input, Some("pw")), Err(expected));
        }
        assert!(accounts.list().unwrap().is_empty());
        assert!(!accounts.path.exists(), "a refused save wrote the file");
    }

    #[test]
    fn a_pasted_address_keeps_only_the_host_and_an_empty_folder_is_the_root() {
        let dir = tempfile::tempdir().unwrap();
        let (accounts, _) = accounts_in(&dir);
        let saved = accounts
            .save(
                draft(FtpRole::Bundles, "К", " FTP://maps.example.org/ ", " maps "),
                None,
            )
            .unwrap();
        assert_eq!(saved.host, "maps.example.org");
        assert_eq!(saved.login, "maps");
        assert_eq!(saved.folder, "/");
    }

    #[test]
    fn a_damaged_file_is_reported_and_not_written_over() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ftp-accounts.json");
        std::fs::write(&path, "{ not json").unwrap();
        let accounts = FtpAccounts::open(path.clone(), Box::new(MemorySecrets::default()));

        let listed = accounts.list();
        assert!(
            matches!(listed, Err(FtpAccountError::Unreadable(ref d)) if d.contains("ftp-accounts.json"))
        );
        let saved = accounts.save(draft(FtpRole::Results, "К", "a.example.org", "u"), None);
        assert!(matches!(saved, Err(FtpAccountError::Unreadable(_))));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{ not json");
    }

    #[test]
    fn a_file_from_a_newer_build_is_not_read_in_part() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ftp-accounts.json");
        std::fs::write(&path, r#"{"version": 99, "accounts": []}"#).unwrap();
        let accounts = FtpAccounts::open(path, Box::new(MemorySecrets::default()));
        assert!(matches!(
            accounts.list(),
            Err(FtpAccountError::Unreadable(_))
        ));
    }

    /// The real credential store, which a plain `cargo test` does not touch:
    /// `cargo test --lib system_credential_store -- --ignored`. On macOS this
    /// files, reads back and deletes a password under «ozi-rs FTP» in the
    /// login keychain — the path the application takes, from a test binary.
    #[test]
    #[ignore = "writes to the real credential store"]
    fn system_credential_store_round_trip() {
        let id = format!("test-{}", uuid::Uuid::new_v4());
        let secrets = SystemSecrets;
        secrets.set(&id, "пароль-проверка").unwrap();
        assert_eq!(
            secrets.get(&id).unwrap().as_deref(),
            Some("пароль-проверка")
        );
        secrets.delete(&id).unwrap();
        assert_eq!(secrets.get(&id).unwrap(), None);
        // Deleting what is not there is not an error.
        secrets.delete(&id).unwrap();
    }

    #[test]
    fn errors_carry_the_key_the_frontend_translates() {
        assert_eq!(
            FtpAccountError::MissingHost.to_string(),
            "ftp.error.missingHost"
        );
        assert_eq!(
            FtpAccountError::Store("denied".to_owned()).to_string(),
            "ftp.error.store: denied"
        );
    }

    // ── Checking an account, against a scripted server on localhost ──────

    /// Serve one connection with a minimal FTP dialogue: the password `right`
    /// logs in, `/results` exists, everything else is refused the way real
    /// servers refuse it.
    fn scripted_server() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            let Ok((stream, _)) = listener.accept() else {
                return;
            };
            let mut writer = stream.try_clone().unwrap();
            let mut reader = BufReader::new(stream);
            writer.write_all(b"220 DB Based FTP ready\r\n").unwrap();
            let mut cwd = "/".to_owned();
            let mut line = String::new();
            loop {
                line.clear();
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    return;
                }
                let (command, argument) = line
                    .trim_end()
                    .split_once(' ')
                    .map_or((line.trim_end(), ""), |(c, a)| (c, a));
                let reply = match command {
                    "USER" => "331 Password required\r\n".to_owned(),
                    "PASS" if argument == "right" => "230 Logged in\r\n".to_owned(),
                    "PASS" => "530 Authentication failed\r\n".to_owned(),
                    "CWD" if argument == "/results" => {
                        cwd = argument.to_owned();
                        "250 Directory changed\r\n".to_owned()
                    }
                    "CWD" => "550 No such directory\r\n".to_owned(),
                    "PWD" => format!("257 \"{cwd}\" is current directory\r\n"),
                    "QUIT" => {
                        let _ = writer.write_all(b"221 Bye\r\n");
                        return;
                    }
                    _ => "502 Not implemented\r\n".to_owned(),
                };
                if writer.write_all(reply.as_bytes()).is_err() {
                    return;
                }
            }
        });
        port
    }

    fn local(port: u16, folder: &str) -> FtpAccount {
        FtpAccount {
            id: "a".to_owned(),
            role: FtpRole::Results,
            name: "К".to_owned(),
            host: "127.0.0.1".to_owned(),
            port,
            login: "crew".to_owned(),
            folder: folder.to_owned(),
        }
    }

    const LIMIT: Duration = Duration::from_secs(5);

    #[test]
    fn a_right_account_logs_in_and_reaches_its_folder() {
        let port = scripted_server();
        assert_eq!(
            check_account(&local(port, "/results"), "right", LIMIT),
            FtpCheck::Ok {
                folder: "/results".to_owned(),
                greeting: Some("220 DB Based FTP ready".to_owned()),
            }
        );
    }

    #[test]
    fn a_wrong_password_is_a_refused_login_with_the_servers_words() {
        let port = scripted_server();
        assert_eq!(
            check_account(&local(port, "/results"), "wrong", LIMIT),
            FtpCheck::LoginRefused("530 Authentication failed".to_owned())
        );
    }

    #[test]
    fn a_folder_that_is_not_there_is_named_as_such() {
        let port = scripted_server();
        assert_eq!(
            check_account(&local(port, "/nowhere"), "right", LIMIT),
            FtpCheck::NoFolder("550 No such directory".to_owned())
        );
    }

    #[test]
    fn nothing_listening_is_unreachable() {
        let port = {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap().port()
        };
        assert!(matches!(
            check_account(&local(port, "/"), "right", LIMIT),
            FtpCheck::Unreachable(_)
        ));
    }

    #[test]
    fn a_server_that_never_greets_is_given_up_on_within_the_limit() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let silent = std::thread::spawn(move || {
            let accepted = listener.accept();
            std::thread::sleep(Duration::from_secs(3));
            drop(accepted);
        });
        let started = Instant::now();
        let outcome = check_account(&local(port, "/"), "right", Duration::from_millis(300));
        assert!(matches!(outcome, FtpCheck::Unreachable(_)), "{outcome:?}");
        assert!(started.elapsed() < Duration::from_secs(2));
        drop(silent);
    }
}
