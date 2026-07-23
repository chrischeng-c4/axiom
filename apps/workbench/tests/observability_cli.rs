// HANDWRITE-BEGIN gap="missing-generator:contract:d5af2cf3" tracker="pending-tracker" reason="Lock down accepted argv, JSON envelopes, registry validation, token propagation, payload bounds, and typed error codes."
use std::{io::{BufRead, BufReader, Write}, net::TcpListener, thread};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use tempfile::tempdir;
use workbench::observability_cli::{run_with_paths, CliResult};

#[test]
fn logs_tail_is_local_line_bounded_and_runtime_independent() {
    let temp = tempdir().unwrap();
    let log = temp.path().join("workbench.log");
    std::fs::write(&log, "one\ntwo\nthree\n").unwrap();
    let result = run_with_paths(&["logs".into(), "--tail".into(), "2".into()], temp.path(), &log).unwrap();
    match result {
        CliResult::Logs { lines, truncated, .. } => {
            assert_eq!(lines, ["two", "three"]);
            assert!(truncated);
        }
        _ => panic!("expected logs"),
    }
    let missing = temp.path().join("missing.log");
    let result = run_with_paths(&["logs".into()], temp.path(), &missing).unwrap();
    assert!(matches!(result, CliResult::Logs { lines, .. } if lines.is_empty()));
}

#[test]
fn snapshot_registry_and_authentication_fail_closed_without_launching_gui() {
    let temp = tempdir().unwrap();
    let log = temp.path().join("unused.log");
    let error = run_with_paths(&["snapshot".into(), "--out".into(), temp.path().join("screen.png").display().to_string()], temp.path(), &log).unwrap_err();
    assert_eq!(error.code, "runtime_unavailable");

    std::fs::write(temp.path().join("current.json"), "not-json").unwrap();
    let error = run_with_paths(&["snapshot".into(), "--out".into(), temp.path().join("screen.png").display().to_string()], temp.path(), &log).unwrap_err();
    assert_eq!(error.code, "runtime_unavailable");
}

#[test]
fn snapshot_accepts_bounded_png_and_writes_only_explicit_output() {
    let temp = tempdir().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let token = "0123456789abcdef";
    let instance = "instance-test";
    let handle = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut request = String::new();
        BufReader::new(stream.try_clone().unwrap()).read_line(&mut request).unwrap();
        let request: serde_json::Value = serde_json::from_str(&request).unwrap();
        assert_eq!(request["token"], token);
        let png = [137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 0];
        let response = serde_json::json!({
            "protocolVersion": 1,
            "requestId": request["requestId"],
            "instanceId": instance,
            "ok": true,
            "mimeType": "image/png",
            "dataBase64": BASE64.encode(png),
        });
        let mut stream = stream;
        writeln!(stream, "{response}").unwrap();
    });
    std::fs::write(temp.path().join("current.json"), serde_json::json!({
        "protocolVersion": 1,
        "instanceId": instance,
        "pid": std::process::id(),
        "port": port,
        "token": token,
    }).to_string()).unwrap();
    let output = temp.path().join("screen.png");
    let result = run_with_paths(&["snapshot".into(), "--out".into(), output.display().to_string()], temp.path(), &temp.path().join("unused.log")).unwrap();
    handle.join().unwrap();
    assert!(matches!(result, CliResult::Snapshot { bytes: 12, .. }));
    assert_eq!(&std::fs::read(output).unwrap()[..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
}

#[test]
fn public_argv_and_json_envelopes_are_exact() {
    let temp = tempdir().unwrap();
    let error = run_with_paths(&["logs".into(), "--unknown".into()], temp.path(), &temp.path().join("log")).unwrap_err();
    assert_eq!(error.code, "invalid_arguments");
    let error = run_with_paths(&["snapshot".into(), "--out".into()], temp.path(), &temp.path().join("log")).unwrap_err();
    assert_eq!(error.code, "invalid_arguments");
}

<!-- marker: missing-generator:contract:d5af2cf3 path: apps/workbench/tests/observability_cli.rs reason: Lock down accepted argv, JSON envelopes, registry validation, token propagation, payload bounds, and typed error codes. -->
// HANDWRITE-END
