use datatest_stable::harness;
/// File-based test harness for Mamba compiler.
///
/// Auto-discovers `.py` files under `tests/cpython/` and runs them through
/// the appropriate pipeline stage based on `# RUN:` / `# EXPECT:` /
/// `# EXPECT-ERROR:` comment directives.
use mamba::codegen::cranelift::jit::{CraneliftJitBackend, JIT_LOCK};
use mamba::codegen::{CodegenBackend, CodegenOutput};
use mamba::lower::{lower_hir_to_mir_with_symbols, lower_module};
use mamba::parser;
use mamba::source::span::FileId;
use mamba::types::TypeChecker;

// ── Directive parsing ────────────────────────────────────────────

struct Directives {
    run: String,
    expect: Option<i64>,
    expect_error: Option<String>,
    xfail: bool,
    xfail_reason: Option<String>,
}

fn parse_directives(src: &str) -> Directives {
    let mut run = String::new();
    let mut expect = None;
    let mut expect_error = None;
    let mut xfail = false;
    let mut xfail_reason = None;

    for line in src.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("# RUN:") {
            run = rest.trim().to_string();
        } else if let Some(rest) = trimmed.strip_prefix("# EXPECT:") {
            expect = Some(
                rest.trim()
                    .parse::<i64>()
                    .expect("EXPECT value must be i64"),
            );
        } else if let Some(rest) = trimmed.strip_prefix("# EXPECT-ERROR:") {
            expect_error = Some(rest.trim().to_string());
        } else if trimmed == "# XFAIL" || trimmed == "# XFAIL:" {
            xfail = true;
        } else if let Some(rest) = trimmed.strip_prefix("# REASON:") {
            xfail_reason = Some(rest.trim().to_string());
        }
    }

    assert!(!run.is_empty(), "fixture missing # RUN: directive");
    Directives {
        run,
        expect,
        expect_error,
        xfail,
        xfail_reason,
    }
}

// ── Pipeline runners ─────────────────────────────────────────────

fn run_jit(src: &str, directives: &Directives, path: &std::path::Path) {
    // XFAIL fixtures are skipped here — run_jit's panic-based assertions don't
    // play well with catch_unwind across the JIT boundary, so we simply note
    // the expected failure and return.
    if directives.xfail {
        let reason = directives.xfail_reason.as_deref().unwrap_or("inline XFAIL");
        eprintln!("  [xfail-skip] {}: {reason}", path.display());
        return;
    }

    // Recover from a poisoned lock: a previous JIT fixture may have panicked
    // (e.g. an `assert_eq!` failure mid-test) while holding JIT_LOCK. We don't
    // share JIT state across fixtures, so the inner data is still safe to use.
    let _jit_guard = JIT_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    let module = parser::parse(src, FileId(0))
        .unwrap_or_else(|e| panic!("{}: parse failed: {e}", path.display()));
    let mut checker = TypeChecker::new();
    let _ = checker.check_module(&module);
    let hir = lower_module(&module, &checker)
        .unwrap_or_else(|e| panic!("{}: HIR lowering failed: {e:?}", path.display()));
    let mir = lower_hir_to_mir_with_symbols(&hir, &checker.tcx, &checker.symbols);

    let mut backend = CraneliftJitBackend::new()
        .unwrap_or_else(|e| panic!("{}: JIT init failed: {e}", path.display()));
    let output = backend
        .codegen(&mir, &checker.tcx)
        .unwrap_or_else(|e| panic!("{}: JIT codegen failed: {e}", path.display()));

    match output {
        CodegenOutput::Jit { entry } => {
            let f: fn() -> i64 = unsafe { std::mem::transmute(entry) };
            let result = f();
            if let Some(expected) = directives.expect {
                assert_eq!(
                    result,
                    expected,
                    "{}: expected {expected}, got {result}",
                    path.display()
                );
            }
        }
        _ => panic!("{}: expected JIT output", path.display()),
    }
}

