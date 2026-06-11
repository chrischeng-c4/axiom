//! The measure → compare → gate pipeline. For each cell, measure the base then
//! each peer (SEQUENTIALLY — concurrent load generators would contend and
//! poison the ratio), compute `ratio = peer/base`, classify, fold findings, and
//! gate WIN breaches as `PinRegression` (exit 2) via rig's worst-wins builder.

use std::path::PathBuf;

use rig::pins::BaselineStore;
use rig::report::{finding_id, Finding, Invoke, Kind, ReportBuilder, Severity};

use crate::compare::{classify, Classification};
use crate::measure::{fully_failed, load_is_honest, measure};
use crate::report::{ArenaReport, ComparisonRow, PeerCell};
use crate::spec::{Cell, LoadShape, Spec};

/// Knobs for one comparison run.
#[derive(Debug, Clone, Default)]
pub struct RunOpts {
    /// Record each measured ratio back as the new baseline (no gating this run).
    pub update_baselines: bool,
    /// Override the spec's baseline path (used by tests for isolation).
    pub baseline_path: Option<PathBuf>,
    /// Whether `--strict` makes a missing baseline a failure (default Info).
    pub strict: bool,
}

/// Run the whole comparison and produce one `arena.report/1`.
pub fn run(spec: &Spec, opts: &RunOpts) -> ArenaReport {
    let mut b = ReportBuilder::new("run", spec.name.clone());
    b.add_criterion(format!(
        "every WIN cell beats its peer by max(1.0, {}*baseline)",
        spec.ratchet
    ));

    let baseline_path = opts
        .baseline_path
        .clone()
        .or_else(|| spec.baseline.as_ref().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(".arena/baselines.json"));
    let mut store = BaselineStore::load_at(baseline_path);

    let mut comparison = Vec::new();

    for cell in &spec.cells {
        let metric = spec.cell_metric(cell).to_string();
        match measure_cell(spec, cell, &metric, opts, &mut store, &mut b) {
            Some(row) => comparison.push(row),
            None => {} // a tool_error was recorded; keep going to report all cells
        }
    }

    if opts.update_baselines {
        if let Err(e) = store.save() {
            b.add_missing(format!("could not persist baselines: {e}"));
        } else {
            b.add_missing("baselines updated; no gating performed this run".to_string());
        }
    }

    b.agent_prompt(prompt(spec, &comparison));
    ArenaReport::wrap(b.finalize(), comparison)
}

/// Measure one cell across its targets and fold the per-peer findings.
fn measure_cell(
    spec: &Spec,
    cell: &Cell,
    metric: &str,
    opts: &RunOpts,
    store: &mut BaselineStore,
    b: &mut ReportBuilder,
) -> Option<ComparisonRow> {
    // --- base ---
    let base_ct = cell.targets.get(&spec.base)?;
    let base_target = spec.targets.get(&spec.base)?;
    let base_load = target_load(spec, &spec.base, b)?;
    let base_stats = match measure(base_target, base_ct, base_load) {
        Ok(s) => s,
        Err(e) => {
            b.tool_error(
                5,
                format!("cell `{}` base `{}` aborted: {e}", cell.name, spec.base),
            );
            return None;
        }
    };
    if fully_failed(&base_stats) {
        b.add_finding(finding(
            Kind::ScenarioError,
            Severity::High,
            &format!("{}/{}", cell.name, spec.base),
            format!(
                "base target `{}` was unreachable for cell `{}`",
                spec.base, cell.name
            ),
            "Check the base service is up and the request is valid.",
            serde_json::json!({ "error_rate": base_stats.error_rate, "total": base_stats.total }),
        ));
        return None;
    }
    let base_value = base_stats.get(metric).unwrap_or(f64::NAN);
    let base_honest = load_is_honest(base_load, &base_stats);
    if !base_honest {
        b.add_finding(load_honesty_finding(
            &cell.name,
            &spec.base,
            base_load.target_qps,
            &base_stats,
        ));
    }

    // --- peers (sequential) ---
    let mut peers = Vec::new();
    for (tid, ct) in &cell.targets {
        if tid == &spec.base {
            continue;
        }
        let load = match target_load(spec, tid, b) {
            Some(l) => l,
            None => continue,
        };
        let Some(peer_target) = spec.targets.get(tid) else {
            continue;
        };
        let stats = match measure(peer_target, ct, load) {
            Ok(s) => s,
            Err(e) => {
                b.tool_error(5, format!("cell `{}` peer `{tid}` aborted: {e}", cell.name));
                continue;
            }
        };
        let unreachable = fully_failed(&stats);
        let peer_honest = load_is_honest(load, &stats) && !unreachable;
        if !peer_honest {
            b.add_finding(load_honesty_finding(
                &cell.name,
                tid,
                load.target_qps,
                &stats,
            ));
        }
        let value = stats.get(metric).unwrap_or(f64::NAN);
        let ratio = if base_value > 0.0 {
            value / base_value
        } else {
            f64::NAN
        };
        let trustworthy = base_honest && peer_honest && ratio.is_finite();

        let key = format!("{}/{}/{}", spec.name, cell.name, tid);
        let baseline = store.get(&key, "ratio").map(|e| e.value);
        let class = classify(&ct.gate, ratio, spec.ratchet, baseline, ct.floor);

        // Gate / fold findings (only when trustworthy — an untrustworthy ratio
        // must never fail or pass a build).
        if trustworthy && !opts.update_baselines {
            fold_class(b, spec, cell, tid, ratio, &class, baseline, opts.strict);
        }
        if opts.update_baselines && trustworthy && ct.gate != "exempt" {
            store.record(&key, "ratio", ratio);
        }

        peers.push(PeerCell {
            target: tid.clone(),
            value,
            ratio,
            gate: ct.gate.clone(),
            verdict: if trustworthy {
                class.label()
            } else {
                "untrustworthy".to_string()
            },
            baseline,
            trustworthy,
        });
    }

    Some(ComparisonRow {
        cell: cell.name.clone(),
        metric: metric.to_string(),
        base_target: spec.base.clone(),
        base_value,
        peers,
    })
}

