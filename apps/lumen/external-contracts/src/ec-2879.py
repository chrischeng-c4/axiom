from __future__ import annotations

import json
import os
import re
import hashlib
from pathlib import Path
from typing import Any


AUTH_SCHEMA = "axiom.gcp.lumen.auth.acceptance.v1"
CLEANUP_SCHEMA = "axiom.gcp.operator.cleanup.v1"
LUMEN_AUDIENCE = "lumen.axiom.dev"
ISSUER_REVOCATION_BOUND_SECONDS = 120
LUMEN_REVOCATION_BOUND_SECONDS = 360

GOOGLE_REJECTION_ROWS = (
    "probe-google-access-token-refused.status.txt",
    "probe-google-id-token-refused.status.txt",
    "probe-human-google-token-refused.status.txt",
)

LEAST_PRIVILEGE_ROWS = {
    "probe-reader-search-granted.status.txt": "200",
    "probe-reader-stats-granted.status.txt": "200",
    "probe-reader-index-denied.status.txt": "403",
    "probe-reader-create-denied.status.txt": "403",
    "probe-reader-admin-denied.status.txt": "403",
    "probe-reader-other-collection.status.txt": "403",
    "probe-foreign-namespace-denied.status.txt": "403",
    "probe-unbound-search-denied.status.txt": "403",
    "probe-writer-index-granted.status.txt": "200",
    "probe-writer-search-granted.status.txt": "200",
    "probe-writer-create-denied.status.txt": "403",
    "probe-writer-admin-denied.status.txt": "403",
    "probe-writer-other-collection.status.txt": "403",
    "probe-collection-admin-instance-denied.status.txt": "403",
    "probe-operator-admin-granted.status.txt": "200",
    "probe-operator-search-denied.status.txt": "403",
}


class EvidenceError(ValueError):
    """Raised when a retained GKE bundle cannot prove the #2879 boundary."""


def _read_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as error:
        raise EvidenceError(f"missing retained evidence: {path}") from error
    except json.JSONDecodeError as error:
        raise EvidenceError(f"invalid JSON retained evidence: {path}") from error
    if not isinstance(value, dict):
        raise EvidenceError(f"retained evidence must be a JSON object: {path}")
    return value


def _require_status(path: Path, expected: str) -> None:
    try:
        line = path.read_text(encoding="utf-8").strip()
    except FileNotFoundError as error:
        raise EvidenceError(f"missing HTTP status row: {path}") from error
    if not re.search(rf"-> {re.escape(expected)} \(expected {re.escape(expected)}\)$", line):
        raise EvidenceError(f"{path} does not prove expected HTTP {expected}: {line!r}")


def _require_bool(container: dict[str, Any], key: str, label: str) -> None:
    if container.get(key) is not True:
        raise EvidenceError(f"{label} must be explicitly true")


def _snapshot_manifest(root: Path, excluded: set[Path] | None = None) -> tuple[list[dict[str, Any]], str]:
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
                "prefix_sha256": f"sha256:{hashlib.sha256(data).hexdigest()}",
            }
        )
    encoded = json.dumps(entries, separators=(",", ":"), sort_keys=True).encode("utf-8")
    return entries, f"sha256:{hashlib.sha256(encoded).hexdigest()}"