fn run_parse(src: &str, directives: &Directives, path: &std::path::Path) {
    let result = parser::parse(src, FileId(0));

    // Handle xfail: expected failures pass silently, unexpected passes warn.
    if directives.xfail {
        let reason = directives.xfail_reason.as_deref().unwrap_or("inline XFAIL");
        match result {
            Err(_) => {
                eprintln!("  [xfail] {}: {reason}", path.display());
                return;
            }
            Ok(_) => {
                eprintln!(
                    "  [xpass] {} passed unexpectedly (xfail reason: {reason}). \
                     Consider removing # XFAIL.",
                    path.display()
                );
                return;
            }
        }
    }

    match result {
        Ok(_) => {
            if let Some(expected) = &directives.expect_error {
                panic!(
                    "{}: expected parse error containing '{}', but parsing succeeded",
                    path.display(),
                    expected,
                );
            }
        }
        Err(e) => {
            if let Some(expected) = &directives.expect_error {
                let msg = e.to_string();
                assert!(
                    msg.contains(expected.as_str()),
                    "{}: expected parse error containing '{}', got: {}",
                    path.display(),
                    expected,
                    msg,
                );
            } else {
                panic!("{}: parse failed: {e}", path.display());
            }
        }
    }
}

fn run_typecheck(src: &str, directives: &Directives, path: &std::path::Path) {
    let module = parser::parse(src, FileId(0))
        .unwrap_or_else(|e| panic!("{}: parse failed: {e}", path.display()));
    let mut checker = TypeChecker::new();
    let errors = checker.check_module(&module);
    let error_msgs: Vec<String> = errors.into_iter().map(|e| e.to_string()).collect();

    if let Some(expected_substr) = &directives.expect_error {
        assert!(
            error_msgs
                .iter()
                .any(|msg| msg.contains(expected_substr.as_str())),
            "{}: expected error containing '{}', got: {:?}",
            path.display(),
            expected_substr,
            error_msgs,
        );
    } else {
        assert!(
            error_msgs.is_empty(),
            "{}: expected no errors, got: {:?}",
            path.display(),
            error_msgs,
        );
    }
}

// ── Harness entry point ──────────────────────────────────────────

fn run_fixture(path: &std::path::Path) -> datatest_stable::Result<()> {
    // Skip conformance/ — handled by conformance_tests harness.
    // Skip bench/ — those are perf inputs for `mamba bench`, not pipeline
    // fixtures, and they intentionally lack `# RUN:` directives.
    // Skip cpython_lib_test/ — vendored CPython 3.12 Lib/test seeds owned by
    // cpython_lib_test_runner.rs (drift-baseline harness); they are byte-for-byte
    // upstream files and intentionally have no `# RUN:` directives.
    if path.components().any(|c| {
        let s = c.as_os_str();
        s == "conformance" || s == "bench" || s == "cpython_lib_test"
    }) {
        return Ok(());
    }

    let src = std::fs::read_to_string(path)?;

    // Dimension-first migration: record-based conformance fixtures (the
    // `[tool.mamba]` tree) live directly under tests/cpython and are owned
    // by the live-CPython-oracle runner. Only `# RUN:`-directive fixtures
    // (today: `_regression/core/grammar/`) belong to this pipeline harness;
    // everything else is skipped, mirroring the runner's inverse
    // `[pipeline-skip]` rule.
    if !src
        .lines()
        .any(|line| line.trim_start().starts_with("# RUN:"))
    {
        return Ok(());
    }

    let directives = parse_directives(&src);

    match directives.run.as_str() {
        "jit" => run_jit(&src, &directives, path),
        "parse" => run_parse(&src, &directives, path),
        "typecheck" => run_typecheck(&src, &directives, path),
        other => panic!("{}: unknown RUN mode '{other}'", path.display()),
    }

    Ok(())
}

harness!(run_fixture, "tests/cpython", r".*\.py$");
