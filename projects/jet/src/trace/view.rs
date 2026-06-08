// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-trace.md#schema
// CODEGEN-BEGIN
//! `jet trace view` entry point — unzip, start HTTP server, open browser.
//!
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R5
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R12

use crate::trace::archive::read_manifest_from_zip;
use crate::trace::server::{build_router, ViewerState};
use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::path::PathBuf;

/// Run `jet trace view <file>`.
///
/// 1. Parse the trace zip to get the manifest.
/// 2. Bind a `TcpListener` on `127.0.0.1:<port>` (0 = free port).
/// 3. Spawn the axum HTTP server.
/// 4. Print the URL to stdout.
/// 5. Optionally open the browser.
/// 6. Block until Ctrl-C.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R5
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R12
pub async fn run(file: PathBuf, port: u16, no_open: bool) -> Result<()> {
    // Validate file exists.
    if !file.exists() {
        anyhow::bail!("Trace file not found: {}", file.display());
    }

    // Parse manifest — validates the archive format.
    let manifest = read_manifest_from_zip(&file)
        .with_context(|| format!("Invalid trace format in {}", file.display()))?;

    // Bind listener on loopback.
    // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R12
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("Failed to bind trace viewer on {addr}"))?;
    let bound_addr = listener
        .local_addr()
        .context("Failed to get bound address")?;
    let url = format!("http://{bound_addr}");

    let state = ViewerState::new(file.clone(), manifest);
    let app = build_router(state);

    // Print the URL before spawning the server.
    println!("Trace viewer running at {url}");

    // Open browser unless suppressed.
    if !no_open {
        if let Err(e) = open::that(&url) {
            eprintln!("Warning: could not open browser automatically ({e}). Visit {url} manually.");
        }
    }

    // Serve until Ctrl-C.
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("HTTP server error")?;

    println!("Trace viewer stopped.");
    Ok(())
}

/// `jet trace show <file>` — print manifest summary to stdout.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R5
pub fn show(file: &PathBuf) -> Result<()> {
    if !file.exists() {
        anyhow::bail!("Trace file not found: {}", file.display());
    }
    let manifest = read_manifest_from_zip(file)
        .with_context(|| format!("Invalid trace format in {}", file.display()))?;

    let duration_ms = manifest.finished_at.saturating_sub(manifest.started_at);
    let action_count = manifest
        .events
        .iter()
        .filter(|e| matches!(e, crate::trace::manifest::TraceEvent::ActionStep(_)))
        .count();
    let network_count = manifest
        .events
        .iter()
        .filter(|e| matches!(e, crate::trace::manifest::TraceEvent::Network(_)))
        .count();
    let console_count = manifest
        .events
        .iter()
        .filter(|e| matches!(e, crate::trace::manifest::TraceEvent::Console(_)))
        .count();

    println!("Test:    {}", manifest.test_title);
    println!("File:    {}", manifest.spec_file);
    println!("Outcome: {}", manifest.outcome);
    println!("Duration: {duration_ms}ms");
    println!("Steps:   {action_count} actions, {network_count} network, {console_count} console");
    println!("Assets:  {}", manifest.assets.len());
    Ok(())
}

