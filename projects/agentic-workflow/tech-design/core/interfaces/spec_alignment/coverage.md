---
id: projects-sdd-src-spec-alignment-coverage-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: traceability-closure-gate
    claim: traceability-closure-gate
    coverage: full
    rationale: "Spec alignment interfaces implement TD/source annotation and coverage checks used by the traceability closure gate."
---

# Standardized projects/agentic-workflow/src/spec_alignment/coverage.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/spec_alignment/coverage.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `analyze` | projects/agentic-workflow/src/spec_alignment/coverage.rs | function | pub | 29 | analyze(     spec_dir: &Path,     source_dirs: &[&Path],     orphan_requirements: Vec<OrphanRequirementEntry>,     daemon_ready: bool, ) -> CoverageReport |
| `analyze_with_precomputed` | projects/agentic-workflow/src/spec_alignment/coverage.rs | function | pub | 64 | analyze_with_precomputed(     spec_dir: &Path,     source_dirs: &[&Path],     orphan_requirements: Vec<OrphanRequirementEntry>,     precomputed_requirements: HashMap<String, HashMap<String, Option<String>>>,     schema_mismatches: Vec<SchemaStructMismatchEntry>,     daemon_ready: bool, ) -> CoverageReport |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: handwrite-gap standardize:claim-code -->

