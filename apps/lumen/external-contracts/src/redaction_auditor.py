#!/usr/bin/env python3
"""Credential-destroying, controller-committed redaction auditor for #2879."""

from __future__ import annotations

import argparse
import base64
from datetime import datetime, timezone
import hashlib
import json
import os
import re
import shutil
from pathlib import Path
from typing import Any, Mapping


LIVE_SCAN_SCHEMA = "axiom.lumen.ec.redaction-live-scan.v4"
AUDIT_SCHEMA = "axiom.lumen.ec.redaction-audit.v6"


class RedactionAuditError(ValueError):
    """Raised when retained evidence cannot independently prove redaction."""


def _digest(data: bytes) -> str:
    return f"sha256:{hashlib.sha256(data).hexdigest()}"


def _require_sha256(value: object, label: str) -> str:
    if not isinstance(value, str) or not re.fullmatch(r"sha256:[0-9a-f]{64}", value):
        raise RedactionAuditError(f"{label} must be a sha256 digest")
    return value


def _observed_action(sequence: int, kind: str, **details: Any) -> dict[str, Any]:
    return {
        "sequence": sequence,
        "kind": kind,
        "observed_at": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        **details,
    }


def snapshot_manifest(root: Path, excluded: set[Path] | None = None) -> tuple[list[dict[str, Any]], str]:
    excluded = {path.resolve() for path in excluded or set()}
    entries: list[dict[str, Any]] = []
    for path in sorted(root.rglob("*")):
        if not path.is_file() or path.resolve() in excluded:
            continue
        data = path.read_bytes()
        entries.append({"path": path.relative_to(root).as_posix(), "bytes": len(data), "sha256": _digest(data)})
    return entries, _digest(json.dumps(entries, separators=(",", ":"), sort_keys=True).encode("utf-8"))


def _safe_relative_path(value: str) -> Path:
    path = Path(value)
    if not value or path.is_absolute() or ".." in path.parts or path == Path("."):
        raise RedactionAuditError("controller credential path must be a safe nonempty relative path")
    return path


def credential_digests(credentials: Mapping[str, bytes]) -> dict[str, str]:
    result: dict[str, str] = {}
    for path, value in credentials.items():
        _safe_relative_path(path)
        if not isinstance(value, bytes) or not value:
            raise RedactionAuditError("controller credential bytes must be nonempty")
        result[path] = _digest(value)
    if not result or len(result) != len(credentials):
        raise RedactionAuditError("controller commitment needs distinct credential paths")
    return dict(sorted(result.items()))


def _binding_index(credentials: Mapping[str, bytes], bindings: list[dict[str, Any]]) -> list[dict[str, Any]]:
    digests = credential_digests(credentials)
    if not isinstance(bindings, list) or len(bindings) != len(digests):
        raise RedactionAuditError("controller credentials need one binding per exact credential path")
    indexed: dict[str, dict[str, Any]] = {}
    for binding in bindings:
        if not isinstance(binding, dict) or set(binding) != {"path", "class", "issuer_kind", "audience", "observation_id", "fingerprint"}:
            raise RedactionAuditError("controller credential binding has unknown or missing fields")
        path, fingerprint = binding.get("path"), binding.get("fingerprint")
        if not isinstance(path, str) or path in indexed or digests.get(path) != fingerprint or not isinstance(binding.get("class"), str) or not isinstance(binding.get("issuer_kind"), str) or not isinstance(binding.get("observation_id"), str) or not binding["observation_id"]:
            raise RedactionAuditError("controller credential binding does not map exact path and full bytes")
        indexed[path] = binding
    return [indexed[path] for path in sorted(indexed)]


def controller_commitment(run_id: str, credentials: Mapping[str, bytes], bindings: list[dict[str, Any]]) -> str:
    if not isinstance(run_id, str) or not run_id:
        raise RedactionAuditError("run id is required for a controller commitment")
    payload = {"credential_bindings": _binding_index(credentials, bindings), "credential_digests": credential_digests(credentials), "run_id": run_id}
    return _digest(json.dumps(payload, separators=(",", ":"), sort_keys=True).encode("utf-8"))


def _credential_forms(value: bytes) -> set[bytes]:
    urlsafe = base64.urlsafe_b64encode(value)
    standard = base64.b64encode(value)
    forms = {value, value.hex().encode("ascii"), standard, standard.rstrip(b"="), urlsafe, urlsafe.rstrip(b"=")}
    try:
        decoded = value.decode("utf-8")
    except UnicodeDecodeError:
        return forms
    forms.add(json.dumps(decoded, ensure_ascii=False).encode("utf-8"))
    return forms


