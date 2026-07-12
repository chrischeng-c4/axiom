// HANDWRITE-BEGIN gap="missing-generator:e2e-test:dc898059" tracker="pending-tracker" reason="R7/AC3/AC4/AC5: `container_available()` skip helper (mirrors `vat_cluster.rs`'s Docker-gated pattern and `vat_sandbox_microvm.rs`'s container-gated tests) plus `build_fails_missing_dockerfile` (no subprocess, always runs) and the container-gated `build_produces_tagged_image_visible_in_container_image_list` test asserting both a successful `BuildReport` and that singular-noun `container image list` (not the plural `container images`, per the Phase 0 spike's R7 finding) shows the built tag."

use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use vat::commands::build;

/// Skip helper: checks if the `container` CLI is available, mirroring
/// vat_sandbox_microvm.rs's container-gated pattern.
fn container_available() -> bool {
    vat::sandbox::microvm::available()
}

/// AC3: build_image fails cleanly with a clear error when the Dockerfile path
/// does not exist. No subprocess is spawned; this test always runs without
/// requiring the container CLI.
#[test]
fn build_fails_missing_dockerfile() {
    let context = PathBuf::from("/tmp/nonexistent-context");
    let dockerfile = PathBuf::from("/tmp/nonexistent-dockerfile");
    let tag = "test:latest";
    let build_args: Vec<(String, String)> = vec![];

    let result = build::build_image(&context, &dockerfile, tag, &build_args);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("dockerfile not found"),
        "error should mention missing dockerfile: {}",
        err_msg
    );
}

/// AC4/AC5: build_produces_tagged_image_visible_in_container_image_list.
/// Writes a minimal, valid Dockerfile to a tempdir, runs vat build against it,
/// and asserts both a successful BuildReport and that `container image list`
/// (singular noun, not plural) shows the built tag. Skips cleanly when the
/// container CLI is not installed.
#[test]
fn build_produces_tagged_image_visible_in_container_image_list() {
    if !container_available() {
        eprintln!("container CLI not installed; skipping build smoke test");
        return;
    }

    // Create a temporary directory with a minimal Dockerfile.
    let tempdir = tempfile::tempdir().expect("create tempdir");
    let dockerfile_path = tempdir.path().join("Dockerfile");
    let mut dockerfile = File::create(&dockerfile_path).expect("create Dockerfile");
    writeln!(dockerfile, "FROM busybox:latest").expect("write Dockerfile");
    writeln!(dockerfile, "RUN echo 'test build'").expect("write Dockerfile");

    let context = tempdir.path().to_path_buf();
    let tag = "vat-test-build:latest";
    let build_args: Vec<(String, String)> = vec![];

    // Call build_image directly (the in-process entry point).
    let result = build::build_image(&context, &dockerfile_path, tag, &build_args);

    // Assert successful BuildReport.
    assert!(result.is_ok(), "build_image should succeed: {:?}", result);
    let report = result.unwrap();

    assert_eq!(report.tag, tag);
    assert!(
        report.dockerfile.contains("Dockerfile"),
        "dockerfile path should be in report"
    );
    assert_eq!(report.build_args, BTreeMap::new());
    assert!(report.duration_ms > 0);

    // Verify the tag appears in `container image list` (singular noun). The CLI
    // renders NAME and TAG as separate whitespace-padded table columns (e.g.
    // "vat-test-build  latest"), not a single "name:tag" token, so a literal
    // substring check against the colon-joined tag never matches; split on the
    // colon and require both columns on the same row instead.
    let output = Command::new("container")
        .args(&["image", "list"])
        .output()
        .expect("run container image list");

    let image_list = String::from_utf8_lossy(&output.stdout);
    let (image_name, image_tag) = tag.split_once(':').unwrap_or((tag, "latest"));
    let found = image_list.lines().any(|line| {
        let mut cols = line.split_whitespace();
        cols.next() == Some(image_name) && cols.next() == Some(image_tag)
    });
    assert!(
        found,
        "container image list should show name {} tag {}: {}",
        image_name, image_tag, image_list
    );

    // Cleanup: delete the image so it doesn't pollute the system.
    let _ = Command::new("container")
        .args(&["image", "rm", tag])
        .output();
}
// HANDWRITE-END
