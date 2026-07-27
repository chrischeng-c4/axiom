"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/semantic/agentic-workflow-gen-rust.md`.

Migrated by batch `semantic-semantic-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:semantic/semantic-agentic-workflow-gen-rust"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/semantic/agentic-workflow-gen-rust.md"
__legacy_td_digest__ = "sha256:95baf2db7e494aeae8f6cca411bb94bb00edb69fdc824725ea271576c66bc284"


def render_markdown() -> Annotated[str, "sha256:95baf2db7e494aeae8f6cca411bb94bb00edb69fdc824725ea271576c66bc284"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: semantic-agentic-workflow-gen-rust\nsummary: Semantic coverage for \"apps/agentic-workflow/src/gen/rust\"\nfill_sections: [schema, tests, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: cb-lifecycle-dispatch\n    claim: cb-lifecycle-dispatch\n    coverage: full\n    rationale: \"This semantic TD covers TD/CB generation, parsing, validation, and code artifact lifecycle source behavior.\"\n---\n\n# Semantic TD: agentic-workflow/gen/rust\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\nsemantic_domain:\n  key: \"agentic-workflow/gen/rust\"\n  source_group: \"apps/agentic-workflow/src/gen/rust\"\n  coverage_kind: semantic\n  evidence:\n    source_units:\n      - path: \"apps/agentic-workflow/src/gen/rust/rpc_api.rs\"\n        language: \"rust\"\n        ownership_state: \"codegen\"\n        generator_primitives: [\"data_model\"]\n        symbols:\n          - name: \"RpcApiGenOutput\"\n            kind: \"struct\"\n            public: true\n        source_evidence_node:\n          layer: \"backend\"\n          ecosystem: \"rust\"\n          role: \"source\"\n          section_type: \"schema\"\n          domain: \"apps/agentic-workflow/src/gen/rust\"\n      - path: \"apps/agentic-workflow/src/gen/rust/db_model.rs\"\n        language: \"rust\"\n        ownership_state: \"codegen\"\n        generator_primitives: [\"data_model\"]\n        symbols:\n          - name: \"DbModelGenOutput\"\n            kind: \"struct\"\n            public: true\n        source_evidence_node:\n          layer: \"backend\"\n          ecosystem: \"rust\"\n          role: \"source\"\n          section_type: \"schema\"\n          domain: \"apps/agentic-workflow/src/gen/rust\"\n```\n\n## Tests\n<!-- type: tests lang: yaml -->\n\n```yaml\ntests:\n  coverage_kind: semantic\n  strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives\n  evidence:\n    source_tests: []\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\ncoverage_kind: semantic\nchanges:\n  - path: \"apps/agentic-workflow/src/gen/rust/rpc_api.rs\"\n    action: modify\n    section: schema\n    description: |\n      Existing source behavior is covered by this feature/domain semantic TD.\n    impl_mode: hand-written\n  - path: \"apps/agentic-workflow/src/gen/rust/db_model.rs\"\n    action: modify\n    section: schema\n    description: |\n      Existing source behavior is covered by this feature/domain semantic TD.\n    impl_mode: hand-written\n  - action: annotate\n    section: unit-test\n    impl_mode: hand-written\n    description: \"Traceability metadata edge for the unit-test section.\"\n\n```\n"
