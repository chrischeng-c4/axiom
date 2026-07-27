"""Canonical Python tech design migrated from `apps/agentic-workflow/tech-design/core/logic/python-reference-http-db.md`.

Migrated by batch `semantic-core-logic-01`.
"""

from __future__ import annotations

from typing import Annotated


__aw_artifact_id__ = "artifact:core-logic/core-logic-python-reference-http-db"
__legacy_td_path__ = "apps/agentic-workflow/tech-design/core/logic/python-reference-http-db.md"
__legacy_td_digest__ = "sha256:6670fced2995dae8a3dd5fb2f3365839b01dbc50a5bd0ed592633388a369aae4"


def render_markdown() -> Annotated[str, "sha256:6670fced2995dae8a3dd5fb2f3365839b01dbc50a5bd0ed592633388a369aae4"]:
    """Render the preserved legacy design byte-for-byte."""

    return "---\nid: aw-python-reference-http-db\nsummary: \"Record a DDD FastAPI/Pydantic/SQLAlchemy/SQLite reference without treating SQLite or the example as a framework contract.\"\nfill_sections: [logic, unit-test, e2e-test, changes]\ncapability_refs:\n  - id: td-cb-lifecycle-automation\n    role: primary\n    gap: python-reference-http-db\n    claim: python-reference-http-db\n    coverage: full\n    rationale: \"A second, materially different Python reference is required before deriving TD semantics.\"\n---\n\n# Python Reference: HTTP and Database\n\n## Logic\n\nThe reference maps one create/read product journey into ordinary Python DDD\npaths: `domain/product.py` owns product invariants, `application/products.py`\nowns use cases and a repository protocol, `interface/http.py` owns FastAPI and\nPydantic DTOs, and `infrastructure/` owns SQLAlchemy and SQLite records.\n\nObserved semantic roles are distinct: domain value, boundary DTO, persistence\nrecord, repository adapter, application use case, and public HTTP contract.\nThe boundary forbids caller-supplied persistence IDs; SQLite's unique SKU is\ntranslated into a stable 409 response.\n\nUnsupported assumptions: this example does not establish a general Python\nframework, production database compatibility, migration behavior, async\ndatabase policy, OpenAPI lowering, or multi-target generation. SQLite only\nverifies this local schema and constraint journey.\n\n## Unit Test\n\nThe product unit test calls the application use case through an in-memory\nrepository and imports neither FastAPI nor SQLAlchemy.\n\n## E2E Test\n\nIndependent EC tests exercise the FastAPI boundary with HTTP requests: create\nand read, duplicate-SKU constraint mapping, and attempts to inject a\npersistence ID or malformed SKU.\n\n## Changes\n\n`tests/fixtures/python_spec_http_db/` remains an installable Python project;\nthe Rust fixture invokes its declared test group through `uv run` in a\ntemporary environment.\n"
