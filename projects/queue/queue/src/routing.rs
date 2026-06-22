//! Task routing system for directing tasks to specific queues.
//!
//! Similar to Celery's CELERY_ROUTES, this module allows routing tasks
//! to different queues based on task name, patterns, or custom logic.
//!
//! # Example
//! ```rust,ignore
//! use cclab_queue::routing::{Router, RouterConfig, Route};
//!
//! // Simple route configuration
//! let config = RouterConfig::new()
//!     .route("send_email", "email")           // Exact match
//!     .route_glob("tasks.math.*", "math")     // Glob pattern
//!     .route_fn("urgent_*", |task, _args| {   // Custom logic
//!         if task.contains("urgent") {
//!             Some("high-priority".to_string())
//!         } else {
//!             None
//!         }
//!     })
//!     .default_queue("default");
//!
//! let router = Router::new(config);
//! let queue = router.route("send_email", &args);  // Returns "email"
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Route definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// Pattern to match (exact, glob, or regex)
    pub pattern: String,
    /// Target queue name
    pub queue: String,
    /// Pattern type
    #[serde(default)]
    pub pattern_type: PatternType,
}

/// Pattern matching type
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PatternType {
    /// Exact string match
    #[default]
    Exact,
    /// Glob pattern (e.g., "tasks.*", "email.send.*")
    Glob,
    /// Regex pattern
    Regex,
}

/// Custom routing function type
pub type RouteFn = Arc<dyn Fn(&str, &serde_json::Value) -> Option<String> + Send + Sync>;

/// Router configuration builder
#[derive(Default)]
pub struct RouterConfig {
    routes: Vec<Route>,
    custom_routes: Vec<(String, RouteFn)>,
    default_queue: String,
}

impl RouterConfig {
    /// Create a new router configuration
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            custom_routes: Vec::new(),
            default_queue: "default".to_string(),
        }
    }

    /// Add an exact match route
    pub fn route(mut self, task_name: &str, queue: &str) -> Self {
        self.routes.push(Route {
            pattern: task_name.to_string(),
            queue: queue.to_string(),
            pattern_type: PatternType::Exact,
        });
        self
    }

    /// Add a glob pattern route (e.g., "tasks.math.*")
    pub fn route_glob(mut self, pattern: &str, queue: &str) -> Self {
        self.routes.push(Route {
            pattern: pattern.to_string(),
            queue: queue.to_string(),
            pattern_type: PatternType::Glob,
        });
        self
    }

    /// Add a regex pattern route
    pub fn route_regex(mut self, pattern: &str, queue: &str) -> Self {
        self.routes.push(Route {
            pattern: pattern.to_string(),
            queue: queue.to_string(),
            pattern_type: PatternType::Regex,
        });
        self
    }

    /// Add a custom routing function
    pub fn route_fn<F>(mut self, name: &str, f: F) -> Self
    where
        F: Fn(&str, &serde_json::Value) -> Option<String> + Send + Sync + 'static,
    {
        self.custom_routes.push((name.to_string(), Arc::new(f)));
        self
    }

    /// Set the default queue for unmatched tasks
    pub fn default_queue(mut self, queue: &str) -> Self {
        self.default_queue = queue.to_string();
        self
    }

    /// Build the router
    pub fn build(self) -> Router {
        Router::new(self)
    }
}

/// Task router
pub struct Router {
    routes: Vec<Route>,
    custom_routes: Vec<(String, RouteFn)>,
    default_queue: String,
    // Compiled regex patterns (lazy)
    regex_cache: std::sync::RwLock<HashMap<String, regex::Regex>>,
}

