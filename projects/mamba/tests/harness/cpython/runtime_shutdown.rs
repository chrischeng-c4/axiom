// HANDWRITE-BEGIN gap="missing-generator:hand-written:63c8d753" tracker="standardize-gap-projects-mamba-tests-runtime-shutdown-conformance-tests-rs" reason="Existing hand-written code in projects/mamba/tests/runtime_shutdown_conformance_tests.rs requires tracked generator coverage."
// "
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

#[path = "harness_common.rs"]
mod common;
use common::mamba_bin;

fn fixture_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/cpython")
        .join(rel)
}

fn status_detail(status: ExitStatus) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return format!("signal {signal}");
        }
    }

    match status.code() {
        Some(code) => format!("exit code {code}"),
        None => "unknown process status".to_string(),
    }
}

fn assert_fixture_exits_successfully(rel: &str) {
    let fixture = fixture_path(rel);
    assert!(
        Path::new(&fixture).exists(),
        "missing conformance fixture {}",
        fixture.display()
    );

    let output = Command::new(mamba_bin())
        .arg("run")
        .arg(&fixture)
        .output()
        .unwrap_or_else(|err| panic!("failed to execute mamba for {rel}: {err}"));

    assert!(
        output.status.success(),
        "mamba run {rel} ended with {}\nstdout:\n{}\nstderr:\n{}",
        status_detail(output.status),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn previously_crashing_fixtures_exit_without_shutdown_signal() {
    for rel in [
        // Relocated to _regression/ by the dimension-first migration (no-record
        // regression fixture). The std-libs/* entries below predate this change
        // and were already absent on disk.
        "_regression/core/imports/test_import.py",
        "std-libs/itertools/edges.py",
        "std-libs/re/broad.py",
        "std-libs/re/ops_broad.py",
    ] {
        assert_fixture_exits_successfully(rel);
    }
}
// HANDWRITE-END
