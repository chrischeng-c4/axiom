// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
// CODEGEN-BEGIN
//! Tailwind CSS configuration model.
//!
//! Supports loading from `tailwind.config.js` (minimal JS object literal parser)
//! and from the `css.tailwind` section of `jet.toml`.
//! JS config takes precedence when both files exist.

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

// ─── public types ────────────────────────────────────────────────────────────

/// Tailwind dark-mode strategy.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DarkMode {
    #[default]
    /// `.dark` class on an ancestor element (default Tailwind v3 strategy).
    Class,
    /// `@media (prefers-color-scheme: dark)` — not fully implemented, treated as Class.
    Media,
}

/// Subset of `theme` / `theme.extend` relevant to the JIT engine.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
#[derive(Debug, Clone, Default)]
pub struct ThemeConfig {
    /// `theme.extend.colors`: maps utility suffix → CSS value.
    ///
    /// Example: `{ "primary": "hsl(var(--primary))" }` enables `text-primary`.
    pub extend_colors: HashMap<String, String>,
}

/// Parsed Tailwind configuration.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
#[derive(Debug, Clone)]
pub struct TailwindConfig {
    /// Content glob patterns — e.g. `["./src/**/*.{ts,tsx}"]`.
    pub content: Vec<String>,
    /// Dark mode strategy (only `class` is fully supported).
    pub dark_mode: DarkMode,
    /// Theme configuration.
    pub theme: ThemeConfig,
    /// Plugin names present in the config (e.g. `"tailwindcss-animate"`).
    pub plugins: Vec<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
impl Default for TailwindConfig {
    fn default() -> Self {
        Self {
            content: vec!["./src/**/*.{ts,tsx,js,jsx}".to_string()],
            dark_mode: DarkMode::Class,
            theme: ThemeConfig::default(),
            plugins: vec![],
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
impl TailwindConfig {
    // ── loaders ─────────────────────────────────────────────────────────

    /// Load config from `project_root`, preferring `tailwind.config.js` over
    /// the `css.tailwind` section in `jet.toml`.  Falls back to
    /// `TailwindConfig::default()` if neither file exists.
    pub fn load(project_root: &Path) -> Result<Self> {
        let js_path = project_root.join("tailwind.config.js");
        let toml_path = project_root.join("jet.toml");

        if js_path.exists() {
            return Self::from_js(&js_path);
        }
        if toml_path.exists() {
            return Self::from_toml(&toml_path);
        }
        Ok(Self::default())
    }

    /// Parse `tailwind.config.js` using a minimal JS object-literal evaluator.
    ///
    /// Handles the common `module.exports = { ... }` pattern.
    pub fn from_js(path: &Path) -> Result<Self> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Cannot read {:?}: {}", path, e))?;
        parse_js_config(&source)
    }

    /// Parse the `css.tailwind` section from `jet.toml`.
    pub fn from_toml(path: &Path) -> Result<Self> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Cannot read {:?}: {}", path, e))?;
        parse_yaml_config(&source)
    }
}

// ─── JS config parser ────────────────────────────────────────────────────────

/// Minimal JS object-literal parser for `tailwind.config.js`.
///
/// Supports:
/// - `module.exports = { ... }`
/// - `content: [...]` array of string literals
/// - `darkMode: "class"` / `darkMode: "media"`
/// - `theme: { extend: { colors: { key: "value" } } }`
/// - `plugins: [require("name"), ...]` — plugin names extracted from `require()`
fn parse_js_config(source: &str) -> Result<TailwindConfig> {
    let mut config = TailwindConfig::default();

    // Strip comments
    let stripped = strip_js_comments(source);

    // Extract module.exports object body
    let body = extract_exports_body(&stripped).unwrap_or(&stripped);

    // content
    if let Some(arr) = extract_array_field(body, "content") {
        config.content = extract_string_array(&arr);
    }

    // darkMode
    if let Some(val) = extract_string_field(body, "darkMode") {
        config.dark_mode = match val.trim_matches('"').trim_matches('\'') {
            "media" => DarkMode::Media,
            _ => DarkMode::Class,
        };
    }

    // plugins
    config.plugins = extract_plugins(body);

    // theme.extend.colors
    if let Some(theme_body) = extract_object_field(body, "theme") {
        if let Some(extend_body) = extract_object_field(&theme_body, "extend") {
            if let Some(colors_body) = extract_object_field(&extend_body, "colors") {
                config.theme.extend_colors = extract_string_map(&colors_body);
            }
        }
    }

    Ok(config)
}

// ─── YAML config parser ───────────────────────────────────────────────────────

fn parse_yaml_config(source: &str) -> Result<TailwindConfig> {
    let value: serde_yaml::Value =
        serde_yaml::from_str(source).map_err(|e| anyhow::anyhow!("YAML parse error: {}", e))?;

    let tailwind = match value.get("css").and_then(|c| c.get("tailwind")) {
        Some(t) => t,
        None => return Ok(TailwindConfig::default()),
    };

    let mut config = TailwindConfig::default();

    if let Some(content) = tailwind.get("content").and_then(|c| c.as_sequence()) {
        config.content = content
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
    }

    if let Some(dark) = tailwind.get("darkMode").and_then(|d| d.as_str()) {
        config.dark_mode = match dark {
            "media" => DarkMode::Media,
            _ => DarkMode::Class,
        };
    }

    if let Some(plugins) = tailwind.get("plugins").and_then(|p| p.as_sequence()) {
        config.plugins = plugins
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
    }

    if let Some(colors) = tailwind
        .get("theme")
        .and_then(|t| t.get("extend"))
        .and_then(|e| e.get("colors"))
        .and_then(|c| c.as_mapping())
    {
        for (k, v) in colors {
            if let (Some(key), Some(val)) = (k.as_str(), v.as_str()) {
                config
                    .theme
                    .extend_colors
                    .insert(key.to_string(), val.to_string());
            }
        }
    }

    Ok(config)
}

// ─── JS parsing helpers ───────────────────────────────────────────────────────

fn strip_js_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    // Track whether the parser is inside a string literal to avoid treating
    // `/*` inside glob patterns (e.g. `./src/**/*.tsx`) as block comments.
    let mut in_string = false;
    let mut string_char = '"';
    let mut prev = '\0';

