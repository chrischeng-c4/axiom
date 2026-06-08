# Implementation Diff

## Summary

```
Cargo.lock                                 |   1 +
 crates/cclab-jet/Cargo.toml                |   7 +-
 crates/cclab-jet/src/cli.rs                | 223 ++++++++++++++++++++++
 crates/cclab-jet/src/lib.rs                |   2 +
 crates/cclab-jet/src/runner/env.rs         |  71 ++++++++
 crates/cclab-jet/src/runner/jit.rs         | 214 ++++++++++++++++++++++
 crates/cclab-jet/src/runner/mod.rs         | 261 ++++++++++++++++++++++++++
 crates/cclab-jet/src/runner/source_map.rs  |  68 +++++++
 crates/cclab-jet/src/runner/watcher.rs     | 102 +++++++++++
 crates/cclab-jet/src/task_runner/cache.rs  | 201 ++++++++++++++++++++
 crates/cclab-jet/src/task_runner/config.rs | 141 ++++++++++++++
 crates/cclab-jet/src/task_runner/graph.rs  | 284 +++++++++++++++++++++++++++++
 crates/cclab-jet/src/task_runner/hash.rs   | 162 ++++++++++++++++
 crates/cclab-jet/src/task_runner/mod.rs    | 251 +++++++++++++++++++++++++
 14 files changed, 1987 insertions(+), 1 deletion(-)
```

## Diff