impl Router {
    /// Create a new router from configuration
    pub fn new(config: RouterConfig) -> Self {
        Self {
            routes: config.routes,
            custom_routes: config.custom_routes,
            default_queue: config.default_queue,
            regex_cache: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Create a router with no custom routes (for deserialization)
    pub fn from_routes(routes: Vec<Route>, default_queue: String) -> Self {
        Self {
            routes,
            custom_routes: Vec::new(),
            default_queue,
            regex_cache: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Route a task to a queue
    ///
    /// Returns the target queue name based on routing rules.
    /// Falls back to default queue if no route matches.
    pub fn route(&self, task_name: &str, args: &serde_json::Value) -> String {
        // Try custom routes first
        for (_, route_fn) in &self.custom_routes {
            if let Some(queue) = route_fn(task_name, args) {
                return queue;
            }
        }

        // Try pattern routes
        for route in &self.routes {
            if self.matches(&route.pattern, &route.pattern_type, task_name) {
                return route.queue.clone();
            }
        }

        // Fall back to default
        self.default_queue.clone()
    }

    /// Check if a pattern matches a task name
    fn matches(&self, pattern: &str, pattern_type: &PatternType, task_name: &str) -> bool {
        match pattern_type {
            PatternType::Exact => pattern == task_name,
            PatternType::Glob => self.glob_match(pattern, task_name),
            PatternType::Regex => self.regex_match(pattern, task_name),
        }
    }

    /// Glob pattern matching
    fn glob_match(&self, pattern: &str, task_name: &str) -> bool {
        // Simple glob: * matches any sequence, ? matches single char
        let regex_pattern = pattern
            .replace('.', "\\.")
            .replace('*', ".*")
            .replace('?', ".");

        if let Ok(re) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
            re.is_match(task_name)
        } else {
            false
        }
    }

    /// Regex pattern matching
    fn regex_match(&self, pattern: &str, task_name: &str) -> bool {
        // Check cache first
        {
            let cache = self.regex_cache.read().unwrap();
            if let Some(re) = cache.get(pattern) {
                return re.is_match(task_name);
            }
        }

        // Compile and cache
        if let Ok(re) = regex::Regex::new(pattern) {
            let matches = re.is_match(task_name);
            self.regex_cache
                .write()
                .unwrap()
                .insert(pattern.to_string(), re);
            matches
        } else {
            false
        }
    }

    /// Get the default queue name
    pub fn default_queue(&self) -> &str {
        &self.default_queue
    }

    /// Get all configured routes
    pub fn routes(&self) -> &[Route] {
        &self.routes
    }
}

impl Default for Router {
    fn default() -> Self {
        Self {
            routes: Vec::new(),
            custom_routes: Vec::new(),
            default_queue: "default".to_string(),
            regex_cache: std::sync::RwLock::new(HashMap::new()),
        }
    }
}

// Allow Router to be shared across threads
unsafe impl Send for Router {}
unsafe impl Sync for Router {}

/// Routes configuration for serialization
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoutesConfig {
    /// List of routes
    #[serde(default)]
    pub routes: Vec<Route>,
    /// Default queue
    #[serde(default = "default_queue_name")]
    pub default_queue: String,
}

fn default_queue_name() -> String {
    "default".to_string()
}

impl RoutesConfig {
    /// Load routes from environment variable
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        if let Ok(json) = std::env::var("TASK_ROUTES") {
            Ok(serde_json::from_str(&json)?)
        } else {
            Ok(Self::default())
        }
    }

    /// Convert to Router
    pub fn into_router(self) -> Router {
        Router::from_routes(self.routes, self.default_queue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_route() {
        let router = RouterConfig::new()
            .route("send_email", "email")
            .route("process_payment", "payments")
            .build();

        assert_eq!(router.route("send_email", &serde_json::json!({})), "email");
        assert_eq!(
            router.route("process_payment", &serde_json::json!({})),
            "payments"
        );
        assert_eq!(
            router.route("unknown_task", &serde_json::json!({})),
            "default"
        );
    }

    #[test]
    fn test_glob_route() {
        let router = RouterConfig::new()
            .route_glob("email.*", "email")
            .route_glob("math.*", "math")
            .route_glob("tasks.*.urgent", "high-priority")
            .build();

        assert_eq!(router.route("email.send", &serde_json::json!({})), "email");
        assert_eq!(
            router.route("email.receive", &serde_json::json!({})),
            "email"
        );
        assert_eq!(router.route("math.add", &serde_json::json!({})), "math");
        assert_eq!(
            router.route("tasks.email.urgent", &serde_json::json!({})),
            "high-priority"
        );
        assert_eq!(
            router.route("other.task", &serde_json::json!({})),
            "default"
        );
    }

    #[test]
    fn test_regex_route() {
        let router = RouterConfig::new()
            .route_regex(r"^user_\d+$", "users")
            .route_regex(r"^report_.*_monthly$", "reports")
            .build();

        assert_eq!(router.route("user_123", &serde_json::json!({})), "users");
        assert_eq!(router.route("user_456", &serde_json::json!({})), "users");
        assert_eq!(
            router.route("report_sales_monthly", &serde_json::json!({})),
            "reports"
        );
        assert_eq!(router.route("user_abc", &serde_json::json!({})), "default");
    }

    #[test]
    fn test_custom_route_fn() {
        let router = RouterConfig::new()
            .route_fn("priority_router", |task_name, args| {
                // Route based on args
                if let Some(priority) = args.get("priority").and_then(|v| v.as_str()) {
                    if priority == "high" {
                        return Some("high-priority".to_string());
                    }
                }
                // Route based on task name
                if task_name.starts_with("urgent_") {
                    return Some("urgent".to_string());
                }
                None
            })
            .build();

        assert_eq!(
            router.route("any_task", &serde_json::json!({"priority": "high"})),
            "high-priority"
        );
        assert_eq!(
            router.route("urgent_task", &serde_json::json!({})),
            "urgent"
        );
        assert_eq!(
            router.route("normal_task", &serde_json::json!({})),
            "default"
        );
    }

    #[test]
    fn test_route_priority() {
        // Custom routes take precedence over pattern routes
        let router = RouterConfig::new()
            .route_fn("custom", |task_name, _| {
                if task_name == "special_task" {
                    Some("custom-queue".to_string())
                } else {
                    None
                }
            })
            .route("special_task", "pattern-queue")
            .build();

        // Custom route wins
        assert_eq!(
            router.route("special_task", &serde_json::json!({})),
            "custom-queue"
        );
    }

    #[test]
    fn test_default_queue() {
        let router = RouterConfig::new().default_queue("my-default").build();

        assert_eq!(
            router.route("any_task", &serde_json::json!({})),
            "my-default"
        );
    }

    #[test]
    fn test_routes_config_serde() {
        let config = RoutesConfig {
            routes: vec![
                Route {
                    pattern: "email.*".to_string(),
                    queue: "email".to_string(),
                    pattern_type: PatternType::Glob,
                },
                Route {
                    pattern: "process_payment".to_string(),
                    queue: "payments".to_string(),
                    pattern_type: PatternType::Exact,
                },
            ],
            default_queue: "default".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: RoutesConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.routes.len(), 2);
        assert_eq!(parsed.routes[0].pattern, "email.*");
        assert_eq!(parsed.routes[0].pattern_type, PatternType::Glob);
    }

    #[test]
    fn test_routes_from_json() {
        let json = r#"{
            "routes": [
                {"pattern": "email.*", "queue": "email", "pattern_type": "glob"},
                {"pattern": "^user_\\d+$", "queue": "users", "pattern_type": "regex"}
            ],
            "default_queue": "worker"
        }"#;

        let config: RoutesConfig = serde_json::from_str(json).unwrap();
        let router = config.into_router();

        assert_eq!(router.route("email.send", &serde_json::json!({})), "email");
        assert_eq!(router.route("user_123", &serde_json::json!({})), "users");
        assert_eq!(router.route("other", &serde_json::json!({})), "worker");
    }
}