    while let Some(c) = chars.next() {
        if in_string {
            // Only the matching unescaped quote closes the string.
            if c == string_char && prev != '\\' {
                in_string = false;
            }
            out.push(c);
            prev = c;
            continue;
        }

        // Start of a string literal
        if c == '"' || c == '\'' {
            in_string = true;
            string_char = c;
            out.push(c);
            prev = c;
            continue;
        }

        // Possible comment start (only outside string literals)
        if c == '/' {
            match chars.peek() {
                Some('/') => {
                    // line comment — skip to end of line
                    for ch in chars.by_ref() {
                        if ch == '\n' {
                            out.push('\n');
                            break;
                        }
                    }
                    prev = '\n';
                    continue;
                }
                Some('*') => {
                    // block comment — skip to */
                    chars.next();
                    let mut prev_c = ' ';
                    for ch in chars.by_ref() {
                        if prev_c == '*' && ch == '/' {
                            break;
                        }
                        prev_c = ch;
                    }
                    prev = ' ';
                    continue;
                }
                _ => {}
            }
        }

        out.push(c);
        prev = c;
    }
    out
}

fn extract_exports_body<'a>(source: &'a str) -> Option<&'a str> {
    let pat = "module.exports";
    let start = source.find(pat)?;
    let after = &source[start + pat.len()..];
    let eq = after.find('=')?;
    let after_eq = &after[eq + 1..];
    let brace = after_eq.find('{')?;
    let body_start = after_eq[brace + 1..].as_ptr() as usize - source.as_ptr() as usize;
    // Find matching closing brace
    let body = &source[body_start..];
    let close = find_matching_brace(body)?;
    Some(&body[..close])
}

fn extract_object_field<'a>(body: &'a str, key: &str) -> Option<String> {
    let pat = format!("{}:", key);
    let start = body.find(&pat)?;
    let after = &body[start + pat.len()..];
    let trimmed = after.trim_start();
    if !trimmed.starts_with('{') {
        return None;
    }
    let inner = &trimmed[1..];
    let close = find_matching_brace(inner)?;
    Some(inner[..close].to_string())
}

