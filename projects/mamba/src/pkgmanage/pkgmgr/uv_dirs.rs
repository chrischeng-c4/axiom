// XDG / uv-style cache, data, config path resolver (Tick 62).
//
// uv stores its on-disk state in three directories:
//   * cache  — downloaded wheels, built sdists, simple-index metadata
//   * data   — managed Python installs, uv tool installs
//   * config — uv.toml + per-user defaults
//
// On every platform the resolution order is:
//   1. `UV_CACHE_DIR` / `UV_DATA_DIR` / `UV_CONFIG_DIR` env var (uv-compat)
//   2. The platform-native location:
//      * Linux: $XDG_*_HOME/uv (then ~/.cache/uv etc. fallback)
//      * macOS: ~/Library/Caches/uv (cache),
//               ~/Library/Application Support/uv (data + config)
//      * Windows: %LOCALAPPDATA%\uv\cache (cache),
//                 %APPDATA%\uv\data (data),
//                 %APPDATA%\uv (config)
//   3. tmp_dir/uv-* as the last-ditch fallback when HOME / APPDATA are
//      missing.
//
// `resolve_dirs` takes an `EnvLookup` and a `Platform` so callers can
// inject a fake environment in tests — production callers use
// `EnvLookup::from_process_env()` and `Platform::current()`. Pure-data
// path computation; no filesystem side effects.

use std::collections::HashMap;
use std::path::PathBuf;

/// Platform discriminator. The macOS / Windows arms differ enough from
/// freedesktop XDG that they get their own match arms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    Macos,
    Windows,
}

impl Platform {
    /// Detect the current platform from cfg. Returns `Linux` for any
    /// unrecognized Unix (BSD / illumos / etc.) so callers still get a
    /// usable default.
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Platform::Macos
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else {
            Platform::Linux
        }
    }
}

/// Pluggable env lookup. Tests inject a `HashMap`; production calls
/// `EnvLookup::from_process_env()` to read the real process env.
#[derive(Debug, Clone, Default)]
pub struct EnvLookup {
    vars: HashMap<String, String>,
}

impl EnvLookup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_pairs<I, K, V>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let mut out = HashMap::new();
        for (k, v) in pairs {
            out.insert(k.into(), v.into());
        }
        Self { vars: out }
    }

    pub fn from_process_env() -> Self {
        Self {
            vars: std::env::vars().collect(),
        }
    }

    /// Returns the value when set *and non-empty*. uv treats unset and
    /// empty-string the same way for these variables.
    fn get(&self, key: &str) -> Option<&str> {
        match self.vars.get(key) {
            Some(v) if !v.is_empty() => Some(v.as_str()),
            _ => None,
        }
    }
}

/// Resolved uv directories. All three paths are always populated —
/// `resolve_dirs` falls back to `tmp_dir/uv-*` when HOME / APPDATA are
/// missing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UvDirs {
    pub cache: PathBuf,
    pub data: PathBuf,
    pub config: PathBuf,
}

/// Resolve the uv directory triple from the given env + platform.
pub fn resolve_dirs(env: &EnvLookup, platform: Platform, tmp_dir: &str) -> UvDirs {
    UvDirs {
        cache: resolve_cache(env, platform, tmp_dir),
        data: resolve_data(env, platform, tmp_dir),
        config: resolve_config(env, platform, tmp_dir),
    }
}

fn resolve_cache(env: &EnvLookup, platform: Platform, tmp_dir: &str) -> PathBuf {
    if let Some(v) = env.get("UV_CACHE_DIR") {
        return PathBuf::from(v);
    }
    match platform {
        Platform::Linux => {
            if let Some(xdg) = env.get("XDG_CACHE_HOME") {
                return PathBuf::from(xdg).join("uv");
            }
            if let Some(home) = env.get("HOME") {
                return PathBuf::from(home).join(".cache").join("uv");
            }
            PathBuf::from(tmp_dir).join("uv-cache")
        }
        Platform::Macos => {
            if let Some(home) = env.get("HOME") {
                return PathBuf::from(home)
                    .join("Library")
                    .join("Caches")
                    .join("uv");
            }
            PathBuf::from(tmp_dir).join("uv-cache")
        }
        Platform::Windows => {
            if let Some(localapp) = env.get("LOCALAPPDATA") {
                return PathBuf::from(localapp).join("uv").join("cache");
            }
            PathBuf::from(tmp_dir).join("uv-cache")
        }
    }
}

fn resolve_data(env: &EnvLookup, platform: Platform, tmp_dir: &str) -> PathBuf {
    if let Some(v) = env.get("UV_DATA_DIR") {
        return PathBuf::from(v);
    }
    match platform {
        Platform::Linux => {
            if let Some(xdg) = env.get("XDG_DATA_HOME") {
                return PathBuf::from(xdg).join("uv");
            }
            if let Some(home) = env.get("HOME") {
                return PathBuf::from(home).join(".local").join("share").join("uv");
            }
            PathBuf::from(tmp_dir).join("uv-data")
        }
        Platform::Macos => {
            if let Some(home) = env.get("HOME") {
                return PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("uv");
            }
            PathBuf::from(tmp_dir).join("uv-data")
        }
        Platform::Windows => {
            if let Some(appdata) = env.get("APPDATA") {
                return PathBuf::from(appdata).join("uv").join("data");
            }
            PathBuf::from(tmp_dir).join("uv-data")
        }
    }
}

