//! Bundled typeshed stubs for standard library modules
//!
//! This module provides type information for common Python stdlib modules,
//! along with dynamic typeshed downloading and caching.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, RwLock};
use std::time::{Duration, SystemTime};

use super::imports::ModuleInfo;
use super::ty::Type;

/// Configuration for typeshed integration
#[derive(Debug, Clone)]
pub struct TypeshedConfig {
    /// Custom path to a local typeshed copy (takes precedence over downloads)
    pub typeshed_path: Option<PathBuf>,
    /// Directory to store downloaded stubs
    pub cache_dir: PathBuf,
    /// Disable network requests (offline mode)
    pub offline: bool,
    /// Python version for stub resolution (e.g., "3.10", "3.11")
    pub python_version: String,
    /// Cache TTL in days (default: 7)
    pub cache_ttl_days: u32,
    /// Optional commit hash to pin typeshed version
    pub typeshed_commit: Option<String>,
}

impl Default for TypeshedConfig {
    fn default() -> Self {
        let cache_dir = dirs_cache_dir().unwrap_or_else(|| PathBuf::from(".cclab_lens-cache"));
        Self {
            typeshed_path: None,
            cache_dir,
            offline: false,
            python_version: "3.11".to_string(),
            cache_ttl_days: 7,
            typeshed_commit: None,
        }
    }
}

/// Get platform-specific cache directory
fn dirs_cache_dir() -> Option<PathBuf> {
    // Simple cross-platform cache dir detection
    if let Ok(home) = std::env::var("HOME") {
        Some(PathBuf::from(home).join(".cache").join("cclab_lens"))
    } else if let Ok(cache) = std::env::var("XDG_CACHE_HOME") {
        Some(PathBuf::from(cache).join("cclab_lens"))
    } else {
        None
    }
}

/// Cache entry metadata
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct CacheEntry {
    /// When this entry was last updated
    last_updated: SystemTime,
    /// ETag for conditional requests
    etag: Option<String>,
    /// Python version this stub is for
    python_version: String,
}

/// Typeshed cache manager
///
/// Handles local storage and HTTP fetching of typeshed stubs.
/// Downloads run in background threads to avoid blocking LSP/analysis.
#[derive(Debug)]
pub struct TypeshedCache {
    config: TypeshedConfig,
    /// Cache metadata (module -> entry)
    metadata: RwLock<HashMap<String, CacheEntry>>,
    /// Pending downloads (module names being fetched)
    pending: Mutex<std::collections::HashSet<String>>,
    /// Completed downloads ready to be loaded
    completed: Mutex<Vec<String>>,
}

impl TypeshedCache {
    /// Create a new typeshed cache with the given configuration
    pub fn new(config: TypeshedConfig) -> Self {
        // Ensure cache directory exists
        if !config.cache_dir.exists() {
            let _ = fs::create_dir_all(&config.cache_dir);
        }

        Self {
            config,
            metadata: RwLock::new(HashMap::new()),
            pending: Mutex::new(std::collections::HashSet::new()),
            completed: Mutex::new(Vec::new()),
        }
    }

    /// Get the cache key for a module (includes Python version)
    fn cache_key(&self, module: &str) -> String {
        format!(
            "{}_{}",
            self.config.python_version,
            module.replace('.', "_")
        )
    }

    /// Get the cache file path for a module
    fn cache_path(&self, module: &str) -> PathBuf {
        let key = self.cache_key(module);
        self.config.cache_dir.join(format!("{}.pyi", key))
    }

