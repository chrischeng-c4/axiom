#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
from pathlib import Path
from typing import Any


AUDIT_SCHEMA = "axiom.lumen.ec.redaction-audit.v1"
CANARY_BYTES = 32
MIN_CANARY_BYTES = 16


class RedactionAuditError(ValueError):
    """Raised when live credentials appear in the retained evidence corpus."""


def _digest(data: bytes) -> str:
    return f"sha256:{hashlib.sha256(data).hexdigest()}"


def snapshot_manifest(root: Path, excluded: set[Path] | None = None) -> tuple[list[dict[str, Any]], str]:
    excluded = {path.resolve() for path in excluded or set()}
    entries: list[dict[str, Any]] = []
    for path in sorted(root.rglob("*")):
        if not path.is_file() or path.resolve() in excluded:
            continue
        data = path.read_bytes()
        entries.append(
            {
                "path": path.relative_to(root).as_posix(),
                "bytes": len(data),
                "prefix_sha256": _digest(data),
            }
        )
    return entries, _digest(json.dumps(entries, separators=(",", ":"), sort_keys=True).encode("utf-8"))


def _credential_canaries(credential_dir: Path) -> list[str]:
    canaries: set[str] = set()
    for path in sorted(credential_dir.rglob("*")):
        if not path.is_file():
            continue
        value = re.sub(rb"\s+", b"", path.read_bytes())
        if len(value) < MIN_CANARY_BYTES:
            continue
        canaries.add(_digest(value[-CANARY_BYTES:]))
    if not canaries:
        raise RedactionAuditError("ephemeral credential directory contains no usable credential canary")
    return sorted(canaries)


def audit(evidence_root: Path, credential_dir: Path, output: Path) -> dict[str, Any]:
    if not evidence_root.is_dir():
        raise RedactionAuditError(f"live evidence directory does not exist: {evidence_root}")
    if not credential_dir.is_dir():
        raise RedactionAuditError(f"ephemeral credential directory does not exist: {credential_dir}")
    canaries = _credential_canaries(credential_dir)
    excluded = {output}
    manifest, snapshot_digest = snapshot_manifest(evidence_root, excluded)
    evidence = [
        path.read_bytes()
        for path in sorted(evidence_root.rglob("*"))
        if path.is_file() and path.resolve() not in {output.resolve()}
    ]
    for credential in sorted(credential_dir.rglob("*")):
        if not credential.is_file():
            continue
        value = re.sub(rb"\s+", b"", credential.read_bytes())
        if len(value) < MIN_CANARY_BYTES:
            continue
        canary = value[-CANARY_BYTES:]
        if any(canary in retained for retained in evidence):
            raise RedactionAuditError(f"credential canary leaked from {credential.name}")
    if any(re.search(rb'"token"\s*:', retained) for retained in evidence):
        raise RedactionAuditError("retained evidence contains a token field")
    proof = {
        "schema": AUDIT_SCHEMA,
        "status": "passed",
        "auditor_source_digest": _digest(Path(__file__).read_bytes()),
        "credential_canary_digests": canaries,
        "forbidden_token_fields_absent": True,
        "snapshot_manifest": manifest,
        "snapshot_digest": snapshot_digest,
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(proof, sort_keys=True) + "\n", encoding="utf-8")
    return proof


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--evidence-root", required=True, type=Path)
    parser.add_argument("--credential-dir", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    try:
        audit(args.evidence_root, args.credential_dir, args.output)
    except RedactionAuditError as error:
        raise SystemExit(str(error)) from error


if __name__ == "__main__":
    main()