/// Turn a classification into findings on the worst-wins builder.
fn fold_class(
    b: &mut ReportBuilder,
    spec: &Spec,
    cell: &Cell,
    peer: &str,
    ratio: f64,
    class: &Classification,
    baseline: Option<f64>,
    strict: bool,
) {
    let subject = format!("{}/{}", cell.name, peer);
    let ev = serde_json::json!({ "cell": cell.name, "peer": peer, "ratio": ratio, "baseline": baseline });
    match class {
        Classification::Win { req, ok: false } => {
            b.add_finding(Finding {
                id: finding_id(Kind::PinRegression, &subject),
                severity: Severity::High,
                kind: Kind::PinRegression,
                title: format!("WIN regression: base lost ground vs `{peer}` on `{}`", cell.name),
                detail: format!("ratio {ratio:.2} is below the ratcheted requirement {req:.2}"),
                remediation: format!(
                    "Restore the win, or (if the new number is intended) re-record with `arena run --spec <spec> --update-baselines`."
                ),
                invoke: Invoke::command("arena run --spec <spec>"),
                evidence: ev,
            });
        }
        Classification::Win { ok: true, .. } if baseline.is_none() => {
            b.add_finding(Finding {
                id: finding_id(Kind::PinMissingBaseline, &subject),
                severity: if strict {
                    Severity::High
                } else {
                    Severity::Info
                },
                kind: Kind::PinMissingBaseline,
                title: format!("no baseline yet for WIN cell `{}` vs `{peer}`", cell.name),
                detail: format!("first run measured ratio {ratio:.2}; nothing to ratchet against"),
                remediation: "Record it with `arena run --spec <spec> --update-baselines`."
                    .to_string(),
                invoke: Invoke::command("arena run --spec <spec> --update-baselines"),
                evidence: ev,
            });
        }
        _ => { /* Win-ok, Target, Exempt: comparison-table only, no finding */ }
    }
    let _ = spec;
}

/// The load shape for a target id, or a tool_error if it has none.
fn target_load<'a>(spec: &'a Spec, tid: &str, b: &mut ReportBuilder) -> Option<&'a LoadShape> {
    match spec.targets.get(tid).and_then(|t| t.load.as_ref()) {
        Some(load) => Some(load),
        None => {
            b.tool_error(3, format!("target `{tid}` is missing its load shape"));
            None
        }
    }
}

fn load_honesty_finding(
    cell: &str,
    target: &str,
    target_qps: u32,
    stats: &rig::engine::loadgen::LoadStats,
) -> Finding {
    let subject = format!("{cell}/{target}");
    Finding {
        id: finding_id(Kind::LoadHonesty, &subject),
        severity: Severity::Medium,
        kind: Kind::LoadHonesty,
        title: format!("`{target}` did not sustain the offered load on `{cell}` — ratio untrustworthy"),
        detail: format!(
            "achieved {:.1} qps vs offered {target_qps}; the tail is a queueing artifact, not a fair measurement",
            stats.achieved_qps
        ),
        remediation: "Lower target_qps for this comparison, or give the target more headroom.".to_string(),
        invoke: Invoke::command("arena run --spec <spec>"),
        evidence: serde_json::json!({ "achieved_qps": stats.achieved_qps, "target_qps": target_qps, "error_rate": stats.error_rate }),
    }
}

fn finding(
    kind: Kind,
    severity: Severity,
    subject: &str,
    title: String,
    remediation: &str,
    evidence: serde_json::Value,
) -> Finding {
    Finding {
        id: finding_id(kind, subject),
        severity,
        kind,
        title,
        detail: String::new(),
        remediation: remediation.to_string(),
        invoke: Invoke::command("arena run --spec <spec>"),
        evidence,
    }
}

fn prompt(spec: &Spec, comparison: &[ComparisonRow]) -> String {
    let cells = comparison.len();
    format!(
        "arena compared {} target(s) across {cells} cell(s) for `{}`. Inspect `comparison[]` (per cell: base_value + peers[].ratio/verdict) and `findings[]` (WIN regressions are exit 2).",
        spec.targets.len(),
        spec.name,
    )
}
