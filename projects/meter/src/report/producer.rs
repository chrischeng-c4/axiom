// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-report-producer-rs.md#source
// CODEGEN-BEGIN
//! The ONE producer trait at the report boundary.
//!
//! Engine result structs stay UNTOUCHED; the mapping from engine output to
//! [`Finding`]s lives here. This wave ships the `TestResult -> TestFailure`
//! path used by `meter test` and the `AuditResult -> RustVuln/RustWarning` path
//! used by `meter audit`; the other engine impls (ProfileResult, BoundaryMetrics,
//! RegressionReport, FuzzResult, InjectionResult) land in later waves.

use super::finding::{finding_id, Finding, Invoke, Kind, Location, Severity};

/// Map an engine result into zero or more [`Finding`]s.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-producer-rs.md#source
pub trait IntoFindings {
    /// Produce the findings this engine result represents.
    fn into_findings(&self) -> Vec<Finding>;
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-producer-rs.md#source
impl IntoFindings for crate::runner::TestResult {
    /// A failed/errored delegated test becomes one informational `TestFailure`.
    /// Passed/skipped tests produce nothing.
    fn into_findings(&self) -> Vec<Finding> {
        use crate::runner::TestStatus;
        match self.status {
            TestStatus::Failed | TestStatus::Error => {
                let name = &self.meta.full_name;
                let tail = self
                    .error
                    .clone()
                    .or_else(|| self.stack_trace.clone())
                    .unwrap_or_default();
                vec![test_failure_finding(
                    name,
                    &tail,
                    self.meta.file_path.clone(),
                    self.meta.line_number,
                )]
            }
            TestStatus::Passed | TestStatus::Skipped => Vec::new(),
        }
    }
}

/// Build a `TestFailure` finding for a single failed test `name`.
///
/// Public so `capture::delegate` can construct findings directly from parsed
/// runner output without materializing a full `TestResult`.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-producer-rs.md#source
pub fn test_failure_finding(
    name: &str,
    stdout_tail: &str,
    file: Option<String>,
    line: Option<u32>,
) -> Finding {
    let location = if file.is_some() || line.is_some() {
        Some(Location {
            file,
            line,
            symbol: Some(name.to_string()),
        })
    } else {
        None
    };
    Finding {
        id: finding_id(Kind::TestFailure, name),
        severity: Severity::High,
        kind: Kind::TestFailure,
        title: format!("test failed: {name}"),
        detail: if stdout_tail.is_empty() {
            format!("Delegated test `{name}` did not pass.")
        } else {
            format!("Delegated test `{name}` did not pass:\n{stdout_tail}")
        },
        remediation: format!(
            "Re-run `{name}` in isolation, inspect the assertion, and fix the code or the test."
        ),
        invoke: Invoke::command(format!("cargo test {name} -- --exact --nocapture")),
        evidence: serde_json::json!({
            "name": name,
            "stdout_tail": stdout_tail,
        }),
        location,
    }
}

/// Build a single generic `TestFailure` finding for a delegated run that exited
/// non-zero but produced no parseable per-test failures.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-producer-rs.md#source
pub fn generic_test_failure(target: &str, exit_code: i32) -> Finding {
    Finding {
        id: finding_id(Kind::TestFailure, "delegated-run"),
        severity: Severity::High,
        kind: Kind::TestFailure,
        title: "delegated test run failed".to_string(),
        detail: format!(
            "The delegated test runner for `{target}` exited with code {exit_code} but no \
             individual test failures could be parsed from its output."
        ),
        remediation:
            "Re-run the test command directly and read the live stderr to locate the failure."
                .to_string(),
        invoke: Invoke::command(format!("cargo test {target}")),
        evidence: serde_json::json!({
            "name": "delegated-run",
            "exit_code": exit_code,
        }),
        location: None,
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-producer-rs.md#source
impl IntoFindings for crate::baseline::RegressionReport {
    /// Map each detected [`Regression`](crate::baseline::Regression) to a
    /// `Finding{kind: Regression}`. Improvements and unchanged benchmarks
    /// produce nothing — `meter bench` only surfaces slowdowns.
    ///
    /// Severity maps off the engine's [`RegressionSeverity`](crate::baseline::RegressionSeverity):
    /// `Severe` (> failure threshold, default 15%) => `High`, `Moderate`
    /// (> warning threshold, default 5%) => `Medium`, `Minor` => `Low`. Only
    /// `Severe`/`Moderate` (medium-or-worse) elevate the overall status to
    /// `Regression` (exit 2) in `finalize()`; a `Minor`-only report stays a
    /// plain `Findings` (exit 1).
    ///
    /// The `id` is deterministic (`"regression:{benchmark_name}"`), so repeated
    /// comparisons of the same baseline produce byte-identical finding ids.
    fn into_findings(&self) -> Vec<Finding> {
        self.regressions.iter().map(regression_finding).collect()
    }
}

/// Map an engine [`RegressionSeverity`](crate::baseline::RegressionSeverity) to a
/// report [`Severity`].
fn severity_from_regression(sev: crate::baseline::RegressionSeverity) -> Severity {
    use crate::baseline::RegressionSeverity;
    match sev {
        RegressionSeverity::Severe => Severity::High,
        RegressionSeverity::Moderate => Severity::Medium,
        RegressionSeverity::Minor => Severity::Low,
    }
}

/// Build a `Regression` finding from one detected
/// [`Regression`](crate::baseline::Regression).
fn regression_finding(r: &crate::baseline::Regression) -> Finding {
    let bench = &r.name;
    let severity = severity_from_regression(r.severity);
    let percentile = r.percentile_type.name();
    Finding {
        id: finding_id(Kind::Regression, bench),
        severity,
        kind: Kind::Regression,
        title: format!("performance regression: {bench}"),
        detail: format!(
            "`{bench}` is {:+.2}% slower on `{percentile}` ({:.4}ms baseline -> {:.4}ms current).",
            r.percent_change, r.baseline_value_ms, r.current_value_ms
        ),
        remediation: format!(
            "Investigate the slowdown in `{bench}`, then re-run `meter bench` to confirm the regression is resolved."
        ),
        invoke: Invoke::command(format!("meter bench --target {bench}")),
        evidence: serde_json::json!({
            "bench": bench,
            "baseline_ms": r.baseline_value_ms,
            "current_ms": r.current_value_ms,
            "percent_change": r.percent_change,
            "severity": severity.as_str(),
            "percentile": percentile,
            "ci_overlap": r.ci_overlap,
        }),
        location: None,
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-producer-rs.md#source
impl IntoFindings for crate::performance::profiler::PhaseBreakdown {
    /// Map a recorded phase breakdown into per-phase `BoundaryCost` findings —
    /// the `meter profile --phases` EMBED path (no child spawn, no sampler).
    ///
    /// Each phase becomes one `Finding{kind: BoundaryCost}` with evidence
    /// `{phase, self_ns, total_ns, pct, samples}`:
    /// - `self_ns` = that phase's recorded total time (the time attributable to
    ///   the phase itself).
    /// - `total_ns` = the breakdown's `total_duration_ns` (the whole-operation
    ///   wall, the inclusive denominator the phase is measured against).
    /// - `pct` = `self_ns / total_ns` (fraction of the operation in this phase).
    /// - `samples` = the phase's entry `count`.
    ///
    /// Findings are PRE-SORTED by `self_ns` desc (the builder re-sorts by
    /// severity/id, but emitting in cost order keeps the producer stable). All
    /// boundary-cost findings are informational (`Info`): they describe where
    /// time goes, they are not failures.
    fn into_findings(&self) -> Vec<Finding> {
        let total_ns = self.total_duration_ns;
        let mut entries: Vec<(&String, &crate::performance::profiler::PhaseTiming)> =
            self.phases.iter().collect();
        // Deterministic order: self_ns desc, ties by phase name asc.
        entries.sort_by(|a, b| b.1.total_ns.cmp(&a.1.total_ns).then_with(|| a.0.cmp(b.0)));
        entries
            .into_iter()
            .map(|(phase, timing)| {
                boundary_cost_finding(phase, timing.total_ns, total_ns, timing.count)
            })
            .collect()
    }
}

/// Build one `BoundaryCost` finding for a phase.
fn boundary_cost_finding(phase: &str, self_ns: u64, total_ns: u64, samples: u64) -> Finding {
    let pct = if total_ns > 0 {
        self_ns as f64 / total_ns as f64
    } else {
        0.0
    };
    Finding {
        id: finding_id(Kind::BoundaryCost, phase),
        severity: Severity::Info,
        kind: Kind::BoundaryCost,
        title: format!("boundary cost: {phase} ({:.1}%)", pct * 100.0),
        detail: format!(
            "Phase `{phase}` accounts for {:.3}ms of the {:.3}ms operation ({:.1}%, over {samples} entries).",
            self_ns as f64 / 1e6,
            total_ns as f64 / 1e6,
            pct * 100.0,
        ),
        remediation: format!(
            "If `{phase}` dominates, reduce its work or move it off the critical path; re-profile to confirm."
        ),
        invoke: Invoke::command("meter profile --phases <breakdown.json>"),
        evidence: serde_json::json!({
            "phase": phase,
            "self_ns": self_ns,
            "total_ns": total_ns,
            "pct": pct,
            "samples": samples,
        }),
        location: None,
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-producer-rs.md#source
impl IntoFindings for crate::rust_runner::AuditResult {
    /// Map each `cargo audit` advisory to a `RustVuln` finding and each warning
    /// to a `RustWarning` finding. Both id schemes are deterministic so repeated
    /// audits of the same lockfile produce byte-identical finding ids. The
    /// builder re-sorts (severity desc, id asc) on `finalize`, so ordering here
    /// is not load-bearing — but the producer is already stable.
    fn into_findings(&self) -> Vec<Finding> {
        let mut findings = Vec::new();
        for v in &self.vulnerabilities {
            findings.push(rust_vuln_finding(v));
        }
        for w in &self.warnings {
            findings.push(rust_warning_finding(w));
        }
        findings
    }
}

/// Build a `RustVuln` finding from one cargo-audit advisory.
fn rust_vuln_finding(v: &crate::rust_runner::Vulnerability) -> Finding {
    let package = &v.package.name;
    let version = &v.package.version;
    let severity = severity_from_cvss(v.cvss.as_deref());
    let detail = if v.description.is_empty() {
        format!(
            "Advisory {} affects `{package}` {version}: {}.",
            v.id, v.title
        )
    } else {
        format!(
            "Advisory {} affects `{package}` {version}: {}\n{}",
            v.id, v.title, v.description
        )
    };
    Finding {
        id: finding_id(Kind::RustVuln, &v.id),
        severity,
        kind: Kind::RustVuln,
        title: v.title.clone(),
        detail,
        remediation: format!("Upgrade `{package}` past the affected range."),
        invoke: Invoke::command(format!("cargo update -p {package}")),
        evidence: serde_json::json!({
            "advisory_id": v.id,
            "package": package,
            "version": version,
            "cvss": v.cvss,
            "title": v.title,
        }),
        location: None,
    }
}

/// Build a `RustWarning` finding from one cargo-audit warning (yanked,
/// unmaintained, unsound, etc.). Warnings carry no advisory severity, so they
/// map to `low` (informational `info` only when the package is unknown).
fn rust_warning_finding(w: &crate::rust_runner::AuditWarning) -> Finding {
    let package = w
        .package
        .as_ref()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "<unknown>".to_string());
    let severity = if w.package.is_some() {
        Severity::Low
    } else {
        Severity::Info
    };
    let detail = match &w.message {
        Some(m) if !m.is_empty() => format!("`{package}` has a `{}` warning: {m}", w.kind),
        _ => format!("`{package}` has a `{}` warning.", w.kind),
    };
    Finding {
        id: finding_id(Kind::RustWarning, format!("{package}:{}", w.kind)),
        severity,
        kind: Kind::RustWarning,
        title: format!("{} warning: {package}", w.kind),
        detail,
        remediation: format!(
            "Review `{package}` (`{}`); consider an alternative or a maintained fork.",
            w.kind
        ),
        invoke: Invoke::command(format!("cargo update -p {package}")),
        evidence: serde_json::json!({
            "package": package,
            "kind": w.kind,
        }),
        location: None,
    }
}

// === Fuzz + injection producers (Wave 6, `meter fuzz`) ============================
//
// These map the security-engine outputs to `FuzzCrash`/`Injection` findings with
// BYTE-REPRODUCIBLE ids. The id of a fuzz crash embeds `blake3(input)[..8]` (8
// hex chars of a stable content hash of the crashing input) so the SAME crashing
// input produces the SAME id across separate process runs — the determinism
// contract from AGENT-SURFACE §3. Injection ids embed `blake3(payload)[..8]` for
// the same reason. blake3 is a tiny pure-Rust hasher pulled in only under the
// `capture` feature (the only feature that drives the fuzzers), so this block is
// `capture`-gated.

/// Truncated content hash used in fuzz/injection finding ids: the first 8 hex
/// characters of `blake3(bytes)`. Stable across runs and processes, so ids built
/// from the same crashing input / payload are byte-identical — the
/// reproducibility contract.
#[cfg(feature = "capture")]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-producer-rs.md#source
pub fn hash8(bytes: &[u8]) -> String {
    let digest = blake3::hash(bytes);
    // `to_hex()` is lowercase, deterministic, and platform-independent.
    digest.to_hex().as_str()[..8].to_string()
}

/// A single injection leak surfaced by `meter fuzz` (`demo-sql` / future endpoint
/// injection paths). Carries the payload that got through, its category, and
/// whether it was reflected verbatim. The capture layer builds these from the
/// engine's `InjectionResult`/`InjectionTest` output; the producer turns each
/// into a deterministic `Injection` finding.
#[cfg(feature = "capture")]
#[derive(Debug, Clone)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-producer-rs.md#source
pub struct InjectionHit {
    /// The injection payload that was not blocked.
    pub payload: String,
    /// The payload category slug (e.g. `"sql_injection"`).
    pub category: String,
    /// `true` if the payload was reflected verbatim (Allowed); `false` if it was
    /// modified-then-accepted (Sanitized).
    pub reflected: bool,
}

/// Build a `FuzzCrash` finding from one engine [`FuzzCrash`](crate::security::FuzzCrash).
///
/// `id = "fuzz_crash:{target}:{hash8}"` where `hash8 = blake3(input)[..8]` — a
/// stable content hash of the crashing input, so repeated seeded runs against the
/// same target produce byte-identical ids. `evidence = {input_b64, panic_msg,
/// strategy, seed}` (the input is base64-encoded so non-UTF-8/control bytes
/// survive a JSON round-trip intact).
#[cfg(feature = "capture")]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-producer-rs.md#source
pub fn fuzz_crash_finding(
    target: &str,
    crash: &crate::security::FuzzCrash,
    strategy: &str,
    seed: u64,
) -> Finding {
    let input_bytes = crash.input.as_bytes();
    let hash = hash8(input_bytes);
    let input_b64 = base64_encode(input_bytes);
    Finding {
        id: format!("fuzz_crash:{target}:{hash}"),
        severity: Severity::High,
        kind: Kind::FuzzCrash,
        title: format!("fuzz crash in `{target}` ({hash})"),
        detail: format!(
            "Fuzzing `{target}` (seed {seed}, strategy `{strategy}`) produced an input that the \
             target rejected/crashed on: {}",
            crash.error
        ),
        remediation: format!(
            "Decode `evidence.input_b64` and reproduce the crash, then harden `{target}` against \
             that input. Re-run `meter fuzz --target {target} --seed {seed}` to confirm the fix."
        ),
        invoke: Invoke::command(format!("meter fuzz --target {target} --seed {seed}")),
        evidence: serde_json::json!({
            "input_b64": input_b64,
            "panic_msg": crash.error,
            "strategy": strategy,
            "seed": seed,
        }),
        location: None,
    }
}

/// Build an `Injection` finding from one [`InjectionHit`].
///
/// `id = "injection:{target}:{category}:{hash8}"` where `hash8 =
/// blake3(payload)[..8]`, so the same leaked payload yields the same id across
/// runs. `evidence = {payload, category, reflected}`.
#[cfg(feature = "capture")]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-producer-rs.md#source
pub fn injection_finding(target: &str, hit: &InjectionHit) -> Finding {
    let hash = hash8(hit.payload.as_bytes());
    let category = &hit.category;
    Finding {
        id: format!("injection:{target}:{category}:{hash}"),
        severity: Severity::High,
        kind: Kind::Injection,
        title: format!("injection leak in `{target}` ({category})"),
        detail: format!(
            "The `{target}` validator did not block a `{category}` payload (reflected: {}): {}",
            hit.reflected, hit.payload
        ),
        remediation: format!(
            "Reject or fully neutralize the payload in `evidence.payload` (category `{category}`). \
             Use parameterized queries / strict allow-lists, then re-run `meter fuzz --target {target}`."
        ),
        invoke: Invoke::command(format!("meter fuzz --target {target}")),
        evidence: serde_json::json!({
            "payload": hit.payload,
            "category": category,
            "reflected": hit.reflected,
        }),
        location: None,
    }
}

/// Map an engine [`FuzzResult`](crate::security::FuzzResult) into `FuzzCrash`
/// findings. The bare `FuzzResult` carries no target/seed/strategy context (those
/// live on the capture-layer invocation), so this convenience impl stamps a
/// generic `target = "fuzz"`, `seed = 0`, `strategy = "unknown"`. The CLI path in
/// `capture::fuzz` calls [`fuzz_crash_finding`] DIRECTLY with the real
/// target/seed so its ids are byte-reproducible per invocation; this impl exists
/// so `FuzzResult` satisfies the `IntoFindings` boundary trait uniformly.
#[cfg(feature = "capture")]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-producer-rs.md#source
impl IntoFindings for crate::security::FuzzResult {
    fn into_findings(&self) -> Vec<Finding> {
        self.crashes
            .iter()
            .map(|c| fuzz_crash_finding("fuzz", c, "unknown", 0))
            .collect()
    }
}

/// Map a single engine [`InjectionResult`](crate::security::InjectionResult) into
/// zero-or-one `Injection` findings. `InjectionResult` is an ENUM with no payload
/// text of its own, so only the leak VARIANTS map to a finding, and they carry a
/// placeholder payload/category (the capture layer's `demo-sql` path uses the
/// richer [`injection_finding`] with the real payload). `Blocked`/`Error` are the
/// safe/non-leak outcomes and produce nothing.
#[cfg(feature = "capture")]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-producer-rs.md#source
impl IntoFindings for crate::security::InjectionResult {
    fn into_findings(&self) -> Vec<Finding> {
        use crate::security::InjectionResult;
        let reflected = match self {
            InjectionResult::Allowed => true,
            InjectionResult::Sanitized => false,
            // Blocked / Error are NOT leaks: no finding.
            InjectionResult::Blocked | InjectionResult::Error(_) => return Vec::new(),
        };
        vec![injection_finding(
            "injection",
            &InjectionHit {
                payload: String::new(),
                category: "unknown".to_string(),
                reflected,
            },
        )]
    }
}

/// Minimal standard base64 (RFC 4648) encoder for fuzz-crash inputs. We encode
/// the RAW crashing bytes so non-UTF-8 / control bytes survive the JSON round
/// trip intact and the agent can decode the EXACT input. Kept local (a few
/// lines) to avoid taking a base64 crate dependency for one field.
#[cfg(feature = "capture")]
fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Map a cargo-audit `cvss` field to a [`Severity`].
///
/// The field is most commonly a CVSS v3 vector string (e.g.
/// `"CVSS:3.1/AV:L/AC:H/..."`) or, less commonly, a bare base score. We extract
/// the first parseable base score from the string and bucket it:
/// `>= 9.0` critical, `>= 7.0` high, `>= 4.0` medium, else low. When no numeric
/// score can be recovered (the common vector-only case, or `None`), we default
/// to `high` — the conservative choice for an unrated advisory.
fn severity_from_cvss(cvss: Option<&str>) -> Severity {
    match cvss.and_then(cvss_base_score) {
        Some(score) if score >= 9.0 => Severity::Critical,
        Some(score) if score >= 7.0 => Severity::High,
        Some(score) if score >= 4.0 => Severity::Medium,
        Some(_) => Severity::Low,
        None => Severity::High,
    }
}

/// Recover a numeric base score from a cargo-audit cvss field, if one is
/// present. Accepts a bare `"7.5"` or a leading base score (e.g.
/// `"9.1/CVSS:3.1/..."`); a pure CVSS vector string yields `None` (its only
/// numbers are the `CVSS:3.1` version + metric values, not a base score), so it
/// falls through to the conservative default `high`.
fn cvss_base_score(cvss: &str) -> Option<f64> {
    let cvss = cvss.trim();
    // Fast path: the whole field is a bare score.
    if let Ok(score) = cvss.parse::<f64>() {
        return Some(score);
    }
    // A field that starts with the `CVSS:` prefix is a pure metric vector; its
    // embedded numbers are the spec version + metric codes, NOT a base score, so
    // do NOT mine it. Default-high applies.
    if cvss.starts_with("CVSS:") {
        return None;
    }
    // Otherwise accept a LEADING numeric token only (e.g. a base score prefixed
    // onto a vector: `9.1/CVSS:3.1/...`). The first token before any non-numeric
    // separator is the candidate.
    let lead: String = cvss
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    if let Ok(score) = lead.parse::<f64>() {
        if (0.0..=10.0).contains(&score) {
            return Some(score);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::{TestMeta, TestResult, TestStatus};
    use crate::rust_runner::{AuditResult, AuditWarning, Package, Vulnerability};

    fn result(status: TestStatus) -> TestResult {
        TestResult {
            meta: TestMeta::new("mod::name"),
            status,
            duration_ms: 1,
            error: Some("assertion failed".into()),
            stack_trace: None,
            profile_metrics: None,
            stress_metrics: None,
            started_at: String::new(),
        }
    }

    #[test]
    fn failed_test_produces_finding() {
        let f = result(TestStatus::Failed).into_findings();
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].kind, Kind::TestFailure);
        assert!(f[0].id.starts_with("test_failure:"));
        assert_eq!(f[0].evidence["name"], "mod::name");
    }

    #[test]
    fn error_test_produces_finding() {
        assert_eq!(result(TestStatus::Error).into_findings().len(), 1);
    }

    #[test]
    fn passed_test_produces_nothing() {
        assert!(result(TestStatus::Passed).into_findings().is_empty());
        assert!(result(TestStatus::Skipped).into_findings().is_empty());
    }

    #[test]
    fn generic_failure_has_stable_id() {
        let f = generic_test_failure("-p meter", 101);
        assert_eq!(f.id, "test_failure:delegated-run");
        assert_eq!(f.evidence["exit_code"], 101);
    }

    fn vuln(id: &str, cvss: Option<&str>) -> Vulnerability {
        Vulnerability {
            id: id.to_string(),
            package: Package {
                name: "time".into(),
                version: "0.1.45".into(),
            },
            severity: None,
            title: "Potential segfault in the time crate".into(),
            description: "Unix-like operating systems may segfault.".into(),
            cvss: cvss.map(|s| s.to_string()),
        }
    }

    #[test]
    fn audit_maps_vuln_to_rust_vuln_finding() {
        let result = AuditResult {
            vulnerabilities: vec![vuln("RUSTSEC-2020-0071", Some("7.5"))],
            warnings: vec![],
        };
        let f = result.into_findings();
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].kind, Kind::RustVuln);
        assert_eq!(f[0].id, "rust_vuln:RUSTSEC-2020-0071");
        assert_eq!(f[0].severity, Severity::High);
        assert_eq!(f[0].invoke.command, "cargo update -p time");
        assert_eq!(f[0].evidence["advisory_id"], "RUSTSEC-2020-0071");
        assert_eq!(f[0].evidence["package"], "time");
        assert_eq!(f[0].evidence["version"], "0.1.45");
        assert_eq!(f[0].evidence["cvss"], "7.5");
        assert!(f[0].remediation.contains("Upgrade `time`"));
    }

    #[test]
    fn audit_id_is_deterministic() {
        let result = AuditResult {
            vulnerabilities: vec![vuln("RUSTSEC-2020-0071", None)],
            warnings: vec![],
        };
        let a = result.clone().into_findings();
        let b = result.into_findings();
        assert_eq!(a[0].id, b[0].id);
    }

    #[test]
    fn cvss_buckets_severity() {
        assert_eq!(severity_from_cvss(Some("9.8")), Severity::Critical);
        assert_eq!(severity_from_cvss(Some("7.5")), Severity::High);
        assert_eq!(severity_from_cvss(Some("5.0")), Severity::Medium);
        assert_eq!(severity_from_cvss(Some("2.1")), Severity::Low);
        // Unknown / vector-only / None default to high.
        assert_eq!(severity_from_cvss(None), Severity::High);
        assert_eq!(
            severity_from_cvss(Some("CVSS:3.1/AV:L/AC:H/PR:N/UI:N/S:C/C:N/I:N/A:H")),
            Severity::High
        );
    }

    #[test]
    fn cvss_extracts_embedded_score() {
        // A vector that also carries a base score should bucket on that score.
        assert_eq!(
            severity_from_cvss(Some("9.1/CVSS:3.1/AV:N")),
            Severity::Critical
        );
    }

    #[test]
    fn audit_maps_warning_to_rust_warning_finding() {
        let result = AuditResult {
            vulnerabilities: vec![],
            warnings: vec![AuditWarning {
                kind: "unmaintained".into(),
                package: Some(Package {
                    name: "ansi_term".into(),
                    version: "0.12.1".into(),
                }),
                message: Some("crate is unmaintained".into()),
            }],
        };
        let f = result.into_findings();
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].kind, Kind::RustWarning);
        assert_eq!(f[0].id, "rust_warning:ansi_term:unmaintained");
        assert_eq!(f[0].severity, Severity::Low);
        assert_eq!(f[0].evidence["package"], "ansi_term");
        assert_eq!(f[0].evidence["kind"], "unmaintained");
    }

    #[test]
    fn audit_warning_without_package_is_info() {
        let result = AuditResult {
            vulnerabilities: vec![],
            warnings: vec![AuditWarning {
                kind: "yanked".into(),
                package: None,
                message: None,
            }],
        };
        let f = result.into_findings();
        assert_eq!(f[0].severity, Severity::Info);
        assert_eq!(f[0].id, "rust_warning:<unknown>:yanked");
    }

    #[test]
    fn audit_empty_produces_no_findings() {
        let result = AuditResult {
            vulnerabilities: vec![],
            warnings: vec![],
        };
        assert!(result.into_findings().is_empty());
    }

    // --- regression producer (Wave 4) ---

    use crate::baseline::{
        PercentileType, Regression, RegressionReport, RegressionSeverity, RegressionSummary,
    };

    fn regression(name: &str, severity: RegressionSeverity, pct: f64) -> Regression {
        Regression {
            name: name.to_string(),
            percentile_type: PercentileType::Mean,
            baseline_value_ms: 10.0,
            current_value_ms: 10.0 * (1.0 + pct / 100.0),
            percent_change: pct,
            ci_overlap: false,
            severity,
        }
    }

    fn regression_report(regressions: Vec<Regression>) -> RegressionReport {
        let n = regressions.len();
        RegressionReport {
            baseline_timestamp: "2026-01-01T00:00:00Z".to_string(),
            current_timestamp: "2026-01-02T00:00:00Z".to_string(),
            regressions,
            improvements: Vec::new(),
            summary: RegressionSummary {
                total_benchmarks: n,
                regressions_found: n,
                improvements_found: 0,
                unchanged: 0,
            },
        }
    }

    #[test]
    fn regression_report_maps_severe_to_high_finding() {
        let report = regression_report(vec![regression(
            "bench_sha256",
            RegressionSeverity::Severe,
            20.0,
        )]);
        let f = report.into_findings();
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].kind, Kind::Regression);
        assert_eq!(f[0].id, "regression:bench_sha256");
        assert_eq!(f[0].severity, Severity::High);
        // Evidence carries the bench/baseline/current/percent/severity proof.
        assert_eq!(f[0].evidence["bench"], "bench_sha256");
        assert_eq!(f[0].evidence["baseline_ms"], 10.0);
        assert_eq!(f[0].evidence["current_ms"], 12.0);
        assert_eq!(f[0].evidence["percent_change"], 20.0);
        assert_eq!(f[0].evidence["severity"], "high");
        // Remediation + invoke point the agent at the named benchmark.
        assert!(f[0].invoke.command.contains("bench_sha256"));
        assert!(f[0].remediation.contains("bench_sha256"));
    }

