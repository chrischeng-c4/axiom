"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/semantic/agentic-workflow-generate-generators-tests.md`.

Migrated by batch `semantic-semantic-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:semantic/semantic-agentic-workflow-generate-generators-tests"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/semantic/agentic-workflow-generate-generators-tests.md"
__legacy_td_digest__ = "sha256:6f6e72e7a6a61b5cead62f94f2bd34c4ceb9dc526b987de9ae4d1f16c2283025"


def render_markdown() -> Annotated[str, "sha256:6f6e72e7a6a61b5cead62f94f2bd34c4ceb9dc526b987de9ae4d1f16c2283025"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: semantic-agentic-workflow-generate-generators-tests\nsummary: Semantic coverage for \"apps/agentic-workflow/src/generate/generators/tests\"\nfill_sections: [schema, tests, changes]\ncapability_refs:\n  - id: existing-project-standardization\n    role: primary\n    gap: cb-and-cold-verification-gates\n    claim: cb-and-cold-verification-gates\n    coverage: full\n    rationale: \"This semantic TD covers verification source behavior used by existing-project standardization gates.\"\n---\n\n# Semantic TD: agentic-workflow/generate/generators/tests\n\n## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\nsemantic_domain:\n  key: \"agentic-workflow/generate/generators/tests\"\n  source_group: \"apps/agentic-workflow/src/generate/generators/tests\"\n  coverage_kind: semantic\n  evidence:\n    source_units:\n      - path: \"apps/agentic-workflow/src/generate/generators/tests/cli_subcommand_test.rs\"\n        language: \"rust\"\n        ownership_state: \"codegen\"\n        generator_primitives: [\"test_case\"]\n        source_evidence_node:\n          layer: \"backend\"\n          ecosystem: \"rust\"\n          role: \"test\"\n          section_type: \"tests\"\n          domain: \"apps/agentic-workflow/src/generate/generators/tests\"\n      - path: \"apps/agentic-workflow/src/generate/generators/tests/module_facade_test.rs\"\n        language: \"rust\"\n        ownership_state: \"codegen\"\n        generator_primitives: [\"test_case\"]\n        source_evidence_node:\n          layer: \"backend\"\n          ecosystem: \"rust\"\n          role: \"test\"\n          section_type: \"tests\"\n          domain: \"apps/agentic-workflow/src/generate/generators/tests\"\n```\n\n## Tests\n<!-- type: tests lang: yaml -->\n\n```yaml\ntests:\n  coverage_kind: semantic\n  strategy: preserve observed source behavior while semantic coverage is promoted toward generator primitives\n  evidence:\n    source_tests:\n      - path: \"apps/agentic-workflow/src/generate/generators/tests/cli_subcommand_test.rs\"\n      - path: \"apps/agentic-workflow/src/generate/generators/tests/module_facade_test.rs\"\n```\n\n## Changes\n<!-- type: changes lang: yaml -->\n\n```yaml\ncoverage_kind: semantic\nchanges:\n  - path: \"apps/agentic-workflow/src/generate/generators/tests/cli_subcommand_test.rs\"\n    action: modify\n    section: schema\n    description: |\n      Existing source behavior is covered by this feature/domain semantic TD.\n    impl_mode: hand-written\n  - path: \"apps/agentic-workflow/src/generate/generators/tests/module_facade_test.rs\"\n    action: modify\n    section: schema\n    description: |\n      Existing source behavior is covered by this feature/domain semantic TD.\n    impl_mode: hand-written\n  - action: annotate\n    section: unit-test\n    impl_mode: hand-written\n    description: \"Traceability metadata edge for the unit-test section.\"\n\n```\n"