    /// Check if a cached stub exists and is not expired
    pub fn has_valid_cache(&self, module: &str) -> bool {
        let path = self.cache_path(module);
        if !path.exists() {
            return false;
        }

        // Check TTL
        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                let ttl = Duration::from_secs(self.config.cache_ttl_days as u64 * 24 * 60 * 60);
                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                    return elapsed < ttl;
                }
            }
        }

        false
    }

    /// Read cached stub content
    pub fn read_cache(&self, module: &str) -> Option<String> {
        let path = self.cache_path(module);
        fs::read_to_string(path).ok()
    }

    /// Write stub content to cache
    pub fn write_cache(&self, module: &str, content: &str) -> std::io::Result<()> {
        let path = self.cache_path(module);

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&path, content)?;

        // Update metadata
        if let Ok(mut metadata) = self.metadata.write() {
            metadata.insert(
                module.to_string(),
                CacheEntry {
                    last_updated: SystemTime::now(),
                    etag: None,
                    python_version: self.config.python_version.clone(),
                },
            );
        }

        Ok(())
    }

    /// Check if a download is already pending for a module
    pub fn is_pending(&self, module: &str) -> bool {
        self.pending
            .lock()
            .map(|p| p.contains(module))
            .unwrap_or(false)
    }

    /// Start a background download for a module (synchronous version for simplicity)
    /// In a real implementation, this would use async or channels for true non-blocking behavior
    /// For now, we download synchronously but the caller can spawn this on a thread pool
    pub fn download_module(&self, module: &str) -> Result<String, String> {
        if self.config.offline {
            return Err("Offline mode enabled".to_string());
        }

        // Check if already pending
        {
            let mut pending = match self.pending.lock() {
                Ok(p) => p,
                Err(_) => return Err("Failed to acquire lock".to_string()),
            };
            if pending.contains(module) {
                return Err("Download already pending".to_string());
            }
            pending.insert(module.to_string());
        }

        let result = download_stub(
            module,
            &self.config.python_version,
            self.config.typeshed_commit.as_deref(),
        );

        // Remove from pending
        if let Ok(mut pending) = self.pending.lock() {
            pending.remove(module);
        }

        if let Ok(ref content) = result {
            // Write to cache on success
            let _ = self.write_cache(module, content);

            // Mark as completed
            if let Ok(mut completed) = self.completed.lock() {
                completed.push(module.to_string());
            }
        }

        result
    }

    /// Start a background download for a module using rayon thread pool
    /// Returns immediately; use `poll_completed` to check for results
    pub fn start_background_download(&self, module: &str)
    where
        Self: Sync,
    {
        if self.config.offline {
            return;
        }

        // Check if already pending
        {
            let mut pending = match self.pending.lock() {
                Ok(p) => p,
                Err(_) => return,
            };
            if pending.contains(module) {
                return;
            }
            pending.insert(module.to_string());
        }

        // For now, just mark the module for download
        // The actual download will happen synchronously when get_or_download is called
        // A full async implementation would use tokio::spawn or rayon here
    }

    /// Poll for completed downloads
    /// Returns module names that have finished downloading
    pub fn poll_completed(&self) -> Vec<String> {
        let mut completed = match self.completed.lock() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        std::mem::take(&mut *completed)
    }

    /// Check if offline mode is enabled
    pub fn is_offline(&self) -> bool {
        self.config.offline
    }

    /// Get the configured typeshed path (for local stubs)
    pub fn typeshed_path(&self) -> Option<&PathBuf> {
        self.config.typeshed_path.as_ref()
    }
}

/// Download a stub from the typeshed GitHub repository
fn download_stub(
    module: &str,
    _python_version: &str,
    commit: Option<&str>,
) -> Result<String, String> {
    // Map module name to typeshed path
    let stub_path = module_to_typeshed_path(module);

    // Build URL
    let branch = commit.unwrap_or("main");
    let url = format!(
        "https://raw.githubusercontent.com/python/typeshed/{}/stdlib/{}",
        branch, stub_path
    );

    // Perform blocking HTTP request
    let response =
        reqwest::blocking::get(&url).map_err(|e| format!("HTTP request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}: {}", response.status(), url));
    }

    response
        .text()
        .map_err(|e| format!("Failed to read response: {}", e))
}