fn resolve_config(env: &EnvLookup, platform: Platform, tmp_dir: &str) -> PathBuf {
    if let Some(v) = env.get("UV_CONFIG_DIR") {
        return PathBuf::from(v);
    }
    match platform {
        Platform::Linux => {
            if let Some(xdg) = env.get("XDG_CONFIG_HOME") {
                return PathBuf::from(xdg).join("uv");
            }
            if let Some(home) = env.get("HOME") {
                return PathBuf::from(home).join(".config").join("uv");
            }
            PathBuf::from(tmp_dir).join("uv-config")
        }
        Platform::Macos => {
            if let Some(home) = env.get("HOME") {
                return PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("uv");
            }
            PathBuf::from(tmp_dir).join("uv-config")
        }
        Platform::Windows => {
            if let Some(appdata) = env.get("APPDATA") {
                return PathBuf::from(appdata).join("uv");
            }
            PathBuf::from(tmp_dir).join("uv-config")
        }
    }
}

/// Convenience: resolved subpaths beneath `data` that uv writes to.
impl UvDirs {
    pub fn python_install_dir(&self) -> PathBuf {
        self.data.join("python")
    }
    pub fn tool_install_dir(&self) -> PathBuf {
        self.data.join("tools")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> EnvLookup {
        EnvLookup::from_pairs(pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())))
    }

    // ---------- UV_* overrides ----------

    #[test]
    fn uv_cache_dir_overrides_platform_default_on_linux() {
        let e = env(&[("UV_CACHE_DIR", "/custom/cache"), ("HOME", "/home/u")]);
        let d = resolve_dirs(&e, Platform::Linux, "/tmp");
        assert_eq!(d.cache, PathBuf::from("/custom/cache"));
    }

    #[test]
    fn uv_data_dir_overrides_platform_default_on_macos() {
        let e = env(&[("UV_DATA_DIR", "/custom/data"), ("HOME", "/Users/u")]);
        let d = resolve_dirs(&e, Platform::Macos, "/tmp");
        assert_eq!(d.data, PathBuf::from("/custom/data"));
    }

    #[test]
    fn uv_config_dir_overrides_platform_default_on_windows() {
        let e = env(&[
            ("UV_CONFIG_DIR", "C:\\custom\\config"),
            ("APPDATA", "C:\\Users\\u\\AppData\\Roaming"),
        ]);
        let d = resolve_dirs(&e, Platform::Windows, "C:\\Temp");
        assert_eq!(d.config, PathBuf::from("C:\\custom\\config"));
    }

    #[test]
    fn empty_uv_cache_dir_falls_through_to_platform() {
        let e = env(&[("UV_CACHE_DIR", ""), ("HOME", "/home/u")]);
        let d = resolve_dirs(&e, Platform::Linux, "/tmp");
        assert_eq!(d.cache, PathBuf::from("/home/u/.cache/uv"));
    }

    // ---------- Linux XDG ----------

    #[test]
    fn linux_xdg_cache_home_wins_over_home() {
        let e = env(&[("XDG_CACHE_HOME", "/x/cache"), ("HOME", "/home/u")]);
        let d = resolve_dirs(&e, Platform::Linux, "/tmp");
        assert_eq!(d.cache, PathBuf::from("/x/cache/uv"));
    }

    #[test]
    fn linux_home_fallback_for_cache() {
        let e = env(&[("HOME", "/home/u")]);
        let d = resolve_dirs(&e, Platform::Linux, "/tmp");
        assert_eq!(d.cache, PathBuf::from("/home/u/.cache/uv"));
    }

    #[test]
    fn linux_home_fallback_for_data() {
        let e = env(&[("HOME", "/home/u")]);
        let d = resolve_dirs(&e, Platform::Linux, "/tmp");
        assert_eq!(d.data, PathBuf::from("/home/u/.local/share/uv"));
    }

    #[test]
    fn linux_home_fallback_for_config() {
        let e = env(&[("HOME", "/home/u")]);
        let d = resolve_dirs(&e, Platform::Linux, "/tmp");
        assert_eq!(d.config, PathBuf::from("/home/u/.config/uv"));
    }

    #[test]
    fn linux_xdg_data_home_wins_over_home() {
        let e = env(&[("XDG_DATA_HOME", "/x/data"), ("HOME", "/home/u")]);
        let d = resolve_dirs(&e, Platform::Linux, "/tmp");
        assert_eq!(d.data, PathBuf::from("/x/data/uv"));
    }

    #[test]
    fn linux_xdg_config_home_wins_over_home() {
        let e = env(&[("XDG_CONFIG_HOME", "/x/config"), ("HOME", "/home/u")]);
        let d = resolve_dirs(&e, Platform::Linux, "/tmp");
        assert_eq!(d.config, PathBuf::from("/x/config/uv"));
    }

    #[test]
    fn linux_no_home_falls_back_to_tmp() {
        let e = env(&[]);
        let d = resolve_dirs(&e, Platform::Linux, "/tmp");
        assert_eq!(d.cache, PathBuf::from("/tmp/uv-cache"));
        assert_eq!(d.data, PathBuf::from("/tmp/uv-data"));
        assert_eq!(d.config, PathBuf::from("/tmp/uv-config"));
    }

    // ---------- macOS ----------

    #[test]
    fn macos_cache_in_library_caches() {
        let e = env(&[("HOME", "/Users/u")]);
        let d = resolve_dirs(&e, Platform::Macos, "/tmp");
        assert_eq!(d.cache, PathBuf::from("/Users/u/Library/Caches/uv"));
    }

    #[test]
    fn macos_data_and_config_share_application_support() {
        let e = env(&[("HOME", "/Users/u")]);
        let d = resolve_dirs(&e, Platform::Macos, "/tmp");
        let expected = PathBuf::from("/Users/u/Library/Application Support/uv");
        assert_eq!(d.data, expected);
        assert_eq!(d.config, expected);
    }

    #[test]
    fn macos_no_home_falls_back_to_tmp() {
        let e = env(&[]);
        let d = resolve_dirs(&e, Platform::Macos, "/tmp");
        assert_eq!(d.cache, PathBuf::from("/tmp/uv-cache"));
        assert_eq!(d.data, PathBuf::from("/tmp/uv-data"));
        assert_eq!(d.config, PathBuf::from("/tmp/uv-config"));
    }

    #[test]
    fn macos_ignores_xdg_vars() {
        let e = env(&[("HOME", "/Users/u"), ("XDG_CACHE_HOME", "/xdg/cache")]);
        let d = resolve_dirs(&e, Platform::Macos, "/tmp");
        assert_eq!(d.cache, PathBuf::from("/Users/u/Library/Caches/uv"));
    }

    // ---------- Windows ----------

    #[test]
    fn windows_cache_in_localappdata() {
        let e = env(&[("LOCALAPPDATA", "C:\\Users\\u\\AppData\\Local")]);
        let d = resolve_dirs(&e, Platform::Windows, "C:\\Temp");
        assert_eq!(
            d.cache,
            PathBuf::from("C:\\Users\\u\\AppData\\Local")
                .join("uv")
                .join("cache")
        );
    }

    #[test]
    fn windows_data_in_appdata_subdir() {
        let e = env(&[("APPDATA", "C:\\Users\\u\\AppData\\Roaming")]);
        let d = resolve_dirs(&e, Platform::Windows, "C:\\Temp");
        assert_eq!(
            d.data,
            PathBuf::from("C:\\Users\\u\\AppData\\Roaming")
                .join("uv")
                .join("data")
        );
    }

    #[test]
    fn windows_config_in_appdata() {
        let e = env(&[("APPDATA", "C:\\Users\\u\\AppData\\Roaming")]);
        let d = resolve_dirs(&e, Platform::Windows, "C:\\Temp");
        assert_eq!(
            d.config,
            PathBuf::from("C:\\Users\\u\\AppData\\Roaming").join("uv")
        );
    }

    #[test]
    fn windows_no_appdata_falls_back_to_tmp() {
        let e = env(&[]);
        let d = resolve_dirs(&e, Platform::Windows, "C:\\Temp");
        assert_eq!(d.cache, PathBuf::from("C:\\Temp").join("uv-cache"));
        assert_eq!(d.data, PathBuf::from("C:\\Temp").join("uv-data"));
        assert_eq!(d.config, PathBuf::from("C:\\Temp").join("uv-config"));
    }

    // ---------- Convenience subdirs ----------

    #[test]
    fn python_install_dir_is_under_data() {
        let dirs = UvDirs {
            cache: PathBuf::from("/c"),
            data: PathBuf::from("/d"),
            config: PathBuf::from("/cfg"),
        };
        assert_eq!(dirs.python_install_dir(), PathBuf::from("/d/python"));
    }

    #[test]
    fn tool_install_dir_is_under_data() {
        let dirs = UvDirs {
            cache: PathBuf::from("/c"),
            data: PathBuf::from("/d"),
            config: PathBuf::from("/cfg"),
        };
        assert_eq!(dirs.tool_install_dir(), PathBuf::from("/d/tools"));
    }

    // ---------- Platform::current ----------

    #[test]
    fn platform_current_is_well_defined() {
        let _ = Platform::current();
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn platform_current_is_macos_when_compiled_on_macos() {
        assert_eq!(Platform::current(), Platform::Macos);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn platform_current_is_linux_when_compiled_on_linux() {
        assert_eq!(Platform::current(), Platform::Linux);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn platform_current_is_windows_when_compiled_on_windows() {
        assert_eq!(Platform::current(), Platform::Windows);
    }

    // ---------- from_process_env smoke ----------

    #[test]
    fn from_process_env_loads_some_variables() {
        let e = EnvLookup::from_process_env();
        let _ = resolve_dirs(&e, Platform::current(), "/tmp");
    }
}
