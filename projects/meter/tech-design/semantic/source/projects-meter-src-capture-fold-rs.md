---
id: projects-meter-src-capture-fold-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-use-first-cli
    role: primary
    gap: delegated-runner-exit-code-contract
    claim: delegated-runner-exit-code-contract
    coverage: full
    rationale: "Source template implements meter agent-facing CLI, runner, or report surfaces."
---

# Standardized projects/meter/src/capture/fold.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/capture/fold.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Hotspot` | projects/meter/src/capture/fold.rs | struct | pub | 30 |  |
| `aggregate` | projects/meter/src/capture/fold.rs | function | pub | 54 | aggregate(stacks: &[FoldedStack], effective_hz: f64) -> Vec<Hotspot> |
| `fold_hotspots` | projects/meter/src/capture/fold.rs | function | pub | 129 | fold_hotspots(     stacks: &[FoldedStack],     effective_hz: f64,     fail_hot: Option<f64>, ) -> Vec<Finding> |
| `to_flamegraph` | projects/meter/src/capture/fold.rs | function | pub | 193 | to_flamegraph(stacks: &[FoldedStack]) -> FlamegraphData |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Fold folded stacks into ranked per-symbol hot spots — the C1 contract.
//!
//! [`fold_hotspots`] aggregates a `Vec<FoldedStack>` (root->leaf frame lists +
//! leaf sample counts) into per-symbol `self`/`total`/`samples` metrics:
//!
//! - `self` (samples) = samples whose LEAF is that symbol.
//! - `total` (samples) = samples whose stack CONTAINS that symbol (inclusive).
//! - `pct` = `self_samples / total_samples_overall` (fraction of all samples
//!   spent with this symbol as the leaf).
//! - `ns ≈ samples * (1e9 / effective_hz)`.
//!
//! The result is a `Vec<Finding{kind:Hotspot}>` PRE-SORTED by `self_ns` DESC,
//! with `evidence = {symbol, self_ns, total_ns, pct, samples, rank}` — the
//! DEFAULT stdout of `meter profile`. The SVG is a separate `--human`-only artifact
//! ([`to_flamegraph`]); it never appears in the JSON report.

use std::collections::HashMap;

use crate::performance::profiler::FlamegraphData;
use crate::report::finding::{finding_id, Finding, Invoke, Kind, Location, Severity};

use super::sampler::FoldedStack;

/// Per-symbol aggregated hot-spot metrics (the raw numbers behind a `Hotspot`
/// finding's evidence).
#[derive(Debug, Clone, PartialEq)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-fold-rs.md#source
pub struct Hotspot {
    /// The symbol (function) name.
    pub symbol: String,
    /// Samples whose LEAF is this symbol (exclusive / self time).
    pub self_samples: u64,
    /// Samples whose stack CONTAINS this symbol (inclusive / total time).
    pub total_samples: u64,
    /// Self time in nanoseconds (`self_samples * ns_per_sample`).
    pub self_ns: u64,
    /// Total (inclusive) time in nanoseconds.
    pub total_ns: u64,
    /// `self_samples / total_samples_overall` — fraction of all wall samples
    /// spent with this symbol as the leaf.
    pub pct: f64,
    /// 1-based rank by `self_ns` descending.
    pub rank: usize,
}