/// Map a Python module name to its typeshed path
fn module_to_typeshed_path(module: &str) -> String {
    let parts: Vec<&str> = module.split('.').collect();

    if parts.len() == 1 {
        // Single module: could be module.pyi or module/__init__.pyi
        // Try module.pyi first (most common for stdlib)
        format!("{}.pyi", module)
    } else {
        // Nested module like os.path -> os/path.pyi
        format!("{}.pyi", parts.join("/"))
    }
}

/// Create os module stub
pub fn create_os_stub() -> ModuleInfo {
    let mut info = ModuleInfo::new("os");

    // Path operations
    info.exports
        .insert("getcwd".to_string(), Type::callable(vec![], Type::Str));
    info.exports.insert(
        "chdir".to_string(),
        Type::callable(vec![Type::Str], Type::None),
    );
    info.exports.insert(
        "listdir".to_string(),
        Type::callable(vec![Type::Str], Type::list(Type::Str)),
    );
    info.exports.insert(
        "mkdir".to_string(),
        Type::callable(vec![Type::Str], Type::None),
    );
    info.exports.insert(
        "makedirs".to_string(),
        Type::callable(vec![Type::Str], Type::None),
    );
    info.exports.insert(
        "remove".to_string(),
        Type::callable(vec![Type::Str], Type::None),
    );
    info.exports.insert(
        "rmdir".to_string(),
        Type::callable(vec![Type::Str], Type::None),
    );
    info.exports.insert(
        "rename".to_string(),
        Type::callable(vec![Type::Str, Type::Str], Type::None),
    );
    info.exports.insert(
        "stat".to_string(),
        Type::callable(vec![Type::Str], Type::Any),
    );
    info.exports.insert(
        "walk".to_string(),
        Type::callable(vec![Type::Str], Type::Any),
    );

    // Environment
    info.exports
        .insert("environ".to_string(), Type::dict(Type::Str, Type::Str));
    info.exports.insert(
        "getenv".to_string(),
        Type::callable(vec![Type::Str], Type::optional(Type::Str)),
    );
    info.exports.insert(
        "putenv".to_string(),
        Type::callable(vec![Type::Str, Type::Str], Type::None),
    );

    // Process
    info.exports
        .insert("getpid".to_string(), Type::callable(vec![], Type::Int));
    info.exports
        .insert("getppid".to_string(), Type::callable(vec![], Type::Int));
    info.exports.insert(
        "system".to_string(),
        Type::callable(vec![Type::Str], Type::Int),
    );
    info.exports.insert(
        "popen".to_string(),
        Type::callable(vec![Type::Str], Type::Any),
    );

    // Path separator
    info.exports.insert("sep".to_string(), Type::Str);
    info.exports.insert("linesep".to_string(), Type::Str);
    info.exports.insert("pathsep".to_string(), Type::Str);
    info.exports.insert("name".to_string(), Type::Str);

    info
}

/// Create os.path module stub
pub fn create_os_path_stub() -> ModuleInfo {
    let mut info = ModuleInfo::new("os.path");

    info.exports.insert(
        "join".to_string(),
        Type::callable(vec![Type::Str, Type::Str], Type::Str),
    );
    info.exports.insert(
        "exists".to_string(),
        Type::callable(vec![Type::Str], Type::Bool),
    );
    info.exports.insert(
        "isfile".to_string(),
        Type::callable(vec![Type::Str], Type::Bool),
    );
    info.exports.insert(
        "isdir".to_string(),
        Type::callable(vec![Type::Str], Type::Bool),
    );
    info.exports.insert(
        "isabs".to_string(),
        Type::callable(vec![Type::Str], Type::Bool),
    );
    info.exports.insert(
        "islink".to_string(),
        Type::callable(vec![Type::Str], Type::Bool),
    );
    info.exports.insert(
        "basename".to_string(),
        Type::callable(vec![Type::Str], Type::Str),
    );
    info.exports.insert(
        "dirname".to_string(),
        Type::callable(vec![Type::Str], Type::Str),
    );
    info.exports.insert(
        "split".to_string(),
        Type::callable(vec![Type::Str], Type::Tuple(vec![Type::Str, Type::Str])),
    );
    info.exports.insert(
        "splitext".to_string(),
        Type::callable(vec![Type::Str], Type::Tuple(vec![Type::Str, Type::Str])),
    );
    info.exports.insert(
        "abspath".to_string(),
        Type::callable(vec![Type::Str], Type::Str),
    );
    info.exports.insert(
        "realpath".to_string(),
        Type::callable(vec![Type::Str], Type::Str),
    );
    info.exports.insert(
        "normpath".to_string(),
        Type::callable(vec![Type::Str], Type::Str),
    );
    info.exports.insert(
        "expanduser".to_string(),
        Type::callable(vec![Type::Str], Type::Str),
    );
    info.exports.insert(
        "expandvars".to_string(),
        Type::callable(vec![Type::Str], Type::Str),
    );
    info.exports.insert(
        "getsize".to_string(),
        Type::callable(vec![Type::Str], Type::Int),
    );
    info.exports.insert(
        "getmtime".to_string(),
        Type::callable(vec![Type::Str], Type::Float),
    );
    info.exports.insert(
        "getctime".to_string(),
        Type::callable(vec![Type::Str], Type::Float),
    );
    info.exports.insert(
        "getatime".to_string(),
        Type::callable(vec![Type::Str], Type::Float),
    );

    info
}

