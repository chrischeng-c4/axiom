---
id: projects-meter-src-capture-fuzz-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: legacy-carried-internals
    role: primary
    gap: seeded-fuzz-and-injection-finding-generation
    claim: seeded-fuzz-and-injection-finding-generation
    coverage: full
    rationale: "Source template implements meter security, fuzzing, injection, or audit surfaces."
---

# Standardized projects/meter/src/capture/fuzz.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/capture/fuzz.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `DEFAULT_SEED` | projects/meter/src/capture/fuzz.rs | constant | pub | 39 |  |
| `FuzzCaptureOutcome` | projects/meter/src/capture/fuzz.rs | struct | pub | 65 |  |
| `FuzzTargetError` | projects/meter/src/capture/fuzz.rs | struct | pub | 79 |  |
| `demo_crash_target` | projects/meter/src/capture/fuzz.rs | function | pub | 100 | demo_crash_target(input: &str) -> Result<(), String> |
| `demo_vulnerable_sanitizer` | projects/meter/src/capture/fuzz.rs | function | pub | 121 | demo_vulnerable_sanitizer(input: &str) -> Result<String, String> |
| `fuzz_http` | projects/meter/src/capture/fuzz.rs | function | pub | 230 | fuzz_http(     url: &str,     method: &str,     seed: u64, ) -> Result<FuzzCaptureOutcome, FuzzTargetError> |
| `run_demo_crash` | projects/meter/src/capture/fuzz.rs | function | pub | 148 | run_demo_crash(seed: u64) -> FuzzResult |
| `run_demo_sql` | projects/meter/src/capture/fuzz.rs | function | pub | 158 | run_demo_sql() -> Vec<InjectionHit> |
| `run_demo_target` | projects/meter/src/capture/fuzz.rs | function | pub | 192 | run_demo_target(target: &str, seed: u64) -> Result<FuzzCaptureOutcome, FuzzTargetError> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/meter/src/capture/fuzz.rs -->
````rust
//! `meter fuzz` capture — drive the security engines (擷取) with a `--seed`.
//!
//! The CLI has NO caller-supplied sanitizer/parse closure: an agent invoking
//! `meter fuzz` on the command line cannot pass a Rust `Fn`. So this module ships a
//! small set of NAMED, clearly-labeled BUILT-IN DEMO TARGETS so the verb is
//! invokable end-to-end AND the byte-reproducibility (determinism) gate can run
//! with no server:
//!
//! - `demo-crash` — runs the mutation [`Fuzzer`](crate::security::Fuzzer)
//!   against a deliberately error-prone demo parse function, seeded from
//!   `--seed`. The mutation sequence (and therefore the crashing inputs and
//!   their ids) is byte-reproducible across separate process runs.
//! - `demo-sql` — runs the [`SqlInjectionTester`](crate::security::SqlInjectionTester)
//!   against a deliberately-vulnerable demo sanitizer over the
//!   [`PayloadDatabase`](crate::security::PayloadDatabase) SQL-injection corpus.
//!   Payloads that slip through (allowed/sanitized) become `Injection` findings.
//!
//! The `demo-` prefix is deliberate: these are SELF-CONTAINED demonstrations of
//! the fuzzing/injection engines, NOT a scan of any real code in the workspace.
//!
//! There is also a REAL path: [`fuzz_http`] points the async HTTP fuzzer at a
//! live `--url` (via [`fuzz_http_endpoint`](crate::security::AsyncFuzzer::fuzz_http_endpoint)),
//! which is where the engine fuzzes an actual endpoint.
//!
//! Determinism note: the demo crash function and the SQL demo sanitizer are
//! defined HERE (not in the frozen engine files), so the engine modules stay
//! untouched and consumable by pgkit.

use crate::report::finding::Finding;
use crate::report::producer::{fuzz_crash_finding, injection_finding, InjectionHit};
use crate::security::{
    FuzzConfig, FuzzResult, Fuzzer, InjectionResult, PayloadDatabase, SqlInjectionTester,
};

/// Default mutation seed when `--seed` is omitted. Fixed so the DEFAULT (no
/// `--seed`) invocation is itself byte-reproducible across runs.
pub const DEFAULT_SEED: u64 = 0;

/// Number of mutation iterations the demo crash fuzzer performs. Small + fixed
/// so a run is fast and fully deterministic given the seed.
const DEMO_CRASH_ITERATIONS: u32 = 256;

/// Per-iteration timeout (ms) for the demo crash run. Set far larger than any
/// real run could take so the engine's wall-clock early-break NEVER triggers —
/// making the number of iterations (and the crash set) a pure function of the
/// seed, not of timing. Must stay small enough that the engine's internal
/// `timeout_ms * (iteration + 1)` cannot overflow `u64` for `DEMO_CRASH_ITERATIONS`.
const DEMO_CRASH_TIMEOUT_MS: u64 = 1_000_000;