/// Aggregate folded stacks into ranked [`Hotspot`]s, sorted by `self_ns` desc.
///
/// `effective_hz` maps samples -> ns (`ns = samples * 1e9 / hz`). Returns an
/// empty vec for empty input. Ordering is deterministic: primary key `self_ns`
/// desc, ties broken by `symbol` asc so equal-weight symbols rank stably.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-fold-rs.md#source
pub fn aggregate(stacks: &[FoldedStack], effective_hz: f64) -> Vec<Hotspot> {
    let total_samples_overall: u64 = stacks.iter().map(|s| s.count).sum();
    if total_samples_overall == 0 {
        return Vec::new();
    }
    let ns_per_sample = if effective_hz > 0.0 {
        1e9 / effective_hz
    } else {
        0.0
    };

    // self_samples: keyed on the leaf symbol.
    // total_samples: keyed on every DISTINCT symbol in a stack (count a stack's
    // samples once per symbol it contains, even if the symbol repeats via
    // recursion).
    let mut self_by_symbol: HashMap<&str, u64> = HashMap::new();
    let mut total_by_symbol: HashMap<&str, u64> = HashMap::new();

    for stack in stacks {
        if let Some(leaf) = stack.leaf() {
            *self_by_symbol.entry(leaf).or_insert(0) += stack.count;
        }
        // Distinct symbols in this stack get the stack's samples added to their
        // inclusive total exactly once.
        let mut seen: Vec<&str> = Vec::with_capacity(stack.frames.len());
        for f in &stack.frames {
            if !seen.contains(&f.as_str()) {
                seen.push(f.as_str());
                *total_by_symbol.entry(f.as_str()).or_insert(0) += stack.count;
            }
        }
    }

    // Build a hotspot per symbol that has a leaf (self) presence. A pure-caller
    // symbol with no self samples is not a hot spot by self-time; it still has a
    // total, but the ranked self-time list only includes leaves.
    let mut hotspots: Vec<Hotspot> = self_by_symbol
        .iter()
        .map(|(&symbol, &self_samples)| {
            let total_samples = *total_by_symbol.get(symbol).unwrap_or(&self_samples);
            Hotspot {
                symbol: symbol.to_string(),
                self_samples,
                total_samples,
                self_ns: (self_samples as f64 * ns_per_sample) as u64,
                total_ns: (total_samples as f64 * ns_per_sample) as u64,
                pct: self_samples as f64 / total_samples_overall as f64,
                rank: 0,
            }
        })
        .collect();

    // Sort by self_ns desc, ties by symbol asc for determinism. Sorting on
    // self_samples is equivalent to self_ns (monotone in samples) and avoids
    // float ordering pitfalls.
    hotspots.sort_by(|a, b| {
        b.self_samples
            .cmp(&a.self_samples)
            .then_with(|| a.symbol.cmp(&b.symbol))
    });
    for (i, h) in hotspots.iter_mut().enumerate() {
        h.rank = i + 1;
    }
    hotspots
}

/// Build the ranked `Hotspot` findings from folded stacks, PRE-SORTED by
/// `self_ns` desc. `fail_hot`, when set, marks any hot spot whose `pct` exceeds
/// the threshold as `High` severity (the rest are `Info`); without it every hot
/// spot is informational.
///
/// This is the DEFAULT stdout of `meter profile`. The findings carry
/// `evidence = {symbol, self_ns, total_ns, pct, samples, rank}` — the C1
/// contract. `module_for` (optional) supplies a source symbol/location hint.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-fold-rs.md#source
pub fn fold_hotspots(
    stacks: &[FoldedStack],
    effective_hz: f64,
    fail_hot: Option<f64>,
) -> Vec<Finding> {
    let hotspots = aggregate(stacks, effective_hz);
    let total_self: u64 = hotspots.iter().map(|h| h.self_samples).sum();
    let unsym_self: u64 = hotspots
        .iter()
        .filter(|h| h.symbol == UNSYMBOLICATED_SYMBOL)
        .map(|h| h.self_samples)
        .sum();
    let unsym_dominant =
        total_self > 0 && unsym_self as f64 / total_self as f64 > UNSYMBOLICATED_WARN_SHARE;
    hotspots
        .into_iter()
        .map(|h| {
            if unsym_dominant && h.symbol == UNSYMBOLICATED_SYMBOL {
                unsymbolicated_finding(&h, fail_hot)
            } else {
                hotspot_finding(&h, fail_hot)
            }
        })
        .collect()
}

/// The leaf symbol the platform sampler emits for a frame it could not
/// symbolicate (stripped binary / missing debug info).
const UNSYMBOLICATED_SYMBOL: &str = "???";

/// Self-sample share above which unsymbolicated leaves mean the MEASUREMENT is
/// broken (the target binary has no symbols), not that the workload is hot in
/// unknown code. Past this share the `???` hot spot is escalated to a warning
/// so the report is non-clean and carries a rebuild-with-symbols remediation.
const UNSYMBOLICATED_WARN_SHARE: f64 = 0.5;