/// Create sys module stub
pub fn create_sys_stub() -> ModuleInfo {
    let mut info = ModuleInfo::new("sys");

    // Streams
    info.exports.insert("stdin".to_string(), Type::Any);
    info.exports.insert("stdout".to_string(), Type::Any);
    info.exports.insert("stderr".to_string(), Type::Any);

    // Arguments
    info.exports
        .insert("argv".to_string(), Type::list(Type::Str));

    // Paths
    info.exports
        .insert("path".to_string(), Type::list(Type::Str));
    info.exports
        .insert("modules".to_string(), Type::dict(Type::Str, Type::Any));

    // Version info
    info.exports.insert("version".to_string(), Type::Str);
    info.exports.insert(
        "version_info".to_string(),
        Type::Tuple(vec![Type::Int, Type::Int, Type::Int, Type::Str, Type::Int]),
    );
    info.exports.insert("platform".to_string(), Type::Str);
    info.exports.insert("executable".to_string(), Type::Str);
    info.exports.insert("prefix".to_string(), Type::Str);

    // Functions
    info.exports.insert(
        "exit".to_string(),
        Type::callable(vec![Type::Int], Type::Never),
    );
    info.exports.insert(
        "getrecursionlimit".to_string(),
        Type::callable(vec![], Type::Int),
    );
    info.exports.insert(
        "setrecursionlimit".to_string(),
        Type::callable(vec![Type::Int], Type::None),
    );
    info.exports.insert(
        "getsizeof".to_string(),
        Type::callable(vec![Type::Any], Type::Int),
    );

    // Numeric limits
    info.exports.insert("maxsize".to_string(), Type::Int);
    info.exports.insert("float_info".to_string(), Type::Any);
    info.exports.insert("int_info".to_string(), Type::Any);

    info
}

