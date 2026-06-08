// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/config/platform_sync_config_preamble_source.md#source
// CODEGEN-BEGIN
//! Platform configuration

use crate::models::change::{RepoPlatformConfig, TechDesignPlatformConfig};
use crate::Result;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/config/platform_sync.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Authentication configuration.
/// @spec projects/agentic-workflow/tech-design/core/config/platform_sync.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Path to .env file.
    pub envfile: Option<String>,
    /// Field name in .env file.
    pub envfield: Option<String>,
}

/// Label configuration.
/// @spec projects/agentic-workflow/tech-design/core/config/platform_sync.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LabelConfig {
    /// Auto-create labels if they don't exist.
    #[serde(default)]
    pub auto_create: bool,
    /// Label for proposal issues.
    pub proposal: Option<String>,
    /// Label for spec issues.
    pub spec: Option<String>,
    /// Status labels.
    #[serde(default)]
    pub status: Option<StatusLabels>,
    /// Scope label configuration.
    #[serde(default)]
    pub scope: Option<ScopeConfig>,
}

/// Platform configuration from .aw/config.toml.
/// @spec projects/agentic-workflow/tech-design/core/config/platform_sync.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    /// Platform type: github or gitlab.
    #[serde(rename = "type")]
    pub platform_type: String,
    /// Repository in owner/repo format.
    pub repo: String,
    /// Optional self-hosted base host (e.g. 'gitlab.example.com'). None = use platform default.
    #[serde(default)]
    pub host: Option<String>,
    /// Authentication configuration.
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    /// Label configuration.
    #[serde(default)]
    pub labels: Option<LabelConfig>,
    /// Title format configuration.
    #[serde(default)]
    pub title: Option<TitleConfig>,
    /// Legacy envfile field.
    #[serde(default)]
    pub envfile: Option<String>,
    /// Legacy envfield field.
    #[serde(default)]
    pub envfield: Option<String>,
}

/// Auto-detection for scope labels.
/// @spec projects/agentic-workflow/tech-design/core/config/platform_sync.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScopeAutoDetect {
    /// Regex to extract scope from path.
    pub path_regex: Option<String>,
}

/// Scope label configuration (e.g., crate:xxx).
/// @spec projects/agentic-workflow/tech-design/core/config/platform_sync.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScopeConfig {
    /// Enable scope labels.
    pub enabled: bool,
    /// Label pattern (e.g., crate:{scope}).
    pub pattern: Option<String>,
    /// Auto-detection configuration.
    #[serde(default)]
    pub auto_detect: Option<ScopeAutoDetect>,
}

/// Status labels mapping.
/// @spec projects/agentic-workflow/tech-design/core/config/platform_sync.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusLabels {
    /// Draft status label.
    pub draft: Option<String>,
    /// Review status label.
    pub review: Option<String>,
    /// Approved status label.
    pub approved: Option<String>,
    /// Implementing status label.
    pub implementing: Option<String>,
    /// Done status label.
    pub done: Option<String>,
}

