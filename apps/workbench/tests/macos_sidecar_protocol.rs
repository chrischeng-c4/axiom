// HANDWRITE-BEGIN gap="missing-generator:unit-test:eccdc595" tracker="pending-tracker" reason="Exercise the built sidecar through its exact protocol and real shell PTYs for versioning, bytes, cwd, isolation, lifecycle, and failures."
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::Mutex,
    thread,
    time::{Duration, Instant},
};

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use serde_json::{json, Value};

const TERMINAL_CORE_SOURCE: &str = include_str!("../src/terminal_core.rs");
static TEST_LOCK: Mutex<()> = Mutex::new(());

struct Sidecar {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_request_id: u64,
}

impl Sidecar {
    fn spawn() -> Self {
        Self::spawn_with_path(None)
    }

    fn spawn_with_path(path: Option<&str>) -> Self {
        let mut command = Command::new(env!("CARGO_BIN_EXE_workbench-core"));
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        if let Some(path) = path {
            command.env("PATH", path);
        }
        let mut child = command.spawn().expect("start workbench-core sidecar");
        let stdin = child.stdin.take().expect("sidecar stdin");
        let stdout = BufReader::new(child.stdout.take().expect("sidecar stdout"));
        Self {
            child,
            stdin,
            stdout,
            next_request_id: 1,
        }
    }

    fn send(&mut self, method: &str, params: Value) -> Value {
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        self.send_with_id(request_id, 1, method, params)
    }

    fn send_with_id(
        &mut self,
        request_id: u64,
        protocol_version: u16,
        method: &str,
        params: Value,
    ) -> Value {
        let request = json!({
            "protocolVersion": protocol_version,
            "requestId": request_id,
            "method": method,
            "params": params,
        });
        serde_json::to_writer(&mut self.stdin, &request).expect("write request JSON");
        self.stdin.write_all(b"\n").expect("write request frame");
        self.stdin.flush().expect("flush request");
        let mut line = String::new();
        assert!(
            self.stdout.read_line(&mut line).expect("read response") > 0,
            "sidecar closed before responding"
        );
        let response: Value = serde_json::from_str(&line).expect("response JSON");
        assert_eq!(response["protocolVersion"], 1);
        assert_eq!(response["requestId"], request_id);
        response
    }
}

impl Drop for Sidecar {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = serde_json::to_writer(
                &mut self.stdin,
                &json!({
                    "protocolVersion": 1,
                    "requestId": self.next_request_id,
                    "method": "shutdown",
                    "params": {},
                }),
            );
            let _ = self.stdin.write_all(b"\n");
            let _ = self.stdin.flush();
            let _ = self.child.wait();
        }
    }
}

fn session_frame(response: &Value) -> &Value {
    assert_eq!(response["ok"], true, "sidecar error: {response}");
    assert_eq!(response["result"]["kind"], "session");
    &response["result"]["frame"]
}

fn poll_until_exited(sidecar: &mut Sidecar, tab_id: &str) -> (Vec<u8>, Value) {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut output = Vec::new();
    loop {
        let response = sidecar.send("poll", json!({ "tabId": tab_id }));
        let frame = session_frame(&response);
        output.extend(
            BASE64_STANDARD
                .decode(frame["outputBase64"].as_str().unwrap_or_default())
                .expect("frame Base64"),
        );
        if frame["snapshot"]["running"] == false {
            return (output, frame["snapshot"].clone());
        }
        assert!(Instant::now() < deadline, "tab {tab_id} did not exit");
        thread::sleep(Duration::from_millis(20));
    }
}

fn wait_for_interactive_shell(sidecar: &mut Sidecar, tab_id: &str) {
    thread::sleep(Duration::from_millis(150));
    let response = sidecar.send("poll", json!({ "tabId": tab_id }));
    assert_eq!(session_frame(&response)["snapshot"]["running"], true);
}

#[test]
fn protocol_version_and_invalid_requests_fail_closed() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let mut sidecar = Sidecar::spawn();
    let hello = sidecar.send("hello", json!({}));
    assert_eq!(hello["ok"], true);
    assert_eq!(
        hello["result"]["profiles"],
        json!(["claude", "codex", "agy", "shell"])
    );

    let wrong_version = sidecar.send_with_id(70, 99, "hello", json!({}));
    assert_eq!(wrong_version["error"]["code"], "unsupportedVersion");
    let accepted = sidecar.send_with_id(71, 1, "hello", json!({}));
    assert_eq!(accepted["ok"], true);
    let duplicate = sidecar.send_with_id(71, 1, "hello", json!({}));
    assert_eq!(duplicate["error"]["code"], "duplicateRequest");

    let invalid_tab = sidecar.send(
        "launch",
        json!({
            "tabId": "unsafe/tab",
            "profile": "shell",
            "cwd": std::env::temp_dir(),
        }),
    );
    assert_eq!(invalid_tab["error"]["code"], "invalidTab");
    let missing = sidecar.send("poll", json!({ "tabId": "never-started" }));
    assert_eq!(missing["error"]["code"], "missingSession");
}