    #[test]
    fn regression_report_maps_moderate_to_medium_finding() {
        let report = regression_report(vec![regression(
            "bench_parse",
            RegressionSeverity::Moderate,
            8.0,
        )]);
        let f = report.into_findings();
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].severity, Severity::Medium);
        assert_eq!(f[0].evidence["severity"], "medium");
    }

    #[test]
    fn regression_report_maps_minor_to_low_finding() {
        let report = regression_report(vec![regression(
            "bench_noop",
            RegressionSeverity::Minor,
            3.0,
        )]);
        let f = report.into_findings();
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].severity, Severity::Low);
        assert_eq!(f[0].evidence["severity"], "low");
    }

    #[test]
    fn regression_report_id_is_deterministic() {
        let report = regression_report(vec![regression(
            "bench_sha256",
            RegressionSeverity::Severe,
            20.0,
        )]);
        let a = report.clone().into_findings();
        let b = report.into_findings();
        assert_eq!(a[0].id, b[0].id);
    }

    #[test]
    fn regression_report_empty_produces_no_findings() {
        let report = regression_report(vec![]);
        assert!(report.into_findings().is_empty());
    }

    // --- boundary-cost producer (Wave 5, `meter profile --phases` embed path) ---

    use crate::performance::profiler::PhaseBreakdown;
    use std::collections::HashMap;

    fn breakdown() -> PhaseBreakdown {
        let mut times: HashMap<String, Vec<u64>> = HashMap::new();
        // RustConvert dominates (6ms over 2 entries), PythonExtract second (3ms).
        times.insert("PythonExtract".to_string(), vec![1_500_000, 1_500_000]);
        times.insert("RustConvert".to_string(), vec![3_000_000, 3_000_000]);
        PhaseBreakdown::from_times(times, 2, 9_000_000)
    }

    #[test]
    fn phase_breakdown_maps_to_boundary_cost_findings() {
        let f = breakdown().into_findings();
        assert_eq!(f.len(), 2);
        assert!(f.iter().all(|x| x.kind == Kind::BoundaryCost));
        // Emitted self_ns desc: RustConvert (6ms) before PythonExtract (3ms).
        assert_eq!(f[0].evidence["phase"], "RustConvert");
        assert_eq!(f[0].id, "boundary:RustConvert");
        assert_eq!(f[0].evidence["self_ns"], 6_000_000u64);
        assert_eq!(f[0].evidence["total_ns"], 9_000_000u64);
        assert_eq!(f[0].evidence["samples"], 2);
        // pct = 6/9 ≈ 0.6667.
        let pct = f[0].evidence["pct"].as_f64().unwrap();
        assert!((pct - 0.6666).abs() < 0.01);
    }

    #[test]
    fn boundary_cost_findings_are_info_severity() {
        let f = breakdown().into_findings();
        assert!(f.iter().all(|x| x.severity == Severity::Info));
    }

    #[test]
    fn boundary_cost_evidence_has_full_contract_shape() {
        let f = breakdown().into_findings();
        for key in ["phase", "self_ns", "total_ns", "pct", "samples"] {
            assert!(f[0].evidence.get(key).is_some(), "missing `{key}`");
        }
    }

    #[test]
    fn empty_breakdown_produces_no_findings() {
        let pb = PhaseBreakdown::new();
        assert!(pb.into_findings().is_empty());
    }
}

