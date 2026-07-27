"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/fillback/strategy.md`.

Migrated by batch `projection-core-interfaces-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-fillback-strategy"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/fillback/strategy.md"
__legacy_projection_digest__ = "sha256:4834cecd7b41ad7154875e34873dcaf248b0ff88ba59449e932d334889b1024a"


def render_markdown() -> Annotated[str, "sha256:4834cecd7b41ad7154875e34873dcaf248b0ff88ba59449e932d334889b1024a"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-fillback-strategy-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: existing-project-standardization\n    role: primary\n    gap: brownfield-takeover-surface\n    claim: brownfield-takeover-surface\n    coverage: full\n    rationale: \"Fillback interfaces support brownfield takeover by deriving TD/spec coverage from existing project artifacts.\"\n---\n\n# Standardized apps/agentic-workflow/src/fillback/strategy.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/fillback/strategy.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\nNo public AST symbols.\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap standardize:claim-code -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/fillback/strategy.rs -->\n```rust\nuse crate::Result;\nuse async_trait::async_trait;\nuse std::path::Path;\n\n/// Common interface for all import strategies\n///\n/// Each strategy (OpenSpec, Speckit, Code) implements this trait to provide\n/// a consistent way to execute imports and detect if they can handle a given source.\n#[async_trait]\n/// @spec apps/agentic-workflow/tech-design/core/interfaces/fillback/strategy.md#source\npub trait ImportStrategy: Send + Sync {\n    /// Execute the import strategy\n    ///\n    /// # Arguments\n    /// * `source` - Path to the source directory or file to import from\n    /// * `change_id` - The change ID to create/populate in .aw/changes/\n    ///\n    /// # Errors\n    /// Returns an error if the import fails for any reason (parsing, file I/O, etc.)\n    async fn execute(&self, source: &Path, change_id: &str) -> Result<()>;\n\n    /// Check if this strategy can handle the given source\n    ///\n    /// Used for auto-detection when strategy is set to \"auto\".\n    /// Each strategy implements its own detection logic.\n    ///\n    /// # Arguments\n    /// * `source` - Path to check\n    ///\n    /// # Returns\n    /// `true` if this strategy can handle the source, `false` otherwise\n    fn can_handle(&self, source: &Path) -> bool;\n\n    /// Get the name of this strategy for display purposes\n    fn name(&self) -> &'static str;\n}\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/fillback/strategy.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:standardize:claim-code>\"\n    description: |\n      Source template owns the complete fillback import strategy trait module.\n```\n"