/// `jet trace extract <file> <dir>` — extract the zip to a directory.
// @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R5
pub fn extract(file: &PathBuf, dir: &PathBuf) -> Result<()> {
    if !file.exists() {
        anyhow::bail!("Trace file not found: {}", file.display());
    }
    std::fs::create_dir_all(dir)
        .with_context(|| format!("Failed to create output dir: {}", dir.display()))?;

    let f =
        std::fs::File::open(file).with_context(|| format!("Failed to open {}", file.display()))?;
    let mut archive = zip::ZipArchive::new(f)
        .with_context(|| format!("Failed to parse zip: {}", file.display()))?;

    // GH #3078 — zip-slip guard: reject `..` and absolute-path entries.
    // We use enclosed_name() (sanitizes traversal) and then canonicalize
    // the parent dir to verify the resolved path actually stays under `dir`.
    let dir_canon = dir
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize output dir: {}", dir.display()))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_name = entry.name().to_string();
        let safe_name = entry.enclosed_name().ok_or_else(|| {
            anyhow::anyhow!(
                "Refusing to extract unsafe zip entry '{}' (path traversal or absolute path)",
                entry_name
            )
        })?;
        let out_path = dir.join(&safe_name);
        if entry_name.ends_with('/') {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
                let parent_canon = parent.canonicalize().with_context(|| {
                    format!("Failed to canonicalize entry parent: {}", parent.display())
                })?;
                if !parent_canon.starts_with(&dir_canon) {
                    anyhow::bail!(
                        "Refusing to extract '{}' — resolved path '{}' escapes output dir '{}'",
                        entry_name,
                        parent_canon.display(),
                        dir_canon.display(),
                    );
                }
            }
            let mut out_file = std::fs::File::create(&out_path)
                .with_context(|| format!("Failed to create {}", out_path.display()))?;
            std::io::copy(&mut entry, &mut out_file)?;
        }
    }

    println!("Extracted to {}", dir.display());
    Ok(())
}

/// Wait for Ctrl-C.
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl-C signal");
}

#[cfg(test)]
mod tests {
    //! GH #3078 — zip-slip guard for `jet trace extract`.

    use super::*;
    use std::io::Write;
    use tempfile::TempDir;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    /// Build a minimal zip in `path` containing `entries` (name → bytes).
    fn build_zip(path: &PathBuf, entries: &[(&str, &[u8])]) {
        let f = std::fs::File::create(path).unwrap();
        let mut z = ZipWriter::new(f);
        let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        for (name, bytes) in entries {
            z.start_file(*name, opts).unwrap();
            z.write_all(bytes).unwrap();
        }
        z.finish().unwrap();
    }

    /// Happy path: normal entries (including subdir) extract correctly.
    #[test]
    fn extract_writes_normal_entries() {
        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("ok.zip");
        build_zip(
            &zip_path,
            &[
                ("manifest.ndjson", b"{}\n"),
                ("assets/screenshot-0.png", b"PNGDATA"),
            ],
        );
        let out_dir = tmp.path().join("out");
        std::fs::create_dir_all(&out_dir).unwrap();

        extract(&zip_path, &out_dir).expect("safe zip must extract");
        assert_eq!(
            std::fs::read_to_string(out_dir.join("manifest.ndjson")).unwrap(),
            "{}\n"
        );
        assert_eq!(
            std::fs::read(out_dir.join("assets/screenshot-0.png")).unwrap(),
            b"PNGDATA"
        );
    }

    /// Zip-slip: an entry named `../escape.txt` must be rejected, and no
    /// file may be written outside `out_dir`.
    #[test]
    fn extract_rejects_dot_dot_traversal() {
        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("evil.zip");
        build_zip(
            &zip_path,
            &[("manifest.ndjson", b"{}\n"), ("../escape.txt", b"PWN")],
        );
        let out_dir = tmp.path().join("out");
        std::fs::create_dir_all(&out_dir).unwrap();

        let err = extract(&zip_path, &out_dir).expect_err("zip with `..` entry must be rejected");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("path traversal")
                || msg.contains("escapes output dir")
                || msg.contains("unsafe zip entry"),
            "diagnostic must explain why, got: {msg}",
        );
        // Sibling-of-out_dir file path must NOT exist.
        assert!(
            !tmp.path().join("escape.txt").exists(),
            "no file may be written outside out_dir"
        );
    }

    /// Zip-slip: an absolute-path entry (`/tmp/pwn.txt`) must also be
    /// rejected by `enclosed_name()`.
    #[test]
    fn extract_rejects_absolute_path_entry() {
        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("abs.zip");
        build_zip(
            &zip_path,
            &[("manifest.ndjson", b"{}\n"), ("/abs/pwn.txt", b"PWN")],
        );
        let out_dir = tmp.path().join("out");
        std::fs::create_dir_all(&out_dir).unwrap();

        let err = extract(&zip_path, &out_dir)
            .expect_err("zip with absolute-path entry must be rejected");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("unsafe zip entry") || msg.contains("path traversal"),
            "diagnostic must explain why, got: {msg}",
        );
    }
}
// CODEGEN-END