/// The `???` hot spot when it DOMINATES self time: a `Medium` (at least)
/// finding telling the agent the profile is unusable until the target is
/// rebuilt with symbols, instead of a "clean" report ranking `???` as the
/// thing to optimize.
fn unsymbolicated_finding(h: &Hotspot, fail_hot: Option<f64>) -> Finding {
    let mut f = hotspot_finding(h, fail_hot);
    // Escalate to at least Medium (a `--fail-hot` High stays High).
    if matches!(f.severity, Severity::Low | Severity::Info) {
        f.severity = Severity::Medium;
    }
    let pct_display = h.pct * 100.0;
    f.title = format!(
        "unsymbolicated frames dominate sampling ({pct_display:.1}% self) — target has no symbols"
    );
    f.detail = format!(
        "{:.1}% of self samples fold to `???` leaves, so the ranked hot spots cannot name the \
         code that is actually hot. The sampled binary was most likely built without symbols \
         (e.g. a release profile with `strip = true`).",
        pct_display,
    );
    f.remediation = "Rebuild the target with symbols and re-profile: \
                     `CARGO_PROFILE_RELEASE_STRIP=false CARGO_PROFILE_RELEASE_DEBUG=true \
                     cargo build --release`, or add a dedicated `[profile.profiling]` that \
                     inherits release with `strip = false, debug = true`."
        .to_string();
    f
}

/// Convert one aggregated [`Hotspot`] into a `Finding{kind:Hotspot}`.
fn hotspot_finding(h: &Hotspot, fail_hot: Option<f64>) -> Finding {
    let pct_display = h.pct * 100.0;
    // Severity: a hot spot that breaches `--fail-hot` is High (it is the thing
    // to fix); otherwise it is informational. Hot spots are NOT failures by
    // default — finding where time goes is the whole point.
    let severity = match fail_hot {
        Some(threshold) if pct_display > threshold => Severity::High,
        _ => Severity::Info,
    };
    Finding {
        id: finding_id(Kind::Hotspot, &h.symbol),
        severity,
        kind: Kind::Hotspot,
        title: format!("hot spot: {} ({:.1}% self)", h.symbol, pct_display),
        detail: format!(
            "`{}` is the innermost frame in {} of {} sampled stacks ({:.1}% self time, \
             {:.3}ms self / {:.3}ms total).",
            h.symbol,
            h.self_samples,
            h.total_samples,
            pct_display,
            h.self_ns as f64 / 1e6,
            h.total_ns as f64 / 1e6,
        ),
        remediation: format!(
            "Investigate `{}`: it dominates self time. Reduce its work, cache its result, \
             or move it off the hot path, then re-run `meter profile` to confirm.",
            h.symbol
        ),
        invoke: Invoke::command(format!(
            "meter profile --fail-hot {:.0}",
            pct_display.max(1.0)
        )),
        evidence: serde_json::json!({
            "symbol": h.symbol,
            "self_ns": h.self_ns,
            "total_ns": h.total_ns,
            "pct": h.pct,
            "samples": h.self_samples,
            "rank": h.rank,
        }),
        location: Some(Location {
            file: None,
            line: None,
            symbol: Some(h.symbol.clone()),
        }),
    }
}