// === Fuzz + injection producer tests (Wave 6) =================================
#[cfg(all(test, feature = "capture"))]
mod fuzz_tests {
    use super::*;
    use crate::security::FuzzCrash;

    fn crash(input: &str, error: &str) -> FuzzCrash {
        FuzzCrash {
            input: input.to_string(),
            error: error.to_string(),
            iteration: 7,
        }
    }

    #[test]
    fn hash8_is_eight_lowercase_hex_chars() {
        let h = hash8(b"hello");
        assert_eq!(h.len(), 8);
        assert!(h
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn hash8_is_stable_for_the_same_bytes() {
        // The reproducibility primitive: identical bytes => identical hash.
        assert_eq!(hash8(b"' OR 1=1--"), hash8(b"' OR 1=1--"));
        assert_ne!(hash8(b"a"), hash8(b"b"));
    }

    #[test]
    fn fuzz_crash_finding_has_reproducible_id_and_evidence() {
        let c = crash("bad\0input", "embedded NUL");
        let f = fuzz_crash_finding("demo-crash", &c, "mutation:all", 42);
        assert_eq!(f.kind, Kind::FuzzCrash);
        // id = fuzz_crash:{target}:{blake3(input)[..8]}.
        let expected_hash = hash8("bad\0input".as_bytes());
        assert_eq!(f.id, format!("fuzz_crash:demo-crash:{expected_hash}"));
        // evidence carries the full {input_b64, panic_msg, strategy, seed} shape.
        assert_eq!(f.evidence["panic_msg"], "embedded NUL");
        assert_eq!(f.evidence["strategy"], "mutation:all");
        assert_eq!(f.evidence["seed"], 42);
        assert!(f.evidence["input_b64"].is_string());
    }

    #[test]
    fn fuzz_crash_id_is_byte_identical_across_calls() {
        let c = crash("payload", "boom");
        let a = fuzz_crash_finding("t", &c, "s", 1);
        let b = fuzz_crash_finding("t", &c, "s", 1);
        assert_eq!(a.id, b.id);
    }

    #[test]
    fn fuzz_crash_input_b64_decodes_back_to_input() {
        // Round-trip the base64 so non-UTF8/control bytes are recoverable.
        let c = crash("a;b'c", "meta");
        let f = fuzz_crash_finding("t", &c, "s", 0);
        let b64 = f.evidence["input_b64"].as_str().unwrap();
        assert_eq!(b64, base64_encode("a;b'c".as_bytes()));
    }

    #[test]
    fn fuzz_result_into_findings_maps_all_crashes() {
        let result = crate::security::FuzzResult {
            iterations: 10,
            crashes: vec![crash("x", "e1"), crash("y", "e2")],
            duration_ms: 1,
        };
        let f = result.into_findings();
        assert_eq!(f.len(), 2);
        assert!(f.iter().all(|x| x.kind == Kind::FuzzCrash));
        // The bare-impl convenience path stamps a generic target/seed.
        assert!(f.iter().all(|x| x.id.starts_with("fuzz_crash:fuzz:")));
    }

    #[test]
    fn injection_finding_has_reproducible_id_and_evidence() {
        let hit = InjectionHit {
            payload: "' OR 1=1--".to_string(),
            category: "sql_injection".to_string(),
            reflected: true,
        };
        let f = injection_finding("demo-sql", &hit);
        assert_eq!(f.kind, Kind::Injection);
        let expected_hash = hash8("' OR 1=1--".as_bytes());
        assert_eq!(
            f.id,
            format!("injection:demo-sql:sql_injection:{expected_hash}")
        );
        assert_eq!(f.evidence["payload"], "' OR 1=1--");
        assert_eq!(f.evidence["category"], "sql_injection");
        assert_eq!(f.evidence["reflected"], true);
    }

    #[test]
    fn injection_id_is_byte_identical_across_calls() {
        let hit = InjectionHit {
            payload: "admin'--".to_string(),
            category: "sql_injection".to_string(),
            reflected: false,
        };
        let a = injection_finding("demo-sql", &hit);
        let b = injection_finding("demo-sql", &hit);
        assert_eq!(a.id, b.id);
    }

    #[test]
    fn injection_result_enum_maps_only_leak_variants() {
        use crate::security::InjectionResult;
        // Allowed / Sanitized are leaks => one finding each.
        assert_eq!(InjectionResult::Allowed.into_findings().len(), 1);
        assert_eq!(InjectionResult::Sanitized.into_findings().len(), 1);
        // Blocked / Error are safe => no finding.
        assert!(InjectionResult::Blocked.into_findings().is_empty());
        assert!(InjectionResult::Error("x".into())
            .into_findings()
            .is_empty());
    }
}
// CODEGEN-END
