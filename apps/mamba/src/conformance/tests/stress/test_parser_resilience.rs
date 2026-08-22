//! Parser resilience & fuzzing stress tests (#gen12_fuzzing).
//!
//! Verifies parser handling of garbage/malformed inputs, deep nesting,
//! unterminated literals, and extreme indentation without panic or SIGABRT.

use crate::parser;
use crate::source::span::FileId;
use sha2::{Digest, Sha256};

/// Simple deterministic pseudo-random number generator for property-based fuzzing.
struct FuzzRng {
    state: u64,
}

impl FuzzRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.state
    }

    fn next_usize(&mut self, max: usize) -> usize {
        if max == 0 {
            0
        } else {
            (self.next_u64() as usize) % max
        }
    }

    fn fill_bytes(&mut self, buf: &mut [u8]) {
        for chunk in buf.chunks_mut(8) {
            let val = self.next_u64().to_ne_bytes();
            let len = chunk.len();
            chunk.copy_from_slice(&val[..len]);
        }
    }
}

/// Property-based test: 500 iterations of random ASCII and UTF-8 bytes.
/// Verifies parser returns Ok or Err, but NEVER panics or aborts.
#[test]
fn test_parser_random_bytes_resilience() {
    let mut rng = FuzzRng::new(0xDEADBEEF12345678);

    for len in [0, 1, 2, 5, 10, 32, 64, 128, 256, 512] {
        for _ in 0..50 {
            let mut bytes = vec![0u8; len];
            rng.fill_bytes(&mut bytes);

            // Test 1: lossy UTF-8 conversion
            let src = String::from_utf8_lossy(&bytes);
            let _ = std::panic::catch_unwind(|| {
                let _ = parser::parse(&src, FileId(0));
            });

            // Test 2: valid printable ASCII subset with random tokens
            let tokens = ["def ", "class ", "if ", "else: ", "for ", "in ", "return ",
                           "pass", "break", "continue", "import ", "from ", "try: ",
                           "except ", "finally: ", "lambda ", "yield ", "async ",
                           "+", "-", "*", "/", "==", "!=", "=", "(", ")", "[", "]",
                           "{", "}", ":", ",", ".", ";", "\n", "\t", " ", "0", "42",
                           "\"foo\"", "'bar'", "x", "y", "áóí", "None", "True", "False"];
            let mut src_acc = String::new();
            let count = rng.next_usize(30);
            for _ in 0..count {
                let idx = rng.next_usize(tokens.len());
                src_acc.push_str(tokens[idx]);
            }

            let _ = std::panic::catch_unwind(|| {
                let _ = parser::parse(&src_acc, FileId(0));
            });
        }
    }
}

/// Test unterminated string literals (single, double, triple-quoted, raw, byte, f-strings).
#[test]
fn test_parser_unterminated_literals() {
    let prefixes = ["", "r", "b", "f", "rf", "fr", "rb", "br"];
    let quotes = ["'", "\"", "'''", "\"\"\""];

    for prefix in prefixes {
        for quote in quotes {
            let src = format!("{prefix}{quote}hello world this is unterminated");
            let result = std::panic::catch_unwind(|| parser::parse(&src, FileId(0)));
            assert!(result.is_ok(), "parser panicked on unterminated quote {prefix}{quote}");

            // With backslash escapes at the end
            let src_esc = format!("{prefix}{quote}hello world \\");
            let result_esc = std::panic::catch_unwind(|| parser::parse(&src_esc, FileId(0)));
            assert!(result_esc.is_ok(), "parser panicked on escape at EOF in string");
        }
    }
}

const DEEP_PARSER_CHILD_ENV: &str = "MAMBA_DEEP_PARSER_CHILD_V1";
const DEEP_PARSER_EXACT_TEST: &str = "conformance::tests_subdirs::stress::test_parser_resilience::test_parser_deeply_nested_expressions";
const DEEP_PARSER_CHILD_SUMMARY: &str =
    "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured;";
const DEEP_PARSER_MARKERS: [&str; 4] = [
    "MAMBA_DEEP_PARSER_CASE_OK:v1:parentheses:200",
    "MAMBA_DEEP_PARSER_CASE_OK:v1:binary_chain:200",
    "MAMBA_DEEP_PARSER_CASE_OK:v1:attribute_chain:150",
    "MAMBA_DEEP_PARSER_CHILD_OK:v1",
];
const DEEP_PARSER_CHILD_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
const DEEP_PARSER_CLEANUP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
const DEEP_PARSER_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(10);

