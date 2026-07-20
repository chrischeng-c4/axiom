// HANDWRITE-BEGIN gap="missing-generator:unit-test:1bead46d" tracker="pending-tracker" reason="Prove real-PTY nested cwd transitions, fragmented telemetry, invalid transitions, prompt non-scraping, and folder-registry immutability."
use std::io::Read;
use std::path::Path;
use std::thread;

use workbench::cwd_context::{
    cwd_telemetry_frame, ActiveCwdContext, CwdTelemetryDecoder, CwdTelemetrySource,
    CWD_TELEMETRY_PROTOCOL,
};
use workbench::folder_shell::ShellState;
use workbench::native_agent_pty::{PtyCommand, PtySession, PtySize};

fn test_size() -> PtySize {
    PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    }
}

fn raw_frame(uri: &str) -> Vec<u8> {
    format!("\x1b]7;{uri}\x07").into_bytes()
}

/// @spec apps/workbench/tech-design/logic/synchronize-authoritative-pty-cwd-into-workbench-active-context.md#unit-test
#[test]
fn decoder_is_fragment_safe_and_never_scrapes_ordinary_output() {
    let root = tempfile::tempdir().unwrap();
    let first = root.path().join("first folder");
    let second = root.path().join("second");
    std::fs::create_dir_all(&first).unwrap();
    std::fs::create_dir_all(&second).unwrap();
    let root_path = root.path().canonicalize().unwrap();
    let first_path = first.canonicalize().unwrap();
    let second_path = second.canonicalize().unwrap();
    let mut context = ActiveCwdContext::new(&root_path).unwrap();

    for ordinary in [
        b"prompt cwd=/tmp/forged\n".as_slice(),
        b"$ cd /private/tmp/also-forged\n".as_slice(),
        b"file://localhost/private/tmp/plain-text\n".as_slice(),
        b"\x1b]7".as_slice(),
    ] {
        assert!(context.push_output(ordinary).is_empty());
        assert_eq!(context.current(), root_path);
    }

    let first_frame = cwd_telemetry_frame(&first_path).unwrap();
    let mut updates = Vec::new();
    for byte in first_frame.as_bytes() {
        updates.extend(context.push_output(&[*byte]));
    }
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].path, first_path);

    let mut second_frame = cwd_telemetry_frame(&second_path).unwrap().into_bytes();
    assert_eq!(second_frame.pop(), Some(0x07));
    second_frame.extend_from_slice(b"\x1b\\");
    let midpoint = second_frame.len() / 2;
    assert!(context.push_output(&second_frame[..midpoint]).is_empty());
    let updates = context.push_output(&second_frame[midpoint..]);
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].path, second_path);

    let mut decoder = CwdTelemetryDecoder::default();
    let mut oversized = b"\x1b]7;".to_vec();
    oversized.resize(CwdTelemetryDecoder::max_pending_bytes() + 1, b'x');
    assert!(decoder.push(&oversized).is_empty());
    assert_eq!(decoder.pending_len(), 0);
}

/// @spec apps/workbench/tech-design/logic/synchronize-authoritative-pty-cwd-into-workbench-active-context.md#unit-test
#[test]
fn decoder_validates_local_existing_directories() {
    let root = tempfile::tempdir().unwrap();
    let valid = root.path().join("valid directory");
    let file = root.path().join("not-a-directory.txt");
    std::fs::create_dir_all(&valid).unwrap();
    std::fs::write(&file, b"not a cwd").unwrap();
    let root_path = root.path().canonicalize().unwrap();
    let valid_path = valid.canonicalize().unwrap();
    let mut context = ActiveCwdContext::new(&root_path).unwrap();

    let valid_frame = cwd_telemetry_frame(&valid_path).unwrap();
    let updates = context.push_output(valid_frame.as_bytes());
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].path, valid_path);
    assert_eq!(updates[0].source, CwdTelemetrySource::Osc7);
    assert!(context.push_output(valid_frame.as_bytes()).is_empty());

    let prior = context.current().to_path_buf();
    let invalid = [
        raw_frame("https://localhost/tmp"),
        raw_frame("file://remote.example/tmp"),
        raw_frame("file://localhost/definitely/missing/workbench-cwd"),
        raw_frame(&format!("file://localhost{}", file.display())),
        b"\x1b]7;not a URI\x07".to_vec(),
    ];
    for frame in invalid {
        assert!(context.push_output(&frame).is_empty());
        assert_eq!(context.current(), prior);
    }
}