/// Title format configuration.
/// @spec projects/agentic-workflow/tech-design/core/config/platform_sync.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TitleConfig {
    /// Format for proposal issues.
    pub proposal: Option<String>,
    /// Format for spec issues.
    pub spec: Option<String>,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/config/platform_sync_config_runtime_source.md#source
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/config/platform_sync_config_runtime_source.md#source
impl PlatformConfig {
    /// Load platform config from project
    pub fn load(project_root: &Path) -> Result<Self> {
        // Try TOML first, then YAML for backwards compatibility
        let toml_path = project_root.join(".aw/config.toml");
        let yaml_path = project_root.join(".aw/config.yaml");

        let (config_path, is_toml) = if toml_path.exists() {
            (toml_path, true)
        } else if yaml_path.exists() {
            (yaml_path, false)
        } else {
            anyhow::bail!(
                "Platform config not found at .aw/config.toml.\n\
                 Create it with:\n\
                 \n\
                 [platform]\n\
                 type = \"github\"\n\
                 repo = \"owner/repo\"\n\
                 \n\
                 [platform.auth]\n\
                 envfile = \".env\"\n\
                 envfield = \"GITHUB_TOKEN\""
            );
        };

        let content = fs::read_to_string(&config_path)?;
        let config: ConfigFile = if is_toml {
            toml::from_str(&content)?
        } else {
            serde_yaml::from_str(&content)?
        };

        // Try [agentic_workflow.issue_platform] first, fall back to [platform]
        config
            .agentic_workflow
            .and_then(|s| s.issue_platform)
            .or(config.platform)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No platform section in config file.\n\
                     Add:\n\
                     \n\
                     [agentic_workflow.issue_platform]\n\
                     type = \"github\"\n\
                     repo = \"owner/repo\"\n\
                     \n\
                     Or rerun aw init to refresh platform settings"
                )
            })
    }

    /// Get envfile from auth config or legacy field
    fn get_envfile(&self) -> Option<&str> {
        self.auth
            .as_ref()
            .and_then(|a| a.envfile.as_deref())
            .or(self.envfile.as_deref())
    }

    /// Get envfield from auth config or legacy field
    fn get_envfield(&self) -> Option<&str> {
        self.auth
            .as_ref()
            .and_then(|a| a.envfield.as_deref())
            .or(self.envfield.as_deref())
    }

    /// Get token from configured source
    pub fn get_token(&self, project_root: &Path) -> Result<Option<String>> {
        let envfield = self.get_envfield();

        // First try environment variable directly
        if let Some(field) = envfield {
            if let Ok(token) = std::env::var(field) {
                if !token.is_empty() {
                    return Ok(Some(token));
                }
            }
        }

        // Then try .env file
        if let (Some(envfile), Some(envfield)) = (self.get_envfile(), envfield) {
            let env_path = resolve_path(envfile, project_root);

            if env_path.exists() {
                let env_vars = parse_env_file(&env_path)?;
                if let Some(token) = env_vars.get(envfield) {
                    if !token.is_empty() {
                        return Ok(Some(token.clone()));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Get proposal label
    pub fn proposal_label(&self) -> Option<&str> {
        self.labels.as_ref().and_then(|l| l.proposal.as_deref())
    }

    /// Get spec label
    pub fn spec_label(&self) -> Option<&str> {
        self.labels.as_ref().and_then(|l| l.spec.as_deref())
    }

    /// Format proposal title
    pub fn format_proposal_title(&self, change_id: &str, title: &str) -> String {
        self.title
            .as_ref()
            .and_then(|t| t.proposal.as_ref())
            .map(|fmt| {
                fmt.replace("{change_id}", change_id)
                    .replace("{title}", title)
            })
            .unwrap_or_else(|| format!("[{}] {}", change_id, title))
    }

    /// Format spec title
    pub fn format_spec_title(&self, change_id: &str, spec_id: &str) -> String {
        self.title
            .as_ref()
            .and_then(|t| t.spec.as_ref())
            .map(|fmt| {
                fmt.replace("{change_id}", change_id)
                    .replace("{spec_id}", spec_id)
            })
            .unwrap_or_else(|| format!("[{}/spec] {}", change_id, spec_id))
    }

    /// Extract scope labels from affected code paths
    pub fn extract_scope_labels(&self, affected_code: &[String]) -> Vec<String> {
        let scope_config = match self.labels.as_ref().and_then(|l| l.scope.as_ref()) {
            Some(s) if s.enabled => s,
            _ => return Vec::new(),
        };

        let pattern = match scope_config.pattern.as_ref() {
            Some(p) => p,
            None => return Vec::new(),
        };

        let regex_str = match scope_config
            .auto_detect
            .as_ref()
            .and_then(|a| a.path_regex.as_ref())
        {
            Some(r) => r,
            None => return Vec::new(),
        };

        let regex = match Regex::new(regex_str) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        let mut scopes: Vec<String> = affected_code
            .iter()
            .filter_map(|path| {
                regex
                    .captures(path)
                    .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
            })
            .collect();

        scopes.sort();
        scopes.dedup();

        scopes
            .into_iter()
            .map(|scope| pattern.replace("{scope}", &scope))
            .collect()
    }
}

/// Full config file structure
#[derive(Debug, Deserialize)]
struct ConfigFile {
    /// New namespaced path: [agentic_workflow.issue_platform]
    agentic_workflow: Option<SddSection>,
    /// Legacy path: [platform]
    platform: Option<PlatformConfig>,
}

/// Wrapper for [agentic_workflow] section
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SddSection {
    issue_platform: Option<PlatformConfig>,
    #[serde(default)]
    repo_platform: Option<RepoPlatformConfig>,
    #[serde(default)]
    tech_design_platform: Option<TechDesignPlatformConfig>,
}

/// Resolve path with $project_dir variable
/// Security: Only allows paths within project root (no .. traversal or absolute paths)
fn resolve_path(path: &str, project_root: &Path) -> std::path::PathBuf {
    let resolved = path.replace("$project_dir", ".");

    // Security check: reject absolute paths and path traversal
    let path_obj = std::path::Path::new(&resolved);
    if path_obj.is_absolute() {
        eprintln!(
            "Warning: Absolute paths not allowed in config, using project-relative: {}",
            path
        );
        return project_root.join(path_obj.file_name().unwrap_or_default());
    }

    // Reject paths with .. components
    for component in path_obj.components() {
        if let std::path::Component::ParentDir = component {
            eprintln!(
                "Warning: Path traversal (..) not allowed in config, using project-relative: {}",
                path
            );
            return project_root.join(path_obj.file_name().unwrap_or_default());
        }
    }

    project_root.join(path_obj)
}

/// Parse .env file into key-value pairs
/// Supports:
/// - KEY=value
/// - KEY="value with spaces"
/// - KEY='value with spaces'
/// - export KEY=value (strips export prefix)
/// - Comments with # (including # inside quoted values)
/// - Escaped characters in quoted values
fn parse_env_file(path: &Path) -> Result<HashMap<String, String>> {
    let content = fs::read_to_string(path)?;
    let mut vars = HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Strip 'export ' prefix if present
        let line = line.strip_prefix("export ").unwrap_or(line);

        // Parse KEY=value or KEY="value"
        if let Some((key, rest)) = line.split_once('=') {
            let key = key.trim();
            if key.is_empty() {
                continue;
            }

            let rest = rest.trim();

            // Parse value with proper quote handling
            let value = if rest.starts_with('"') {
                // Double-quoted: find closing quote, handle escapes
                parse_quoted_value(rest, '"')
            } else if rest.starts_with('\'') {
                // Single-quoted: find closing quote (no escape handling)
                parse_quoted_value(rest, '\'')
            } else {
                // Unquoted: take until # or end of line
                rest.split('#').next().unwrap_or("").trim().to_string()
            };

            vars.insert(key.to_string(), value);
        }
    }

    Ok(vars)
}

/// Parse a quoted value, handling escapes for double quotes
fn parse_quoted_value(s: &str, quote: char) -> String {
    let inner = &s[1..]; // Skip opening quote
    let mut result = String::new();
    let mut chars = inner.chars().peekable();
    let is_double_quote = quote == '"';

    while let Some(c) = chars.next() {
        if c == quote {
            // Found closing quote
            break;
        } else if is_double_quote && c == '\\' {
            // Handle escape sequences in double quotes
            if let Some(&next) = chars.peek() {
                match next {
                    'n' => {
                        result.push('\n');
                        chars.next();
                    }
                    't' => {
                        result.push('\t');
                        chars.next();
                    }
                    '\\' | '"' => {
                        result.push(next);
                        chars.next();
                    }
                    _ => result.push(c), // Keep backslash for unknown escapes
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_env_file() {
        let temp_dir = TempDir::new().unwrap();
        let env_path = temp_dir.path().join(".env");

        fs::write(
            &env_path,
            r#"
# Comment
GITHUB_TOKEN=ghp_test123
GITLAB_TOKEN="glpat-test456"
EMPTY=
"#,
        )
        .unwrap();

        let vars = parse_env_file(&env_path).unwrap();
        assert_eq!(vars.get("GITHUB_TOKEN"), Some(&"ghp_test123".to_string()));
        assert_eq!(vars.get("GITLAB_TOKEN"), Some(&"glpat-test456".to_string()));
        assert_eq!(vars.get("EMPTY"), Some(&"".to_string()));
    }

    #[test]
    fn test_parse_env_file_advanced() {
        let temp_dir = TempDir::new().unwrap();
        let env_path = temp_dir.path().join(".env");

        fs::write(
            &env_path,
            r#"
# Export prefix support
export API_KEY=secret123

# Hash inside quotes (not a comment)
DB_URL="postgres://user:pass#123@localhost/db"

# Single quotes
SINGLE='value with spaces'

# Inline comment (unquoted)
PORT=3000 # default port

# Escape sequences in double quotes
ESCAPED="line1\nline2\ttab"
"#,
        )
        .unwrap();

        let vars = parse_env_file(&env_path).unwrap();
        assert_eq!(vars.get("API_KEY"), Some(&"secret123".to_string()));
        assert_eq!(
            vars.get("DB_URL"),
            Some(&"postgres://user:pass#123@localhost/db".to_string())
        );
        assert_eq!(vars.get("SINGLE"), Some(&"value with spaces".to_string()));
        assert_eq!(vars.get("PORT"), Some(&"3000".to_string()));
        assert_eq!(vars.get("ESCAPED"), Some(&"line1\nline2\ttab".to_string()));
    }

    #[test]
    fn test_resolve_path() {
        let project_root = Path::new("/home/user/project");

        // Normal relative path
        assert_eq!(
            resolve_path(".env", project_root),
            Path::new("/home/user/project/.env")
        );

        // $project_dir variable substitution
        assert_eq!(
            resolve_path("$project_dir/.env", project_root),
            Path::new("/home/user/project/./.env")
        );

        // Security: absolute paths are blocked (uses filename only)
        assert_eq!(
            resolve_path("/absolute/path/.env", project_root),
            Path::new("/home/user/project/.env")
        );

        // Security: path traversal is blocked (uses filename only)
        assert_eq!(
            resolve_path("../../../etc/passwd", project_root),
            Path::new("/home/user/project/passwd")
        );
    }

    #[test]
    fn test_load_config_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let cclab_dir = temp_dir.path().join(".aw");
        fs::create_dir_all(&cclab_dir).unwrap();

        fs::write(
            cclab_dir.join("config.yaml"),
            r#"
platform:
  type: github
  repo: owner/repo
  envfile: .env
  envfield: GITHUB_TOKEN
"#,
        )
        .unwrap();

        let config = PlatformConfig::load(temp_dir.path()).unwrap();
        assert_eq!(config.platform_type, "github");
        assert_eq!(config.repo, "owner/repo");
        assert_eq!(config.envfile, Some(".env".to_string()));
        assert_eq!(config.envfield, Some("GITHUB_TOKEN".to_string()));
    }

    #[test]
    fn test_load_config_toml() {
        let temp_dir = TempDir::new().unwrap();
        let cclab_dir = temp_dir.path().join(".aw");
        fs::create_dir_all(&cclab_dir).unwrap();

        fs::write(
            cclab_dir.join("config.toml"),
            r#"
[platform]
type = "github"
repo = "owner/repo"

[platform.auth]
envfile = ".env"
envfield = "GITHUB_TOKEN"

[platform.labels]
auto_create = true
proposal = "cclab:sdd:proposal"
spec = "cclab:sdd:spec"

[platform.labels.scope]
enabled = true
pattern = "crate:{scope}"

[platform.labels.scope.auto_detect]
path_regex = "crates/cclab-([^/]+)/"
"#,
        )
        .unwrap();

        let config = PlatformConfig::load(temp_dir.path()).unwrap();
        assert_eq!(config.platform_type, "github");
        assert_eq!(config.repo, "owner/repo");
        assert_eq!(config.proposal_label(), Some("cclab:sdd:proposal"));
        assert_eq!(config.spec_label(), Some("cclab:sdd:spec"));
    }

    #[test]
    fn test_get_token_from_env_file() {
        let temp_dir = TempDir::new().unwrap();

        // Create .env file
        fs::write(temp_dir.path().join(".env"), "GITHUB_TOKEN=test_token_123").unwrap();

        let config = PlatformConfig {
            platform_type: "github".to_string(),
            repo: "owner/repo".to_string(),
            host: None,
            auth: Some(AuthConfig {
                envfile: Some(".env".to_string()),
                envfield: Some("GITHUB_TOKEN".to_string()),
            }),
            labels: None,
            title: None,
            envfile: None,
            envfield: None,
        };

        let token = config.get_token(temp_dir.path()).unwrap();
        assert_eq!(token, Some("test_token_123".to_string()));
    }

    #[test]
    fn test_extract_scope_labels() {
        let config = PlatformConfig {
            platform_type: "github".to_string(),
            repo: "owner/repo".to_string(),
            host: None,
            auth: None,
            labels: Some(LabelConfig {
                auto_create: true,
                proposal: None,
                spec: None,
                status: None,
                scope: Some(ScopeConfig {
                    enabled: true,
                    pattern: Some("crate:{scope}".to_string()),
                    auto_detect: Some(ScopeAutoDetect {
                        path_regex: Some("(?:crates|projects)/(?:cclab-)?([^/]+)/".to_string()),
                    }),
                }),
            }),
            title: None,
            envfile: None,
            envfield: None,
        };

        let affected = vec![
            "projects/agentic-workflow/src/lib.rs".to_string(),
            "projects/agentic-workflow/src/generate/lib.rs".to_string(),
            "projects/agentic-workflow/src/types.rs".to_string(), // duplicate
        ];

        let labels = config.extract_scope_labels(&affected);
        assert_eq!(labels, vec!["crate:agentic-workflow"]);
    }

    #[test]
    fn test_load_config_toml_sdd_issue_platform() {
        let temp_dir = TempDir::new().unwrap();
        let cclab_dir = temp_dir.path().join(".aw");
        fs::create_dir_all(&cclab_dir).unwrap();

        fs::write(
            cclab_dir.join("config.toml"),
            r#"
version = "0.3.13"

[agentic_workflow.issue_platform]
type = "gitlab"
repo = "myorg/myrepo"
auth_method = "cli"
"#,
        )
        .unwrap();

        let config = PlatformConfig::load(temp_dir.path()).unwrap();
        assert_eq!(config.platform_type, "gitlab");
        assert_eq!(config.repo, "myorg/myrepo");
    }

    #[test]
    fn test_load_config_toml_sdd_over_legacy() {
        // [agentic_workflow.issue_platform] takes priority over [platform]
        let temp_dir = TempDir::new().unwrap();
        let cclab_dir = temp_dir.path().join(".aw");
        fs::create_dir_all(&cclab_dir).unwrap();

        fs::write(
            cclab_dir.join("config.toml"),
            r#"
[platform]
type = "github"
repo = "old/repo"

[agentic_workflow.issue_platform]
type = "gitlab"
repo = "new/repo"
"#,
        )
        .unwrap();

        let config = PlatformConfig::load(temp_dir.path()).unwrap();
        assert_eq!(config.platform_type, "gitlab");
        assert_eq!(config.repo, "new/repo");
    }

    #[test]
    fn test_format_titles() {
        let config = PlatformConfig {
            platform_type: "github".to_string(),
            repo: "owner/repo".to_string(),
            host: None,
            auth: None,
            labels: None,
            title: Some(TitleConfig {
                proposal: Some("[{change_id}] {title}".to_string()),
                spec: Some("[{change_id}/spec] {spec_id}".to_string()),
            }),
            envfile: None,
            envfield: None,
        };

        assert_eq!(
            config.format_proposal_title("platform-sync", "Platform Sync Feature"),
            "[platform-sync] Platform Sync Feature"
        );
        assert_eq!(
            config.format_spec_title("platform-sync", "github-client"),
            "[platform-sync/spec] github-client"
        );
    }
}
// CODEGEN-END