fn spawn_deep_parser_pipe_drainer<R>(
    mut reader: R,
) -> std::thread::JoinHandle<(Vec<u8>, Option<String>)>
where
    R: std::io::Read + Send + 'static,
{
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let error = reader.read_to_end(&mut bytes).err().map(|err| err.to_string());
        (bytes, error)
    })
}

fn join_deep_parser_pipe_drainer(
    handle: std::thread::JoinHandle<(Vec<u8>, Option<String>)>,
    stream: &str,
) -> (String, Option<String>) {
    match handle.join() {
        Ok((bytes, error)) => (String::from_utf8_lossy(&bytes).into_owned(), error),
        Err(_) => (
            String::new(),
            Some(format!("{stream} drainer thread panicked")),
        ),
    }
}

#[cfg(unix)]
fn configure_deep_parser_process_group(command: &mut std::process::Command) {
    use std::os::unix::process::CommandExt;

    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) == -1 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        });
    }
}

#[cfg(not(unix))]
fn configure_deep_parser_process_group(_command: &mut std::process::Command) {}

#[cfg(unix)]
fn request_deep_parser_termination(
    child: &std::process::Child,
    force: bool,
) -> std::io::Result<()> {
    let signal = if force { libc::SIGKILL } else { libc::SIGTERM };
    let process_group = -(child.id() as libc::pid_t);
    if unsafe { libc::kill(process_group, signal) } == -1 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(unix))]
fn request_deep_parser_termination(
    child: &mut std::process::Child,
    _force: bool,
) -> std::io::Result<()> {
    child.kill()
}

fn terminate_and_reap_deep_parser_child(
    child: &mut std::process::Child,
    reason: &str,
    supervisor_errors: &mut Vec<String>,
) -> std::process::ExitStatus {
    let cleanup_deadline = std::time::Instant::now() + DEEP_PARSER_CLEANUP_TIMEOUT;

    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status,
            Ok(None) => {}
            Err(err) => {
                supervisor_errors.push(format!(
                    "try_wait during {reason} cleanup failed: {err}"
                ))
            }
        }

        if std::time::Instant::now() >= cleanup_deadline {
            eprintln!(
                "deep-parser child cleanup deadline expired without direct-child reap: \
                 reason={reason}, pid={}, supervisor_errors={supervisor_errors:?}",
                child.id()
            );
            if let Err(err) = request_deep_parser_termination(child, true) {
                eprintln!("deep-parser final termination request failed: {err}");
            }
            std::process::abort();
        }

        if let Err(err) = request_deep_parser_termination(child, false) {
            supervisor_errors.push(format!("termination during {reason} cleanup failed: {err}"));

            // A failed termination request may race with an already-exited
            // child. Re-probe immediately before retrying within the deadline.
            match child.try_wait() {
                Ok(Some(status)) => return status,
                Ok(None) => {}
                Err(probe_err) => supervisor_errors.push(format!(
                    "try_wait after termination during {reason} cleanup failed: {probe_err}"
                )),
            }
        }

        match child.try_wait() {
            Ok(Some(status)) => return status,
            Ok(None) => {}
            Err(err) => supervisor_errors.push(format!(
                "try_wait after termination during {reason} cleanup failed: {err}"
            )),
        }

        let now = std::time::Instant::now();
        if now < cleanup_deadline {
            std::thread::sleep(
                DEEP_PARSER_POLL_INTERVAL.min(cleanup_deadline.saturating_duration_since(now)),
            );
        }
    }
}