def _scan_corpus(evidence_root: Path, credentials: Mapping[str, bytes], excluded: set[Path]) -> None:
    retained = [
        path.read_bytes() for path in sorted(evidence_root.rglob("*"))
        if path.is_file() and path.resolve() not in excluded
    ]
    for value in credentials.values():
        if any(form in item for form in _credential_forms(value) for item in retained):
            raise RedactionAuditError("a controller-committed complete credential or encoding appears in retained evidence")
    sensitive = re.compile(rb'"(?:access_token|id_token|refresh_token|token)"\s*:', re.IGNORECASE)
    if any(sensitive.search(item) for item in retained):
        raise RedactionAuditError("retained evidence contains a credential-bearing structured field")


def scan_live_credentials(credential_dir: Path, run_id: str, controller_credentials: Mapping[str, bytes], credential_bindings: list[dict[str, Any]], output: Path) -> dict[str, Any]:
    if not credential_dir.is_dir():
        raise RedactionAuditError(f"ephemeral credential directory does not exist: {credential_dir}")
    expected = {path: value for path, value in controller_credentials.items()}
    expected_digests = credential_digests(expected)
    observed_files = {path.relative_to(credential_dir).as_posix(): path.read_bytes() for path in credential_dir.rglob("*") if path.is_file()}
    if observed_files != expected:
        raise RedactionAuditError("live credential paths or complete bytes do not match the controller-held credentials")
    bindings = _binding_index(expected, credential_bindings)
    commitment = controller_commitment(run_id, expected, bindings)
    actions = [_observed_action(1, "live-credential-scan", credential_count=len(expected))]
    shutil.rmtree(credential_dir)
    if credential_dir.exists():
        raise RedactionAuditError("credential directory remained after the audited destruction action")
    actions.append(_observed_action(2, "credential-directory-destroyed", directory_absent=True))
    proof = {
        "schema": LIVE_SCAN_SCHEMA,
        "run_id": run_id,
        "controller_commitment": commitment,
        "auditor_source_digest": _digest(Path(__file__).read_bytes()),
        "credential_paths": sorted(expected),
        "credential_digests": expected_digests,
        "credential_bindings": bindings,
        "actions": actions,
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(proof, sort_keys=True) + "\n", encoding="utf-8")
    return proof


def _read_live_scan(path: Path, run_id: str, expected_commitment: str) -> dict[str, Any]:
    try:
        live = json.loads(path.read_text(encoding="utf-8"))
    except (FileNotFoundError, json.JSONDecodeError) as error:
        raise RedactionAuditError("live redaction scan is missing or invalid") from error
    if not isinstance(live, dict) or live.get("schema") != LIVE_SCAN_SCHEMA or live.get("run_id") != run_id:
        raise RedactionAuditError("live redaction scan belongs to another run or schema")
    if live.get("controller_commitment") != expected_commitment:
        raise RedactionAuditError("live redaction scan does not carry the controller commitment")
    digests = live.get("credential_digests")
    paths = live.get("credential_paths")
    if not isinstance(digests, dict) or not isinstance(paths, list) or paths != sorted(digests) or not paths or any(_safe_relative_path(path) is None or _require_sha256(digest, "live credential") != digest for path, digest in digests.items()):
        raise RedactionAuditError("live redaction scan has invalid credential paths or fingerprints")
    actions = live.get("actions")
    if not isinstance(actions, list) or len(actions) != 2 or not all(isinstance(item, dict) for item in actions):
        raise RedactionAuditError("live scan does not retain the observed scan then destruction action order")
    scan, destruction = actions
    if (
        scan.get("sequence") != 1
        or scan.get("kind") != "live-credential-scan"
        or not isinstance(scan.get("observed_at"), str)
        or scan.get("credential_count") != len(digests)
        or destruction.get("sequence") != 2
        or destruction.get("kind") != "credential-directory-destroyed"
        or not isinstance(destruction.get("observed_at"), str)
        or destruction.get("directory_absent") is not True
    ):
        raise RedactionAuditError("live scan action sequence is not complete credential scan then destruction")
    if live.get("auditor_source_digest") != _digest(Path(__file__).read_bytes()):
        raise RedactionAuditError("live redaction scan was not made by this auditor source")
    return live