def _verify_redaction_proof(evidence_root: Path, proof_path: Path) -> None:
    proof = _read_json(proof_path)
    if proof.get("schema") != "axiom.lumen.ec.redaction-audit.v1":
        raise EvidenceError("redaction proof has an unsupported schema")
    if proof.get("status") != "passed":
        raise EvidenceError("redaction auditor did not pass")
    auditor_source = Path(__file__).with_name("redaction_auditor.py").read_bytes()
    expected_auditor_digest = f"sha256:{hashlib.sha256(auditor_source).hexdigest()}"
    if proof.get("auditor_source_digest") != expected_auditor_digest:
        raise EvidenceError("redaction proof was not produced by the reviewed auditor source")
    manifest = proof.get("snapshot_manifest")
    if not isinstance(manifest, list) or not manifest:
        raise EvidenceError("redaction proof contains no immutable evidence snapshot")
    snapshot_digest = f"sha256:{hashlib.sha256(json.dumps(manifest, separators=(',', ':'), sort_keys=True).encode('utf-8')).hexdigest()}"
    if proof.get("snapshot_digest") != snapshot_digest:
        raise EvidenceError("redaction proof snapshot digest is invalid")
    for entry in manifest:
        if not isinstance(entry, dict):
            raise EvidenceError("redaction proof snapshot has an invalid file entry")
        path = entry.get("path")
        byte_count = entry.get("bytes")
        prefix_digest = entry.get("prefix_sha256")
        if (
            not isinstance(path, str)
            or not isinstance(byte_count, int)
            or byte_count < 0
            or not isinstance(prefix_digest, str)
            or not prefix_digest.startswith("sha256:")
        ):
            raise EvidenceError("redaction proof snapshot has an invalid file digest entry")
        relative = Path(path)
        if relative.is_absolute() or ".." in relative.parts:
            raise EvidenceError("redaction proof snapshot path escapes retained evidence")
        retained = evidence_root / relative
        if not retained.is_file():
            raise EvidenceError(f"redaction snapshot file disappeared: {path}")
        value = retained.read_bytes()
        if len(value) < byte_count:
            raise EvidenceError(f"redaction snapshot file was truncated: {path}")
        actual_digest = f"sha256:{hashlib.sha256(value[:byte_count]).hexdigest()}"
        if actual_digest != prefix_digest:
            raise EvidenceError(f"redaction snapshot prefix changed: {path}")
    canaries = proof.get("credential_canary_digests")
    if not isinstance(canaries, list) or not canaries or not all(
        isinstance(value, str) and value.startswith("sha256:") for value in canaries
    ):
        raise EvidenceError("redaction proof contains no digest-bound credential canary audit")
    if proof.get("forbidden_token_fields_absent") is not True:
        raise EvidenceError("redaction auditor found a retained token field")


