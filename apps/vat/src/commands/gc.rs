// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-commands-gc-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! `vat gc` — report and prune retained vat workspaces.

use std::collections::BTreeSet;
use std::path::Path;
use std::process::{Command, ExitCode};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use walkdir::WalkDir;

use crate::paths;
use crate::state::Status;
use crate::store;

#[derive(Debug, Clone)]
pub struct Args {
    pub execute: bool,
    pub keep_last: usize,
    pub include_failed: bool,
    pub include_snapshots: bool,
    pub older_than_days: Option<i64>,
    pub measure: bool,
    pub apparent: bool,
    pub json: bool,
}

#[derive(Debug, Serialize)]
struct GcReport {
    store_root: String,
    dry_run: bool,
    policy: GcPolicy,
    total_count: usize,
    candidate_count: usize,
    deleted_count: usize,
    skipped_count: usize,
    total_disk_size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_apparent_size_bytes: Option<u64>,
    reclaimable_disk_size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reclaimable_apparent_size_bytes: Option<u64>,
    entries: Vec<GcEntry>,
}

#[derive(Debug, Serialize)]
struct GcPolicy {
    keep_last: usize,
    include_failed: bool,
    include_snapshots: bool,
    older_than_days: Option<i64>,
    measure: bool,
    apparent: bool,
}

#[derive(Debug, Serialize)]
struct GcEntry {
    id: String,
    status: String,
    updated_at: DateTime<Utc>,
    age_days: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    disk_size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    apparent_size_bytes: Option<u64>,
    candidate: bool,
    deleted: bool,
    reason: String,
}

pub fn exec(args: Args) -> Result<ExitCode> {
    let mut report = build_report(&args)?;
    if args.execute {
        for entry in &mut report.entries {
            if !entry.candidate {
                continue;
            }
            store::remove(&entry.id).with_context(|| format!("gc remove {}", entry.id))?;
            entry.deleted = true;
            report.deleted_count += 1;
        }
    }

    if args.json {
        crate::commands::print_json(&report, false)?;
    } else {
        print_human(&report);
    }
    Ok(ExitCode::SUCCESS)
}

fn build_report(args: &Args) -> Result<GcReport> {
    let store_root = paths::vats_dir()?;
    let mut vats = store::list()?;
    vats.sort_by(|a, b| b.meta.updated_at.cmp(&a.meta.updated_at));
    let protected = vats
        .iter()
        .take(args.keep_last)
        .map(|vat| vat.meta.id.clone())
        .collect::<BTreeSet<_>>();

    let now = Utc::now();
    let mut entries = Vec::new();
    let measure_disk = args.measure || args.apparent;
    let mut total_disk_size_bytes = measure_disk.then_some(0_u64);
    let mut total_apparent_size_bytes = args.apparent.then_some(0_u64);
    let mut reclaimable_disk_size_bytes = measure_disk.then_some(0_u64);
    let mut reclaimable_apparent_size_bytes = args.apparent.then_some(0_u64);

    for vat in &vats {
        let disk_size_bytes = measure_disk.then(|| disk_size(&vat.dir)).flatten();
        let apparent_size_bytes = args.apparent.then(|| apparent_size(&vat.dir));
        total_disk_size_bytes = add_optional(total_disk_size_bytes, disk_size_bytes);
        total_apparent_size_bytes = add_optional(total_apparent_size_bytes, apparent_size_bytes);
        let age_days = (now - vat.meta.updated_at).num_days().max(0);
        let (candidate, reason) =
            candidate_reason(args, &protected, &vat.meta.status, &vat.meta.id, age_days);
        if candidate {
            reclaimable_disk_size_bytes =
                add_optional(reclaimable_disk_size_bytes, disk_size_bytes);
            reclaimable_apparent_size_bytes =
                add_optional(reclaimable_apparent_size_bytes, apparent_size_bytes);
        }
        entries.push(GcEntry {
            id: vat.meta.id.clone(),
            status: status_label(&vat.meta.status),
            updated_at: vat.meta.updated_at,
            age_days,
            apparent_size_bytes,
            disk_size_bytes,
            candidate,
            deleted: false,
            reason,
        });
    }

    let candidate_count = entries.iter().filter(|entry| entry.candidate).count();
    let skipped_count = entries.len().saturating_sub(candidate_count);
    Ok(GcReport {
        store_root: store_root.to_string_lossy().into_owned(),
        dry_run: !args.execute,
        policy: GcPolicy {
            keep_last: args.keep_last,
            include_failed: args.include_failed,
            include_snapshots: args.include_snapshots,
            older_than_days: args.older_than_days,
            measure: args.measure,
            apparent: args.apparent,
        },
        total_count: entries.len(),
        candidate_count,
        deleted_count: 0,
        skipped_count,
        total_apparent_size_bytes,
        total_disk_size_bytes,
        reclaimable_apparent_size_bytes,
        reclaimable_disk_size_bytes,
        entries,
    })
}

