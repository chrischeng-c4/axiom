// `mamba auth` — plaintext credential store for package indexes.

use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read as _;
use std::path::{Path, PathBuf};

use crate::pkgmanage::pkgmgr::auth_header::basic_auth;

const CREDENTIALS_DIR_ENV: &str = "MAMBA_CREDENTIALS_DIR";
const TOKEN_USERNAME: &str = "__token__";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredCredential {
    service: String,
    username: String,
    token: String,
}

pub fn cmd_auth(sub: &ArgMatches) -> Result<()> {
    match sub.subcommand() {
        Some(("login", cmd)) => cmd_login(cmd),
        Some(("logout", cmd)) => cmd_logout(cmd),
        Some(("token", cmd)) => cmd_token(cmd),
        Some(("dir", cmd)) => cmd_dir(cmd),
        Some((other, _)) => bail!("unknown auth subcommand `{other}`"),
        None => bail!("`mamba auth` requires a subcommand: login | logout | token | dir"),
    }
}

fn cmd_login(sub: &ArgMatches) -> Result<()> {
    let service = service_arg(sub)?;
    let username = sub
        .get_one::<String>("username")
        .cloned()
        .unwrap_or_else(|| TOKEN_USERNAME.to_string());
    let token = sub
        .get_one::<String>("token")
        .context("missing --token TOKEN; use --token - to read from stdin")
        .and_then(|raw| read_token(raw))?;

    let dir = credentials_dir()?;
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = fs::metadata(&dir)
            .with_context(|| format!("stat {}", dir.display()))?
            .permissions();
        perm.set_mode(0o700);
        fs::set_permissions(&dir, perm).with_context(|| format!("chmod {}", dir.display()))?;
    }

    let normalized = normalize_service(service);
    let path = credential_path_in(&dir, &normalized, &username);
    let body = toml::to_string_pretty(&StoredCredential {
        service: normalized.clone(),
        username: username.clone(),
        token,
    })?;
    fs::write(&path, body).with_context(|| format!("write {}", path.display()))?;
    eprintln!("stored credentials for {normalized} as {username}");
    Ok(())
}

fn cmd_logout(sub: &ArgMatches) -> Result<()> {
    let service = normalize_service(service_arg(sub)?);
    let username = username_or_token(sub);
    let path = credential_path(&service, &username)?;
    if !path.exists() {
        eprintln!("no_op: no credentials for {service} as {username}");
        return Ok(());
    }
    fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?;
    Ok(())
}

fn cmd_token(sub: &ArgMatches) -> Result<()> {
    let service = normalize_service(service_arg(sub)?);
    let username = username_or_token(sub);
    let path = credential_path(&service, &username)?;
    let cred = read_credential(&path)?;
    println!("{}", cred.token);
    Ok(())
}

fn cmd_dir(sub: &ArgMatches) -> Result<()> {
    let dir = credentials_dir()?;
    if let Some(service) = sub.get_one::<String>("service") {
        let normalized = normalize_service(service);
        let username = username_or_token(sub);
        println!(
            "{}",
            credential_path_in(&dir, &normalized, &username).display()
        );
    } else {
        println!("{}", dir.display());
    }
    Ok(())
}

fn read_token(raw: &str) -> Result<String> {
    let mut token = if raw == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .context("read token from stdin")?;
        buf
    } else {
        raw.to_string()
    };
    while token.ends_with(['\n', '\r']) {
        token.pop();
    }
    if token.is_empty() {
        bail!("token cannot be empty");
    }
    Ok(token)
}

fn read_credential(path: &Path) -> Result<StoredCredential> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

pub fn authorization_for_url(url: &str) -> Result<Option<String>> {
    let service = normalize_service(url);
    authorization_for_service(&service)
}

pub fn authorization_for_service(service: &str) -> Result<Option<String>> {
    let dir = match credentials_dir() {
        Ok(dir) => dir,
        Err(_) => return Ok(None),
    };
    if !dir.is_dir() {
        return Ok(None);
    }

    let normalized = normalize_service(service);
    let token_path = credential_path_in(&dir, &normalized, TOKEN_USERNAME);
    if token_path.exists() {
        let cred = read_credential(&token_path)?;
        return credential_to_authorization(&cred);
    }

    let mut candidates = fs::read_dir(&dir)
        .with_context(|| format!("read {}", dir.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("read entries {}", dir.display()))?;
    candidates.sort_by_key(|entry| entry.path());
    for entry in candidates {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let prefix = format!("{}__", safe_filename(&normalized));
        if !name.starts_with(&prefix) || !name.ends_with(".toml") {
            continue;
        }
        let cred = read_credential(&path)?;
        if cred.service == normalized {
            return credential_to_authorization(&cred);
        }
    }
    Ok(None)
}

fn credential_to_authorization(cred: &StoredCredential) -> Result<Option<String>> {
    Ok(Some(
        basic_auth(&cred.username, &cred.token).map_err(anyhow::Error::from)?,
    ))
}

fn service_arg(sub: &ArgMatches) -> Result<&str> {
    sub.get_one::<String>("service")
        .map(String::as_str)
        .context("missing required <service>")
}

fn username_or_token(sub: &ArgMatches) -> String {
    sub.get_one::<String>("username")
        .cloned()
        .unwrap_or_else(|| TOKEN_USERNAME.to_string())
}

fn credential_path(service: &str, username: &str) -> Result<PathBuf> {
    Ok(credential_path_in(&credentials_dir()?, service, username))
}

fn credential_path_in(dir: &Path, service: &str, username: &str) -> PathBuf {
    dir.join(format!(
        "{}__{}.toml",
        safe_filename(service),
        safe_filename(username)
    ))
}

pub fn credentials_dir() -> Result<PathBuf> {
    if let Some(env_root) = std::env::var_os(CREDENTIALS_DIR_ENV) {
        let p = PathBuf::from(env_root);
        if p.as_os_str().is_empty() {
            bail!("${CREDENTIALS_DIR_ENV} is set but empty");
        }
        return Ok(p);
    }
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
        let p = PathBuf::from(xdg);
        if !p.as_os_str().is_empty() {
            return Ok(p.join("mamba").join("credentials"));
        }
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .context("no $HOME and no $MAMBA_CREDENTIALS_DIR — set $MAMBA_CREDENTIALS_DIR")?;
    Ok(home
        .join(".local")
        .join("share")
        .join("mamba")
        .join("credentials"))
}

fn normalize_service(service: &str) -> String {
    let trimmed = service.trim();
    let after_scheme = trimmed
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(trimmed);
    let without_path = after_scheme.split('/').next().unwrap_or(after_scheme);
    let without_auth = without_path
        .rsplit_once('@')
        .map(|(_, host)| host)
        .unwrap_or(without_path);
    without_auth.to_ascii_lowercase()
}

fn safe_filename(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "service".to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_url_to_lowercase_host_port() {
        assert_eq!(
            normalize_service("https://User@Repo.EXAMPLE:8443/simple"),
            "repo.example:8443"
        );
    }

    #[test]
    fn safe_filename_keeps_host_chars() {
        assert_eq!(safe_filename("repo.example:8443"), "repo.example_8443");
    }
}