/// @spec apps/workbench/tech-design/logic/synchronize-authoritative-pty-cwd-into-workbench-active-context.md#unit-test
#[cfg(unix)]
#[test]
fn real_pty_updates_active_context_from_osc7() {
    let root = tempfile::tempdir().unwrap();
    let nested = root.path().join("nested");
    let file = root.path().join("file.txt");
    let missing = root.path().join("missing");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(&file, b"not a directory").unwrap();
    let root_path = root.path().canonicalize().unwrap();
    let nested_path = nested.canonicalize().unwrap();

    let mut folders = ShellState::default();
    folders.register_path(&root_path).unwrap();
    let folder_snapshot = folders.clone();

    let script = concat!(
        "printf 'PROTOCOL=%s\\n' \"$WORKBENCH_CWD_TELEMETRY\"; ",
        "cd \"$1\" && printf '\\033]7;file://localhost%s\\007' \"$PWD\"; ",
        "cd \"$2\" 2>/dev/null || printf 'FAILED_MISSING:%s\\n' \"$2\"; ",
        "cd \"$3\" 2>/dev/null || printf 'FAILED_FILE:%s\\n' \"$3\"; ",
        "printf 'ordinary cwd=/tmp/forged\\n'; exit 0"
    );
    let command = PtyCommand::new("/bin/sh", &root_path).args([
        "-c",
        script,
        "workbench-cwd-fixture",
        nested_path.to_str().unwrap(),
        missing.to_str().unwrap(),
        file.to_str().unwrap(),
    ]);
    let session = PtySession::spawn(&command, test_size()).unwrap();
    let mut reader = session.try_clone_reader().unwrap();
    let output_thread = thread::spawn(move || {
        let mut output = Vec::new();
        reader.read_to_end(&mut output).unwrap();
        output
    });
    assert!(session.wait().unwrap().success());
    let output = output_thread.join().unwrap();

    let mut context = ActiveCwdContext::new(&root_path).unwrap();
    let mut updates = Vec::new();
    for chunk in output.chunks(3) {
        updates.extend(context.push_output(chunk));
    }
    assert_eq!(updates.len(), 1, "{}", String::from_utf8_lossy(&output));
    assert_eq!(updates[0].path, nested_path);
    assert_eq!(updates[0].source, CwdTelemetrySource::Osc7);
    assert_eq!(context.current(), nested_path);
    assert_eq!(folders, folder_snapshot);

    let visible = String::from_utf8_lossy(&output);
    assert!(visible.contains(&format!("PROTOCOL={CWD_TELEMETRY_PROTOCOL}")));
    assert!(visible.contains("FAILED_MISSING"));
    assert!(visible.contains("FAILED_FILE"));
    assert!(visible.contains("ordinary cwd=/tmp/forged"));
}

/// @spec apps/workbench/tech-design/logic/synchronize-authoritative-pty-cwd-into-workbench-active-context.md#unit-test
#[test]
fn failed_transitions_never_mutate_context_or_launch_folders() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("file.txt");
    std::fs::write(&file, b"not a directory").unwrap();
    let root_path = root.path().canonicalize().unwrap();
    let mut folders = ShellState::default();
    folders.register_path(&root_path).unwrap();
    let folder_snapshot = folders.clone();
    let mut context = ActiveCwdContext::new(&root_path).unwrap();

    let failures = [
        b"cd: no such file or directory: /tmp/missing\n".to_vec(),
        raw_frame("file://remote.example/tmp"),
        raw_frame("file://localhost/tmp/workbench-missing-directory"),
        raw_frame(&format!("file://localhost{}", file.display())),
    ];
    for failure in failures {
        assert!(context.push_output(&failure).is_empty());
    }

    assert_eq!(context.current(), root_path);
    assert_eq!(folders, folder_snapshot);
    assert!(cwd_telemetry_frame(&file).is_err());
    assert!(ActiveCwdContext::new(Path::new("/definitely/missing/workbench")).is_err());
}

<!-- marker: missing-generator:unit-test:1bead46d path: apps/workbench/tests/pty_cwd_context.rs reason: Prove real-PTY nested cwd transitions, fragmented telemetry, invalid transitions, prompt non-scraping, and folder-registry immutability. -->
// HANDWRITE-END
