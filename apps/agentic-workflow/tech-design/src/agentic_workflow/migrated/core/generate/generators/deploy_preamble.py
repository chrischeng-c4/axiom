"""Canonical Python producer for `apps/agentic-workflow/tech-design/core/generate/generators/deploy_preamble.md`.

Migrated by batch `projection-core-generate-02`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-generate/core-generate-generators-deploy-preamble"
__legacy_projection_path__ = "apps/agentic-workflow/tech-design/core/generate/generators/deploy_preamble.md"
__legacy_projection_digest__ = "sha256:7288338fa751f0bd719cd2cd9e23b66427c18df68970fbd28b201aacb4433819"


def render_markdown() -> Annotated[str, "sha256:7288338fa751f0bd719cd2cd9e23b66427c18df68970fbd28b201aacb4433819"]:
    """Render the preserved generated projection byte-for-byte."""

    return "---\nid: sdd-generate-generators-deploy-preamble\nfill_sections: [overview, source, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections.\"\n---\n\n# DeployGenerator Preamble Source\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nPublic API manifest for `apps/agentic-workflow/src/generate/generators/deploy.rs` generated from AST during Score force-regeneration standardization.\n\n### Symbols\n\n| Name | Target | Kind | Visibility | Line | Signature |\n|------|--------|------|------------|------|-----------|\n| `DeployGenerator` | apps/agentic-workflow/src/generate/generators/deploy.rs | struct | pub | 30 |  |\n| `new` | apps/agentic-workflow/src/generate/generators/deploy.rs | function | pub | 38 | new() -> Self |\n## Source\n<!-- type: source lang: rust -->\n<!-- source-from-target: handwrite-gap generate-generators-deploy-preamble -->\n\n<!-- source-snapshot: path=apps/agentic-workflow/src/generate/generators/deploy.rs -->\n```rust\n\n//! Kubernetes Deployment + Service manifest generator\n//!\n//! Generates Kubernetes manifests from a [`DeploySpec`] (deploy section type):\n//!\n//! | Output file        | Description                                    |\n//! |--------------------|------------------------------------------------|\n//! | `deployment.yaml`  | `apps/v1 Deployment` resource                  |\n//! | `service.yaml`     | `v1 Service` (ClusterIP) resource              |\n//!\n//! The generator implements [`SpecIRGenerator`] and only accepts\n//! [`SpecIR::Deploy`] variants.\n\nuse super::common::{\n    GeneratedFile, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy, SpecIRGenerator,\n};\nuse crate::generate::engine::TemplateEngine;\nuse crate::generate::spec_ir::{DeploySpec, EnvVar, SpecIR};\nuse serde::Serialize;\n\n// ---------------------------------------------------------------------------\n// DeployGenerator\n// ---------------------------------------------------------------------------\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\nchanges:\n  - path: apps/agentic-workflow/src/generate/generators/deploy.rs\n    action: modify\n    section: source\n    impl_mode: codegen\n    replaces:\n      - \"<handwrite-gap:generate-generators-deploy-preamble>\"\n    description: \"Source template owns module docs, imports, and the generator section header.\"\n```\n"
