// HANDWRITE-BEGIN gap="missing-generator:unit-test:5ca87ac5" tracker="pending-tracker" reason="Prove exact provider commands, recoverable missing binaries, real shell bidirectional IO, selected cwd, resize, interrupt, exit, and cleanup."
use std::path::PathBuf;

use workbench::native_agent_pty::{
    AgentKind, AgentLaunchCommand, PtyCommand, PtyLaunchError, PtyRuntime, PtySession, PtySize,
};

#[cfg(unix)]
use std::io::{BufRead, BufReader, Read};
#[cfg(unix)]
use std::process::Command;
#[cfg(unix)]
use std::sync::mpsc::{self, Receiver};
#[cfg(unix)]
use std::thread;
#[cfg(unix)]
use std::time::{Duration, Instant};

fn test_size() -> PtySize {
    PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    }
}

/// @spec apps/workbench/tech-design/interfaces/cli/launch-native-claude-code-codex-and-agy-clis-through-a-real-pty.md#unit-test
#[test]
fn adapter_commands_are_exact_and_claude_is_default() {
    assert_eq!(AgentKind::default(), AgentKind::ClaudeCode);
    let cwd = PathBuf::from("/tmp/workbench-selected-folder");
    let expected = [
        (AgentKind::ClaudeCode, "claude", "Claude Code"),
        (AgentKind::Codex, "codex", "Codex"),
        (AgentKind::Agy, "agy", "AGY"),
    ];

    for (kind, program, label) in expected {
        let command = AgentLaunchCommand::for_kind(kind, &cwd);
        assert_eq!(command.agent, kind);
        assert_eq!(command.program, PathBuf::from(program));
        assert!(command.args.is_empty(), "{label} gained hidden arguments");
        assert_eq!(command.cwd, cwd);
        assert_eq!(kind.program(), program);
        assert_eq!(kind.label(), label);
        assert_eq!(command.as_pty_command().program, PathBuf::from(program));
    }
    assert_eq!(AgentKind::ALL, expected.map(|(kind, _, _)| kind));
}

/// @spec apps/workbench/tech-design/interfaces/cli/launch-native-claude-code-codex-and-agy-clis-through-a-real-pty.md#unit-test
#[cfg(unix)]
#[test]
fn missing_vendor_binaries_are_recoverable() {
    let empty_path = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let runtime = PtyRuntime::with_search_path(empty_path.path().as_os_str());

    for kind in AgentKind::ALL {
        let command = AgentLaunchCommand::for_kind(kind, cwd.path());
        match runtime.spawn_agent(&command, test_size()) {
            Err(PtyLaunchError::UnavailableBinary { program }) => {
                assert_eq!(program, PathBuf::from(kind.program()));
            }
            Err(other) => panic!("unexpected {kind:?} launch error: {other}"),
            Ok(_) => panic!("{kind:?} unexpectedly resolved inside an empty PATH"),
        }
    }

    let status = runtime
        .spawn(
            &PtyCommand::new("/bin/sh", cwd.path()).args(["-c", "exit 0"]),
            test_size(),
        )
        .unwrap()
        .wait()
        .unwrap();
    assert!(
        status.success(),
        "failed vendor launch poisoned the runtime"
    );
}

/// @spec apps/workbench/tech-design/interfaces/cli/launch-native-claude-code-codex-and-agy-clis-through-a-real-pty.md#unit-test
#[cfg(unix)]
#[test]
fn real_pty_round_trip_resize_cwd_and_exit() {
    let cwd = tempfile::tempdir().unwrap();
    let canonical_cwd = cwd.path().canonicalize().unwrap();
    let script = concat!(
        "printf 'READY:%s\\n' \"$PWD\"; ",
        "IFS= read -r line; ",
        "printf 'ECHO:%s\\n' \"$line\"; ",
        "stty size; ",
        "exit 7"
    );
    let command = PtyCommand::new("/bin/sh", &canonical_cwd).args(["-c", script]);
    let mut session = PtySession::spawn(&command, test_size()).unwrap();
    let mut reader = session.try_clone_reader().unwrap();
    let output_thread = thread::spawn(move || {
        let mut output = String::new();
        reader.read_to_string(&mut output).unwrap();
        output
    });

    let resized = PtySize {
        rows: 42,
        cols: 132,
        pixel_width: 0,
        pixel_height: 0,
    };
    session.resize(resized).unwrap();
    let observed = session.size().unwrap();
    assert_eq!((observed.rows, observed.cols), (42, 132));
    session.write_all(b"hello-from-workbench\n").unwrap();
    let status = session.wait().unwrap();
    let output = output_thread.join().unwrap();

    assert_eq!(status.exit_code(), 7);
    assert!(
        output.contains(&format!("READY:{}", canonical_cwd.display())),
        "selected cwd missing from PTY output: {output:?}"
    );
    assert!(output.contains("ECHO:hello-from-workbench"), "{output:?}");
    assert!(
        output.contains("42 132"),
        "resized terminal missing: {output:?}"
    );
}