fn run_bounded_parser_child(
    child_env: &str,
    exact_test: &str,
    child_summary: &str,
    markers: &[&str],
    label: &str,
) {
    let executable = std::env::current_exe().expect("failed to resolve current libtest executable");
    let mut command = std::process::Command::new(&executable);
    command
        .arg(exact_test)
        .arg("--exact")
        .arg("--nocapture")
        .env(child_env, "1")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    configure_deep_parser_process_group(&mut command);
    let mut child = command.spawn().unwrap_or_else(|err| {
        panic!(
            "failed to spawn {label} child {}: {err}",
            executable.display()
        )
    });

    let child_stdout = child.stdout.take().unwrap_or_else(|| {
        let mut supervisor_errors = Vec::new();
        let _ = terminate_and_reap_deep_parser_child(
            &mut child,
            "missing stdout pipe",
            &mut supervisor_errors,
        );
        panic!("{label} child stdout pipe was not available");
    });
    let child_stderr = child.stderr.take().unwrap_or_else(|| {
        let mut supervisor_errors = Vec::new();
        let _ = terminate_and_reap_deep_parser_child(
            &mut child,
            "missing stderr pipe",
            &mut supervisor_errors,
        );
        panic!("{label} child stderr pipe was not available");
    });
    let stdout_drainer = spawn_deep_parser_pipe_drainer(child_stdout);
    let stderr_drainer = spawn_deep_parser_pipe_drainer(child_stderr);

    let deadline = std::time::Instant::now() + DEEP_PARSER_CHILD_TIMEOUT;
    let mut timed_out = false;
    let mut supervisor_errors = Vec::new();
    let mut direct_child_reaped = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                direct_child_reaped = true;
                break Some(status);
            }
            Ok(None) => {
                let now = std::time::Instant::now();
                if now >= deadline {
                    timed_out = true;
                    let status = terminate_and_reap_deep_parser_child(
                        &mut child,
                        "timeout",
                        &mut supervisor_errors,
                    );
                    direct_child_reaped = true;
                    break Some(status);
                }
                std::thread::sleep(
                    DEEP_PARSER_POLL_INTERVAL.min(deadline.saturating_duration_since(now)),
                );
            }
            Err(err) => {
                supervisor_errors.push(format!("try_wait failed: {err}"));
                let status = terminate_and_reap_deep_parser_child(
                    &mut child,
                    "try_wait failure",
                    &mut supervisor_errors,
                );
                direct_child_reaped = true;
                break Some(status);
            }
        }
    };

    let (stdout, stdout_error) = join_deep_parser_pipe_drainer(stdout_drainer, "stdout");
    let (stderr, stderr_error) = join_deep_parser_pipe_drainer(stderr_drainer, "stderr");
    let stdout_drainer_joined = true;
    let stderr_drainer_joined = true;
    if let Some(err) = stdout_error {
        supervisor_errors.push(err);
    }
    if let Some(err) = stderr_error {
        supervisor_errors.push(err);
    }

    let diagnostics = format!(
        "{label} child status={status:?}, timed_out={timed_out}, direct_child_reaped={direct_child_reaped}, \
         stdout_drainer_joined={stdout_drainer_joined}, stderr_drainer_joined={stderr_drainer_joined}, \
         supervisor_errors={supervisor_errors:?}\n\
         --- child stdout ---\n{stdout}\n\
         --- child stderr ---\n{stderr}"
    );
    assert!(!timed_out, "{label} child exceeded 30 seconds\n{diagnostics}");
    assert!(
        supervisor_errors.is_empty(),
        "{label} child supervision failed\n{diagnostics}"
    );
    assert!(
        direct_child_reaped && stdout_drainer_joined && stderr_drainer_joined,
        "{label} child containment evidence incomplete\n{diagnostics}"
    );
    assert!(
        status.as_ref().is_some_and(|status| status.success()),
        "{label} child exited unsuccessfully\n{diagnostics}"
    );

    let combined_output = format!("{stdout}\n{stderr}");
    let summary_count = combined_output.matches(child_summary).count();
    assert_eq!(
        summary_count, 1,
        "{label} child must emit exactly one successful one-test summary, got {summary_count}\n{diagnostics}"
    );
    for marker in markers {
        let count = combined_output.matches(marker).count();
        assert_eq!(
            count, 1,
            "{label} child marker {marker:?} must occur exactly once, got {count}\n{diagnostics}"
        );
    }

    print!("{stdout}");
    eprint!("{stderr}");
}

const UNMATCHED_DELIMITER_CHILD_ENV: &str = "MAMBA_UNMATCHED_DELIMITER_CHILD_V1";
const UNMATCHED_DELIMITER_EXACT_TEST: &str =
    "conformance::tests_subdirs::stress::test_parser_resilience::test_parser_unmatched_delimiters";
const UNMATCHED_DELIMITER_CHILD_SUMMARY: &str =
    "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured;";