def audit_terminal(evidence_root: Path, credential_dir: Path, live_scan_path: Path, run_id: str, controller_credentials: Mapping[str, bytes], credential_bindings: list[dict[str, Any]], output: Path) -> dict[str, Any]:
    if not evidence_root.is_dir():
        raise RedactionAuditError(f"retained evidence directory does not exist: {evidence_root}")
    credentials = dict(controller_credentials)
    bindings = _binding_index(credentials, credential_bindings)
    commitment = controller_commitment(run_id, credentials, bindings)
    live = _read_live_scan(live_scan_path, run_id, commitment)
    if live["credential_digests"] != credential_digests(credentials) or live.get("credential_bindings") != bindings:
        raise RedactionAuditError("terminal audit controller credentials disagree with the live scan")
    if credential_dir.exists():
        raise RedactionAuditError("credential directory was recreated or remains present at terminal audit")
    _scan_corpus(evidence_root, credentials, {output})
    manifest, snapshot_digest = snapshot_manifest(evidence_root, {output})
    proof = {
        "schema": AUDIT_SCHEMA,
        "status": "passed",
        "run_id": run_id,
        "controller_commitment": commitment,
        "auditor_source_digest": _digest(Path(__file__).read_bytes()),
        "live_scan_digest": _digest(live_scan_path.read_bytes()),
        "credential_paths": sorted(credentials),
        "credential_bindings": bindings,
        "actions": [_observed_action(3, "terminal-corpus-credential-scan", credential_count=len(credentials), credential_directory_absent=True)],
        "forbidden_credential_fields_absent": True,
        "snapshot_manifest": manifest,
        "snapshot_digest": snapshot_digest,
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(proof, sort_keys=True) + "\n", encoding="utf-8")
    return proof


def _parse_controller_credentials(text: str) -> tuple[dict[str, bytes], list[dict[str, Any]]]:
    try:
        parsed = json.loads(text)
    except json.JSONDecodeError as error:
        raise RedactionAuditError("controller credential input is not JSON") from error
    if not isinstance(parsed, dict) or set(parsed) != {"credentials", "bindings"} or not isinstance(parsed["credentials"], dict) or not all(isinstance(path, str) and isinstance(value, str) for path, value in parsed["credentials"].items()):
        raise RedactionAuditError("controller credential input must carry base64 credentials and bindings")
    try:
        return {path: base64.b64decode(value, validate=True) for path, value in parsed["credentials"].items()}, parsed["bindings"]
    except ValueError as error:
        raise RedactionAuditError("controller credential input contains invalid base64") from error


def _read_controller_credentials_fd(fd: int) -> tuple[dict[str, bytes], list[dict[str, Any]]]:
    if fd < 0:
        raise RedactionAuditError("controller credential FD must be nonnegative")
    try:
        # Duplicate only the inherited descriptor, then close it immediately:
        # controller canaries never acquire a retained filesystem path.
        with os.fdopen(os.dup(fd), "r", encoding="utf-8") as stream:
            return _parse_controller_credentials(stream.read())
    except OSError as error:
        raise RedactionAuditError("controller credential FD cannot be read") from error


def main() -> None:
    parser = argparse.ArgumentParser()
    commands = parser.add_subparsers(dest="command", required=True)
    live = commands.add_parser("live-scan")
    live.add_argument("--credential-dir", required=True, type=Path)
    live.add_argument("--controller-canary-fd", required=True, type=int)
    live.add_argument("--run-id", required=True)
    live.add_argument("--output", required=True, type=Path)
    terminal = commands.add_parser("terminal-audit")
    terminal.add_argument("--evidence-root", required=True, type=Path)
    terminal.add_argument("--credential-dir", required=True, type=Path)
    terminal.add_argument("--live-scan", required=True, type=Path)
    terminal.add_argument("--controller-canary-fd", required=True, type=int)
    terminal.add_argument("--run-id", required=True)
    terminal.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    try:
        credentials, bindings = _read_controller_credentials_fd(args.controller_canary_fd)
        if args.command == "live-scan":
            scan_live_credentials(args.credential_dir, args.run_id, credentials, bindings, args.output)
        else:
            audit_terminal(args.evidence_root, args.credential_dir, args.live_scan, args.run_id, credentials, bindings, args.output)
    except RedactionAuditError as error:
        raise SystemExit(str(error)) from error


if __name__ == "__main__":
    main()
