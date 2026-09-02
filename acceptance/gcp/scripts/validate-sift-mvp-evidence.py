#!/usr/bin/env python3
"""Validate Sift MVP evidence with the repository JSON Schema."""

from __future__ import annotations

import argparse
import copy
import json
import sys
from pathlib import Path

try:
    import jsonschema
except ImportError as error:  # pragma: no cover - exercised by the shell preflight
    raise SystemExit(
        "python package 'jsonschema' is required for Sift MVP evidence validation"
    ) from error


def load_json(path: Path) -> object:
    try:
        with path.open("r", encoding="utf-8") as handle:
            return json.load(handle)
    except (OSError, json.JSONDecodeError) as error:
        raise SystemExit(f"cannot read JSON evidence {path}: {error}") from error


def normalize_verification(document: object) -> object:
    if not isinstance(document, dict):
        raise SystemExit("verification evidence must be a JSON object")
    sift = document.get("acceptance", {}).get("sift")
    if not isinstance(sift, dict):
        raise SystemExit("verification evidence must contain acceptance.sift")
    expected = (
        document.get("schema"),
        sift.get("schema"),
        sift.get("status"),
        sift.get("cleanup_evidence"),
    )
    if expected != (
        "axiom.gcp.operator.verification.v1",
        "axiom.gcp.sift.mvp.verification.v1",
        "verification-passed",
        None,
    ):
        raise SystemExit("Sift verification evidence is not in the pre-cleanup state")

    normalized = copy.deepcopy(document)
    normalized["schema"] = "axiom.gcp.operator.acceptance.v1"
    normalized_sift = normalized["acceptance"]["sift"]
    normalized_sift["schema"] = "axiom.gcp.sift.mvp.acceptance.v1"
    normalized_sift["status"] = "passed"
    normalized_sift["cleanup_evidence"] = {
        "schema": "axiom.gcp.operator.cleanup.v1",
        "project_id": normalized["project_id"],
        "region": normalized["region"],
        "gke_zone": normalized["gke_zone"],
        "run_id": normalized["run_id"],
        "verified_at": "1970-01-01T00:00:00Z",
        "status": "clean",
        "preserved": {"artifact_registry": True, "preexisting_apis": True},
    }
    return normalized


def validate(schema_path: Path, document_path: Path, mode: str) -> None:
    schema = load_json(schema_path)
    validator_class = jsonschema.validators.validator_for(schema)
    validator_class.check_schema(schema)
    document = load_json(document_path)
    if mode == "verification":
        document = normalize_verification(document)

    errors = sorted(
        validator_class(schema).iter_errors(document),
        key=lambda error: tuple(str(part) for part in error.absolute_path),
    )
    if not errors:
        return
    for error in errors[:20]:
        path = ".".join(str(part) for part in error.absolute_path) or "<root>"
        print(f"{document_path}:{path}: {error.message}", file=sys.stderr)
    if len(errors) > 20:
        print(f"{len(errors) - 20} more schema errors omitted", file=sys.stderr)
    raise SystemExit(1)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--schema", type=Path, required=True)
    parser.add_argument("--document", type=Path)
    parser.add_argument(
        "--mode", choices=("verification", "acceptance"), default="acceptance"
    )
    parser.add_argument("--schema-only", action="store_true")
    args = parser.parse_args()

    schema = load_json(args.schema)
    validator_class = jsonschema.validators.validator_for(schema)
    validator_class.check_schema(schema)
    if args.schema_only:
        return
    if args.document is None:
        parser.error("--document is required unless --schema-only is used")
    validate(args.schema, args.document, args.mode)


if __name__ == "__main__":
    main()