fn extract_array_field<'a>(body: &'a str, key: &str) -> Option<String> {
    let pat = format!("{}:", key);
    let start = body.find(&pat)?;
    let after = &body[start + pat.len()..];
    let trimmed = after.trim_start();
    if !trimmed.starts_with('[') {
        return None;
    }
    let inner = &trimmed[1..];
    let close = find_matching_bracket(inner)?;
    Some(inner[..close].to_string())
}

fn extract_string_field<'a>(body: &'a str, key: &str) -> Option<String> {
    let pat = format!("{}:", key);
    let start = body.find(&pat)?;
    let after = &body[start + pat.len()..].trim_start();
    // Find quoted value
    let quote = after.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest = &after[1..];
    let end = rest.find(quote)?;
    Some(rest[..end].to_string())
}

fn extract_string_array(arr_body: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut remaining = arr_body;
    while let Some(start) = remaining.find(|c| c == '"' || c == '\'') {
        let quote = remaining.chars().nth(start).unwrap();
        let inner = &remaining[start + 1..];
        if let Some(end) = inner.find(quote) {
            result.push(inner[..end].to_string());
            remaining = &inner[end + 1..];
        } else {
            break;
        }
    }
    result
}

fn extract_string_map(obj_body: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    // Match: key: "value" or key: 'value'
    let re_pattern = regex::Regex::new(r#"(\w[\w-]*):\s*["']([^"']+)["']"#).unwrap();
    for cap in re_pattern.captures_iter(obj_body) {
        map.insert(cap[1].to_string(), cap[2].to_string());
    }
    map
}

fn extract_plugins(body: &str) -> Vec<String> {
    let mut plugins = Vec::new();
    // Match: require("plugin-name") or require('plugin-name')
    let re = regex::Regex::new(r#"require\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap();
    // Find plugins array
    if let Some(arr) = extract_array_field(body, "plugins") {
        for cap in re.captures_iter(&arr) {
            plugins.push(cap[1].to_string());
        }
    }
    plugins
}

fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut string_char = '"';
    let mut prev = '\0';
    for (i, c) in s.char_indices() {
        if in_string {
            if c == string_char && prev != '\\' {
                in_string = false;
            }
        } else {
            match c {
                '"' | '\'' => {
                    in_string = true;
                    string_char = c;
                }
                '{' => depth += 1,
                '}' => {
                    if depth == 0 {
                        return Some(i);
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }
        prev = c;
    }
    None
}

// ─── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// T16: tailwind.config.js Parsing (R11)
    ///
    /// Verifies that a tailwind.config.js file is parsed correctly.
    #[test]
    fn t16_tailwind_config_js_parsing() {
        let dir = TempDir::new().unwrap();
        let config_file = dir.path().join("tailwind.config.js");
        fs::write(
            &config_file,
            r#"module.exports = {
  content: ["./src/**/*.tsx"],
  darkMode: "class",
  plugins: []
}"#,
        )
        .unwrap();

        let config = TailwindConfig::from_js(&config_file);
        assert!(
            config.is_ok(),
            "Should parse tailwind.config.js: {:?}",
            config
        );
        let config = config.unwrap();

        assert_eq!(
            config.content,
            vec!["./src/**/*.tsx"],
            "Should parse content array"
        );
        assert_eq!(
            config.dark_mode,
            DarkMode::Class,
            "Should parse darkMode: class"
        );
    }

    /// Unit test: plugins array parsed from tailwind.config.js.
    #[test]
    fn tailwind_config_js_plugins_parsed() {
        let dir = TempDir::new().unwrap();
        let config_file = dir.path().join("tailwind.config.js");
        fs::write(
            &config_file,
            r#"module.exports = {
  content: ["./src/**/*.{ts,tsx}"],
  darkMode: "class",
  theme: { extend: { colors: { primary: "hsl(var(--primary))" } } },
  plugins: [require("tailwindcss-animate"), require("@tailwindcss/typography")]
}"#,
        )
        .unwrap();

        let config = TailwindConfig::from_js(&config_file).unwrap();

        assert!(
            config.plugins.contains(&"tailwindcss-animate".to_string()),
            "Should parse tailwindcss-animate plugin: {:?}",
            config.plugins
        );
        assert!(
            config
                .plugins
                .contains(&"@tailwindcss/typography".to_string()),
            "Should parse @tailwindcss/typography plugin: {:?}",
            config.plugins
        );
    }

    /// Unit test: theme.extend.colors parsed.
    #[test]
    fn tailwind_config_js_theme_colors_parsed() {
        let dir = TempDir::new().unwrap();
        let config_file = dir.path().join("tailwind.config.js");
        fs::write(
            &config_file,
            r##"module.exports = {
  content: ["./src/**/*.tsx"],
  theme: { extend: { colors: { primary: "hsl(var(--primary))", accent: "#ff6600" } } }
}"##,
        )
        .unwrap();

        let config = TailwindConfig::from_js(&config_file).unwrap();

        assert!(
            config.theme.extend_colors.contains_key("primary"),
            "Should parse primary color: {:?}",
            config.theme.extend_colors
        );
        assert_eq!(
            config
                .theme
                .extend_colors
                .get("primary")
                .map(|s| s.as_str()),
            Some("hsl(var(--primary))"),
            "Should parse primary color value"
        );
    }

    /// Unit test: darkMode: "media" is parsed correctly.
    #[test]
    fn tailwind_config_js_dark_mode_media() {
        let dir = TempDir::new().unwrap();
        let config_file = dir.path().join("tailwind.config.js");
        fs::write(
            &config_file,
            r#"module.exports = { content: [], darkMode: "media" }"#,
        )
        .unwrap();

        let config = TailwindConfig::from_js(&config_file).unwrap();
        assert_eq!(
            config.dark_mode,
            DarkMode::Media,
            "Should parse darkMode: media"
        );
    }

    /// Unit test: TailwindConfig::load falls back to default when no config files exist.
    #[test]
    fn tailwind_config_load_defaults_when_no_files() {
        let dir = TempDir::new().unwrap(); // empty dir
        let config = TailwindConfig::load(dir.path());
        assert!(config.is_ok(), "Should return Ok(default): {:?}", config);
        let config = config.unwrap();
        // Default content glob covers common TS/TSX files
        assert!(
            !config.content.is_empty(),
            "Default content should not be empty"
        );
    }

    /// GH #3086 — pins the contract that `TailwindConfig::load` surfaces
    /// YAML parse errors as `Err` instead of swallowing them. The historical
    /// bug was that three callsites (`dev_server::serve_module`,
    /// `dev_server::rebuild_css`, `bundler::process_css_entry`) wrapped this
    /// call in `.unwrap_or_default()`, so a broken `jet.toml`
    /// `[css.tailwind]` block silently disabled the user's Tailwind config.
    /// We can't catch the silent-swallow at the callsite without spinning up
    /// a dev server, but we can guard the API contract: if this test starts
    /// returning `Ok(...)` for malformed input, the API has regressed and
    /// any future `.unwrap_or_default()` would once again be silent.
    #[test]
    fn tailwind_config_load_surfaces_yaml_parse_error() {
        let dir = TempDir::new().unwrap();
        // Malformed YAML — unterminated string after `content:`.
        fs::write(
            dir.path().join("jet.toml"),
            "css:\n  tailwind:\n    content: \"unterminated\n",
        )
        .unwrap();

        let result = TailwindConfig::load(dir.path());
        assert!(
            result.is_err(),
            "malformed [css.tailwind] block must return Err, got: {result:?}",
        );
    }
}

fn find_matching_bracket(s: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut string_char = '"';
    let mut prev = '\0';
    for (i, c) in s.char_indices() {
        if in_string {
            if c == string_char && prev != '\\' {
                in_string = false;
            }
        } else {
            match c {
                '"' | '\'' => {
                    in_string = true;
                    string_char = c;
                }
                '[' => depth += 1,
                ']' => {
                    if depth == 0 {
                        return Some(i);
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }
        prev = c;
    }
    None
}
// CODEGEN-END
