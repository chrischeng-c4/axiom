// HANDWRITE-BEGIN gap="missing-generator:unit-test:746d21b3" tracker="pending-tracker" reason="Spawn the real desktop binary, wait for host-ready, request shutdown, and require a clean bounded exit."
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const READY_TIMEOUT: Duration = Duration::from_secs(20);
const EXIT_TIMEOUT: Duration = Duration::from_secs(10);

/// @spec apps/workbench/tech-design/interfaces/rest/bootstrap-workbench-product-contract-and-runnable-desktop-applic.md#unit-test
#[test]
#[cfg_attr(
    not(target_os = "macos"),
    ignore = "native launch smoke currently runs on macOS"
)]
fn launches_native_window_and_exits_cleanly() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_workbench"))
        .env("WORKBENCH_SMOKE_CONTROL", "stdio")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn the real Workbench desktop binary");

    let stdout = child.stdout.take().expect("capture Workbench stdout");
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let _ = sender.send(line);
        }
    });

    let ready_deadline = Instant::now() + READY_TIMEOUT;
    let mut saw_ready = false;
    while Instant::now() < ready_deadline {
        let remaining = ready_deadline.saturating_duration_since(Instant::now());
        match receiver.recv_timeout(remaining) {
            Ok(line) if line == workbench::HOST_READY_MARKER => {
                saw_ready = true;
                break;
            }
            Ok(_) => continue,
            Err(_) => break,
        }
    }

    if !saw_ready {
        fail_child(&mut child, "desktop host never emitted its ready marker");
    }

    let stdin = child.stdin.as_mut().expect("capture Workbench stdin");
    stdin.write_all(b"shutdown\n").expect("request shutdown");
    stdin.flush().expect("flush shutdown request");

    let status = wait_for_exit(&mut child, EXIT_TIMEOUT)
        .unwrap_or_else(|| fail_child(&mut child, "desktop host did not exit after shutdown"));
    assert!(status.success(), "desktop host exited with {status}");
}

/// @spec apps/workbench/tech-design/interfaces/rest/bootstrap-workbench-product-contract-and-runnable-desktop-applic.md#unit-test
#[test]
fn desktop_configuration_is_local_and_bounded() {
    let config: serde_json::Value =
        serde_json::from_str(include_str!("../tauri.conf.json")).expect("valid Tauri config");
    assert_eq!(config["build"]["frontendDist"], "ui");
    assert_eq!(config["app"]["windows"].as_array().map(Vec::len), Some(1));
    assert_eq!(config["app"]["windows"][0]["label"], "main");
    assert_eq!(config["bundle"]["active"], false);

    let document = include_str!("../ui/index.html");
    assert!(document.contains("Workbench"));
    assert!(!document.contains("http://"));
    assert!(!document.contains("https://"));
    assert!(!document.contains("TODO"));
}

/// @spec apps/workbench/tech-design/interfaces/rest/bootstrap-workbench-product-contract-and-runnable-desktop-applic.md#unit-test
#[test]
fn product_contract_keeps_native_agents_authoritative() {
    let contract = format!(
        "{}\n{}",
        include_str!("../README.md"),
        include_str!("../CAPABILITIES.md")
    );
    for agent in ["Claude Code", "Codex", "AGY"] {
        assert!(
            contract.contains(agent),
            "missing native-agent boundary for {agent}"
        );
    }
    assert!(contract.contains("authoritative"));
    assert!(contract.contains("optional"));
    assert!(contract.contains("read-only"));
}

/// @spec apps/workbench/tech-design/interfaces/rest/bootstrap-workbench-product-contract-and-runnable-desktop-applic.md#unit-test
#[test]
fn bootstrap_surface_excludes_later_slice_ownership() {
    let source = format!(
        "{}\n{}",
        include_str!("../src/lib.rs"),
        include_str!("../src/main.rs")
    );
    for forbidden in [
        "pub mod pty",
        "pub struct TerminalSession",
        "pub trait ContextRenderer",
        "aw::",
    ] {
        assert!(
            !source.contains(forbidden),
            "bootstrap owns forbidden surface {forbidden}"
        );
    }
}

fn wait_for_exit(child: &mut Child, timeout: Duration) -> Option<std::process::ExitStatus> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Some(status) = child.try_wait().expect("poll Workbench process") {
            return Some(status);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    None
}

fn fail_child(child: &mut Child, message: &str) -> ! {
    let _ = child.kill();
    let _ = child.wait();
    let mut stderr = String::new();
    if let Some(mut stream) = child.stderr.take() {
        let _ = stream.read_to_string(&mut stderr);
    }
    panic!("{message}; stderr: {stderr}");
}
// HANDWRITE-END