const UNMATCHED_OPEN_DELIMITERS: [&str; 3] = ["(", "[", "{"];
const UNMATCHED_CLOSE_DELIMITERS: [&str; 3] = [")", "]", "}"];
const UNMATCHED_DELIMITERS: [&str; 6] = ["(", ")", "[", "]", "{", "}"];
const UNMATCHED_DEPTHS: [usize; 5] = [1, 10, 50, 100, 300];
const UNMATCHED_INTERLEAVED_SEED: u64 = 42;
const UNMATCHED_INTERLEAVED_ROUNDS: usize = 50;
const UNMATCHED_TOKENS_PER_ROUND: usize = 100;
const UNMATCHED_FILE_ID: FileId = FileId(0);
const UNMATCHED_CONTRACT_DIGEST: [u8; 32] = [
    0x58, 0xfd, 0x4d, 0x9e, 0x29, 0x54, 0x74, 0x33,
    0x1b, 0xdd, 0x46, 0x01, 0xaa, 0xab, 0xc5, 0x4e,
    0x6d, 0xc2, 0xac, 0xc2, 0xaf, 0x5c, 0xf5, 0xae,
    0x42, 0xba, 0xe2, 0x9d, 0xc3, 0xb3, 0xfb, 0x30,
];
const UNMATCHED_DELIMITER_MARKERS: [&str; 8] = [
    "MAMBA_UNMATCHED_DELIMITER_GROUP_OK:v1:open:(:5",
    "MAMBA_UNMATCHED_DELIMITER_GROUP_OK:v1:open:[:5",
    "MAMBA_UNMATCHED_DELIMITER_GROUP_OK:v1:open:{:5",
    "MAMBA_UNMATCHED_DELIMITER_GROUP_OK:v1:close:):5",
    "MAMBA_UNMATCHED_DELIMITER_GROUP_OK:v1:close:]:5",
    "MAMBA_UNMATCHED_DELIMITER_GROUP_OK:v1:close:}:5",
    "MAMBA_UNMATCHED_DELIMITER_GROUP_OK:v1:interleaved:50",
    "MAMBA_UNMATCHED_DELIMITER_CHILD_OK:v1:80",
];

fn assert_unmatched_delimiter_contract() {
    assert_eq!(UNMATCHED_OPEN_DELIMITERS, ["(", "[", "{"]);
    assert_eq!(UNMATCHED_CLOSE_DELIMITERS, [")", "]", "}"]);
    assert_eq!(UNMATCHED_DELIMITERS, ["(", ")", "[", "]", "{", "}"]);
    assert_eq!(UNMATCHED_DEPTHS, [1, 10, 50, 100, 300]);
    assert_eq!(UNMATCHED_INTERLEAVED_SEED, 42);
    assert_eq!(UNMATCHED_INTERLEAVED_ROUNDS, 50);
    assert_eq!(UNMATCHED_TOKENS_PER_ROUND, 100);
    assert_eq!(UNMATCHED_FILE_ID, FileId(0));

    let mut digest = Sha256::new();
    digest.update(b"mamba-unmatched-delimiter-contract:v1\0");
    for value in UNMATCHED_OPEN_DELIMITERS
        .iter()
        .chain(UNMATCHED_CLOSE_DELIMITERS.iter())
        .chain(UNMATCHED_DELIMITERS.iter())
    {
        digest.update((value.len() as u64).to_le_bytes());
        digest.update(value.as_bytes());
    }
    for value in UNMATCHED_DEPTHS {
        digest.update((value as u64).to_le_bytes());
    }
    digest.update(UNMATCHED_INTERLEAVED_SEED.to_le_bytes());
    digest.update((UNMATCHED_INTERLEAVED_ROUNDS as u64).to_le_bytes());
    digest.update((UNMATCHED_TOKENS_PER_ROUND as u64).to_le_bytes());
    digest.update((UNMATCHED_FILE_ID.0 as u64).to_le_bytes());
    assert_eq!(digest.finalize().as_slice(), UNMATCHED_CONTRACT_DIGEST);
}