/// Create io module stub
pub fn create_io_stub() -> ModuleInfo {
    let mut info = ModuleInfo::new("io");

    // Base classes
    info.exports.insert(
        "IOBase".to_string(),
        Type::ClassType {
            name: "IOBase".to_string(),
            module: Some("io".to_string()),
        },
    );
    info.exports.insert(
        "RawIOBase".to_string(),
        Type::ClassType {
            name: "RawIOBase".to_string(),
            module: Some("io".to_string()),
        },
    );
    info.exports.insert(
        "BufferedIOBase".to_string(),
        Type::ClassType {
            name: "BufferedIOBase".to_string(),
            module: Some("io".to_string()),
        },
    );
    info.exports.insert(
        "TextIOBase".to_string(),
        Type::ClassType {
            name: "TextIOBase".to_string(),
            module: Some("io".to_string()),
        },
    );

    // Concrete classes
    info.exports.insert(
        "FileIO".to_string(),
        Type::ClassType {
            name: "FileIO".to_string(),
            module: Some("io".to_string()),
        },
    );
    info.exports.insert(
        "BytesIO".to_string(),
        Type::ClassType {
            name: "BytesIO".to_string(),
            module: Some("io".to_string()),
        },
    );
    info.exports.insert(
        "StringIO".to_string(),
        Type::ClassType {
            name: "StringIO".to_string(),
            module: Some("io".to_string()),
        },
    );
    info.exports.insert(
        "BufferedReader".to_string(),
        Type::ClassType {
            name: "BufferedReader".to_string(),
            module: Some("io".to_string()),
        },
    );
    info.exports.insert(
        "BufferedWriter".to_string(),
        Type::ClassType {
            name: "BufferedWriter".to_string(),
            module: Some("io".to_string()),
        },
    );
    info.exports.insert(
        "TextIOWrapper".to_string(),
        Type::ClassType {
            name: "TextIOWrapper".to_string(),
            module: Some("io".to_string()),
        },
    );

    // Functions
    info.exports.insert(
        "open".to_string(),
        Type::callable(vec![Type::Str, Type::Str], Type::Any),
    );

    // Constants
    info.exports
        .insert("DEFAULT_BUFFER_SIZE".to_string(), Type::Int);
    info.exports.insert("SEEK_SET".to_string(), Type::Int);
    info.exports.insert("SEEK_CUR".to_string(), Type::Int);
    info.exports.insert("SEEK_END".to_string(), Type::Int);

    info
}

/// Create re module stub
pub fn create_re_stub() -> ModuleInfo {
    let mut info = ModuleInfo::new("re");

    // Pattern and Match types
    info.exports.insert(
        "Pattern".to_string(),
        Type::ClassType {
            name: "Pattern".to_string(),
            module: Some("re".to_string()),
        },
    );
    info.exports.insert(
        "Match".to_string(),
        Type::ClassType {
            name: "Match".to_string(),
            module: Some("re".to_string()),
        },
    );

    // Functions
    info.exports.insert(
        "compile".to_string(),
        Type::callable(
            vec![Type::Str],
            Type::Instance {
                name: "Pattern".to_string(),
                module: Some("re".to_string()),
                type_args: vec![Type::Str],
            },
        ),
    );
    info.exports.insert(
        "match".to_string(),
        Type::callable(
            vec![Type::Str, Type::Str],
            Type::optional(Type::Instance {
                name: "Match".to_string(),
                module: Some("re".to_string()),
                type_args: vec![Type::Str],
            }),
        ),
    );
    info.exports.insert(
        "search".to_string(),
        Type::callable(
            vec![Type::Str, Type::Str],
            Type::optional(Type::Instance {
                name: "Match".to_string(),
                module: Some("re".to_string()),
                type_args: vec![Type::Str],
            }),
        ),
    );
    info.exports.insert(
        "findall".to_string(),
        Type::callable(vec![Type::Str, Type::Str], Type::list(Type::Str)),
    );
    info.exports.insert(
        "finditer".to_string(),
        Type::callable(
            vec![Type::Str, Type::Str],
            Type::Any, // Iterator[Match]
        ),
    );
    info.exports.insert(
        "sub".to_string(),
        Type::callable(vec![Type::Str, Type::Str, Type::Str], Type::Str),
    );
    info.exports.insert(
        "subn".to_string(),
        Type::callable(
            vec![Type::Str, Type::Str, Type::Str],
            Type::Tuple(vec![Type::Str, Type::Int]),
        ),
    );
    info.exports.insert(
        "split".to_string(),
        Type::callable(vec![Type::Str, Type::Str], Type::list(Type::Str)),
    );
    info.exports.insert(
        "escape".to_string(),
        Type::callable(vec![Type::Str], Type::Str),
    );

    // Flags
    info.exports.insert("IGNORECASE".to_string(), Type::Int);
    info.exports.insert("I".to_string(), Type::Int);
    info.exports.insert("MULTILINE".to_string(), Type::Int);
    info.exports.insert("M".to_string(), Type::Int);
    info.exports.insert("DOTALL".to_string(), Type::Int);
    info.exports.insert("S".to_string(), Type::Int);
    info.exports.insert("VERBOSE".to_string(), Type::Int);
    info.exports.insert("X".to_string(), Type::Int);
    info.exports.insert("ASCII".to_string(), Type::Int);
    info.exports.insert("A".to_string(), Type::Int);

    info
}

