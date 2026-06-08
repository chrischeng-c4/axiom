// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
// CODEGEN-BEGIN
//! JIT engine: transform TypeScript/TSX/JSX via Tree-sitter, execute via Node.js.
//!
//! Pipeline: source file → Tree-sitter parse → strip types / transform JSX
//!           → write temp .js → execute via Node.js child process → cleanup

use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};

use crate::runner::RunResult;
use crate::transform::{TransformOptions, Transformer};

/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub(crate) fn format_jit_watch_no_filename_err(path: &Path) -> String {
    format!(
        "jet watch requires a path with a file name component, but got {:?} (GH #3600)",
        path
    )
}

/// GH #3614 — `transform_file` previously did
/// `.file_stem().and_then(|s| s.to_str()).unwrap_or("jit")` which silently
/// collapsed missing-stem and non-UTF-8-stem into the same shared temp file
/// `<tmp>/jet-jit/jit.js`. Two concurrent JIT runs against different
/// non-UTF-8 paths then race on the same temp path.
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub(crate) fn format_jit_stem_err(path: &Path) -> String {
    format!(
        "GH #3614 jet jit cannot derive a UTF-8 file stem from {:?}; \
         non-UTF-8 or missing-stem paths would collide on the shared \
         temp file <tmp>/jet-jit/jit.js. Rename the source file to use \
         a UTF-8 stem.",
        path
    )
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub(crate) fn safe_jit_stem(path: &Path) -> Result<String, String> {
    match path.file_stem().and_then(|s| s.to_str()) {
        Some(s) => Ok(s.to_string()),
        None => Err(format_jit_stem_err(path)),
    }
}

/// JIT execution engine for TypeScript/JSX files.
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub struct JitEngine {
    transformer: Transformer,
    project_root: PathBuf,
    temp_dir: PathBuf,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
impl JitEngine {
    /// Create a new JIT engine.
    pub fn new(project_root: &Path) -> Result<Self> {
        let temp_dir = std::env::temp_dir().join("jet-jit");
        std::fs::create_dir_all(&temp_dir)?;

        let options = TransformOptions {
            source_maps: true,
            jsx_automatic: true,
            ..Default::default()
        };

        Ok(Self {
            transformer: Transformer::new(options),
            project_root: project_root.to_path_buf(),
            temp_dir,
        })
    }

    /// Transform and execute a file, returning the result.
    pub async fn execute(&self, file: &Path, args: &[String]) -> Result<RunResult> {
        let temp_js = self.transform_file(file)?;

        let env_vars = super::env::build_env(&self.project_root);

        let output = tokio::process::Command::new("node")
            .arg(&temp_js)
            .args(args)
            .current_dir(&self.project_root)
            .envs(&env_vars)
            .output()
            .await
            .with_context(|| format!("Failed to execute: node {}", temp_js.display()))?;

        // Cleanup temp file
        let _ = std::fs::remove_file(&temp_js);

        // GH #3691 — was `output.status.code().unwrap_or(-1)` which
        // silently collapsed signal-killed node processes (SIGSEGV
        // from native-addon crashes, SIGKILL from OOM, SIGINT from
        // Ctrl+C) onto the same -1 as a real -1 return. Route
        // through the shared `safe_runner_exit_code` so the warn
        // names the signal and the likely cause.
        let (code, warn) = super::safe_runner_exit_code(&output.status);
        if let Some(msg) = warn {
            tracing::warn!(target: "jet::runner::jit", "{}", msg);
        }

        Ok(RunResult {
            exit_code: code,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    /// Execute in watch mode: re-transform and restart on file changes.
    pub async fn execute_watch(&self, file: &Path, args: &[String]) -> Result<()> {
        use notify::{Event, EventKind, RecursiveMode, Watcher};
        use std::sync::mpsc;

        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

        let mut watcher = notify::recommended_watcher(tx)?;

        // Watch the file's parent directory
        let watch_dir = file.parent().unwrap_or_else(|| Path::new("."));
        watcher.watch(watch_dir, RecursiveMode::NonRecursive)?;

        let file_name = file
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .ok_or_else(|| anyhow!(format_jit_watch_no_filename_err(file)))?;

        tracing::info!("Watch mode: {}", file.display());

        // Initial run
        let result = self.execute(file, args).await?;
        print!("{}", result.stdout);
        eprint!("{}", result.stderr);

        // Watch loop
        loop {
            match rx.recv() {
                Ok(Ok(event)) => {
                    let is_target =
                        matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_))
                            && event.paths.iter().any(|p| {
                                p.file_name()
                                    .map(|f| f.to_string_lossy() == file_name)
                                    .unwrap_or(false)
                            });

                    if is_target {
                        tracing::info!("File changed, re-running...");
                        let result = self.execute(file, args).await?;
                        print!("{}", result.stdout);
                        eprint!("{}", result.stderr);
                        if result.exit_code != 0 {
                            tracing::warn!("Process exited with code {}", result.exit_code);
                        }
                    }
                }
                Ok(Err(e)) => {
                    tracing::error!("Watch error: {:?}", e);
                }
                Err(_) => break,
            }
        }

        Ok(())
    }

    /// Transform a TS/TSX/JSX file to a temporary JS file.
    fn transform_file(&self, file: &Path) -> Result<PathBuf> {
        let source = std::fs::read_to_string(file)
            .with_context(|| format!("Failed to read {}", file.display()))?;

        let result = self
            .transformer
            .transform_js(&source, file)
            .with_context(|| format!("Failed to transform {}", file.display()))?;

        // Build output code with inline source map
        let output_code = if let Some(ref sm) = result.source_map {
            source_map::append_inline_source_map(&result.code, sm)
        } else {
            result.code.clone()
        };

        // Write to temp file preserving the stem
        let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("jit");
        let temp_path = self.temp_dir.join(format!("{}.js", stem));
        std::fs::write(&temp_path, &output_code)
            .with_context(|| format!("Failed to write temp file: {}", temp_path.display()))?;

        Ok(temp_path)
    }
}

/// Inline source map helper — reused from source_map module.
mod source_map {
    use base64::Engine;

    /// Append an inline source map comment to JS code.
    pub fn append_inline_source_map(code: &str, source_map_json: &str) -> String {
        let encoded = base64::engine::general_purpose::STANDARD.encode(source_map_json.as_bytes());
        format!(
            "{}\n//# sourceMappingURL=data:application/json;base64,{}\n",
            code, encoded
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jit_engine_creation() {
        let dir = tempfile::tempdir().unwrap();
        let engine = JitEngine::new(dir.path());
        assert!(engine.is_ok());
    }

    #[test]
    fn test_transform_typescript_file() {
        let dir = tempfile::tempdir().unwrap();
        let ts_file = dir.path().join("test.ts");
        std::fs::write(&ts_file, "const x: number = 42;\nconsole.log(x);").unwrap();

        let engine = JitEngine::new(dir.path()).unwrap();
        let temp_js = engine.transform_file(&ts_file).unwrap();

        let output = std::fs::read_to_string(&temp_js).unwrap();
        assert!(output.contains("const x"));
        assert!(output.contains("console.log"));
        assert!(!output.contains(": number"));

        // Cleanup
        let _ = std::fs::remove_file(&temp_js);
    }
}

#[cfg(test)]
mod gh3600_watch_filename_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn helper_tags_gh_issue_and_path() {
        let msg = format_jit_watch_no_filename_err(Path::new("/"));
        assert!(msg.contains("GH #3600"), "msg: {msg}");
        assert!(msg.contains("/"), "msg: {msg}");
    }

    #[test]
    fn helper_renders_dot_dot() {
        let msg = format_jit_watch_no_filename_err(Path::new(".."));
        assert!(msg.contains(".."), "msg: {msg}");
        assert!(msg.contains("file name"), "msg: {msg}");
    }

    #[test]
    fn root_path_has_no_file_name() {
        assert!(Path::new("/").file_name().is_none());
    }

    #[test]
    fn parent_token_path_has_no_file_name() {
        let p: PathBuf = PathBuf::from("..");
        assert!(p.file_name().is_none());
    }

    #[test]
    fn normal_path_has_file_name() {
        assert_eq!(
            Path::new("/tmp/a.ts").file_name().unwrap(),
            std::ffi::OsStr::new("a.ts")
        );
    }
}

#[cfg(test)]
mod gh3614_safe_jit_stem_tests {
    //! GH #3614 — `transform_file` previously did
    //! `.file_stem().and_then(|s| s.to_str()).unwrap_or("jit")`. That
    //! silently bucketed missing-stem and non-UTF-8-stem paths into the
    //! same `<tmp>/jet-jit/jit.js`, so two concurrent runs against
    //! different non-UTF-8 inputs would race on the same temp file.
    use super::*;

    #[test]
    fn happy_utf8_stem_returns_stem() {
        let s = safe_jit_stem(Path::new("/tmp/foo.ts")).expect("utf8 stem ok");
        assert_eq!(s, "foo");
    }

    #[test]
    fn nested_utf8_stem_strips_dirs_and_ext() {
        let s = safe_jit_stem(Path::new("/a/b/c/bar.tsx")).expect("utf8 stem ok");
        assert_eq!(s, "bar");
    }

    #[test]
    fn missing_stem_returns_tagged_err() {
        let err = safe_jit_stem(Path::new("/")).expect_err("/ has no stem");
        assert!(err.contains("GH #3614"), "err: {err}");
        assert!(err.contains("collide"), "err: {err}");
    }

    #[test]
    fn dot_dot_path_returns_tagged_err() {
        let err = safe_jit_stem(Path::new("..")).expect_err(".. has no stem");
        assert!(err.contains("GH #3614"), "err: {err}");
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_stem_returns_tagged_err() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        let bad: &OsStr = OsStr::from_bytes(&[0x66, 0x6f, 0xff, 0x6f, b'.', b't', b's']);
        let p = std::path::PathBuf::from(bad);
        let err = safe_jit_stem(&p).expect_err("non-utf8 stem must error");
        assert!(err.contains("GH #3614"), "err: {err}");
        assert!(err.contains("UTF-8") || err.contains("utf-8"), "err: {err}");
    }

    #[test]
    fn helper_message_includes_path_and_tag() {
        let msg = format_jit_stem_err(Path::new("/tmp/x"));
        assert!(msg.contains("GH #3614"), "msg: {msg}");
        assert!(msg.contains("/tmp/x"), "msg: {msg}");
    }
}
// CODEGEN-END
