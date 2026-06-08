// HANDWRITE-BEGIN gap="missing-generator:hand-written:63c8d753" tracker="standardize-gap-projects-mamba-tests-runtime-shutdown-conformance-tests-rs" reason="Existing hand-written code in projects/mamba/tests/runtime_shutdown_conformance_tests.rs requires tracked generator coverage."
// "
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

fn mamba_bin() -> PathBuf {
    option_env!("CARGO_BIN_EXE_mamba")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/mamba")
        })
}

fn fixture_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/cpython")
        .join("fixtures")
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
        "core/imports/test_import.py",
        "std-libs/itertools/edges.py",
        "std-libs/re/broad.py",
        "std-libs/re/ops_broad.py",
    ] {
        assert_fixture_exits_successfully(rel);
    }
}
// HANDWRITE-END