def verify_retained_gke_evidence(bundle_dir: Path, redaction_proof: Path | None = None) -> None:
    """Validate a redacted, successful #2879 GKE evidence bundle.

    The bundle is produced by the real ``acceptance/gcp`` harness and retained
    outside the repository.  This contract deliberately reads only that
    material; it neither imports Lumen source nor treats older bearer/GSA
    evidence as an oracle.
    """

    auth = _read_json(bundle_dir / "lumen-auth-acceptance.json")
    cleanup = _read_json(bundle_dir / "cleanup.json")
    auth_evidence = bundle_dir / "kubernetes" / "auth"

    if auth.get("schema") != AUTH_SCHEMA or auth.get("status") != "passed":
        raise EvidenceError("auth summary is not a successful GKE two-hop proof")
    if auth.get("audience") != LUMEN_AUDIENCE:
        raise EvidenceError("auth summary does not bind the Lumen token audience")
    if not isinstance(auth.get("run_id"), str) or not auth["run_id"]:
        raise EvidenceError("auth summary is missing its run tag")

    issuers = auth.get("issuers")
    if not isinstance(issuers, list) or len(issuers) != 2:
        raise EvidenceError("auth summary must record exactly the human and GSA issuers")
    issuer_by_kind = {
        issuer.get("kind"): issuer
        for issuer in issuers
        if isinstance(issuer, dict) and isinstance(issuer.get("kind"), str)
    }
    if set(issuer_by_kind) != {"google-user", "google-service-account"}:
        raise EvidenceError("auth summary must retain both observed Google issuer kinds")
    for kind, issuer in issuer_by_kind.items():
        username = issuer.get("kubernetes_username")
        if not isinstance(username, str) or not username:
            raise EvidenceError(f"{kind} has no observed kube-apiserver username")
    if issuer_by_kind["google-service-account"].get("cluster_admin") is not False:
        raise EvidenceError("GSA issuer must be the least-privilege non-admin principal")

    sibling_refusals = auth.get("sibling_mint_refusals")
    if not isinstance(sibling_refusals, int) or sibling_refusals < 1:
        raise EvidenceError("no issuer proved sibling-KSA TokenRequest refusal")

    revocation = auth.get("revocation")
    if not isinstance(revocation, dict):
        raise EvidenceError("auth summary is missing revocation measurements")
    if revocation.get("documented_bound_seconds") != LUMEN_REVOCATION_BOUND_SECONDS:
        raise EvidenceError("auth summary does not retain the documented 360s Lumen allow-cache bound")
    issuer_seconds = revocation.get("issuer_token_request_seconds")
    if not isinstance(issuer_seconds, int) or not 0 <= issuer_seconds <= ISSUER_REVOCATION_BOUND_SECONDS:
        raise EvidenceError("issuer TokenRequest revocation is absent or outside its 120s bound")
    lumen_seconds = revocation.get("lumen_authorization_seconds")
    if not isinstance(lumen_seconds, int) or not 0 <= lumen_seconds <= LUMEN_REVOCATION_BOUND_SECONDS:
        raise EvidenceError("Lumen authorization revocation is absent or outside its 360s bound")

    for row in GOOGLE_REJECTION_ROWS:
        _require_status(auth_evidence / row, "401")
    for row, status in LEAST_PRIVILEGE_ROWS.items():
        _require_status(auth_evidence / row, status)

    namespace = auth.get("namespace")
    if not isinstance(namespace, str) or not namespace:
        raise EvidenceError("auth summary is missing the Lumen serving namespace")
    serving_log = (auth_evidence / "serving.log").read_text(encoding="utf-8")
    for service_account in ("auth-reader", "auth-operator"):
        subject = f"system:serviceaccount:{namespace}:{service_account}"
        if subject not in serving_log:
            raise EvidenceError(f"Lumen serving log does not name the KSA request subject {subject}")
    for issuer in issuer_by_kind.values():
        if issuer["kubernetes_username"] in serving_log:
            raise EvidenceError("Lumen serving log leaked a Google issuer as its request identity")

    # Producer-written booleans do not prove a canary sweep occurred. The
    # harness must invoke our independently-owned auditor after the auth
    # summary is written through this machine-checked, fail-closed lifecycle;
    # `runner.py` requires this command shape rather than accepting comments:
    #
    # lumen_auth_redaction_audit_and_destroy() {
    #   "${LUMEN_AUTH_REDACTION_AUDITOR:?required}" \
    #     --evidence-root "$EVIDENCE_DIR" \
    #     --credential-dir "$SECRET_DIR" \
    #     --output "${LUMEN_AUTH_REDACTION_AUDIT_PATH:?required}"
    #   rm -rf "$SECRET_DIR"
    #   SECRET_DIR=""
    # }
    # if [[ -n "${LUMEN_AUTH_REDACTION_AUDITOR:-}" || -n "${LUMEN_AUTH_REDACTION_AUDIT_PATH:-}" ]]; then
    #   [[ -n "${LUMEN_AUTH_REDACTION_AUDITOR:-}" && -n "${LUMEN_AUTH_REDACTION_AUDIT_PATH:-}" ]] || {
    #     echo "LUMEN_AUTH_REDACTION_AUDITOR and LUMEN_AUTH_REDACTION_AUDIT_PATH must be set together" >&2
    #     exit 1
    #   }
    #   lumen_auth_redaction_audit_and_destroy
    # fi
    #
    # This preserves ordinary GKE harness runs when neither EC variable is
    # supplied; half-configuration exits before the callback. An audit failure
    # aborts under `set -e`; on success the credential
    # directory is immediately removed and cleared before any later output or
    # continuation. The proof retains only hashes and byte-prefix digests, not
    # raw credentials or raw canaries.
    # The later runner checks every immutable preexisting prefix in this
    # retained run-level snapshot, allowing only later appends/new files. A
    # summary flag or copied prior proof cannot satisfy the contract.
    _verify_redaction_proof(
        bundle_dir,
        redaction_proof or auth_evidence / "lumen-auth-redaction-audit.json",
    )

    if cleanup.get("schema") != CLEANUP_SCHEMA or cleanup.get("status") != "clean":
        raise EvidenceError("post-run cleanup evidence is not clean")
    if cleanup.get("run_id") != auth["run_id"]:
        raise EvidenceError("cleanup evidence does not belong to the auth evidence run")


def main() -> None:
    value = os.environ.get("LUMEN_AUTH_EVIDENCE_DIR")
    if not value:
        raise SystemExit("set LUMEN_AUTH_EVIDENCE_DIR to the retained GKE evidence bundle")
    verify_retained_gke_evidence(Path(value))


if __name__ == "__main__":
    main()