/// Test unmatched delimiters without allowing parser recursion to exhaust libtest's stack.
#[test]
fn test_parser_unmatched_delimiters() {
    if std::env::var_os(UNMATCHED_DELIMITER_CHILD_ENV).is_some() {
        assert_unmatched_delimiter_contract();
        let worker = std::thread::Builder::new()
            .name("mamba-unmatched-delimiter-64m".to_string())
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                let delimiters = UNMATCHED_DELIMITERS;
                let open_delimiters = UNMATCHED_OPEN_DELIMITERS;
                let close_delimiters = UNMATCHED_CLOSE_DELIMITERS;
                let depths = UNMATCHED_DEPTHS;
                let mut calls = 0;

                for (marker_index, &open) in open_delimiters.iter().enumerate() {
                    let mut group_calls = 0;
                    for depth in depths {
                        let src = open.repeat(depth);
                        let result = std::panic::catch_unwind(|| parser::parse(&src, UNMATCHED_FILE_ID));
                        assert!(
                            result.is_ok(),
                            "parser panicked on unmatched open delimiter: {src}"
                        );
                        group_calls += 1;
                    }
                    assert_eq!(group_calls, 5);
                    calls += group_calls;
                    println!("{}", UNMATCHED_DELIMITER_MARKERS[marker_index]);
                }

                for (marker_index, &close) in close_delimiters.iter().enumerate() {
                    let mut group_calls = 0;
                    for depth in depths {
                        let src = close.repeat(depth);
                        let result = std::panic::catch_unwind(|| parser::parse(&src, UNMATCHED_FILE_ID));
                        assert!(
                            result.is_ok(),
                            "parser panicked on unmatched close delimiter: {src}"
                        );
                        group_calls += 1;
                    }
                    assert_eq!(group_calls, 5);
                    calls += group_calls;
                    println!("{}", UNMATCHED_DELIMITER_MARKERS[marker_index + 3]);
                }

                let mut rng = FuzzRng::new(UNMATCHED_INTERLEAVED_SEED);
                let mut interleaved_calls = 0;
                for _ in 0..UNMATCHED_INTERLEAVED_ROUNDS {
                    let mut src = String::new();
                    for _ in 0..UNMATCHED_TOKENS_PER_ROUND {
                        src.push_str(delimiters[rng.next_usize(delimiters.len())]);
                    }
                    let result = std::panic::catch_unwind(|| parser::parse(&src, UNMATCHED_FILE_ID));
                    assert!(
                        result.is_ok(),
                        "parser panicked on random interleaved delimiters"
                    );
                    interleaved_calls += 1;
                }
                assert_eq!(interleaved_calls, UNMATCHED_INTERLEAVED_ROUNDS);
                calls += interleaved_calls;
                println!("{}", UNMATCHED_DELIMITER_MARKERS[6]);
                assert_eq!(calls, 80);
            })
            .expect("failed to spawn 64 MiB unmatched-delimiter worker");

        worker
            .join()
            .unwrap_or_else(|_| panic!("64 MiB unmatched-delimiter worker panicked"));
        println!("{}", UNMATCHED_DELIMITER_MARKERS[7]);
        return;
    }

    run_bounded_parser_child(
        UNMATCHED_DELIMITER_CHILD_ENV,
        UNMATCHED_DELIMITER_EXACT_TEST,
        UNMATCHED_DELIMITER_CHILD_SUMMARY,
        &UNMATCHED_DELIMITER_MARKERS,
        "unmatched-delimiter",
    );
}

/// Test deeply nested expressions and chained operators.
#[test]
fn test_parser_deeply_nested_expressions() {
    if std::env::var_os(DEEP_PARSER_CHILD_ENV).is_some() {
        let worker = std::thread::Builder::new()
            .name("mamba-deep-parser-64m".to_string())
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                // Deeply nested parentheses
                let depth = 200;
                let mut src = "(".repeat(depth);
                src.push_str("42");
                src.push_str(&")".repeat(depth));
                src.push('\n');
                assert!(
                    parser::parse(&src, FileId(0)).is_ok(),
                    "parser rejected 200-deep nested parens"
                );
                println!("{}", DEEP_PARSER_MARKERS[0]);

                // Chained binary operators: 1 + 1 + 1 + ... + 1
                let mut chained = "1".to_string();
                for _ in 0..200 {
                    chained.push_str(" + 1");
                }
                chained.push('\n');
                assert!(
                    parser::parse(&chained, FileId(0)).is_ok(),
                    "parser rejected 200-chained addition"
                );
                println!("{}", DEEP_PARSER_MARKERS[1]);

                // Chained attribute access: x.a.b.c.d...
                let mut attrs = "x".to_string();
                for i in 0..150 {
                    attrs.push_str(&format!(".attr_{i}"));
                }
                attrs.push('\n');
                assert!(
                    parser::parse(&attrs, FileId(0)).is_ok(),
                    "parser rejected 150-chained attribute access"
                );
                println!("{}", DEEP_PARSER_MARKERS[2]);
            })
            .expect("failed to spawn 64 MiB deep-parser worker");

        worker
            .join()
            .unwrap_or_else(|_| panic!("64 MiB deep-parser worker panicked"));
        println!("{}", DEEP_PARSER_MARKERS[3]);
        return;
    }

    run_bounded_parser_child(
        DEEP_PARSER_CHILD_ENV,
        DEEP_PARSER_EXACT_TEST,
        DEEP_PARSER_CHILD_SUMMARY,
        &DEEP_PARSER_MARKERS,
        "deep-parser",
    );
}

