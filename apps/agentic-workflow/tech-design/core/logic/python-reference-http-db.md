---
id: aw-python-reference-http-db
summary: "Record a DDD FastAPI/Pydantic/SQLAlchemy/SQLite reference without treating SQLite or the example as a framework contract."
fill_sections: [logic, unit-test, e2e-test, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: python-reference-http-db
    claim: python-reference-http-db
    coverage: full
    rationale: "A second, materially different Python reference is required before deriving TD semantics."
---

# Python Reference: HTTP and Database

## Logic

The reference maps one create/read product journey into ordinary Python DDD
paths: `domain/product.py` owns product invariants, `application/products.py`
owns use cases and a repository protocol, `interface/http.py` owns FastAPI and
Pydantic DTOs, and `infrastructure/` owns SQLAlchemy and SQLite records.

Observed semantic roles are distinct: domain value, boundary DTO, persistence
record, repository adapter, application use case, and public HTTP contract.
The boundary forbids caller-supplied persistence IDs; SQLite's unique SKU is
translated into a stable 409 response.

Unsupported assumptions: this example does not establish a general Python
framework, production database compatibility, migration behavior, async
database policy, OpenAPI lowering, or multi-target generation. SQLite only
verifies this local schema and constraint journey.

## Unit Test

The product unit test calls the application use case through an in-memory
repository and imports neither FastAPI nor SQLAlchemy.

## E2E Test

Independent EC tests exercise the FastAPI boundary with HTTP requests: create
and read, duplicate-SKU constraint mapping, and attempts to inject a
persistence ID or malformed SKU.

## Changes

`tests/fixtures/python_spec_http_db/` remains an installable Python project;
the Rust fixture invokes its declared test group through `uv run` in a
temporary environment.