fn candidate_reason(
    args: &Args,
    protected: &BTreeSet<String>,
    status: &Status,
    id: &str,
    age_days: i64,
) -> (bool, String) {
    if matches!(status, Status::Running) {
        return (false, "running".to_string());
    }
    if protected.contains(id) {
        return (false, "kept_newest".to_string());
    }
    if matches!(status, Status::Snapshot) && !args.include_snapshots {
        return (false, "snapshot_retained".to_string());
    }
    if matches!(status, Status::Exited { code } if *code != 0) && !args.include_failed {
        return (false, "failed_retained".to_string());
    }
    if let Some(days) = args.older_than_days {
        if age_days < days {
            return (false, format!("newer_than_{days}d"));
        }
    }
    (true, "candidate".to_string())
}

fn apparent_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|entry| entry.metadata().ok())
        .filter(|meta| meta.is_file())
        .fold(0_u64, |sum, meta| sum.saturating_add(meta.len()))
}

fn disk_size(path: &Path) -> Option<u64> {
    let output = Command::new("du").arg("-sk").arg(path).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let kib = stdout.split_whitespace().next()?.parse::<u64>().ok()?;
    Some(kib.saturating_mul(1024))
}

fn add_optional(sum: Option<u64>, value: Option<u64>) -> Option<u64> {
    Some(sum?.saturating_add(value?))
}

fn status_label(status: &Status) -> String {
    match status {
        Status::Created => "created".to_string(),
        Status::Running => "running".to_string(),
        Status::Exited { code } => format!("exited:{code}"),
        Status::Snapshot => "snapshot".to_string(),
    }
}

fn print_human(report: &GcReport) {
    println!(
        "vat gc: {} candidates, {} skipped, reclaimable {} disk / {} apparent ({})",
        report.candidate_count,
        report.skipped_count,
        format_optional_bytes(report.reclaimable_disk_size_bytes),
        format_optional_bytes(report.reclaimable_apparent_size_bytes),
        if report.dry_run {
            "dry-run; add --execute to delete"
        } else {
            "deleted"
        }
    );
    println!("store {}", report.store_root);
    println!(
        "{:<14} {:<10} {:>9} {:>12} {:>12} {:<16} ACTION",
        "ID", "STATUS", "AGE_DAYS", "DISK", "APPARENT", "REASON"
    );
    for entry in &report.entries {
        println!(
            "{:<14} {:<10} {:>9} {:>12} {:>12} {:<16} {}",
            entry.id,
            entry.status,
            entry.age_days,
            format_optional_bytes(entry.disk_size_bytes),
            format_optional_bytes(entry.apparent_size_bytes),
            entry.reason,
            if entry.deleted {
                "deleted"
            } else if entry.candidate {
                "would-delete"
            } else {
                "keep"
            }
        );
    }
}

fn format_optional_bytes(bytes: Option<u64>) -> String {
    bytes.map(format_bytes).unwrap_or_else(|| "?".to_string())
}

fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;
    const TIB: f64 = GIB * 1024.0;
    let value = bytes as f64;
    if value >= TIB {
        format!("{:.1}T", value / TIB)
    } else if value >= GIB {
        format!("{:.1}G", value / GIB)
    } else if value >= MIB {
        format!("{:.1}M", value / MIB)
    } else if value >= KIB {
        format!("{:.1}K", value / KIB)
    } else {
        format!("{bytes}B")
    }
}
// CODEGEN-END