/// The `MutationStrategy` label recorded in `fuzz_crash` evidence. The demo
/// `Fuzzer` drives the full strategy set internally (the engine selects per
/// iteration); we report the corpus-level mode rather than a per-crash strategy
/// because the engine's `FuzzCrash` does not surface which strategy produced it.
const DEMO_CRASH_STRATEGY: &str = "mutation:all";

/// Result of running a built-in demo / real fuzz target.
///
/// Carries the raw engine output PLUS the context the producer needs for
/// byte-reproducible finding ids (`target` slug + `seed`). The dispatch layer
/// folds [`Self::into_findings`] into the report builder.
#[derive(Debug)]
pub struct FuzzCaptureOutcome {
    /// The named target that was driven (`"demo-crash"`, `"demo-sql"`, or
    /// `"http:<url>"`).
    pub target: String,
    /// The seed the mutation engine was seeded with (recorded in evidence).
    pub seed: u64,
    /// The findings produced (already context-stamped with `target`/`seed`).
    pub findings: Vec<Finding>,
}

/// A target the CLI could not invoke at all (unknown name, unreachable URL).
/// The dispatch layer maps this to a `ToolError(5)` report — NEVER fake-clean.
#[derive(Debug)]
pub struct FuzzTargetError(pub String);

impl std::fmt::Display for FuzzTargetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for FuzzTargetError {}

/// A deliberately error-prone demo "parser". Returns `Err` (a simulated crash /
/// rejection) when the mutated input trips one of a few honest failure modes:
/// an embedded NUL, a non-ASCII byte, or a `'`/`"`/`;` SQL-ish metacharacter.
/// This is NOT real code under test — it exists purely so the mutation `Fuzzer`
/// has something to find, making the verb invokable and the determinism gate
/// runnable with no external server.
///
/// Public so the unit tests can exercise the exact failure modes the demo run
/// relies on.
pub fn demo_crash_target(input: &str) -> Result<(), String> {
    if input.contains('\0') {
        return Err("demo parser: embedded NUL byte rejected".to_string());
    }
    if input.bytes().any(|b| !b.is_ascii()) {
        return Err("demo parser: non-ASCII byte rejected".to_string());
    }
    if input.contains(['\'', '"', ';']) {
        return Err("demo parser: SQL metacharacter rejected".to_string());
    }
    Ok(())
}

/// A deliberately-VULNERABLE demo SQL sanitizer: it only strips single quotes
/// and otherwise accepts the input verbatim, so most injection payloads slip
/// through (allowed or merely sanitized rather than blocked). This is the demo
/// counterpart to a real validator and lets `demo-sql` produce `Injection`
/// findings deterministically.
///
/// Public so the unit tests can assert the sanitizer is genuinely weak.
pub fn demo_vulnerable_sanitizer(input: &str) -> Result<String, String> {
    // Only blocks a literal stacked-DROP; everything else is accepted (quotes
    // merely stripped), so the tester reports Allowed/Sanitized = a leak.
    if input.contains("DROP TABLE") {
        return Err("demo sanitizer: blocked stacked DROP".to_string());
    }
    Ok(input.replace('\'', ""))
}

/// Build the seeded [`FuzzConfig`] used by `demo-crash`. Seeding from `seed`
/// (default [`DEFAULT_SEED`]) makes the mutation sequence — and therefore the
/// crashing inputs and their blake3-derived ids — byte-reproducible across
/// separate process runs.
fn demo_crash_config(seed: u64) -> FuzzConfig {
    FuzzConfig::new()
        .with_iterations(DEMO_CRASH_ITERATIONS)
        .with_seed(seed)
        // A huge-but-non-overflowing per-iteration timeout so the engine's
        // wall-clock early-break never fires: the run length is a pure function
        // of the seed (no time-dependent early break => byte-stable).
        .with_timeout_ms(DEMO_CRASH_TIMEOUT_MS)
}

/// Run the `demo-crash` target: seed the mutation [`Fuzzer`] and fuzz the
/// deliberately error-prone [`demo_crash_target`]. Returns the raw engine
/// [`FuzzResult`] (deterministic given `seed`).
pub fn run_demo_crash(seed: u64) -> FuzzResult {
    let fuzzer = Fuzzer::new(demo_crash_config(seed));
    fuzzer.fuzz(demo_crash_target)
}