/// Push the SAME folded stacks into a [`FlamegraphData`] for the `--human` SVG
/// export. The fold/rank JSON is the default; this is the demoted side artifact.
///
/// Returns a [`FlamegraphData`] ready for
/// [`generate_flamegraph_svg`](crate::performance::profiler::generate_flamegraph_svg).
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-fold-rs.md#source
pub fn to_flamegraph(stacks: &[FoldedStack]) -> FlamegraphData {
    let mut data = FlamegraphData::new();
    for stack in stacks {
        // inferno's folded format is `a;b;c <count>`.
        data.folded_stacks.push(stack.to_folded_line());
    }
    data.sample_count = stacks.iter().map(|s| s.count).sum();
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A synthetic stack set: `hot` is the dominant leaf, `cold` second, plus a
    /// shared caller `main` so total/self differ.
    fn synthetic() -> Vec<FoldedStack> {
        vec![
            // 80 samples terminate in `hot`, under main.
            FoldedStack::new(vec!["main".into(), "hot".into()], 80),
            // 15 samples terminate in `cold`, also under main.
            FoldedStack::new(vec!["main".into(), "cold".into()], 15),
            // 5 samples terminate in `main` itself (self time in the caller).
            FoldedStack::new(vec!["main".into()], 5),
        ]
    }

    // 1000 Hz => 1ms per sample => 1_000_000 ns/sample (round number for asserts).
    const HZ: f64 = 1000.0;

    #[test]
    fn unsymbolicated_dominant_escalates_to_medium_warning() {
        // 70% of self samples fold to `???` — the measurement is broken.
        let stacks = vec![
            FoldedStack::new(vec!["main".into(), "???".into()], 70),
            FoldedStack::new(vec!["main".into(), "hot".into()], 30),
        ];
        let findings = fold_hotspots(&stacks, HZ, None);
        let unsym = findings
            .iter()
            .find(|f| f.id == "hotspot:???")
            .expect("??? finding present");
        assert_eq!(unsym.severity, Severity::Medium);
        assert!(unsym.title.contains("unsymbolicated"), "{}", unsym.title);
        assert!(
            unsym.remediation.contains("CARGO_PROFILE_RELEASE_STRIP"),
            "{}",
            unsym.remediation
        );
        // The symbolized hotspot stays informational.
        let hot = findings.iter().find(|f| f.id == "hotspot:hot").unwrap();
        assert_eq!(hot.severity, Severity::Info);
    }

    #[test]
    fn unsymbolicated_minority_stays_informational() {
        // 20% `???` is normal (JIT frames, dyld stubs) — no escalation.
        let stacks = vec![
            FoldedStack::new(vec!["main".into(), "???".into()], 20),
            FoldedStack::new(vec!["main".into(), "hot".into()], 80),
        ];
        let findings = fold_hotspots(&stacks, HZ, None);
        assert!(findings.iter().all(|f| f.severity == Severity::Info));
        assert!(findings.iter().all(|f| !f.title.contains("unsymbolicated")));
    }

    #[test]
    fn aggregate_splits_self_and_total() {
        let h = aggregate(&synthetic(), HZ);
        // Three leaf symbols: hot, cold, main.
        assert_eq!(h.len(), 3);
        let hot = h.iter().find(|x| x.symbol == "hot").unwrap();
        // self = 80 (only `hot` leaves). total = 80 (only stacks containing hot).
        assert_eq!(hot.self_samples, 80);
        assert_eq!(hot.total_samples, 80);
        // ns = samples * 1e9/1000 = samples * 1_000_000.
        assert_eq!(hot.self_ns, 80_000_000);

        let main = h.iter().find(|x| x.symbol == "main").unwrap();
        // self = 5 (only the lone `main` stack leaves in main).
        assert_eq!(main.self_samples, 5);
        // total = 100 (main is on EVERY stack: 80 + 15 + 5).
        assert_eq!(main.total_samples, 100);
        assert_eq!(main.total_ns, 100_000_000);
    }

    #[test]
    fn pct_is_self_over_total_overall() {
        let h = aggregate(&synthetic(), HZ);
        let hot = h.iter().find(|x| x.symbol == "hot").unwrap();
        // total samples overall = 100; hot self = 80 => 0.80.
        assert!((hot.pct - 0.80).abs() < 1e-9);
        let cold = h.iter().find(|x| x.symbol == "cold").unwrap();
        assert!((cold.pct - 0.15).abs() < 1e-9);
    }

    #[test]
    fn ranked_by_self_ns_desc() {
        let h = aggregate(&synthetic(), HZ);
        // hot (80) first, cold (15) second, main (5) last.
        assert_eq!(h[0].symbol, "hot");
        assert_eq!(h[0].rank, 1);
        assert_eq!(h[1].symbol, "cold");
        assert_eq!(h[1].rank, 2);
        assert_eq!(h[2].symbol, "main");
        assert_eq!(h[2].rank, 3);
        // self_ns strictly descending.
        assert!(h[0].self_ns >= h[1].self_ns);
        assert!(h[1].self_ns >= h[2].self_ns);
    }

    #[test]
    fn fold_hotspots_findings_are_presorted_and_hotspot_kind() {
        let findings = fold_hotspots(&synthetic(), HZ, None);
        assert_eq!(findings.len(), 3);
        // Every finding is a hotspot.
        assert!(findings.iter().all(|f| f.kind == Kind::Hotspot));
        // The first finding is the dominant `hot` leaf.
        assert_eq!(findings[0].id, "hotspot:hot");
        assert_eq!(findings[0].evidence["symbol"], "hot");
        assert_eq!(findings[0].evidence["samples"], 80);
        assert_eq!(findings[0].evidence["self_ns"], 80_000_000u64);
        assert_eq!(findings[0].evidence["rank"], 1);
        // self_ns is sorted DESCENDING across the findings (the C1 contract).
        let self_ns: Vec<u64> = findings
            .iter()
            .map(|f| f.evidence["self_ns"].as_u64().unwrap())
            .collect();
        let mut sorted = self_ns.clone();
        sorted.sort_by(|a, b| b.cmp(a));
        assert_eq!(
            self_ns, sorted,
            "findings must be pre-sorted by self_ns desc"
        );
    }

    #[test]
    fn evidence_has_full_contract_shape() {
        let findings = fold_hotspots(&synthetic(), HZ, None);
        let ev = &findings[0].evidence;
        for key in ["symbol", "self_ns", "total_ns", "pct", "samples", "rank"] {
            assert!(ev.get(key).is_some(), "evidence missing `{key}`");
        }
    }

    #[test]
    fn fail_hot_marks_breaching_hotspot_high() {
        // hot is 80% self; a 50% threshold makes it High, the rest stay Info.
        let findings = fold_hotspots(&synthetic(), HZ, Some(50.0));
        let hot = findings.iter().find(|f| f.id == "hotspot:hot").unwrap();
        assert_eq!(hot.severity, Severity::High);
        let cold = findings.iter().find(|f| f.id == "hotspot:cold").unwrap();
        assert_eq!(cold.severity, Severity::Info);
    }

    #[test]
    fn no_fail_hot_keeps_all_info() {
        let findings = fold_hotspots(&synthetic(), HZ, None);
        assert!(findings.iter().all(|f| f.severity == Severity::Info));
    }

    #[test]
    fn empty_input_yields_no_findings() {
        assert!(fold_hotspots(&[], HZ, None).is_empty());
        assert!(aggregate(&[], HZ).is_empty());
    }

    #[test]
    fn ties_broken_by_symbol_asc_for_determinism() {
        // Two leaves with equal self counts must order by symbol name.
        let stacks = vec![
            FoldedStack::new(vec!["r".into(), "zzz".into()], 10),
            FoldedStack::new(vec!["r".into(), "aaa".into()], 10),
        ];
        let h = aggregate(&stacks, HZ);
        assert_eq!(h[0].symbol, "aaa");
        assert_eq!(h[1].symbol, "zzz");
    }

    #[test]
    fn to_flamegraph_renders_folded_lines() {
        let fg = to_flamegraph(&synthetic());
        assert!(fg.has_data());
        assert_eq!(fg.sample_count, 100);
        assert!(fg.folded_stacks.iter().any(|l| l == "main;hot 80"));
        assert!(fg.folded_stacks.iter().any(|l| l == "main 5"));
    }

    #[test]
    fn total_counts_recursive_symbol_once_per_stack() {
        // A recursive stack `a;a;b` must count `a`'s inclusive total only ONCE
        // for that stack's samples (not twice).
        let stacks = vec![FoldedStack::new(
            vec!["a".into(), "a".into(), "b".into()],
            7,
        )];
        let h = aggregate(&stacks, HZ);
        let a = h.iter().find(|x| x.symbol == "a");
        // `a` is never a leaf here, so it does not appear in the self-keyed list.
        assert!(a.is_none());
        let b = h.iter().find(|x| x.symbol == "b").unwrap();
        assert_eq!(b.self_samples, 7);
        assert_eq!(b.total_samples, 7);
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/meter/src/capture/fold.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Source template for `projects/meter/src/capture/fold.rs` captured during meter full-codegen standardization.
```