/// Test extreme indentation and whitespace mixing.
#[test]
fn test_parser_extreme_indentation() {
    // 500 spaces indentation
    let mut src = "def f():\n".to_string();
    src.push_str(&" ".repeat(500));
    src.push_str("pass\n");

    let res = std::panic::catch_unwind(|| parser::parse(&src, FileId(0)));
    assert!(res.is_ok(), "parser panicked on 500-space indent");

    // Mixed spaces and tabs
    let mut src_mixed = "def f():\n".to_string();
    src_mixed.push_str(" \t \t \t \t \t");
    src_mixed.push_str("x = 1\n");
    let res_mixed = std::panic::catch_unwind(|| parser::parse(&src_mixed, FileId(0)));
    assert!(res_mixed.is_ok(), "parser panicked on mixed spaces and tabs");
}

/// Test Unicode identifier fuzzing, zero-width characters, and RTL overrides.
#[test]
fn test_parser_unicode_fuzzing() {
    let unicode_inputs = [
        "áóí = 10\n",
        "αβγ = 20\n",
        "变量 = 30\n",
        "変数_123 = 40\n",
        "f_🐍 = 50\n",
        "x\u{200B}y = 60\n", // Zero-width space
        "\u{202E}reversed\u{202C} = 70\n", // RTL override
        "s = 'Emoji: 🚀🔥🎉'\n",
    ];

    for src in unicode_inputs {
        let res = std::panic::catch_unwind(|| parser::parse(src, FileId(0)));
        assert!(res.is_ok(), "parser panicked on Unicode input: {:?}", src);
    }
}

/// Test 100-target chained assignment: a = b = c = ... = 42
#[test]
fn test_parser_chained_assignments_stress() {
    use crate::parser::ast::{Expr, Stmt};

    let mut targets = Vec::new();
    for i in 0..100 {
        targets.push(format!("var_{i}"));
    }
    let src = format!("{} = 42\n", targets.join(" = "));
    let res = std::panic::catch_unwind(|| parser::parse(&src, FileId(0)));
    assert!(res.is_ok(), "parser panicked on 100-target assignment");
    let module = res
        .unwrap()
        .expect("100-target chained assignment should parse successfully");
    assert_eq!(
        module.stmts.len(),
        101,
        "expected 101 desugared Stmt::Assign"
    );

    // Stmt 0: __chained_<offset>__ = 42 (temp owns original RHS once)
    let tmp_name = match &module.stmts[0].node {
        Stmt::Assign { target, value } => {
            let name = match &target.node {
                Expr::Ident(n) => n.clone(),
                _ => panic!("expected Ident target for temp assign"),
            };
            assert!(
                name.starts_with("__chained_"),
                "expected temp name starting with __chained_"
            );
            assert!(
                matches!(&value.node, Expr::IntLit(42)),
                "temp should own original RHS IntLit(42)"
            );
            name
        }
        other => panic!("expected Stmt::Assign for temp at stmt 0, got {other:?}"),
    };

    // Stmts 1..=100: var_0..var_99 = __chained__ in source order
    for i in 0..100 {
        let expected_var = format!("var_{i}");
        match &module.stmts[i + 1].node {
            Stmt::Assign { target, value } => {
                assert!(
                    matches!(&target.node, Expr::Ident(n) if n == &expected_var),
                    "expected target {expected_var} at stmt {}, got {:?}",
                    i + 1,
                    target.node
                );
                assert!(
                    matches!(&value.node, Expr::Ident(n) if n == &tmp_name),
                    "target {expected_var} should reference temp {tmp_name}"
                );
            }
            other => panic!(
                "expected Stmt::Assign for target '{expected_var}' at stmt {}, got {other:?}",
                i + 1
            ),
        }
    }
}
