"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/interfaces/spec_ir/mod.md`.

Migrated by batch `projection-core-interfaces-03`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-interfaces/core-interfaces-spec-ir-mod"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/interfaces/spec_ir/mod.md"
__legacy_projection_digest__ = "sha256:207cff7ba21c6f575e993fe151fe20babf18e842fd41472d9804b5449821b275"


def render_markdown() -> Annotated[str, "sha256:207cff7ba21c6f575e993fe151fe20babf18e842fd41472d9804b5449821b275"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: projects-sdd-src-spec-ir-mod-rs\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Spec IR interfaces drive code artifact generation from TD/spec manifests in the TD/CB lifecycle.\"\n---\n\n# Standardized apps/agentic-workflow/src/spec_ir/mod.rs\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/spec_ir/mod.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `codegen` | apps/agentic-workflow/src/spec_ir/mod.rs | module | pub | 20 |  |\n| `generator` | apps/agentic-workflow/src/spec_ir/mod.rs | module | pub | 21 |  |\n| `migration` | apps/agentic-workflow/src/spec_ir/mod.rs | module | pub | 22 |  |\n| `orchestrator` | apps/agentic-workflow/src/spec_ir/mod.rs | module | pub | 23 |  |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: strip-handwrite -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/spec_ir/mod.rs -->\n````rust\n//! SpecIR YAML Manifest types (k8s/Kustomize style)\n//!\n//! Language-agnostic intermediate representation for the spec-to-code pipeline.\n//! SDD writes these YAML files, Lens reads them for codegen.\n//!\n//! ## Manifest format\n//!\n//! ```yaml\n//! apiVersion: cclab.dev/v1\n//! kind: Api\n//! metadata:\n//!   name: user-service\n//!   change_id: genesis-372\n//! spec:\n//!   # kind-specific payload\n//! ```\n\npub mod codegen;\npub mod generator;\npub mod migration;\npub mod orchestrator;\nmod types;\n\npub use types::*;\n````\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/spec_ir/mod.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    description: |\n      Source template owns the complete SpecIR module facade.\n```\n"