/// Run the `demo-sql` target: drive the [`SqlInjectionTester`] against the
/// [`demo_vulnerable_sanitizer`] over the SQL-injection payload corpus, and
/// collect the payloads that were NOT blocked as [`InjectionHit`]s. The payload
/// corpus is a fixed, ordered table, so the hit set is deterministic.
pub fn run_demo_sql() -> Vec<InjectionHit> {
    let tester = SqlInjectionTester::new();
    let db = PayloadDatabase::new();
    let payloads = db.sql_injection().to_vec();
    let results = tester.test(demo_vulnerable_sanitizer, &payloads);

    results
        .into_iter()
        .filter_map(|test| {
            // A payload that was allowed-as-is or merely sanitized but accepted
            // got through the (weak) sanitizer => an injection leak. Blocked /
            // engine-error payloads are the safe outcome and produce no finding.
            match test.actual {
                Some(InjectionResult::Allowed) => Some(InjectionHit {
                    payload: test.payload,
                    category: "sql_injection".to_string(),
                    reflected: true,
                }),
                Some(InjectionResult::Sanitized) => Some(InjectionHit {
                    payload: test.payload,
                    category: "sql_injection".to_string(),
                    // Sanitized = modified-then-accepted; not reflected verbatim.
                    reflected: false,
                }),
                Some(InjectionResult::Blocked) | Some(InjectionResult::Error(_)) | None => None,
            }
        })
        .collect()
}

/// Drive a built-in DEMO target by name. `Ok` with the (possibly empty) finding
/// set on a known target; `Err(FuzzTargetError)` on an unknown target name so the
/// dispatch layer can map it to `ToolError(5)` (un-invocable).
pub fn run_demo_target(target: &str, seed: u64) -> Result<FuzzCaptureOutcome, FuzzTargetError> {
    match target {
        "demo-crash" => {
            let result = run_demo_crash(seed);
            let findings = result
                .crashes
                .iter()
                .map(|c| fuzz_crash_finding(target, c, DEMO_CRASH_STRATEGY, seed))
                .collect();
            Ok(FuzzCaptureOutcome {
                target: target.to_string(),
                seed,
                findings,
            })
        }
        "demo-sql" => {
            let hits = run_demo_sql();
            let findings = hits.iter().map(|h| injection_finding(target, h)).collect();
            Ok(FuzzCaptureOutcome {
                target: target.to_string(),
                seed,
                findings,
            })
        }
        other => Err(FuzzTargetError(format!(
            "unknown fuzz target `{other}`; use `demo-crash`, `demo-sql`, or `--url <endpoint>`"
        ))),
    }
}