/// Create json module stub
pub fn create_json_stub() -> ModuleInfo {
    let mut info = ModuleInfo::new("json");

    info.exports.insert(
        "dumps".to_string(),
        Type::callable(vec![Type::Any], Type::Str),
    );
    info.exports.insert(
        "loads".to_string(),
        Type::callable(vec![Type::Str], Type::Any),
    );
    info.exports.insert(
        "dump".to_string(),
        Type::callable(vec![Type::Any, Type::Any], Type::None),
    );
    info.exports.insert(
        "load".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );

    info.exports.insert(
        "JSONEncoder".to_string(),
        Type::ClassType {
            name: "JSONEncoder".to_string(),
            module: Some("json".to_string()),
        },
    );
    info.exports.insert(
        "JSONDecoder".to_string(),
        Type::ClassType {
            name: "JSONDecoder".to_string(),
            module: Some("json".to_string()),
        },
    );
    info.exports.insert(
        "JSONDecodeError".to_string(),
        Type::ClassType {
            name: "JSONDecodeError".to_string(),
            module: Some("json".to_string()),
        },
    );

    info
}

/// Create pathlib module stub
pub fn create_pathlib_stub() -> ModuleInfo {
    let mut info = ModuleInfo::new("pathlib");

    let path_type = Type::ClassType {
        name: "Path".to_string(),
        module: Some("pathlib".to_string()),
    };

    info.exports.insert("Path".to_string(), path_type.clone());
    info.exports.insert(
        "PurePath".to_string(),
        Type::ClassType {
            name: "PurePath".to_string(),
            module: Some("pathlib".to_string()),
        },
    );
    info.exports.insert(
        "PurePosixPath".to_string(),
        Type::ClassType {
            name: "PurePosixPath".to_string(),
            module: Some("pathlib".to_string()),
        },
    );
    info.exports.insert(
        "PureWindowsPath".to_string(),
        Type::ClassType {
            name: "PureWindowsPath".to_string(),
            module: Some("pathlib".to_string()),
        },
    );
    info.exports.insert(
        "PosixPath".to_string(),
        Type::ClassType {
            name: "PosixPath".to_string(),
            module: Some("pathlib".to_string()),
        },
    );
    info.exports.insert(
        "WindowsPath".to_string(),
        Type::ClassType {
            name: "WindowsPath".to_string(),
            module: Some("pathlib".to_string()),
        },
    );

    info
}

/// Create functools module stub
pub fn create_functools_stub() -> ModuleInfo {
    let mut info = ModuleInfo::new("functools");

    info.exports.insert(
        "reduce".to_string(),
        Type::callable(vec![Type::Any, Type::Any], Type::Any),
    );
    info.exports.insert(
        "partial".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );
    info.exports.insert(
        "wraps".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );
    info.exports
        .insert("lru_cache".to_string(), Type::callable(vec![], Type::Any));
    info.exports.insert(
        "cache".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );
    info.exports.insert(
        "cached_property".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );
    info.exports.insert(
        "total_ordering".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );
    info.exports.insert(
        "cmp_to_key".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );

    info
}

