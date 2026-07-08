// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
// CODEGEN-BEGIN
use anyhow::Result;
use std::collections::HashMap;
use std::env::VarError;
use std::path::{Path, PathBuf};

/// GH #3608 — resolve `~/.npmrc` from $HOME while distinguishing
/// `VarError::NotPresent` (canonical "no HOME", silent) from
/// `VarError::NotUnicode(_)` (real misconfiguration: HOME is set but
/// jet silently doesn't read the user's ~/.npmrc, dropping their
/// auth tokens / scoped registries with no diagnostic). Returns the
/// candidate path + an optional warn message.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn safe_user_npmrc_path(
    home_result: Result<String, VarError>,
) -> (Option<PathBuf>, Option<String>) {
    match home_result {
        Ok(home) => (Some(PathBuf::from(home).join(".npmrc")), None),
        Err(VarError::NotPresent) => (None, None),
        Err(VarError::NotUnicode(_)) => (None, Some(format_safe_user_npmrc_warn("not-unicode"))),
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_safe_user_npmrc_warn(observed_kind: &str) -> String {
    format!(
        "GH #3608 pkg_manager::npmrc: HOME observed as {observed_kind}; \
         the user-level ~/.npmrc will NOT be loaded. Auth tokens, scoped \
         registries, and proxy settings in your home .npmrc will be \
         silently dropped — expect 401s on private scopes or fall-through \
         to the public registry. Re-set HOME with a valid UTF-8 path."
    )
}

/// Format the warning emitted when an existing .npmrc file cannot be parsed.
///
/// Extracted as a free function so unit tests can pin the message shape
/// without having to inspect log capture: the message must name the offending
/// .npmrc path verbatim, preserve the underlying error, mention the GH #3526
/// tag, and hint at the user-visible symptoms (private-registry auth /
/// scope routing / proxy) so users searching their logs for a 401 or a
/// "package not found" failure can land on this line.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_npmrc_read_warn(path: &Path, err: &anyhow::Error) -> String {
    format!(
        "GH #3526 failed to read .npmrc at {}: {}; private-registry auth tokens, scoped-registry routing, and proxy settings in this file will be skipped. Expect 401s on private scopes or 'package not found' against a corporate mirror until this is fixed (check file permissions / EACCES, disk health, or that the path is a file not a directory).",
        path.display(),
        err
    )
}

/// Merged .npmrc config from project → user → global precedence.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, Default)]
pub struct NpmrcConfig {
    pub registry: String,
    pub scoped_registries: HashMap<String, String>,
    pub auth_tokens: HashMap<String, String>,
    pub proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub strict_ssl: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
impl NpmrcConfig {
    /// Load and merge .npmrc from all levels (project > user > global).
    pub fn load(project_dir: &Path) -> Self {
        let mut config = Self {
            registry: "https://registry.npmjs.org/".to_string(),
            strict_ssl: true,
            ..Default::default()
        };

        // Load in reverse precedence order (global first, project last wins)
        let paths = Self::config_paths(project_dir);
        for path in paths {
            if path.exists() {
                // GH #3526 — surface .npmrc read failures instead of
                // silently ignoring the user's settings. The file
                // exists (outer `exists()` was true), so a parse_file
                // Err is a real io::Error: EACCES on a misowned
                // .npmrc, EIO on a flaky disk, EISDIR if it's
                // somehow a directory. Without this warn, the user
                // sees 401s on a private scope or "package not found"
                // on a corporate mirror, with no breadcrumb pointing
                // at the actual unreadable .npmrc.
                match Self::parse_file(&path) {
                    Ok(entries) => config.merge(entries),
                    Err(err) => {
                        tracing::warn!(
                            target: "jet::pkg_manager::npmrc",
                            npmrc = %path.display(),
                            error = %err,
                            "{}",
                            format_npmrc_read_warn(&path, &err)
                        );
                    }
                }
            }
        }

        config
    }

    /// Return config file paths in load order: [global, user, project].
    fn config_paths(project_dir: &Path) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Global: /etc/npmrc or PREFIX/etc/npmrc
        paths.push(PathBuf::from("/etc/npmrc"));

        // User: ~/.npmrc
        // GH #3608 — distinguish HOME NotPresent (silent) from
        // NotUnicode (warn) so misconfigured HOME values don't
        // silently drop the user's auth tokens / scoped registries.
        let (user_npmrc, warn) = safe_user_npmrc_path(std::env::var("HOME"));
        if let Some(msg) = warn {
            tracing::warn!(target: "jet::pkg_manager::npmrc", "{}", msg);
        }
        if let Some(p) = user_npmrc {
            paths.push(p);
        }

        // Project: .npmrc in project root
        paths.push(project_dir.join(".npmrc"));

        paths
    }

    /// Parse an .npmrc file into key-value pairs.
    fn parse_file(path: &Path) -> Result<Vec<(String, String)>> {
        let content = std::fs::read_to_string(path)?;
        let mut entries = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                entries.push((key.trim().to_string(), value.trim().to_string()));
            }
        }

        Ok(entries)
    }

    /// Merge parsed entries into this config. Later calls overwrite earlier.
    fn merge(&mut self, entries: Vec<(String, String)>) {
        for (key, value) in entries {
            if key == "registry" {
                self.registry = value;
            } else if key == "proxy" {
                self.proxy = Some(value);
            } else if key == "https-proxy" || key == "https_proxy" {
                self.https_proxy = Some(value);
            } else if key == "strict-ssl" {
                self.strict_ssl = value != "false";
            } else if key.starts_with('@') && key.ends_with(":registry") {
                // @scope:registry = https://...
                let scope = key.trim_end_matches(":registry");
                self.scoped_registries.insert(scope.to_string(), value);
            } else if key.starts_with("//") && key.ends_with(":_authToken") {
                // //registry.npmjs.org/:_authToken = token
                let registry = key.trim_end_matches(":_authToken");
                self.auth_tokens.insert(registry.to_string(), value);
            }
        }
    }

    /// Get the registry URL for a given package name.
    pub fn registry_for(&self, package_name: &str) -> &str {
        if let Some(scope) = package_name.strip_prefix('@') {
            if let Some(scope_name) = scope.split('/').next() {
                let scope_key = format!("@{}", scope_name);
                if let Some(url) = self.scoped_registries.get(&scope_key) {
                    return url;
                }
            }
        }
        &self.registry
    }

    /// Get the auth token for a given registry URL, if any.
    pub fn auth_token_for(&self, registry_url: &str) -> Option<&str> {
        // Try exact match on //host/path pattern
        for (pattern, token) in &self.auth_tokens {
            if registry_url.contains(pattern.trim_start_matches("//")) {
                return Some(token);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let dir = tempdir().unwrap();
        let config = NpmrcConfig::load(dir.path());
        assert_eq!(config.registry, "https://registry.npmjs.org/");
        assert!(config.strict_ssl);
    }

    #[test]
    fn test_parse_npmrc() {
        let dir = tempdir().unwrap();
        let npmrc_path = dir.path().join(".npmrc");
        std::fs::write(
            &npmrc_path,
            "registry=https://custom.registry.com/\n\
             @myorg:registry=https://npm.myorg.com/\n\
             //npm.myorg.com/:_authToken=secret123\n\
             strict-ssl=false\n\
             # this is a comment\n",
        )
        .unwrap();

        let config = NpmrcConfig::load(dir.path());
        assert_eq!(config.registry, "https://custom.registry.com/");
        assert_eq!(
            config.scoped_registries.get("@myorg"),
            Some(&"https://npm.myorg.com/".to_string())
        );
        assert_eq!(
            config.auth_tokens.get("//npm.myorg.com/"),
            Some(&"secret123".to_string())
        );
        assert!(!config.strict_ssl);
    }

    #[test]
    fn gh3526_format_npmrc_read_warn_names_path_and_error_and_issue() {
        let path = PathBuf::from("/home/user/.npmrc");
        let err = anyhow::anyhow!("Permission denied (os error 13)");
        let msg = format_npmrc_read_warn(&path, &err);
        assert!(
            msg.contains("/home/user/.npmrc"),
            "warning must name the offending .npmrc path verbatim: {msg}"
        );
        assert!(
            msg.contains("Permission denied (os error 13)"),
            "warning must preserve the underlying error verbatim: {msg}"
        );
        assert!(
            msg.contains("GH #3526"),
            "warning must carry the GH #3526 tag so users can grep their logs: {msg}"
        );
    }

    #[test]
    fn gh3526_format_npmrc_read_warn_hints_at_symptoms() {
        let path = PathBuf::from("/etc/npmrc");
        let err = anyhow::anyhow!("Is a directory (os error 21)");
        let msg = format_npmrc_read_warn(&path, &err);
        // The whole point of the warning is to be findable when a user
        // grepping for "401" or "registry" hits the log: pin those keywords.
        assert!(
            msg.contains("auth")
                && (msg.contains("private") || msg.contains("registry") || msg.contains("scope")),
            "warning must mention auth + private/registry/scope so a user debugging a 401 can find it: {msg}"
        );
        assert!(
            msg.contains("proxy"),
            "warning must mention proxy since corporate setups commonly depend on .npmrc proxy: {msg}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn gh3526_load_surfaces_unreadable_npmrc_instead_of_silently_dropping() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let npmrc_path = dir.path().join(".npmrc");
        std::fs::write(&npmrc_path, "registry=https://corp.example.com/\n").unwrap();
        // chmod 000 — owner can't read either; parse_file will return Err
        // with EACCES rather than the previous `exists() && Ok(_)` shortcut.
        std::fs::set_permissions(&npmrc_path, std::fs::Permissions::from_mode(0o000)).unwrap();

        // load() must not panic and must fall back to defaults instead of
        // silently dropping the user's registry config without a peep.
        let config = NpmrcConfig::load(dir.path());
        // The custom registry is unreachable (we couldn't read the file), so
        // load() falls back to the default. The contract this test pins:
        // load() *does not* claim the user's custom registry is in effect
        // when the file is unreadable — and (per the production code path)
        // emits a warning rather than silently swallowing the failure.
        assert_eq!(
            config.registry, "https://registry.npmjs.org/",
            "unreadable .npmrc must not leave stale/silent state; default registry expected"
        );

        // Restore perms so tempdir cleanup can succeed.
        std::fs::set_permissions(&npmrc_path, std::fs::Permissions::from_mode(0o644)).unwrap();
    }

    #[test]
    fn test_scoped_registry_lookup() {
        let mut config = NpmrcConfig::default();
        config.registry = "https://registry.npmjs.org/".to_string();
        config
            .scoped_registries
            .insert("@myorg".to_string(), "https://npm.myorg.com/".to_string());

        assert_eq!(
            config.registry_for("@myorg/my-pkg"),
            "https://npm.myorg.com/"
        );
        assert_eq!(config.registry_for("lodash"), "https://registry.npmjs.org/");
    }
}

#[cfg(test)]
mod gh3608_safe_user_npmrc_path_tests {
    //! GH #3608 — `~/.npmrc` resolution must distinguish HOME NotPresent
    //! (silent) from NotUnicode (warn). The prior `if let Ok(home) = ...`
    //! collapsed both into silent skip, silently dropping the user's
    //! auth tokens / scoped registries on a misconfigured HOME.
    use super::*;

    #[test]
    fn ok_home_yields_npmrc_path() {
        let (path, warn) = safe_user_npmrc_path(Ok("/Users/dev".to_string()));
        assert_eq!(path.unwrap(), PathBuf::from("/Users/dev/.npmrc"));
        assert!(warn.is_none());
    }

    #[test]
    fn not_present_silently_skips() {
        let (path, warn) = safe_user_npmrc_path(Err(VarError::NotPresent));
        assert!(path.is_none());
        assert!(
            warn.is_none(),
            "NotPresent is canonical — must not emit warn"
        );
    }

    #[test]
    fn not_unicode_skips_and_warns() {
        let raw = std::ffi::OsString::from("ignored");
        let (path, warn) = safe_user_npmrc_path(Err(VarError::NotUnicode(raw)));
        assert!(path.is_none(), "no usable HOME → skip");
        let msg = warn.expect("NotUnicode must emit warn");
        assert!(msg.contains("GH #3608"), "msg: {msg}");
        assert!(msg.contains("not-unicode"), "msg: {msg}");
        assert!(msg.contains("HOME"), "msg: {msg}");
    }

    #[test]
    fn warn_helper_names_consequences() {
        let msg = format_safe_user_npmrc_warn("not-unicode");
        assert!(msg.contains("GH #3608"), "msg: {msg}");
        assert!(
            msg.contains("auth tokens") || msg.contains("registries"),
            "must name user-visible consequences, got: {msg}"
        );
    }

    /// Distinguishability: the two error discriminants must produce
    /// distinguishable warn states.
    #[test]
    fn discriminants_distinguishable() {
        let raw = std::ffi::OsString::from("ignored");
        let not_present = safe_user_npmrc_path(Err(VarError::NotPresent)).1;
        let not_unicode = safe_user_npmrc_path(Err(VarError::NotUnicode(raw))).1;
        assert!(not_present.is_none());
        assert!(not_unicode.is_some());
    }
}
// CODEGEN-END
