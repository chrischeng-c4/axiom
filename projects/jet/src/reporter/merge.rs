// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-reporter.md#schema
// CODEGEN-BEGIN
//! Shard merger — stitches N per-shard report directories into a single
//! unified HTML report.
//!
//! The merge algorithm:
//! 1. Read each input directory's `results.ndjson` sidecar (preferred) or fall
//!    back to an empty list.
//! 2. Deduplicate rows by `test_id` (last-writer wins — shards are assumed
//!    disjoint, but dedup handles overlapping coverage in pathological cases).
//! 3. Re-sort by `test_id` for deterministic output.
//! 4. Re-render a unified `index.html` into `output`.
//!
// @spec enhancement-html-reporter-for-native-test-runner-spec#R7

use crate::reporter::html::{render_from_rows, TestRow, REPORT_CSS, REPORT_JS};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Merge N shard report directories into a single unified report at `output`.
///
/// Each input directory should contain a `results.ndjson` sidecar produced by
/// `finalize_with_sidecar`. Missing sidecars are skipped gracefully.
///
/// Deduplication: rows with the same `test_id` are deduplicated (last input
/// directory wins — assumes disjoint shards).
///
/// `output` is created if it does not exist.
// @spec enhancement-html-reporter-for-native-test-runner-spec#R7
pub fn merge_reports(inputs: &[PathBuf], output: &Path) -> Result<()> {
    let shard_count = inputs.len() as u32;
    let mut by_id: HashMap<String, TestRow> = HashMap::new();

    for input_dir in inputs {
        let rows = crate::reporter::html::read_rows_from_dir(input_dir);
        for row in rows {
            by_id.insert(row.test_id.clone(), row);
        }
    }

    // Convert to sorted vec.
    let mut merged: Vec<TestRow> = by_id.into_values().collect();
    merged.sort_by(|a, b| a.test_id.cmp(&b.test_id));

    // If there are multiple shards, annotate the report with a synthetic shard
    // summary ("N shards merged").  We use (1, shard_count) as a convention
    // meaning "1 merged report from N shards".
    let shard_meta = if shard_count > 1 {
        Some((1u32, shard_count))
    } else {
        None
    };

    let html = render_from_rows(&merged, shard_meta);

    std::fs::create_dir_all(output)
        .with_context(|| format!("Failed to create output dir: {}", output.display()))?;

    std::fs::write(output.join("index.html"), &html)
        .context("Failed to write merged index.html")?;
    std::fs::write(output.join("report.js"), REPORT_JS).context("Failed to write report.js")?;
    std::fs::write(output.join("report.css"), REPORT_CSS).context("Failed to write report.css")?;

    // Write NDJSON sidecar for the merged report so it can itself be used as
    // input in a downstream merge (idempotent chaining).
    let ndjson: String = merged
        .iter()
        .map(|row| crate::reporter::html::row_to_ndjson_line(row))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(output.join("results.ndjson"), ndjson)
        .context("Failed to write merged results.ndjson")?;

    println!(
        "Merged report ({} shard(s)): {}/index.html",
        shard_count,
        output.display()
    );
    Ok(())
}
// CODEGEN-END