/// Fuzz a REAL HTTP endpoint at `url` with `method`, seeded from `seed`. This is
/// the non-demo path: it drives [`AsyncFuzzer::fuzz_http_endpoint`] against a
/// live endpoint and maps the resulting crashes to `fuzz_crash` findings.
///
/// `Err(FuzzTargetError)` when the URL is malformed/unreachable to the point the
/// fuzzer cannot run at all; per-request HTTP errors are crashes (findings), not
/// invocation failures. Requires a Tokio runtime to be available to the caller.
pub fn fuzz_http(
    url: &str,
    method: &str,
    seed: u64,
) -> Result<FuzzCaptureOutcome, FuzzTargetError> {
    use crate::security::{AsyncFuzzConfig, AsyncFuzzer};

    // Reject an obviously-malformed URL up front so a bad `--url` is an
    // invocation error (ToolError 5), not a fake-clean run.
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err(FuzzTargetError(format!(
            "`--url {url}` is not a valid http(s) endpoint"
        )));
    }

    let config = AsyncFuzzConfig::new()
        .with_iterations(64)
        .with_seed(seed)
        .with_corpus(vec![
            String::new(),
            "test".to_string(),
            "1".to_string(),
            "' OR 1=1--".to_string(),
        ]);

    // Drive the async HTTP fuzzer on a dedicated current-thread runtime so this
    // function is callable from a sync dispatch path.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| FuzzTargetError(format!("could not start async runtime: {e}")))?;

    let result = runtime.block_on(async {
        let mut fuzzer = AsyncFuzzer::new(config);
        fuzzer.fuzz_http_endpoint(url, method).await
    });

    let target = format!("http:{url}");
    let strategy = format!("http:{}", method.to_uppercase());
    let findings = result
        .crashes
        .iter()
        .map(|c| fuzz_crash_finding(&target, c, &strategy, seed))
        .collect();
    Ok(FuzzCaptureOutcome {
        target,
        seed,
        findings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_crash_target_rejects_nul_and_metacharacters() {
        // The demo parser must have real, deterministic failure modes so the
        // fuzzer can find crashes.
        assert!(demo_crash_target("hello").is_ok());
        assert!(demo_crash_target("with\0nul").is_err());
        assert!(demo_crash_target("quote'here").is_err());
        assert!(demo_crash_target("semi;colon").is_err());
    }

    #[test]
    fn demo_sanitizer_is_genuinely_vulnerable() {
        // A classic injection payload slips through (only quotes stripped),
        // proving the demo sanitizer is weak on purpose.
        let out = demo_vulnerable_sanitizer("' OR 1=1--").unwrap();
        assert!(!out.contains('\''));
        assert!(out.contains("OR 1=1")); // injection body survives => a leak
    }

    #[test]
    fn demo_crash_finds_crashes_for_a_seed() {
        // With a fixed seed the run must surface at least one crash so the verb
        // exercises the findings path (the demo parser is easy to trip).
        let result = run_demo_crash(42);
        assert_eq!(result.iterations, DEMO_CRASH_ITERATIONS);
        assert!(
            !result.crashes.is_empty(),
            "demo-crash with seed 42 should surface crashes"
        );
    }

    #[test]
    fn demo_crash_is_byte_reproducible_across_runs() {
        // THE HARD CONTRACT: two independent seeded runs produce identical
        // crashing inputs (and therefore identical finding ids downstream).
        let a = run_demo_crash(42);
        let b = run_demo_crash(42);
        let ai: Vec<&String> = a.crashes.iter().map(|c| &c.input).collect();
        let bi: Vec<&String> = b.crashes.iter().map(|c| &c.input).collect();
        assert_eq!(ai, bi, "same seed must yield the same crashing inputs");
    }

    #[test]
    fn demo_crash_outcome_findings_are_fuzz_crash_kind() {
        let outcome = run_demo_target("demo-crash", 42).unwrap();
        assert_eq!(outcome.target, "demo-crash");
        assert_eq!(outcome.seed, 42);
        assert!(!outcome.findings.is_empty());
        assert!(outcome
            .findings
            .iter()
            .all(|f| f.kind == crate::report::Kind::FuzzCrash));
    }

    #[test]
    fn demo_crash_finding_ids_are_byte_identical_across_runs() {
        // The full id set must be byte-identical across two independent
        // `run_demo_target` calls for the same seed — the determinism gate.
        let a = run_demo_target("demo-crash", 42).unwrap();
        let b = run_demo_target("demo-crash", 42).unwrap();
        let ai: Vec<&String> = a.findings.iter().map(|f| &f.id).collect();
        let bi: Vec<&String> = b.findings.iter().map(|f| &f.id).collect();
        assert_eq!(ai, bi);
        // ids embed the target + a blake3(input)[..8] suffix.
        assert!(a
            .findings
            .iter()
            .all(|f| f.id.starts_with("fuzz_crash:demo-crash:")));
    }

    #[test]
    fn demo_crash_seed_changes_the_id_set() {
        // Different seeds must (in general) drive a different mutation sequence,
        // so the id sets differ — proving the seed actually threads through.
        let a = run_demo_target("demo-crash", 1).unwrap();
        let b = run_demo_target("demo-crash", 2).unwrap();
        let ai: Vec<&String> = a.findings.iter().map(|f| &f.id).collect();
        let bi: Vec<&String> = b.findings.iter().map(|f| &f.id).collect();
        assert_ne!(ai, bi);
    }

    #[test]
    fn demo_sql_finds_injection_leaks() {
        let outcome = run_demo_target("demo-sql", 0).unwrap();
        assert_eq!(outcome.target, "demo-sql");
        assert!(
            !outcome.findings.is_empty(),
            "the weak demo sanitizer should leak injection payloads"
        );
        assert!(outcome
            .findings
            .iter()
            .all(|f| f.kind == crate::report::Kind::Injection));
        assert!(outcome
            .findings
            .iter()
            .all(|f| f.id.starts_with("injection:demo-sql:sql_injection:")));
    }

    #[test]
    fn demo_sql_is_deterministic() {
        let a = run_demo_target("demo-sql", 0).unwrap();
        let b = run_demo_target("demo-sql", 0).unwrap();
        let ai: Vec<&String> = a.findings.iter().map(|f| &f.id).collect();
        let bi: Vec<&String> = b.findings.iter().map(|f| &f.id).collect();
        assert_eq!(ai, bi);
    }

    #[test]
    fn unknown_target_is_an_invocation_error() {
        let err = run_demo_target("nope", 0).unwrap_err();
        assert!(err.0.contains("unknown fuzz target"));
    }

    #[test]
    fn malformed_url_is_an_invocation_error() {
        // A non-http(s) URL is rejected before any request is attempted, so a
        // bad `--url` becomes ToolError(5), never fake-clean.
        let err = fuzz_http("not-a-url", "GET", 0).unwrap_err();
        assert!(err.0.contains("not a valid http"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/capture/fuzz.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/capture/fuzz.rs` captured during meter full-codegen standardization.
```