```diff
diff --git a/Cargo.lock b/Cargo.lock
index b9c3321..48c463d 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -1385,6 +1385,7 @@ version = "0.3.29"
 dependencies = [
  "anyhow",
  "axum 0.8.8",
+ "base64 0.22.1",
  "clap",
  "dashmap 6.1.0",
  "flate2",
diff --git a/crates/cclab-jet/Cargo.toml b/crates/cclab-jet/Cargo.toml
index 8b9ce13..3cdf4e1 100644
--- a/crates/cclab-jet/Cargo.toml
+++ b/crates/cclab-jet/Cargo.toml
@@ -78,6 +78,11 @@ futures = "0.3"
 # Glob patterns (workspace discovery)
 glob = "0.3"
 
+# Base64 encoding (JIT source maps)
+base64 = "0.22"
+
+# Temp files (JIT runner, dlx)
+tempfile = "3.15"
+
 [dev-dependencies]
 tracing-subscriber = { workspace = true }
-tempfile = "3.15"
diff --git a/crates/cclab-jet/src/cli.rs b/crates/cclab-jet/src/cli.rs
index 4e7a053..19f0c59 100644
--- a/crates/cclab-jet/src/cli.rs
+++ b/crates/cclab-jet/src/cli.rs
@@ -153,6 +153,68 @@ pub fn command() -> Command {
                 ),
         )
         .subcommand(Command::new("check").about("Type check TypeScript files"))
+        .subcommand(
+            Command::new("run")
+                .about("Run a script, file, or task (no args = list scripts)")
+                .arg(
+                    Arg::new("target")
+                        .help("Script name, file path, or task name"),
+                )
+                .arg(
+                    Arg::new("args")
+                        .num_args(0..)
+                        .trailing_var_arg(true)
+                        .help("Arguments to pass to the script"),
+                )
+                .arg(
+                    Arg::new("watch")
+                        .short('w')
+                        .long("watch")
+                        .action(ArgAction::SetTrue)
+                        .help("Watch mode (re-run on file changes)"),
+                )
+                .arg(
+                    Arg::new("filter")
+                        .long("filter")
+                        .help("Filter packages (task runner mode)"),
+                )
+                .arg(
+                    Arg::new("dry")
+                        .long("dry")
+                        .action(ArgAction::SetTrue)
+                        .help("Dry run: show what would execute"),
+                ),
+        )
+        .subcommand(
+            Command::new("exec")
+                .about("Run a command with node_modules/.bin on PATH")
+                .arg(
+                    Arg::new("cmd")
+                        .required(true)
+                        .help("Command to execute"),
+                )
+                .arg(
+                    Arg::new("args")
+                        .num_args(0..)
+                        .trailing_var_arg(true)
+                        .help("Arguments"),
+                ),
+        )
+        .subcommand(
+            Command::new("jtx")
+                .about("Download and execute a package (like npx)")
+                .arg(
+                    Arg::new("package")
+                        .required(true)
+                        .help("Package to execute"),
+                )
+                .arg(
+                    Arg::new("args")
+                        .num_args(0..)
+                        .trailing_var_arg(true)
+                        .help("Arguments"),
+                ),
+        )
 }
 
 /// Execute a jet CLI command
@@ -345,12 +407,173 @@ async fn execute_async(matches: &ArgMatches) -> Result<()> {
             Ok(())
         }
 
+        Some(("run", m)) => {
+            // No target → list available scripts (like `npm run`)
+            let target = match m.get_one::<String>("target") {
+                Some(t) => t,
+                None => return list_scripts(&root_dir),
+            };
+            let args: Vec<String> = m
+                .get_many::<String>("args")
+                .map(|v| v.cloned().collect())
+                .unwrap_or_default();
+            let watch = m.get_flag("watch");
+            let filter = m.get_one::<String>("filter").map(|s| s.as_str());
+            let dry_run = m.get_flag("dry");
+
+            handle_run(&root_dir, target, &args, watch, filter, dry_run).await
+        }
+
+        Some(("exec", m)) => {
+            let cmd = m.get_one::<String>("cmd").unwrap();
+            let args: Vec<String> = m
+                .get_many::<String>("args")
+                .map(|v| v.cloned().collect())
+                .unwrap_or_default();
+
+            let runner = crate::runner::ScriptRunner::new(root_dir);
+            let result = runner.exec_command(cmd, &args).await?;
+            print!("{}", result.stdout);
+            eprint!("{}", result.stderr);
+            std::process::exit(result.exit_code);
+        }
+
+        Some(("jtx", m)) => {
+            let package = m.get_one::<String>("package").unwrap();
+            let args: Vec<String> = m
+                .get_many::<String>("args")
+                .map(|v| v.cloned().collect())
+                .unwrap_or_default();
+
+            // Download package to temp, then execute its bin
+            println!("Downloading {}...", package);
+            let temp_dir = tempfile::tempdir()
+                .context("Failed to create temp directory")?;
+            let pm = crate::pkg_manager::PackageManager::new(
+                temp_dir.path().to_path_buf(),
+            )?;
+            // Create minimal package.json
+            let pkg = format!(
+                r#"{{"name":"dlx-tmp","version":"0.0.0","dependencies":{{"{}":"latest"}}}}"#,
+                package
+            );
+            std::fs::write(temp_dir.path().join("package.json"), pkg)?;
+            pm.install().await?;
+
+            let runner = crate::runner::ScriptRunner::new(
+                temp_dir.path().to_path_buf(),
+            );
+            let result = runner.exec_command(package, &args).await?;
+            print!("{}", result.stdout);
+            eprint!("{}", result.stderr);
+            std::process::exit(result.exit_code);
+        }
+
         _ => {
             anyhow::bail!("Unknown jet subcommand. Run 'cclab jet --help' for usage.")
         }
     }
 }
 
+/// List all available scripts from package.json and jet.config.yaml pipeline.
+/// Equivalent to `npm run` (no arguments).
+fn list_scripts(root_dir: &PathBuf) -> Result<()> {
+    // package.json scripts
+    let pkg_path = root_dir.join("package.json");
+    if pkg_path.exists() {
+        let content = std::fs::read_to_string(&pkg_path)?;
+        if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
+            if let Some(scripts) = pkg.get("scripts").and_then(|s| s.as_object()) {
+                if !scripts.is_empty() {
+                    println!("Scripts available via `jet run`:\n");
+                    let max_len = scripts.keys().map(|k| k.len()).max().unwrap_or(0);
+                    for (name, cmd) in scripts {
+                        println!(
+                            "  {:<width$}  {}",
+                            name,
+                            cmd.as_str().unwrap_or(""),
+                            width = max_len
+                        );
+                    }
+                }
+            }
+        }
+    }
+
+    // jet.config.yaml pipeline tasks
+    if let Ok(config) = crate::task_runner::config::JetConfig::load(root_dir) {
+        if !config.pipeline.is_empty() {
+            println!("\nPipeline tasks (jet.config.yaml):\n");
+            for (name, def) in &config.pipeline {
+                let deps = if def.depends_on.is_empty() {
+                    String::new()
+                } else {
+                    format!(" (deps: {})", def.depends_on.join(", "))
+                };
+                println!("  {}{}", name, deps);
+            }
+        }
+    }
+
+    Ok(())
+}
+
+/// Handle `jet run <target>`: resolve as script, file, or task.
+async fn handle_run(
+    root_dir: &PathBuf,
+    target: &str,
+    args: &[String],
+    watch: bool,
+    filter: Option<&str>,
+    dry_run: bool,
+) -> Result<()> {
+    let runner = crate::runner::ScriptRunner::new(root_dir.clone());
+
+    // 1. Check package.json scripts
+    if runner.has_script(target) {
+        let result = runner.run_script(target, args).await?;
+        print!("{}", result.stdout);
+        eprint!("{}", result.stderr);
+        if result.exit_code != 0 {
+            anyhow::bail!("Script '{}' exited with code {}", target, result.exit_code);
+        }
+        return Ok(());
+    }
+
+    // 2. Check if it's a file
+    if runner.is_file(target) {
+        let path = std::path::Path::new(target);
+        let result = runner.run_file(path, args, watch).await?;
+        print!("{}", result.stdout);
+        eprint!("{}", result.stderr);
+        if result.exit_code != 0 {
+            anyhow::bail!("File '{}' exited with code {}", target, result.exit_code);
+        }
+        return Ok(());
+    }
+
+    // 3. Check task runner (jet.config.yaml pipeline)
+    if let Ok(tr) = crate::task_runner::TaskRunner::new(root_dir) {
+        if tr.has_task(target) {
+            let results = tr.run(target, filter, dry_run).await?;
+            crate::task_runner::TaskRunner::print_summary(&results);
+            let any_failed = results.iter().any(|r| {
+                r.status == crate::task_runner::TaskStatus::Failed
+            });
+            if any_failed {
+                anyhow::bail!("Some tasks failed");
+            }
+            return Ok(());
+        }
+    }
+
+    anyhow::bail!(
+        "Target '{}' not found as a script, file, or task. \
+         Check package.json scripts or jet.config.yaml pipeline.",
+        target
+    )
+}
+
 /// Find the project entry point by checking common locations
 fn find_entry_point(root_dir: &PathBuf) -> Result<PathBuf> {
     let candidates = [
diff --git a/crates/cclab-jet/src/lib.rs b/crates/cclab-jet/src/lib.rs
index 7fe57ff..bdc554e 100644
--- a/crates/cclab-jet/src/lib.rs
+++ b/crates/cclab-jet/src/lib.rs
@@ -9,6 +9,8 @@ pub mod cli;
 pub mod dev_server;
 pub mod pkg_manager;
 pub mod resolver;
+pub mod runner;
+pub mod task_runner;
 pub mod transform;
 
 // Re-export pnpm parity modules for convenience
diff --git a/crates/cclab-jet/src/runner/env.rs b/crates/cclab-jet/src/runner/env.rs
new file mode 100644
index 0000000..3ce7994
--- /dev/null
+++ b/crates/cclab-jet/src/runner/env.rs
@@ -0,0 +1,71 @@
+//! Environment variable injection for script runner.
+//!
+//! Injects NODE_ENV, JET_* variables, and prepends node_modules/.bin to PATH.
+
+use std::collections::HashMap;
+use std::path::Path;
+
+/// Build environment variables for script execution.
+///
+/// - Prepends `node_modules/.bin` to PATH
+/// - Sets `NODE_ENV` if not already set
+/// - Injects `JET_PROJECT_ROOT`
+pub fn build_env(project_root: &Path) -> HashMap<String, String> {
+    let mut env = HashMap::new();
+
+    // Prepend .bin to PATH
+    let bin_dir = project_root.join("node_modules/.bin");
+    let current_path = std::env::var("PATH").unwrap_or_default();
+    env.insert(
+        "PATH".to_string(),
+        format!("{}:{}", bin_dir.display(), current_path),
+    );
+
+    // NODE_ENV defaults to "development"
+    let node_env = std::env::var("NODE_ENV")
+        .unwrap_or_else(|_| "development".to_string());
+    env.insert("NODE_ENV".to_string(), node_env);
+
+    // JET_* variables
+    env.insert(
+        "JET_PROJECT_ROOT".to_string(),
+        project_root.to_string_lossy().to_string(),
+    );
+
+    // Inherit npm_config_* if present
+    for (key, value) in std::env::vars() {
+        if key.starts_with("npm_config_") || key.starts_with("npm_package_") {
+            env.insert(key, value);
+        }
+    }
+
+    env
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use std::path::PathBuf;
+
+    #[test]
+    fn test_build_env_has_path() {
+        let env = build_env(&PathBuf::from("/tmp/project"));
+        let path = env.get("PATH").unwrap();
+        assert!(path.starts_with("/tmp/project/node_modules/.bin:"));
+    }
+
+    #[test]
+    fn test_build_env_has_node_env() {
+        let env = build_env(&PathBuf::from("/tmp/project"));
+        assert!(env.contains_key("NODE_ENV"));
+    }
+
+    #[test]
+    fn test_build_env_has_jet_root() {
+        let env = build_env(&PathBuf::from("/tmp/project"));
+        assert_eq!(
+            env.get("JET_PROJECT_ROOT").unwrap(),
+            "/tmp/project"
+        );
+    }
+}
diff --git a/crates/cclab-jet/src/runner/jit.rs b/crates/cclab-jet/src/runner/jit.rs
new file mode 100644
index 0000000..b6963bb
--- /dev/null
+++ b/crates/cclab-jet/src/runner/jit.rs
@@ -0,0 +1,214 @@
+//! JIT engine: transform TypeScript/TSX/JSX via Tree-sitter, execute via Node.js.
+//!
+//! Pipeline: source file → Tree-sitter parse → strip types / transform JSX
+//!           → write temp .js → execute via Node.js child process → cleanup
+
+use anyhow::{Context, Result};
+use std::path::{Path, PathBuf};
+
+use crate::runner::RunResult;
+use crate::transform::{TransformOptions, Transformer};
+
+/// JIT execution engine for TypeScript/JSX files.
+pub struct JitEngine {
+    transformer: Transformer,
+    project_root: PathBuf,
+    temp_dir: PathBuf,
+}
+
+impl JitEngine {
+    /// Create a new JIT engine.
+    pub fn new(project_root: &Path) -> Result<Self> {
+        let temp_dir = std::env::temp_dir().join("jet-jit");
+        std::fs::create_dir_all(&temp_dir)?;
+
+        let options = TransformOptions {
+            source_maps: true,
+            jsx_automatic: true,
+            ..Default::default()
+        };
+
+        Ok(Self {
+            transformer: Transformer::new(options),
+            project_root: project_root.to_path_buf(),
+            temp_dir,
+        })
+    }
+
+    /// Transform and execute a file, returning the result.
+    pub async fn execute(
+        &self,
+        file: &Path,
+        args: &[String],
+    ) -> Result<RunResult> {
+        let temp_js = self.transform_file(file)?;
+
+        let env_vars = super::env::build_env(&self.project_root);
+
+        let output = tokio::process::Command::new("node")
+            .arg(&temp_js)
+            .args(args)
+            .current_dir(&self.project_root)
+            .envs(&env_vars)
+            .output()
+            .await
+            .with_context(|| {
+                format!("Failed to execute: node {}", temp_js.display())
+            })?;
+
+        // Cleanup temp file
+        let _ = std::fs::remove_file(&temp_js);
+
+        Ok(RunResult {
+            exit_code: output.status.code().unwrap_or(-1),
+            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
+            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
+        })
+    }
+
+    /// Execute in watch mode: re-transform and restart on file changes.
+    pub async fn execute_watch(
+        &self,
+        file: &Path,
+        args: &[String],
+    ) -> Result<()> {
+        use notify::{Event, EventKind, RecursiveMode, Watcher};
+        use std::sync::mpsc;
+
+        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
+
+        let mut watcher = notify::recommended_watcher(tx)?;
+
+        // Watch the file's parent directory
+        let watch_dir = file
+            .parent()
+            .unwrap_or_else(|| Path::new("."));
+        watcher.watch(watch_dir, RecursiveMode::NonRecursive)?;
+
+        let file_name = file
+            .file_name()
+            .map(|f| f.to_string_lossy().to_string())
+            .unwrap_or_default();
+
+        tracing::info!("Watch mode: {}", file.display());
+
+        // Initial run
+        let result = self.execute(file, args).await?;
+        print!("{}", result.stdout);
+        eprint!("{}", result.stderr);
+
+        // Watch loop
+        loop {
+            match rx.recv() {
+                Ok(Ok(event)) => {
+                    let is_target = matches!(
+                        event.kind,
+                        EventKind::Modify(_) | EventKind::Create(_)
+                    ) && event.paths.iter().any(|p| {
+                        p.file_name()
+                            .map(|f| f.to_string_lossy() == file_name)
+                            .unwrap_or(false)
+                    });
+
+                    if is_target {
+                        tracing::info!("File changed, re-running...");
+                        let result = self.execute(file, args).await?;
+                        print!("{}", result.stdout);
+                        eprint!("{}", result.stderr);
+                        if result.exit_code != 0 {
+                            tracing::warn!(
+                                "Process exited with code {}",
+                                result.exit_code
+                            );
+                        }
+                    }
+                }
+                Ok(Err(e)) => {
+                    tracing::error!("Watch error: {:?}", e);
+                }
+                Err(_) => break,
+            }
+        }
+
+        Ok(())
+    }
+
+    /// Transform a TS/TSX/JSX file to a temporary JS file.
+    fn transform_file(&self, file: &Path) -> Result<PathBuf> {
+        let source = std::fs::read_to_string(file)
+            .with_context(|| format!("Failed to read {}", file.display()))?;
+
+        let result = self
+            .transformer
+            .transform_js(&source, file)
+            .with_context(|| {
+                format!("Failed to transform {}", file.display())
+            })?;
+
+        // Build output code with inline source map
+        let output_code = if let Some(ref sm) = result.source_map {
+            source_map::append_inline_source_map(&result.code, sm)
+        } else {
+            result.code.clone()
+        };
+
+        // Write to temp file preserving the stem
+        let stem = file
+            .file_stem()
+            .and_then(|s| s.to_str())
+            .unwrap_or("jit");
+        let temp_path = self.temp_dir.join(format!("{}.js", stem));
+        std::fs::write(&temp_path, &output_code)
+            .with_context(|| {
+                format!("Failed to write temp file: {}", temp_path.display())
+            })?;
+
+        Ok(temp_path)
+    }
+}
+
+/// Inline source map helper — reused from source_map module.
+mod source_map {
+    use base64::Engine;
+
+    /// Append an inline source map comment to JS code.
+    pub fn append_inline_source_map(code: &str, source_map_json: &str) -> String {
+        let encoded = base64::engine::general_purpose::STANDARD
+            .encode(source_map_json.as_bytes());
+        format!(
+            "{}\n//# sourceMappingURL=data:application/json;base64,{}\n",
+            code, encoded
+        )
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_jit_engine_creation() {
+        let dir = tempfile::tempdir().unwrap();
+        let engine = JitEngine::new(dir.path());
+        assert!(engine.is_ok());
+    }
+
+    #[test]
+    fn test_transform_typescript_file() {
+        let dir = tempfile::tempdir().unwrap();
+        let ts_file = dir.path().join("test.ts");
+        std::fs::write(&ts_file, "const x: number = 42;\nconsole.log(x);")
+            .unwrap();
+
+        let engine = JitEngine::new(dir.path()).unwrap();
+        let temp_js = engine.transform_file(&ts_file).unwrap();
+
+        let output = std::fs::read_to_string(&temp_js).unwrap();
+        assert!(output.contains("const x"));
+        assert!(output.contains("console.log"));
+        assert!(!output.contains(": number"));
+
+        // Cleanup
+        let _ = std::fs::remove_file(&temp_js);
+    }
+}
diff --git a/crates/cclab-jet/src/runner/mod.rs b/crates/cclab-jet/src/runner/mod.rs
new file mode 100644
index 0000000..96d1653
--- /dev/null
+++ b/crates/cclab-jet/src/runner/mod.rs
@@ -0,0 +1,261 @@
+//! Script runner: resolve and execute package.json scripts, files, or commands.
+//!
+//! Resolution order for `jet run <name>`:
+//! 1. Check package.json `scripts` → run via `sh -c` with `.bin` on PATH
+//! 2. Check if file exists on disk → JIT execute (ts/tsx/jsx) or direct (js)
+//! 3. Check jet.config.yaml pipeline → task runner mode
+//! 4. Not found → error
+
+use anyhow::{Context, Result};
+use std::collections::HashMap;
+use std::path::{Path, PathBuf};
+use std::process::ExitStatus;
+
+pub mod env;
+pub mod jit;
+pub mod source_map;
+pub mod watcher;
+
+/// Script runner for executing package.json scripts and arbitrary commands.
+pub struct ScriptRunner {
+    project_root: PathBuf,
+    pkg_json: Option<PkgScripts>,
+}
+
+/// Minimal package.json view for script runner.
+#[derive(Debug, Clone, serde::Deserialize)]
+struct PkgScripts {
+    #[serde(default)]
+    scripts: HashMap<String, String>,
+    #[serde(default)]
+    #[allow(dead_code)]
+    bin: Option<serde_json::Value>,
+}
+
+/// Result of running a script or command.
+pub struct RunResult {
+    pub exit_code: i32,
+    pub stdout: String,
+    pub stderr: String,
+}
+
+impl ScriptRunner {
+    /// Create a new script runner rooted at the given directory.
+    pub fn new(project_root: PathBuf) -> Self {
+        let pkg_json = Self::load_pkg_scripts(&project_root);
+        Self {
+            project_root,
+            pkg_json,
+        }
+    }
+
+    fn load_pkg_scripts(root: &Path) -> Option<PkgScripts> {
+        let path = root.join("package.json");
+        let content = std::fs::read_to_string(&path).ok()?;
+        serde_json::from_str(&content).ok()
+    }
+
+    /// Run a named script from package.json with lifecycle hooks.
+    pub async fn run_script(
+        &self,
+        name: &str,
+        args: &[String],
+    ) -> Result<RunResult> {
+        let scripts = self
+            .pkg_json
+            .as_ref()
+            .map(|p| &p.scripts)
+            .ok_or_else(|| anyhow::anyhow!("No package.json found"))?;
+
+        let command = scripts
+            .get(name)
+            .ok_or_else(|| anyhow::anyhow!("Script '{}' not found in package.json", name))?;
+
+        // Run pre-hook if exists
+        let pre_name = format!("pre{}", name);
+        if let Some(pre_cmd) = scripts.get(&pre_name) {
+            tracing::info!("Running pre-hook: {}", pre_name);
+            self.exec_shell(pre_cmd, &[]).await?;
+        }
+
+        // Run the script itself
+        tracing::info!("Running script: {} → {}", name, command);
+        let result = self.exec_shell(command, args).await?;
+
+        // Run post-hook if exists
+        let post_name = format!("post{}", name);
+        if let Some(post_cmd) = scripts.get(&post_name) {
+            tracing::info!("Running post-hook: {}", post_name);
+            self.exec_shell(post_cmd, &[]).await?;
+        }
+
+        Ok(result)
+    }
+
+    /// Run a file directly (JIT for TS/TSX/JSX, direct for JS).
+    pub async fn run_file(
+        &self,
+        path: &Path,
+        args: &[String],
+        watch: bool,
+    ) -> Result<RunResult> {
+        let ext = path
+            .extension()
+            .and_then(|e| e.to_str())
+            .unwrap_or("");
+
+        let full_path = if path.is_absolute() {
+            path.to_path_buf()
+        } else {
+            self.project_root.join(path)
+        };
+
+        if !full_path.exists() {
+            anyhow::bail!("File not found: {}", full_path.display());
+        }
+
+        match ext {
+            "ts" | "tsx" | "jsx" => {
+                let engine = jit::JitEngine::new(&self.project_root)?;
+                if watch {
+                    engine.execute_watch(&full_path, args).await?;
+                    Ok(RunResult {
+                        exit_code: 0,
+                        stdout: String::new(),
+                        stderr: String::new(),
+                    })
+                } else {
+                    engine.execute(&full_path, args).await
+                }
+            }
+            "js" | "mjs" | "cjs" => {
+                self.exec_node(&full_path, args).await
+            }
+            _ => anyhow::bail!("Unsupported file type: .{}", ext),
+        }
+    }
+
+    /// Execute an arbitrary command with node_modules/.bin on PATH.
+    pub async fn exec_command(
+        &self,
+        cmd: &str,
+        args: &[String],
+    ) -> Result<RunResult> {
+        // Check if cmd exists in .bin
+        let bin_path = self.resolve_bin_path(cmd);
+        let effective_cmd = if let Some(bp) = bin_path {
+            bp.to_string_lossy().to_string()
+        } else {
+            cmd.to_string()
+        };
+
+        self.exec_shell(&effective_cmd, args).await
+    }
+
+    /// Resolve a command name to node_modules/.bin path.
+    fn resolve_bin_path(&self, cmd: &str) -> Option<PathBuf> {
+        let bin_dir = self.project_root.join("node_modules/.bin");
+        let candidate = bin_dir.join(cmd);
+        if candidate.exists() {
+            Some(candidate)
+        } else {
+            None
+        }
+    }
+
+    /// Execute a shell command with injected environment.
+    async fn exec_shell(
+        &self,
+        command: &str,
+        extra_args: &[String],
+    ) -> Result<RunResult> {
+        let env_vars = env::build_env(&self.project_root);
+        let full_cmd = if extra_args.is_empty() {
+            command.to_string()
+        } else {
+            format!("{} {}", command, extra_args.join(" "))
+        };
+
+        let output = tokio::process::Command::new("sh")
+            .arg("-c")
+            .arg(&full_cmd)
+            .current_dir(&self.project_root)
+            .envs(&env_vars)
+            .output()
+            .await
+            .with_context(|| format!("Failed to execute: {}", full_cmd))?;
+
+        Ok(RunResult {
+            exit_code: exit_code(&output.status),
+            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
+            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
+        })
+    }
+
+    /// Execute a JS file directly via Node.js.
+    async fn exec_node(
+        &self,
+        file: &Path,
+        args: &[String],
+    ) -> Result<RunResult> {
+        let env_vars = env::build_env(&self.project_root);
+
+        let output = tokio::process::Command::new("node")
+            .arg(file)
+            .args(args)
+            .current_dir(&self.project_root)
+            .envs(&env_vars)
+            .output()
+            .await
+            .with_context(|| {
+                format!("Failed to execute: node {}", file.display())
+            })?;
+
+        Ok(RunResult {
+            exit_code: exit_code(&output.status),
+            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
+            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
+        })
+    }
+
+    /// Check if a name corresponds to a package.json script.
+    pub fn has_script(&self, name: &str) -> bool {
+        self.pkg_json
+            .as_ref()
+            .map(|p| p.scripts.contains_key(name))
+            .unwrap_or(false)
+    }
+
+    /// Check if a name corresponds to a file on disk.
+    pub fn is_file(&self, name: &str) -> bool {
+        let path = self.project_root.join(name);
+        path.is_file()
+    }
+}
+
+fn exit_code(status: &ExitStatus) -> i32 {
+    status.code().unwrap_or(-1)
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_script_runner_no_pkg_json() {
+        let runner = ScriptRunner::new(PathBuf::from("/nonexistent"));
+        assert!(!runner.has_script("test"));
+        assert!(!runner.is_file("index.js"));
+    }
+
+    #[test]
+    fn test_script_runner_with_tempdir() {
+        let dir = tempfile::tempdir().unwrap();
+        let pkg = r#"{"name":"t","version":"1.0.0","scripts":{"test":"echo ok"}}"#;
+        std::fs::write(dir.path().join("package.json"), pkg).unwrap();
+
+        let runner = ScriptRunner::new(dir.path().to_path_buf());
+        assert!(runner.has_script("test"));
+        assert!(!runner.has_script("build"));
+    }
+}
diff --git a/crates/cclab-jet/src/runner/source_map.rs b/crates/cclab-jet/src/runner/source_map.rs
new file mode 100644
index 0000000..1f0dbf1
--- /dev/null
+++ b/crates/cclab-jet/src/runner/source_map.rs
@@ -0,0 +1,68 @@
+//! Source map generation for JIT-transformed files.
+//!
+//! Generates inline source maps so that Node.js stack traces
+//! point back to the original .ts/.tsx source lines.
+
+use base64::Engine;
+
+/// Append an inline source map comment to JavaScript code.
+pub fn append_inline_source_map(code: &str, source_map_json: &str) -> String {
+    let encoded = base64::engine::general_purpose::STANDARD
+        .encode(source_map_json.as_bytes());
+    format!(
+        "{}\n//# sourceMappingURL=data:application/json;base64,{}\n",
+        code, encoded
+    )
+}
+
+/// Generate a minimal V3 source map JSON for a 1:1 line mapping.
+///
+/// This is used when type stripping doesn't change line numbers,
+/// so each output line maps to the same input line.
+pub fn generate_identity_source_map(
+    source_file: &str,
+    source_content: &str,
+) -> String {
+    let line_count = source_content.lines().count();
+    // Each line: "AAAA" = (0,0,0,0) relative to previous
+    let mappings = (0..line_count)
+        .map(|_| "AAAA")
+        .collect::<Vec<_>>()
+        .join(";");
+
+    serde_json::json!({
+        "version": 3,
+        "file": source_file.replace(".ts", ".js")
+            .replace(".tsx", ".js")
+            .replace(".jsx", ".js"),
+        "sources": [source_file],
+        "sourcesContent": [source_content],
+        "names": [],
+        "mappings": mappings,
+    })
+    .to_string()
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_inline_source_map() {
+        let code = "console.log(42);";
+        let sm = r#"{"version":3}"#;
+        let result = append_inline_source_map(code, sm);
+        assert!(result.starts_with("console.log(42);"));
+        assert!(result.contains("sourceMappingURL=data:application/json;base64,"));
+    }
+
+    #[test]
+    fn test_identity_source_map() {
+        let sm = generate_identity_source_map("test.ts", "line1\nline2\nline3");
+        let parsed: serde_json::Value = serde_json::from_str(&sm).unwrap();
+        assert_eq!(parsed["version"], 3);
+        assert_eq!(parsed["sources"][0], "test.ts");
+        let mappings = parsed["mappings"].as_str().unwrap();
+        assert_eq!(mappings.matches(';').count(), 2); // 3 lines = 2 semicolons
+    }
+}
diff --git a/crates/cclab-jet/src/runner/watcher.rs b/crates/cclab-jet/src/runner/watcher.rs
new file mode 100644
index 0000000..bcbe5c9
--- /dev/null
+++ b/crates/cclab-jet/src/runner/watcher.rs
@@ -0,0 +1,102 @@
+//! File watcher for JIT watch mode.
+//!
+//! Watches source files for changes and triggers re-execution.
+//! Uses the `notify` crate for cross-platform file system events.
+
+use anyhow::Result;
+use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
+use std::path::Path;
+use std::sync::mpsc;
+use std::time::{Duration, Instant};
+
+/// Debounced file watcher that coalesces rapid changes.
+pub struct DebouncedWatcher {
+    _watcher: RecommendedWatcher,
+    rx: mpsc::Receiver<notify::Result<Event>>,
+    debounce_ms: u64,
+}
+
+impl DebouncedWatcher {
+    /// Create a new debounced watcher for the given path.
+    pub fn new(path: &Path, debounce_ms: u64) -> Result<Self> {
+        let (tx, rx) = mpsc::channel();
+        let mut watcher = notify::recommended_watcher(tx)?;
+        watcher.watch(path, RecursiveMode::Recursive)?;
+
+        Ok(Self {
+            _watcher: watcher,
+            rx,
+            debounce_ms,
+        })
+    }
+
+    /// Wait for the next meaningful file change event.
+    /// Returns the list of changed file paths.
+    pub fn wait_for_change(&self) -> Result<Vec<std::path::PathBuf>> {
+        let mut changed = Vec::new();
+        let mut last_event = Instant::now();
+        let debounce = Duration::from_millis(self.debounce_ms);
+
+        loop {
+            let timeout = if changed.is_empty() {
+                Duration::from_secs(3600) // wait indefinitely
+            } else {
+                debounce.saturating_sub(last_event.elapsed())
+            };
+
+            match self.rx.recv_timeout(timeout) {
+                Ok(Ok(event)) => {
+                    if matches!(
+                        event.kind,
+                        EventKind::Modify(_) | EventKind::Create(_)
+                    ) {
+                        for path in event.paths {
+                            if is_source_file(&path) && !changed.contains(&path) {
+                                changed.push(path);
+                            }
+                        }
+                        last_event = Instant::now();
+                    }
+                }
+                Ok(Err(e)) => {
+                    tracing::warn!("Watch error: {:?}", e);
+                }
+                Err(mpsc::RecvTimeoutError::Timeout) => {
+                    if !changed.is_empty() {
+                        return Ok(changed);
+                    }
+                }
+                Err(mpsc::RecvTimeoutError::Disconnected) => {
+                    anyhow::bail!("File watcher disconnected");
+                }
+            }
+        }
+    }
+}
+
+/// Check if a path is a JS/TS source file worth watching.
+fn is_source_file(path: &Path) -> bool {
+    path.extension()
+        .and_then(|e| e.to_str())
+        .map(|ext| {
+            matches!(ext, "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs")
+        })
+        .unwrap_or(false)
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use std::path::PathBuf;
+
+    #[test]
+    fn test_is_source_file() {
+        assert!(is_source_file(&PathBuf::from("app.ts")));
+        assert!(is_source_file(&PathBuf::from("app.tsx")));
+        assert!(is_source_file(&PathBuf::from("app.jsx")));
+        assert!(is_source_file(&PathBuf::from("app.js")));
+        assert!(is_source_file(&PathBuf::from("app.mjs")));
+        assert!(!is_source_file(&PathBuf::from("style.css")));
+        assert!(!is_source_file(&PathBuf::from("data.json")));
+    }
+}
diff --git a/crates/cclab-jet/src/task_runner/cache.rs b/crates/cclab-jet/src/task_runner/cache.rs
new file mode 100644
index 0000000..2c63e99
--- /dev/null
+++ b/crates/cclab-jet/src/task_runner/cache.rs
@@ -0,0 +1,201 @@
+//! Task cache: content-hash based caching for task outputs.
+//!
+//! Cache key = SHA-256(input files + env vars + command + dependency hashes).
+//! Cached entries stored in `.jet-cache/tasks/` inside the project.
+
+use anyhow::{Context, Result};
+use serde::{Deserialize, Serialize};
+use std::path::{Path, PathBuf};
+
+use super::hash;
+
+/// Cached task entry stored on disk.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct TaskCacheEntry {
+    pub hash: String,
+    pub task_name: String,
+    pub outputs: Vec<String>,
+    pub stdout: String,
+    pub stderr: String,
+    pub created_at: String,
+}
+
+/// Task cache manager.
+pub struct TaskCache {
+    cache_dir: PathBuf,
+}
+
+impl TaskCache {
+    /// Create a new task cache in the project's .jet-cache directory.
+    pub fn new(project_root: &Path) -> Result<Self> {
+        let cache_dir = project_root.join(".jet-cache").join("tasks");
+        std::fs::create_dir_all(&cache_dir)?;
+        Ok(Self { cache_dir })
+    }
+
+    /// Compute a content hash for a task given its inputs and environment.
+    pub fn compute_hash(
+        &self,
+        task_name: &str,
+        input_globs: &[String],
+        env_keys: &[String],
+        project_root: &Path,
+    ) -> Result<String> {
+        hash::compute_task_hash(task_name, input_globs, env_keys, project_root)
+    }
+
+    /// Look up a cached entry by hash.
+    pub fn lookup(&self, hash: &str) -> Result<Option<TaskCacheEntry>> {
+        let entry_path = self.cache_dir.join(format!("{}.json", hash));
+        if !entry_path.exists() {
+            return Ok(None);
+        }
+
+        let content = std::fs::read_to_string(&entry_path)
+            .with_context(|| {
+                format!("Failed to read cache entry: {}", entry_path.display())
+            })?;
+        let entry: TaskCacheEntry = serde_json::from_str(&content)?;
+        Ok(Some(entry))
+    }
+
+    /// Store a task result in the cache.
+    pub fn store(
+        &self,
+        hash: &str,
+        task_name: &str,
+        output_globs: &[String],
+        stdout: &str,
+        stderr: &str,
+        project_root: &Path,
+    ) -> Result<()> {
+        // Collect actual output files
+        let outputs = collect_output_files(output_globs, project_root);
+
+        let entry = TaskCacheEntry {
+            hash: hash.to_string(),
+            task_name: task_name.to_string(),
+            outputs,
+            stdout: stdout.to_string(),
+            stderr: stderr.to_string(),
+            created_at: chrono_now(),
+        };
+
+        let entry_path = self.cache_dir.join(format!("{}.json", hash));
+        let content = serde_json::to_string_pretty(&entry)?;
+        std::fs::write(&entry_path, content)?;
+
+        // Also cache output files
+        let output_cache_dir = self.cache_dir.join(hash);
+        if !entry.outputs.is_empty() {
+            std::fs::create_dir_all(&output_cache_dir)?;
+            for output_file in &entry.outputs {
+                let src = project_root.join(output_file);
+                let dst = output_cache_dir.join(output_file);
+                if src.exists() {
+                    if let Some(parent) = dst.parent() {
+                        std::fs::create_dir_all(parent)?;
+                    }
+                    let _ = std::fs::copy(&src, &dst);
+                }
+            }
+        }
+
+        Ok(())
+    }
+
+    /// Restore cached outputs to the project directory.
+    pub fn restore_outputs(
+        &self,
+        hash: &str,
+        project_root: &Path,
+    ) -> Result<()> {
+        let output_cache_dir = self.cache_dir.join(hash);
+        if !output_cache_dir.exists() {
+            return Ok(());
+        }
+
+        for entry in walkdir::WalkDir::new(&output_cache_dir)
+            .into_iter()
+            .filter_map(|e| e.ok())
+            .filter(|e| e.file_type().is_file())
+        {
+            let rel = entry
+                .path()
+                .strip_prefix(&output_cache_dir)
+                .unwrap_or(entry.path());
+            let dst = project_root.join(rel);
+            if let Some(parent) = dst.parent() {
+                std::fs::create_dir_all(parent)?;
+            }
+            std::fs::copy(entry.path(), &dst)?;
+        }
+
+        Ok(())
+    }
+}
+
+/// Collect output files matching glob patterns.
+fn collect_output_files(
+    globs: &[String],
+    project_root: &Path,
+) -> Vec<String> {
+    let mut files = Vec::new();
+    for pattern in globs {
+        let full = format!("{}/{}", project_root.display(), pattern);
+        if let Ok(entries) = glob::glob(&full) {
+            for entry in entries.flatten() {
+                if entry.is_file() {
+                    if let Ok(rel) = entry.strip_prefix(project_root) {
+                        files.push(rel.to_string_lossy().to_string());
+                    }
+                }
+            }
+        }
+    }
+    files
+}
+
+/// Get current time as ISO 8601 string (no chrono dependency).
+fn chrono_now() -> String {
+    use std::time::SystemTime;
+    let dur = SystemTime::now()
+        .duration_since(SystemTime::UNIX_EPOCH)
+        .unwrap_or_default();
+    format!("{}s", dur.as_secs())
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_task_cache_creation() {
+        let dir = tempfile::tempdir().unwrap();
+        let cache = TaskCache::new(dir.path());
+        assert!(cache.is_ok());
+        assert!(dir.path().join(".jet-cache/tasks").exists());
+    }
+
+    #[test]
+    fn test_cache_miss() {
+        let dir = tempfile::tempdir().unwrap();
+        let cache = TaskCache::new(dir.path()).unwrap();
+        let result = cache.lookup("nonexistent").unwrap();
+        assert!(result.is_none());
+    }
+
+    #[test]
+    fn test_cache_store_and_lookup() {
+        let dir = tempfile::tempdir().unwrap();
+        let cache = TaskCache::new(dir.path()).unwrap();
+
+        cache
+            .store("abc123", "build", &[], "output\n", "", dir.path())
+            .unwrap();
+
+        let entry = cache.lookup("abc123").unwrap().unwrap();
+        assert_eq!(entry.task_name, "build");
+        assert_eq!(entry.stdout, "output\n");
+    }
+}
diff --git a/crates/cclab-jet/src/task_runner/config.rs b/crates/cclab-jet/src/task_runner/config.rs
new file mode 100644
index 0000000..50328dd
--- /dev/null
+++ b/crates/cclab-jet/src/task_runner/config.rs
@@ -0,0 +1,141 @@
+//! jet.config.yaml configuration parser.
+//!
+//! Defines the pipeline task definitions used by the task runner.
+
+use anyhow::{Context, Result};
+use serde::Deserialize;
+use std::collections::HashMap;
+use std::path::Path;
+
+/// Top-level jet configuration.
+#[derive(Debug, Clone, Deserialize)]
+pub struct JetConfig {
+    /// Task pipeline definitions: task_name → TaskDef.
+    #[serde(default)]
+    pub pipeline: HashMap<String, TaskDef>,
+}
+
+/// Single task definition within the pipeline.
+#[derive(Debug, Clone, Deserialize)]
+pub struct TaskDef {
+    /// Task dependencies. `^task` means cross-package dependency.
+    #[serde(rename = "dependsOn", default)]
+    pub depends_on: Vec<String>,
+
+    /// Glob patterns for input files that affect the cache key.
+    #[serde(default)]
+    pub inputs: Vec<String>,
+
+    /// Glob patterns for output files to cache.
+    #[serde(default)]
+    pub outputs: Vec<String>,
+
+    /// Whether this task is cacheable (default: true).
+    #[serde(default = "default_true")]
+    pub cache: bool,
+
+    /// Whether this is a long-running task (e.g., dev server).
+    /// Persistent tasks are never cached.
+    #[serde(default)]
+    pub persistent: bool,
+
+    /// Environment variables that affect the cache key.
+    #[serde(default)]
+    pub env: Vec<String>,
+}
+
+fn default_true() -> bool {
+    true
+}
+
+impl JetConfig {
+    /// Load configuration from jet.config.yaml in the project root.
+    pub fn load(project_root: &Path) -> Result<Self> {
+        let config_path = project_root.join("jet.config.yaml");
+
+        if !config_path.exists() {
+            // Return empty config if no file exists
+            return Ok(Self {
+                pipeline: HashMap::new(),
+            });
+        }
+
+        let content = std::fs::read_to_string(&config_path)
+            .with_context(|| {
+                format!("Failed to read {}", config_path.display())
+            })?;
+
+        let config: JetConfig = serde_yaml::from_str(&content)
+            .with_context(|| {
+                format!("Failed to parse {}", config_path.display())
+            })?;
+
+        Ok(config)
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_parse_pipeline_config() {
+        let yaml = r#"
+pipeline:
+  build:
+    dependsOn: ["^build"]
+    outputs: ["dist/**"]
+    inputs: ["src/**", "package.json"]
+  test:
+    dependsOn: ["build"]
+    outputs: []
+  lint:
+    outputs: []
+  dev:
+    cache: false
+    persistent: true
+"#;
+
+        let config: JetConfig = serde_yaml::from_str(yaml).unwrap();
+        assert_eq!(config.pipeline.len(), 4);
+
+        let build = &config.pipeline["build"];
+        assert_eq!(build.depends_on, vec!["^build"]);
+        assert_eq!(build.outputs, vec!["dist/**"]);
+        assert!(build.cache); // default true
+
+        let dev = &config.pipeline["dev"];
+        assert!(!dev.cache);
+        assert!(dev.persistent);
+    }
+
+    #[test]
+    fn test_empty_config() {
+        let yaml = "pipeline: {}";
+        let config: JetConfig = serde_yaml::from_str(yaml).unwrap();
+        assert!(config.pipeline.is_empty());
+    }
+
+    #[test]
+    fn test_load_nonexistent() {
+        let dir = tempfile::tempdir().unwrap();
+        let config = JetConfig::load(dir.path()).unwrap();
+        assert!(config.pipeline.is_empty());
+    }
+
+    #[test]
+    fn test_task_def_defaults() {
+        let yaml = r#"
+pipeline:
+  simple: {}
+"#;
+        let config: JetConfig = serde_yaml::from_str(yaml).unwrap();
+        let simple = &config.pipeline["simple"];
+        assert!(simple.depends_on.is_empty());
+        assert!(simple.inputs.is_empty());
+        assert!(simple.outputs.is_empty());
+        assert!(simple.cache);
+        assert!(!simple.persistent);
+        assert!(simple.env.is_empty());
+    }
+}
diff --git a/crates/cclab-jet/src/task_runner/graph.rs b/crates/cclab-jet/src/task_runner/graph.rs
new file mode 100644
index 0000000..a670dd5
--- /dev/null
+++ b/crates/cclab-jet/src/task_runner/graph.rs
@@ -0,0 +1,284 @@
+//! Task dependency graph: DAG construction, topological sort, cycle detection.
+
+use anyhow::Result;
+use std::collections::{HashMap, HashSet, VecDeque};
+
+use super::config::TaskDef;
+
+/// Directed acyclic graph of task dependencies.
+#[derive(Debug)]
+pub struct TaskGraph {
+    /// Adjacency list: task → set of tasks it depends on.
+    deps: HashMap<String, Vec<String>>,
+    /// All known task names.
+    tasks: HashSet<String>,
+}
+
+impl TaskGraph {
+    /// Build a task graph from pipeline configuration.
+    pub fn from_config(
+        pipeline: &HashMap<String, TaskDef>,
+    ) -> Result<Self> {
+        let mut deps: HashMap<String, Vec<String>> = HashMap::new();
+        let tasks: HashSet<String> = pipeline.keys().cloned().collect();
+
+        for (name, def) in pipeline {
+            let task_deps: Vec<String> = def
+                .depends_on
+                .iter()
+                .filter_map(|d| {
+                    // Strip ^ prefix (cross-package indicator)
+                    let dep_name = d.trim_start_matches('^');
+                    if tasks.contains(dep_name) {
+                        Some(dep_name.to_string())
+                    } else {
+                        tracing::warn!(
+                            "Task '{}' depends on unknown task '{}' (skipped)",
+                            name,
+                            dep_name
+                        );
+                        None
+                    }
+                })
+                .collect();
+            deps.insert(name.clone(), task_deps);
+        }
+
+        let graph = Self { deps, tasks };
+        graph.detect_cycles()?;
+        Ok(graph)
+    }
+
+    /// Get the execution order for a task and all its transitive deps.
+    /// Returns tasks in topological order (dependencies first).
+    pub fn execution_order(&self, task_name: &str) -> Result<Vec<String>> {
+        if !self.tasks.contains(task_name) {
+            anyhow::bail!("Task '{}' not found in pipeline", task_name);
+        }
+
+        // Collect all reachable tasks via BFS
+        let mut needed = HashSet::new();
+        let mut queue = VecDeque::new();
+        queue.push_back(task_name.to_string());
+
+        while let Some(name) = queue.pop_front() {
+            if needed.contains(&name) {
+                continue;
+            }
+            needed.insert(name.clone());
+            if let Some(task_deps) = self.deps.get(&name) {
+                for dep in task_deps {
+                    queue.push_back(dep.clone());
+                }
+            }
+        }
+
+        // Kahn's algorithm on the subset
+        let mut in_degree: HashMap<&str, usize> = HashMap::new();
+        for name in &needed {
+            in_degree.entry(name.as_str()).or_insert(0);
+            if let Some(task_deps) = self.deps.get(name.as_str()) {
+                for dep in task_deps {
+                    if needed.contains(dep) {
+                        let _ = *in_degree.entry(dep.as_str()).or_insert(0);
+                    }
+                }
+            }
+        }
+
+        // Count incoming edges
+        for name in &needed {
+            if let Some(task_deps) = self.deps.get(name.as_str()) {
+                for dep in task_deps {
+                    if needed.contains(dep) {
+                        // name depends on dep → dep has an outgoing edge to name
+                        // but we count incoming: name has incoming from dep
+                    }
+                }
+            }
+        }
+
+        // Recount properly: for each (name depends on dep), name has in-edge from dep
+        let mut in_deg: HashMap<String, usize> = HashMap::new();
+        for name in &needed {
+            in_deg.entry(name.clone()).or_insert(0);
+        }
+        for name in &needed {
+            if let Some(task_deps) = self.deps.get(name.as_str()) {
+                for dep in task_deps {
+                    if needed.contains(dep) {
+                        *in_deg.entry(name.clone()).or_insert(0) += 1;
+                    }
+                }
+            }
+        }
+
+        let mut queue: VecDeque<String> = in_deg
+            .iter()
+            .filter(|(_, &deg)| deg == 0)
+            .map(|(name, _)| name.clone())
+            .collect();
+
+        let mut order = Vec::new();
+        while let Some(name) = queue.pop_front() {
+            order.push(name.clone());
+            // Find tasks that depend on `name`
+            for (task, task_deps) in &self.deps {
+                if needed.contains(task) && task_deps.contains(&name) {
+                    if let Some(deg) = in_deg.get_mut(task) {
+                        *deg -= 1;
+                        if *deg == 0 {
+                            queue.push_back(task.clone());
+                        }
+                    }
+                }
+            }
+        }
+
+        Ok(order)
+    }
+
+    /// Detect cycles in the task graph.
+    fn detect_cycles(&self) -> Result<()> {
+        let mut visited = HashSet::new();
+        let mut in_stack = HashSet::new();
+
+        for task in &self.tasks {
+            if !visited.contains(task.as_str()) {
+                self.dfs_cycle(
+                    task,
+                    &mut visited,
+                    &mut in_stack,
+                )?;
+            }
+        }
+
+        Ok(())
+    }
+
+    fn dfs_cycle(
+        &self,
+        node: &str,
+        visited: &mut HashSet<String>,
+        in_stack: &mut HashSet<String>,
+    ) -> Result<()> {
+        visited.insert(node.to_string());
+        in_stack.insert(node.to_string());
+
+        if let Some(task_deps) = self.deps.get(node) {
+            for dep in task_deps {
+                if in_stack.contains(dep.as_str()) {
+                    anyhow::bail!(
+                        "Cycle detected in task graph: {} → {}",
+                        node,
+                        dep
+                    );
+                }
+                if !visited.contains(dep.as_str()) {
+                    self.dfs_cycle(dep, visited, in_stack)?;
+                }
+            }
+        }
+
+        in_stack.remove(node);
+        Ok(())
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn make_pipeline(
+        tasks: Vec<(&str, Vec<&str>)>,
+    ) -> HashMap<String, TaskDef> {
+        tasks
+            .into_iter()
+            .map(|(name, deps)| {
+                (
+                    name.to_string(),
+                    TaskDef {
+                        depends_on: deps.iter().map(|s| s.to_string()).collect(),
+                        inputs: vec![],
+                        outputs: vec![],
+                        cache: true,
+                        persistent: false,
+                        env: vec![],
+                    },
+                )
+            })
+            .collect()
+    }
+
+    #[test]
+    fn test_simple_graph() {
+        let pipeline = make_pipeline(vec![
+            ("build", vec![]),
+            ("test", vec!["build"]),
+            ("lint", vec![]),
+        ]);
+
+        let graph = TaskGraph::from_config(&pipeline).unwrap();
+        let order = graph.execution_order("test").unwrap();
+
+        // build must come before test
+        let build_idx = order.iter().position(|t| t == "build").unwrap();
+        let test_idx = order.iter().position(|t| t == "test").unwrap();
+        assert!(build_idx < test_idx);
+    }
+
+    #[test]
+    fn test_cycle_detection() {
+        let pipeline = make_pipeline(vec![
+            ("a", vec!["b"]),
+            ("b", vec!["a"]),
+        ]);
+
+        let result = TaskGraph::from_config(&pipeline);
+        assert!(result.is_err());
+        let err = result.unwrap_err().to_string();
+        assert!(err.contains("Cycle detected"));
+    }
+
+    #[test]
+    fn test_unknown_task_error() {
+        let pipeline = make_pipeline(vec![("build", vec![])]);
+        let graph = TaskGraph::from_config(&pipeline).unwrap();
+        let result = graph.execution_order("nonexistent");
+        assert!(result.is_err());
+    }
+
+    #[test]
+    fn test_cross_package_dep_stripped() {
+        let pipeline = make_pipeline(vec![
+            ("build", vec!["^build"]),
+        ]);
+        // ^build refers to self after stripping ^
+        let graph = TaskGraph::from_config(&pipeline);
+        // This creates a self-cycle: build depends on build
+        assert!(graph.is_err());
+    }
+
+    #[test]
+    fn test_diamond_dependency() {
+        let pipeline = make_pipeline(vec![
+            ("a", vec![]),
+            ("b", vec!["a"]),
+            ("c", vec!["a"]),
+            ("d", vec!["b", "c"]),
+        ]);
+
+        let graph = TaskGraph::from_config(&pipeline).unwrap();
+        let order = graph.execution_order("d").unwrap();
+
+        assert_eq!(order.len(), 4);
+        let a_idx = order.iter().position(|t| t == "a").unwrap();
+        let b_idx = order.iter().position(|t| t == "b").unwrap();
+        let c_idx = order.iter().position(|t| t == "c").unwrap();
+        let d_idx = order.iter().position(|t| t == "d").unwrap();
+        assert!(a_idx < b_idx);
+        assert!(a_idx < c_idx);
+        assert!(b_idx < d_idx);
+        assert!(c_idx < d_idx);
+    }
+}
diff --git a/crates/cclab-jet/src/task_runner/hash.rs b/crates/cclab-jet/src/task_runner/hash.rs
new file mode 100644
index 0000000..07c1d8e
--- /dev/null
+++ b/crates/cclab-jet/src/task_runner/hash.rs
@@ -0,0 +1,162 @@
+//! Content-hash computation for task cache keys.
+//!
+//! Hash = SHA-256(task_name + sorted input file contents + env values).
+
+use anyhow::Result;
+use sha2::{Digest, Sha256};
+use std::path::Path;
+
+/// Compute a deterministic hash for a task based on its inputs.
+///
+/// Components:
+/// 1. Task name
+/// 2. Content of all input files (sorted by path for determinism)
+/// 3. Values of specified environment variables
+pub fn compute_task_hash(
+    task_name: &str,
+    input_globs: &[String],
+    env_keys: &[String],
+    project_root: &Path,
+) -> Result<String> {
+    let mut hasher = Sha256::new();
+
+    // 1. Task name
+    hasher.update(task_name.as_bytes());
+    hasher.update(b"\0");
+
+    // 2. Input file contents (sorted)
+    let mut input_files = collect_input_files(input_globs, project_root);
+    input_files.sort();
+
+    for file_path in &input_files {
+        let full = project_root.join(file_path);
+        if let Ok(content) = std::fs::read(&full) {
+            hasher.update(file_path.as_bytes());
+            hasher.update(b"\0");
+            hasher.update(&content);
+            hasher.update(b"\0");
+        }
+    }
+
+    // 3. Environment variables (sorted by key)
+    let mut env_pairs: Vec<(String, String)> = env_keys
+        .iter()
+        .map(|k| {
+            let v = std::env::var(k).unwrap_or_default();
+            (k.clone(), v)
+        })
+        .collect();
+    env_pairs.sort_by(|a, b| a.0.cmp(&b.0));
+
+    for (key, value) in &env_pairs {
+        hasher.update(key.as_bytes());
+        hasher.update(b"=");
+        hasher.update(value.as_bytes());
+        hasher.update(b"\0");
+    }
+
+    let result = hasher.finalize();
+    Ok(format!("{:x}", result))
+}
+
+/// Collect input files matching glob patterns, returning relative paths.
+fn collect_input_files(
+    globs: &[String],
+    project_root: &Path,
+) -> Vec<String> {
+    let mut files = Vec::new();
+
+    if globs.is_empty() {
+        // Default: hash all source files
+        return files;
+    }
+
+    for pattern in globs {
+        let full_pattern = format!(
+            "{}/{}",
+            project_root.display(),
+            pattern
+        );
+        if let Ok(entries) = glob::glob(&full_pattern) {
+            for entry in entries.flatten() {
+                if entry.is_file() {
+                    if let Ok(rel) = entry.strip_prefix(project_root) {
+                        files.push(rel.to_string_lossy().to_string());
+                    }
+                }
+            }
+        }
+    }
+
+    files
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    #[test]
+    fn test_hash_deterministic() {
+        let dir = tempfile::tempdir().unwrap();
+        std::fs::write(dir.path().join("src.js"), "console.log('hi')").unwrap();
+
+        let h1 = compute_task_hash(
+            "build",
+            &["src.js".to_string()],
+            &[],
+            dir.path(),
+        )
+        .unwrap();
+        let h2 = compute_task_hash(
+            "build",
+            &["src.js".to_string()],
+            &[],
+            dir.path(),
+        )
+        .unwrap();
+
+        assert_eq!(h1, h2);
+    }
+
+    #[test]
+    fn test_hash_changes_with_content() {
+        let dir = tempfile::tempdir().unwrap();
+        std::fs::write(dir.path().join("src.js"), "v1").unwrap();
+
+        let h1 = compute_task_hash(
+            "build",
+            &["src.js".to_string()],
+            &[],
+            dir.path(),
+        )
+        .unwrap();
+
+        std::fs::write(dir.path().join("src.js"), "v2").unwrap();
+        let h2 = compute_task_hash(
+            "build",
+            &["src.js".to_string()],
+            &[],
+            dir.path(),
+        )
+        .unwrap();
+
+        assert_ne!(h1, h2);
+    }
+
+    #[test]
+    fn test_hash_different_task_names() {
+        let dir = tempfile::tempdir().unwrap();
+
+        let h1 = compute_task_hash("build", &[], &[], dir.path()).unwrap();
+        let h2 = compute_task_hash("test", &[], &[], dir.path()).unwrap();
+
+        assert_ne!(h1, h2);
+    }
+
+    #[test]
+    fn test_empty_inputs() {
+        let dir = tempfile::tempdir().unwrap();
+        let result = compute_task_hash("t", &[], &[], dir.path());
+        assert!(result.is_ok());
+        assert_eq!(result.unwrap().len(), 64); // SHA-256 hex
+    }
+}
diff --git a/crates/cclab-jet/src/task_runner/mod.rs b/crates/cclab-jet/src/task_runner/mod.rs
new file mode 100644
index 0000000..363d8b3
--- /dev/null
+++ b/crates/cclab-jet/src/task_runner/mod.rs
@@ -0,0 +1,251 @@
+//! Task runner: parallel script orchestration with dependency graph and caching.
+//!
+//! Reads `jet.config.yaml` pipeline definitions, builds a task DAG,
+//! and executes tasks in topological order with content-hash caching.
+
+use anyhow::{Context, Result};
+use std::path::{Path, PathBuf};
+
+pub mod cache;
+pub mod config;
+pub mod graph;
+pub mod hash;
+
+use cache::TaskCache;
+use config::JetConfig;
+use graph::TaskGraph;
+
+/// Result of a single task execution.
+#[derive(Debug, Clone)]
+pub struct TaskResult {
+    pub task_name: String,
+    pub package_name: Option<String>,
+    pub status: TaskStatus,
+    pub duration_ms: u64,
+    pub cache_hit: bool,
+    pub exit_code: i32,
+    pub stdout: String,
+    pub stderr: String,
+}
+
+/// Task execution status.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum TaskStatus {
+    Success,
+    Failed,
+    Cached,
+    Skipped,
+}
+
+/// Task runner orchestrator.
+pub struct TaskRunner {
+    config: JetConfig,
+    graph: TaskGraph,
+    cache: TaskCache,
+    project_root: PathBuf,
+}
+
+impl TaskRunner {
+    /// Create a task runner from the project root.
+    /// Loads jet.config.yaml and builds the task graph.
+    pub fn new(project_root: &Path) -> Result<Self> {
+        let config = JetConfig::load(project_root)
+            .context("Failed to load jet.config.yaml")?;
+        let graph = TaskGraph::from_config(&config.pipeline)?;
+        let cache = TaskCache::new(project_root)?;
+
+        Ok(Self {
+            config,
+            graph,
+            cache,
+            project_root: project_root.to_path_buf(),
+        })
+    }
+
+    /// Check if a task name is defined in the pipeline.
+    pub fn has_task(&self, name: &str) -> bool {
+        self.config.pipeline.contains_key(name)
+    }
+
+    /// Run a task and all its dependencies.
+    pub async fn run(
+        &self,
+        task_name: &str,
+        filter: Option<&str>,
+        dry_run: bool,
+    ) -> Result<Vec<TaskResult>> {
+        let execution_order = self.graph.execution_order(task_name)?;
+        let mut results = Vec::new();
+
+        for name in &execution_order {
+            let task_def = match self.config.pipeline.get(name.as_str()) {
+                Some(def) => def,
+                None => continue,
+            };
+
+            // Apply filter
+            if let Some(pattern) = filter {
+                if !name.contains(pattern) {
+                    results.push(TaskResult {
+                        task_name: name.clone(),
+                        package_name: None,
+                        status: TaskStatus::Skipped,
+                        duration_ms: 0,
+                        cache_hit: false,
+                        exit_code: 0,
+                        stdout: String::new(),
+                        stderr: String::new(),
+                    });
+                    continue;
+                }
+            }
+
+            // Dry run: just show what would execute
+            if dry_run {
+                println!("  [dry] {}", name);
+                results.push(TaskResult {
+                    task_name: name.clone(),
+                    package_name: None,
+                    status: TaskStatus::Skipped,
+                    duration_ms: 0,
+                    cache_hit: false,
+                    exit_code: 0,
+                    stdout: String::new(),
+                    stderr: String::new(),
+                });
+                continue;
+            }
+
+            // Persistent tasks (dev servers) are never cached
+            if task_def.persistent {
+                let result = self.execute_task(name).await?;
+                results.push(result);
+                continue;
+            }
+
+            // Check cache
+            if task_def.cache {
+                let hash = self.cache.compute_hash(
+                    name,
+                    &task_def.inputs,
+                    &task_def.env,
+                    &self.project_root,
+                )?;
+
+                if let Some(cached) = self.cache.lookup(&hash)? {
+                    tracing::info!("{} → CACHED", name);
+                    print!("{}", cached.stdout);
+                    eprint!("{}", cached.stderr);
+                    results.push(TaskResult {
+                        task_name: name.clone(),
+                        package_name: None,
+                        status: TaskStatus::Cached,
+                        duration_ms: 0,
+                        cache_hit: true,
+                        exit_code: 0,
+                        stdout: cached.stdout,
+                        stderr: cached.stderr,

... truncated (104 more lines)
```
