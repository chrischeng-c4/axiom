// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Integration tests for `browser::install` and the cache-first launcher.
//!
//! T1 is gated `#[ignore]` and requires real network access plus ~300 MB disk.
//! Run it manually:
//!   cargo test -p jet --test browser_install -- --ignored
//!
//! T2 and T3 run in normal CI (no network required).

// REQ: REQ-R2
// REQ: REQ-R3
// REQ: REQ-R5
// REQ: REQ-R7

use jet::browser::install::DEFAULT_CHROMIUM_REVISION;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

// ---------------------------------------------------------------------------
// T1 — full download + launch (network required, #[ignore])
// ---------------------------------------------------------------------------

/// T1: Download the pinned Chromium revision, assert the binary is executable,
/// launch a browser, navigate to about:blank, close cleanly.
///
/// REQ: REQ-R2, REQ-R3, REQ-R7
#[tokio::test]
#[ignore = "requires network and ~300 MB disk; run with -- --ignored"]
async fn install_chromium_downloads_and_is_launchable() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = jet::browser::install::install_chromium(DEFAULT_CHROMIUM_REVISION, tmp.path())
        .await
        .expect("install_chromium should succeed");

    assert!(path.exists(), "binary path must exist: {}", path.display());
    assert!(path.is_file(), "binary path must be a file");

    #[cfg(unix)]
    {
        let mode = fs::metadata(&path)
            .expect("stat binary")
            .permissions()
            .mode();
        assert!(mode & 0o111 != 0, "binary must have executable bit set");
    }

    let browser = jet::browser::Browser::launch(jet::browser::LaunchOptions {
        executable: Some(path),
        ..Default::default()
    })
    .await
    .expect("Browser::launch should succeed");

    let page = browser.new_page().await.expect("new_page");
    page.goto("about:blank").await.expect("goto about:blank");
    browser.close().await.expect("browser.close");
}

// ---------------------------------------------------------------------------
// T2 — cache-first preference (no network, runs in CI)
// ---------------------------------------------------------------------------

/// T2: Given two cached revision directories, `find_chrome_in` returns the
/// binary from the highest revision number (newest wins).
///
/// REQ: REQ-R5
#[test]
fn find_chrome_prefers_cache() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cache_root = tmp.path();

    // Determine platform binary subpath (mirrors install.rs logic).
    let binary_subpath: &str = if cfg!(target_os = "macos") {
        "chrome-mac/Chromium.app/Contents/MacOS/Chromium"
    } else if cfg!(target_os = "linux") {
        "chrome-linux/chrome"
    } else {
        "chrome-win/chrome.exe"
    };

    // Create two fake cached revisions.
    for rev in &["1300000", "1331488"] {
        let bin = cache_root
            .join(format!("chromium-{}", rev))
            .join(binary_subpath);
        fs::create_dir_all(bin.parent().unwrap()).expect("create dir");
        fs::write(&bin, b"#!/bin/sh\n# fake chrome\n").expect("write fake binary");

        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&bin).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&bin, perms).unwrap();
        }
    }

    let found = jet::browser::launcher::BrowserLauncher::find_chrome_in(cache_root)
        .expect("should find cached binary");

    // Must return the newer revision (1331488), not the older one (1300000).
    let found_str = found.to_string_lossy();
    assert!(
        found_str.contains("1331488"),
        "Expected path to contain revision 1331488, got: {}",
        found_str
    );
    assert!(
        !found_str.contains("1300000"),
        "Must NOT return the older revision 1300000; got: {}",
        found_str
    );
}

// ---------------------------------------------------------------------------
// T3 — unsupported platform returns clear error (no network)
// ---------------------------------------------------------------------------

/// T3: Calling the test-only `install_chromium_for` with an unsupported
/// (os, arch) combo must return `Err` with a message that names the OS and
/// hints at supported platforms.
///
/// REQ: REQ-R3
#[tokio::test]
async fn unsupported_platform_returns_clear_error() {
    let tmp = tempfile::tempdir().expect("tempdir");

    let err =
        jet::browser::install::install_chromium_for("freebsd", "x86_64", "1331488", tmp.path())
            .await
            .expect_err("should fail on unsupported platform");

    let msg = err.to_string();
    assert!(
        msg.contains("freebsd"),
        "Error message must mention the unsupported OS; got: {}",
        msg
    );
    assert!(
        msg.to_lowercase().contains("supported") || msg.to_lowercase().contains("platform"),
        "Error message must hint at supported platforms; got: {}",
        msg
    );

    // No filesystem writes should have occurred under tmp.
    let entries: Vec<_> = fs::read_dir(tmp.path())
        .expect("read_dir")
        .filter_map(|e| e.ok())
        .collect();
    assert!(
        entries.is_empty(),
        "No files should have been written for unsupported platform; found: {:?}",
        entries.iter().map(|e| e.path()).collect::<Vec<_>>()
    );
}
// CODEGEN-END