#[test]
fn default_shell_launches_in_selected_folder_and_preserves_bytes() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    assert!(TERMINAL_CORE_SOURCE.contains("libc::getpwuid"));
    assert!(!TERMINAL_CORE_SOURCE.contains("PathBuf::from(\"/bin/zsh\")"));

    let folder = tempfile::tempdir().expect("selected folder");
    let canonical = folder
        .path()
        .canonicalize()
        .expect("canonical selected folder");
    let mut sidecar = Sidecar::spawn();
    let launched = sidecar.send(
        "launch",
        json!({
            "tabId": "shell-default",
            "profile": "shell",
            "cwd": canonical,
            "rows": 31,
            "cols": 111,
        }),
    );
    let frame = session_frame(&launched);
    assert_eq!(
        frame["snapshot"]["activeCwd"],
        canonical.to_string_lossy().as_ref()
    );
    assert_eq!(frame["snapshot"]["label"], "Shell");
    wait_for_interactive_shell(&mut sidecar, "shell-default");

    let script = b"printf '\\033[31mRAW:%s\\033[0m\\n' \"$PWD\"; exit 7\n";
    let input = sidecar.send(
        "input",
        json!({
            "tabId": "shell-default",
            "dataBase64": BASE64_STANDARD.encode(script),
        }),
    );
    assert_eq!(input["ok"], true);
    let mut output = BASE64_STANDARD
        .decode(
            input["result"]["frame"]["outputBase64"]
                .as_str()
                .unwrap_or_default(),
        )
        .expect("input response Base64");
    let (remaining, snapshot) = poll_until_exited(&mut sidecar, "shell-default");
    output.extend(remaining);
    assert!(
        output.windows(8).any(|window| window == b"\x1b[31mRAW"),
        "raw terminal bytes: {output:?}"
    );
    assert!(String::from_utf8_lossy(&output).contains(canonical.to_string_lossy().as_ref()));
    assert_eq!(snapshot["exitCode"], 7);
}

#[test]
fn tab_sessions_keep_io_and_lifecycle_isolated() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let first = tempfile::tempdir().expect("first cwd");
    let second = tempfile::tempdir().expect("second cwd");
    let mut sidecar = Sidecar::spawn();
    for (tab_id, cwd) in [("tab-one", first.path()), ("tab-two", second.path())] {
        let response = sidecar.send(
            "launch",
            json!({ "tabId": tab_id, "profile": "shell", "cwd": cwd }),
        );
        assert_eq!(response["ok"], true);
    }
    wait_for_interactive_shell(&mut sidecar, "tab-one");
    wait_for_interactive_shell(&mut sidecar, "tab-two");
    let already_running = sidecar.send(
        "launch",
        json!({ "tabId": "tab-one", "profile": "shell", "cwd": first.path() }),
    );
    assert_eq!(already_running["error"]["code"], "alreadyRunning");

    let one = sidecar.send(
        "input",
        json!({
            "tabId": "tab-one",
            "dataBase64": BASE64_STANDARD.encode(b"printf 'ONE:%s\\n' \"$PWD\"; exit 0\n"),
        }),
    );
    assert_eq!(one["ok"], true);
    let two = sidecar.send(
        "input",
        json!({
            "tabId": "tab-two",
            "dataBase64": BASE64_STANDARD.encode(b"printf 'TWO:%s\\n' \"$PWD\"; while :; do sleep 1; done\n"),
        }),
    );
    assert_eq!(two["ok"], true);
    let resized = sidecar.send(
        "resize",
        json!({ "tabId": "tab-two", "rows": 42, "cols": 132 }),
    );
    assert_eq!(resized["ok"], true);

    let (one_output, one_snapshot) = poll_until_exited(&mut sidecar, "tab-one");
    let one_text = String::from_utf8_lossy(&one_output);
    assert!(one_text.contains("ONE:"));
    assert!(!one_text.contains("TWO:"));
    assert_eq!(
        one_snapshot["activeCwd"],
        first
            .path()
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .as_ref()
    );

    let interrupted = sidecar.send("interrupt", json!({ "tabId": "tab-two" }));
    assert_eq!(interrupted["ok"], true);
    thread::sleep(Duration::from_millis(40));
    let terminated = sidecar.send("terminate", json!({ "tabId": "tab-two" }));
    let terminal_frame = session_frame(&terminated);
    let two_output = BASE64_STANDARD
        .decode(terminal_frame["outputBase64"].as_str().unwrap_or_default())
        .expect("second output Base64");
    let two_text = String::from_utf8_lossy(&two_output);
    assert!(!two_text.contains("ONE:"));
    assert_eq!(terminal_frame["snapshot"]["running"], false);
    assert_eq!(
        terminal_frame["snapshot"]["activeCwd"],
        second
            .path()
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .as_ref()
    );

    let relaunch = sidecar.send(
        "launch",
        json!({ "tabId": "tab-one", "profile": "shell", "cwd": second.path() }),
    );
    assert_eq!(relaunch["ok"], true);
    assert_eq!(
        relaunch["result"]["frame"]["snapshot"]["activeCwd"],
        second
            .path()
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .as_ref()
    );
}

#[test]
fn agent_resolution_errors_are_recoverable() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let empty_path = tempfile::tempdir().expect("empty PATH");
    let selected = tempfile::tempdir().expect("selected cwd");
    let mut sidecar = Sidecar::spawn_with_path(Some(empty_path.path().to_str().unwrap()));
    for profile in ["claude", "codex", "agy"] {
        let response = sidecar.send(
            "launch",
            json!({ "tabId": format!("missing-{profile}"), "profile": profile, "cwd": selected.path() }),
        );
        assert_eq!(response["error"]["code"], "unavailableProgram");
    }
    let shell = sidecar.send(
        "launch",
        json!({ "tabId": "still-usable", "profile": "shell", "cwd": selected.path() }),
    );
    assert_eq!(shell["ok"], true);
    let stop = sidecar.send("terminate", json!({ "tabId": "still-usable" }));
    assert_eq!(stop["ok"], true);
}

// HANDWRITE-END