<!-- source-snapshot: path=projects/agentic-workflow/src/spec_alignment/coverage.rs -->
```rust
//! Coverage analysis module.
//!
//! Cross-references `@spec` annotations with spec requirement IDs,
//! queries daemon symbol index for unspecced public functions.
//! Returns `CoverageReport`.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::annotations;
use super::models::{
    CoverageEntry, CoverageReport, OrphanRequirementEntry, SchemaStructMismatchEntry,
    SpecAnnotation, UnspeccedFunction,
};
use super::requirement_coverage;

/// Analyze coverage across spec files and source directories.
///
/// - Collects all spec `.md` files and extracts R{N} requirement IDs
/// - Scans source directories for `@spec` annotations
/// - Cross-references to determine covered/uncovered requirements
/// - Identifies stale annotations (pointing to non-existent specs or IDs)
/// - Optionally queries daemon for unspecced public functions
///
/// `daemon_ready`: if false, skips unspecced function detection.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/coverage.md#source
pub fn analyze(
    spec_dir: &Path,
    source_dirs: &[&Path],
    orphan_requirements: Vec<OrphanRequirementEntry>,
    daemon_ready: bool,
) -> CoverageReport {
    // Collect all spec files and their requirement IDs (reads files)
    let spec_files = collect_spec_files(spec_dir);
    let mut all_requirements: HashMap<String, HashMap<String, Option<String>>> = HashMap::new();

    for spec_path in &spec_files {
        let path_str = spec_path.display().to_string();
        let reqs = requirement_coverage::extract_requirement_ids(&path_str);
        if !reqs.is_empty() {
            all_requirements.insert(path_str, reqs);
        }
    }

    let spec_path_set: HashSet<String> =
        spec_files.iter().map(|p| p.display().to_string()).collect();

    analyze_inner(
        source_dirs,
        orphan_requirements,
        all_requirements,
        spec_path_set,
        Vec::new(),
        daemon_ready,
    )
}

/// Analyze coverage using pre-computed requirement maps (avoids redundant file I/O).
///
/// Use this when the caller has already parsed spec files and extracted requirements.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/coverage.md#source
pub fn analyze_with_precomputed(
    spec_dir: &Path,
    source_dirs: &[&Path],
    orphan_requirements: Vec<OrphanRequirementEntry>,
    precomputed_requirements: HashMap<String, HashMap<String, Option<String>>>,
    schema_mismatches: Vec<SchemaStructMismatchEntry>,
    daemon_ready: bool,
) -> CoverageReport {
    // Build spec path set from directory scan (cheap — only stat calls, no reads)
    let spec_files = collect_spec_files(spec_dir);
    let mut spec_path_set: HashSet<String> =
        spec_files.iter().map(|p| p.display().to_string()).collect();

    // Also include any paths from precomputed_requirements that aren't in the dir scan
    for path in precomputed_requirements.keys() {
        spec_path_set.insert(path.clone());
    }

    analyze_inner(
        source_dirs,
        orphan_requirements,
        precomputed_requirements,
        spec_path_set,
        schema_mismatches,
        daemon_ready,
    )
}

/// Core analysis logic shared by both `analyze` and `analyze_with_precomputed`.
fn analyze_inner(
    source_dirs: &[&Path],
    orphan_requirements: Vec<OrphanRequirementEntry>,
    all_requirements: HashMap<String, HashMap<String, Option<String>>>,
    spec_path_set: HashSet<String>,
    schema_mismatches: Vec<SchemaStructMismatchEntry>,
    daemon_ready: bool,
) -> CoverageReport {
    // Scan source directories for @spec annotations
    let all_annotations = annotations::scan_directories(source_dirs);

    // Build valid (spec_path, req_id) pairs for O(1) lookups
    let mut valid_pairs: HashSet<(String, String)> = HashSet::new();
    // Also map annotation spec_path -> resolved full spec_path for suffix matching
    let mut suffix_map: HashMap<String, String> = HashMap::new();
    for spec_path in &spec_path_set {
        if let Some(reqs) = all_requirements.get(spec_path) {
            for req_id in reqs.keys() {
                valid_pairs.insert((spec_path.clone(), req_id.clone()));
            }
        }
    }

    // Identify stale annotations using valid_pairs for O(1) lookup
    let mut stale_annotations = Vec::new();
    for ann in &all_annotations {
        // Resolve the annotation spec_path to a full path (try as-is, then suffix match)
        let resolved_path = if spec_path_set.contains(&ann.spec_path) {
            Some(ann.spec_path.clone())
        } else {
            // Cache suffix lookups
            if let Some(cached) = suffix_map.get(&ann.spec_path) {
                Some(cached.clone())
            } else {
                let found = spec_path_set
                    .iter()
                    .find(|sp| sp.ends_with(&ann.spec_path))
                    .cloned();
                if let Some(ref resolved) = found {
                    suffix_map.insert(ann.spec_path.clone(), resolved.clone());
                }
                found
            }
        };

        match resolved_path {
            None => {
                stale_annotations.push(ann.clone());
            }
            Some(ref full_path) => {
                if !valid_pairs.contains(&(full_path.clone(), ann.requirement_id.clone())) {
                    stale_annotations.push(ann.clone());
                }
            }
        }
    }

    // Build coverage entries
    let mut covered = Vec::new();
    let mut uncovered_requirements = Vec::new();

    for (spec_path, reqs) in &all_requirements {
        for (req_id, _description) in reqs {
            // Find annotations for this requirement (match by suffix for relative paths)
            let matching_annotations: Vec<SpecAnnotation> = all_annotations
                .iter()
                .filter(|ann| {
                    let ann_matches = ann.spec_path == *spec_path
                        || spec_path.ends_with(&ann.spec_path)
                        || suffix_map
                            .get(&ann.spec_path)
                            .map(|resolved| resolved == spec_path)
                            .unwrap_or(false);
                    ann_matches && ann.requirement_id == *req_id
                })
                .cloned()
                .collect();

            if matching_annotations.is_empty() {
                uncovered_requirements.push(CoverageEntry {
                    requirement_id: req_id.clone(),
                    spec_path: spec_path.clone(),
                    status: "uncovered".to_string(),
                    annotations: Vec::new(),
                });
            } else {
                covered.push(CoverageEntry {
                    requirement_id: req_id.clone(),
                    spec_path: spec_path.clone(),
                    status: "covered".to_string(),
                    annotations: matching_annotations,
                });
            }
        }
    }

    // Detect unspecced public functions (if daemon is ready)
    let unspecced_functions = if daemon_ready {
        detect_unspecced_functions(source_dirs, &all_annotations)
    } else {
        Vec::new()
    };

    // Compute coverage ratio
    let total = covered.len() + uncovered_requirements.len();
    let coverage_ratio = if total == 0 {
        1.0
    } else {
        covered.len() as f64 / total as f64
    };

    CoverageReport {
        covered,
        uncovered_requirements,
        unspecced_functions,
        stale_annotations,
        orphan_requirements,
        schema_struct_mismatches: schema_mismatches,
        coverage_ratio,
    }
}

/// Detect public functions without `@spec` annotations.
///
/// Currently a stub — full implementation requires daemon symbol index integration.
/// When the daemon is ready, this queries `DaemonClient::symbols(file)` for each
/// source file and checks if public functions have annotations nearby.
fn detect_unspecced_functions(
    _source_dirs: &[&Path],
    _annotations: &[SpecAnnotation],
) -> Vec<UnspeccedFunction> {
    // Phase 2 stub: daemon integration for symbol index queries.
    // When daemon is available, this will:
    // 1. For each source file, query daemon.symbols(file)
    // 2. Filter to public symbols (fn_item, impl_method, trait_method)
    // 3. Check if each symbol has a nearby @spec annotation
    // 4. Report unspecced functions
    Vec::new()
}

/// Collect all `.md` spec files recursively from a directory.
fn collect_spec_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if dir.is_file() {
        if dir.extension().map(|e| e == "md").unwrap_or(false) {
            files.push(dir.to_path_buf());
        }
        return files;
    }
    collect_spec_files_recursive(dir, &mut files);
    files.sort();
    files
}

fn collect_spec_files_recursive(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_spec_files_recursive(&path, files);
        } else if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "md" {
                    files.push(path);
                }
            }
        }
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/spec_alignment/coverage.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-gap:standardize:claim-code>"
    description: |
      Source template owns the complete spec-alignment coverage module.
```
