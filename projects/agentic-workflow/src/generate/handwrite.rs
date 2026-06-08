// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/handwrite-marker.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Aggregated output of score sdd coverage over a workspace scan (R4, R5).
/// @spec projects/agentic-workflow/tech-design/core/generate/handwrite-marker.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoverageReport {
    /// All parsed HANDWRITE markers from the scanned workspace.
    pub markers: Vec<HandwriteMarker>,
    /// Count of markers whose tracker equals pending-tracker (R10).
    pub pending_count: usize,
    /// Total marker count across all scanned files.
    pub total_count: usize,
    /// Markers grouped by gap taxonomy code for prioritisation.
    #[serde(default)]
    pub by_gap: Option<std::collections::HashMap<String, Vec<HandwriteMarker>>>,
}

/// Optional fields on a spec changes entry that control HANDWRITE scaffold emission during gen-code (R1, R3). Corresponds to new optional fields added to the ChangeEntry type in spec_ir/types.rs. All three fields are optional; scaffold_handwrite derives defaults for any that are absent.
/// @spec projects/agentic-workflow/tech-design/core/generate/handwrite-marker.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HandwriteEntry {
    /// Codegen gap taxonomy code. If absent, scaffold_handwrite derives the value from the entry's section_type (e.g. section_type=logic → gap="missing-generator:logic"). This derivation is defined in R2/R4.
    #[serde(default)]
    pub gap: Option<String>,
    /// Issue slug tracking this gap. If absent, scaffold_handwrite fills in the sentinel value "pending-tracker" (R3). Markers carrying this sentinel are highlighted by score sdd coverage (R10).
    #[serde(default)]
    pub tracker: Option<String>,
    /// Non-empty description of the codegen gap. Used as the reason attribute in the emitted HANDWRITE marker. If absent, scaffold_handwrite synthesizes a reason from the section_type and target_file path. The parser still rejects a persisted empty-string reason (R9).
    #[serde(default)]
    pub reason: Option<String>,
}

/// Structured record representing one parsed HANDWRITE marker pair. Produced by parse_handwrite_markers() from Rust source files (R2).
/// @spec projects/agentic-workflow/tech-design/core/generate/handwrite-marker.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HandwriteMarker {
    /// Absolute or workspace-relative path to the source file.
    pub file_path: String,
    /// 1-based line number of the begin marker comment.
    pub line_start: usize,
    /// 1-based line number of the end marker comment.
    pub line_end: usize,
    /// Taxonomy code identifying the codegen gap (e.g. logic-flowchart).
    pub gap: String,
    /// Issue slug tracking this gap, or the sentinel value pending-tracker if no issue has been assigned yet (R3, R10).
    pub tracker: String,
    /// Non-empty human-readable description of why this region is hand-written (R9).
    pub reason: String,
}

/// Structured error returned when the parser encounters a malformed or unmatched HANDWRITE marker (R7, R9).
/// @spec projects/agentic-workflow/tech-design/core/generate/handwrite-marker.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandwriteParseError {
    /// Path to the file containing the malformed marker.
    pub file_path: String,
    /// 1-based line number where the error was detected.
    pub line: usize,
    /// Human-readable parse failure description.
    pub message: String,
}
// CODEGEN-END
