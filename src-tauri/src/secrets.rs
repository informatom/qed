//! Passwords and private keys for publish targets, in the operating system's
//! keychain (Keychain on macOS, Credential Manager on Windows, the Secret
//! Service on Linux). Nothing secret is ever written to the site folder.

use keyring::Entry;
use qed_core::sftp::Credentials;
use qed_core::target::{SftpAuth, SftpTarget};

const SERVICE: &str = "qed";

/// Separate entries for a password and a key, so switching between them
/// doesn't lose the other.
fn entry(target: &SftpTarget, auth: SftpAuth) -> Result<Entry, String> {
    let kind = match auth {
        SftpAuth::Password => "password",
        SftpAuth::Key => "key",
    };
    Entry::new(SERVICE, &format!("{}#{kind}", target.keychain_account())).map_err(|e| e.to_string())
}

pub fn store(target: &SftpTarget, auth: SftpAuth, secret: &str) -> Result<(), String> {
    entry(target, auth)?
        .set_password(secret)
        .map_err(|e| format!("can't save to the keychain: {e}"))
}

/// The stored secret, or `None` if there is none yet.
pub fn load(target: &SftpTarget, auth: SftpAuth) -> Result<Option<String>, String> {
    match entry(target, auth)?.get_password() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(format!("can't read the keychain: {error}")),
    }
}

/// The credentials for logging in to `target`, as its `auth` says.
pub fn credentials(target: &SftpTarget) -> Result<Credentials, String> {
    let missing = || match target.auth {
        SftpAuth::Password => format!("no password saved for {}", target.name),
        SftpAuth::Key => format!("no key generated for {}", target.name),
    };
    let secret = load(target, target.auth)?.ok_or_else(missing)?;
    Ok(match target.auth {
        SftpAuth::Password => Credentials::Password(secret),
        SftpAuth::Key => Credentials::Key(secret),
    })
}

/// The private key q.e.d. made for a git remote, by
/// `qed_core::remote::key_account`.
fn remote_entry(account: &str) -> Result<Entry, String> {
    Entry::new(SERVICE, &format!("{account}#key")).map_err(|e| e.to_string())
}

pub fn store_remote_key(account: &str, private_pem: &str) -> Result<(), String> {
    remote_entry(account)?
        .set_password(private_pem)
        .map_err(|e| format!("can't save to the keychain: {e}"))
}

pub fn load_remote_key(account: &str) -> Result<Option<String>, String> {
    match remote_entry(account)?.get_password() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(format!("can't read the keychain: {error}")),
    }
}