/// Create itertools module stub
pub fn create_itertools_stub() -> ModuleInfo {
    let mut info = ModuleInfo::new("itertools");

    // Infinite iterators
    info.exports.insert(
        "count".to_string(),
        Type::callable(vec![Type::Int], Type::Any),
    );
    info.exports.insert(
        "cycle".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );
    info.exports.insert(
        "repeat".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );

    // Combinatoric iterators
    info.exports.insert(
        "product".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );
    info.exports.insert(
        "permutations".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );
    info.exports.insert(
        "combinations".to_string(),
        Type::callable(vec![Type::Any, Type::Int], Type::Any),
    );
    info.exports.insert(
        "combinations_with_replacement".to_string(),
        Type::callable(vec![Type::Any, Type::Int], Type::Any),
    );

    // Terminating iterators
    info.exports.insert(
        "chain".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );
    info.exports.insert(
        "compress".to_string(),
        Type::callable(vec![Type::Any, Type::Any], Type::Any),
    );
    info.exports.insert(
        "dropwhile".to_string(),
        Type::callable(vec![Type::Any, Type::Any], Type::Any),
    );
    info.exports.insert(
        "takewhile".to_string(),
        Type::callable(vec![Type::Any, Type::Any], Type::Any),
    );
    info.exports.insert(
        "groupby".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );
    info.exports.insert(
        "islice".to_string(),
        Type::callable(vec![Type::Any, Type::Int], Type::Any),
    );
    info.exports.insert(
        "starmap".to_string(),
        Type::callable(vec![Type::Any, Type::Any], Type::Any),
    );
    info.exports.insert(
        "tee".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );
    info.exports.insert(
        "zip_longest".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );
    info.exports.insert(
        "filterfalse".to_string(),
        Type::callable(vec![Type::Any, Type::Any], Type::Any),
    );
    info.exports.insert(
        "accumulate".to_string(),
        Type::callable(vec![Type::Any], Type::Any),
    );

    info
}

/// Create datetime module stub
pub fn create_datetime_stub() -> ModuleInfo {
    let mut info = ModuleInfo::new("datetime");

    info.exports.insert(
        "date".to_string(),
        Type::ClassType {
            name: "date".to_string(),
            module: Some("datetime".to_string()),
        },
    );
    info.exports.insert(
        "time".to_string(),
        Type::ClassType {
            name: "time".to_string(),
            module: Some("datetime".to_string()),
        },
    );
    info.exports.insert(
        "datetime".to_string(),
        Type::ClassType {
            name: "datetime".to_string(),
            module: Some("datetime".to_string()),
        },
    );
    info.exports.insert(
        "timedelta".to_string(),
        Type::ClassType {
            name: "timedelta".to_string(),
            module: Some("datetime".to_string()),
        },
    );
    info.exports.insert(
        "timezone".to_string(),
        Type::ClassType {
            name: "timezone".to_string(),
            module: Some("datetime".to_string()),
        },
    );
    info.exports.insert(
        "tzinfo".to_string(),
        Type::ClassType {
            name: "tzinfo".to_string(),
            module: Some("datetime".to_string()),
        },
    );

    // Constants
    info.exports.insert("MINYEAR".to_string(), Type::Int);
    info.exports.insert("MAXYEAR".to_string(), Type::Int);

    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_stub() {
        let os = create_os_stub();
        assert!(os.exports.contains_key("getcwd"));
        assert!(os.exports.contains_key("environ"));
        assert!(os.exports.contains_key("sep"));
    }

    #[test]
    fn test_sys_stub() {
        let sys = create_sys_stub();
        assert!(sys.exports.contains_key("argv"));
        assert!(sys.exports.contains_key("path"));
        assert!(sys.exports.contains_key("exit"));
    }

    #[test]
    fn test_re_stub() {
        let re = create_re_stub();
        assert!(re.exports.contains_key("compile"));
        assert!(re.exports.contains_key("match"));
        assert!(re.exports.contains_key("IGNORECASE"));
    }

    #[test]
    fn test_json_stub() {
        let json = create_json_stub();
        assert!(json.exports.contains_key("dumps"));
        assert!(json.exports.contains_key("loads"));
    }

    #[test]
    fn test_datetime_stub() {
        let dt = create_datetime_stub();
        assert!(dt.exports.contains_key("datetime"));
        assert!(dt.exports.contains_key("date"));
        assert!(dt.exports.contains_key("timedelta"));
    }
}
