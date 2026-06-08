// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-report-emit-rs.md#source
// CODEGEN-BEGIN
//! [`emit`] — print exactly ONE serde_json document to stdout.
//!
//! Everything that is not the report (progress, diagnostics, warnings) goes to
//! stderr. The default form is pretty-printed; `compact` switches to the dense
//! single-line form for byte-stable golden comparisons.

use std::io::Write;

use super::envelope::MeterReport;

/// Serialize `report` and print it as the single stdout JSON document.
///
/// `compact == false` => `to_string_pretty`; `compact == true` => `to_string`.
/// Returns the serialized string for callers that also want it in-hand.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-emit-rs.md#source
pub fn emit(report: &MeterReport, compact: bool) -> String {
    let json = render(report, compact);
    // Exactly one stdout write of the report; trailing newline for line-based
    // tooling. Use the lock to keep the single document atomic.
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    let _ = lock.write_all(json.as_bytes());
    let _ = lock.write_all(b"\n");
    let _ = lock.flush();
    json
}

/// Render `report` to a JSON string without printing.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-emit-rs.md#source
pub fn render(report: &MeterReport, compact: bool) -> String {
    if compact {
        serde_json::to_string(report)
    } else {
        serde_json::to_string_pretty(report)
    }
    .unwrap_or_else(|e| {
        // Serialization of our own owned structs cannot realistically fail, but
        // never panic on the emit path: fall back to a minimal valid JSON doc.
        format!(
            "{{\"schema_version\":\"meter.report/1\",\"error\":\"serialize_failed: {}\"}}",
            e
        )
    })
}

/// Print a diagnostic line to stderr (never stdout).
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-report-emit-rs.md#source
pub fn diag(msg: impl AsRef<str>) {
    let _ = writeln!(std::io::stderr(), "{}", msg.as_ref());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::builder::ReportBuilder;
    use crate::report::envelope::EnvBlock;

    fn env() -> EnvBlock {
        EnvBlock {
            os: "macos".into(),
            arch: "aarch64".into(),
            nextest_present: false,
            sampler_backend: "macos-sample".into(),
            rustc_version: None,
            note: String::new(),
        }
    }

    fn sample_report() -> MeterReport {
        let mut b = ReportBuilder::new("spec", "-");
        b.with_environment(env());
        b.finalize()
    }

    #[test]
    fn render_compact_is_single_line() {
        let r = sample_report();
        let s = render(&r, true);
        assert!(!s.contains('\n'));
        assert!(s.starts_with('{'));
    }

    #[test]
    fn render_pretty_is_multiline() {
        let r = sample_report();
        let s = render(&r, false);
        assert!(s.contains('\n'));
    }

    #[test]
    fn render_roundtrips_to_report() {
        let r = sample_report();
        let s = render(&r, true);
        let back: MeterReport = serde_json::from_str(&s).unwrap();
        assert_eq!(back.verb, "spec");
        assert_eq!(back.schema_version, "meter.report/1");
    }
}
// CODEGEN-END