/// @spec apps/workbench/tech-design/interfaces/cli/launch-native-claude-code-codex-and-agy-clis-through-a-real-pty.md#unit-test
#[cfg(unix)]
#[test]
fn interactive_pty_forces_color_capabilities_without_host_no_color() {
    let cwd = tempfile::tempdir().unwrap();
    let script = "printf 'TERM=%s COLORTERM=%s NO_COLOR_SET=%s\\n' \"$TERM\" \"$COLORTERM\" \"${NO_COLOR+x}\"";
    let command = PtyCommand::new("/bin/sh", cwd.path()).args(["-c", script]);
    let session = PtySession::spawn(&command, test_size()).unwrap();
    let mut reader = session.try_clone_reader().unwrap();
    let output_thread = thread::spawn(move || {
        let mut output = String::new();
        reader.read_to_string(&mut output).unwrap();
        output
    });

    assert!(session.wait().unwrap().success());
    let output = output_thread.join().unwrap();
    assert!(output.contains("TERM=xterm-256color"), "{output:?}");
    assert!(output.contains("COLORTERM=truecolor"), "{output:?}");
    assert!(output.contains("NO_COLOR_SET="), "{output:?}");
}

#[cfg(unix)]
fn line_reader(reader: Box<dyn Read + Send>) -> (Receiver<String>, thread::JoinHandle<String>) {
    let (sender, receiver) = mpsc::channel();
    let handle = thread::spawn(move || {
        let mut reader = BufReader::new(reader);
        let mut complete = String::new();
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    complete.push_str(&line);
                    let _ = sender.send(line);
                }
                Err(error) => {
                    complete.push_str(&format!("<read error: {error}>"));
                    break;
                }
            }
        }
        complete
    });
    (receiver, handle)
}

#[cfg(unix)]
fn receive_until(receiver: &Receiver<String>, needle: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut observed = String::new();
    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match receiver.recv_timeout(remaining.min(Duration::from_millis(250))) {
            Ok(line) => {
                observed.push_str(&line);
                if observed.contains(needle) {
                    return observed;
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    panic!("PTY output never contained {needle:?}: {observed:?}");
}

#[cfg(unix)]
fn process_is_alive(pid: u32) -> bool {
    Command::new("/bin/kill")
        .args(["-0", &pid.to_string()])
        .output()
        .is_ok_and(|output| output.status.success())
}

#[cfg(unix)]
fn assert_process_stops(pid: u32) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if !process_is_alive(pid) {
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("PTY child {pid} remained alive after cleanup");
}

/// @spec apps/workbench/tech-design/interfaces/cli/launch-native-claude-code-codex-and-agy-clis-through-a-real-pty.md#unit-test
#[cfg(unix)]
#[test]
fn real_pty_interrupt_and_termination_cleanup() {
    let cwd = tempfile::tempdir().unwrap();
    let interrupt_script = concat!(
        "trap 'printf \\\"INTERRUPTED\\\\n\\\"; exit 23' INT; ",
        "printf 'SIGNAL_READY\\n'; ",
        "while :; do sleep 1; done"
    );
    let mut session = PtySession::spawn(
        &PtyCommand::new("/bin/sh", cwd.path()).args(["-c", interrupt_script]),
        test_size(),
    )
    .unwrap();
    let (lines, reader_thread) = line_reader(session.try_clone_reader().unwrap());
    receive_until(&lines, "SIGNAL_READY");
    session.interrupt().unwrap();
    receive_until(&lines, "INTERRUPTED");
    let status = session.wait().unwrap();
    let output = reader_thread.join().unwrap();
    assert_eq!(status.exit_code(), 23, "{output:?}");

    let mut terminated = PtySession::spawn(
        &PtyCommand::new("/bin/sh", cwd.path()).args(["-c", "exec sleep 30"]),
        test_size(),
    )
    .unwrap();
    let terminated_pid = terminated.process_id().unwrap();
    assert!(!terminated.terminate().unwrap().success());
    assert_eq!(terminated.process_id(), None);
    assert_process_stops(terminated_pid);

    let abandoned = PtySession::spawn(
        &PtyCommand::new("/bin/sh", cwd.path()).args(["-c", "exec sleep 30"]),
        test_size(),
    )
    .unwrap();
    let abandoned_pid = abandoned.process_id().unwrap();
    drop(abandoned);
    assert_process_stops(abandoned_pid);
}

/// @spec apps/workbench/tech-design/interfaces/cli/launch-native-claude-code-codex-and-agy-clis-through-a-real-pty.md#unit-test
#[test]
fn runtime_has_no_vendor_session_model_or_required_vendor_smoke() {
    let runtime = include_str!("../src/native_agent_pty.rs");
    for forbidden in [
        "resume_id",
        "conversation_store",
        "session_database",
        "history_path",
    ] {
        assert!(
            !runtime.contains(forbidden),
            "runtime owns forbidden {forbidden}"
        );
    }
    assert!(runtime.contains("impl Drop for PtySession"));

    let tests = include_str!("pty_agent_adapters.rs");
    for vendor in ["claude", "codex", "agy"] {
        let forbidden = format!("PtyCommand::new({vendor:?}");
        assert!(
            !tests.contains(&forbidden),
            "test requires installed vendor: {forbidden}"
        );
    }
}

// HANDWRITE-END
