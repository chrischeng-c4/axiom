"""Independent #2879 external-contract oracles.

TD checks parse the actual protocol-located design source tree; CB checks read
only a retained GKE evidence bundle.  Neither path imports Lumen Rust nor a
GKE harness, and no producer summary or pass flag is an oracle by itself.
"""

from __future__ import annotations

import ast
import base64
from datetime import datetime
import hashlib
import json
import re
from pathlib import Path
from typing import Any


TD_SCHEMA = "axiom.lumen.td.auth-handoff.v3"
AUTH_SCHEMA = "axiom.gcp.lumen.auth.acceptance.v2"
RUN_SCHEMA = "axiom.gcp.operator.run.v2"
IMAGES_SCHEMA = "axiom.gcp.lumen.images.v2"
BUILD_SUBMIT_SCHEMA = "axiom.gcp.cloud-build.submit.v1"
BUILD_SOURCE_SCHEMA = "axiom.gcp.cloud-build.source-object.v1"
BUILD_FINAL_SCHEMA = "axiom.gcp.cloud-build.final.v1"
DEPLOYED_CR_SCHEMA = "axiom.gcp.lumen.deployed-cr.v1"
OBSERVATIONS_SCHEMA = "axiom.gcp.lumen.auth.observations.v3"
ISSUER_ACQUISITIONS_SCHEMA = "axiom.gcp.lumen.auth.issuer-acquisitions.v1"
RENDERED_RBAC_SCHEMA = "axiom.gcp.lumen.auth.rendered-rbac.v1"
REVOCATION_SCHEMA = "axiom.gcp.lumen.auth.revocations.v3"
RESIDUE_SCHEMA = "axiom.gcp.lumen.auth.cleanup-observations.v3"
LIVE_REDACTION_SCHEMA = "axiom.lumen.ec.redaction-live-scan.v4"
REDACTION_SCHEMA = "axiom.lumen.ec.redaction-audit.v6"

TD_SOURCE_ROOT = Path("apps/lumen/tech-design/src")
TD_ARTIFACT_PATH = Path(
    "lumen/work_items/wi_12_18_lumen_auth_phase_2_prove_two_hop_ksa_rbac_authorization_on.py"
)
TD_ARTIFACT_ID = (
    "artifact:security-access-kubernetes-native-deployment/"
    "work-item-12-18-lumen-auth-phase-2-prove-two-hop-ksa-rbac-authorization-on-wi-2879"
)
TD_PUBLIC_BOUNDARIES = (
    "acceptance/gcp/scripts/run.sh",
    "acceptance/gcp/scripts/verify-lumen-auth.sh",
    "acceptance/gcp/scripts/cleanup.sh",
    "acceptance/gcp/scripts/verify-clean.sh",
)
TD_CONTRACT_ASSIGNMENT = "__aw_ec_2879_contract__"

LUMEN_AUDIENCE = "lumen.axiom.dev"
KUBERNETES_AUDIENCE = "https://kubernetes.default.svc"
ISSUER_REVOCATION_BOUND_SECONDS = 120
LUMEN_REVOCATION_BOUND_SECONDS = 360
MAX_KSA_TOKEN_LIFETIME_SECONDS = 2700
MAX_CREDENTIAL_DESTRUCTION_SECONDS = 5
REQUEST_MARKER_PREFIX = "authz-"
RUN_LABEL_KEY = "lumen.axiom.dev/run-id"
AUTH_NAMESPACE = "lumen"
CLIENT_NAMESPACE = "lumen-auth-client"
LUMEN_INSTANCE = "lumen-auth"
GRANTED_COLLECTION = "authz"
UNGRANTED_COLLECTION = "authz-other"
LUMEN_API_GROUP = "lumen.axiom.dev"

# RFC 8032 Ed25519 constants.  The verifier is intentionally dependency-free:
# an EC runner must be able to authenticate a retained DSSE envelope without
# importing a producer SDK or accepting a producer-provided success flag.
_ED25519_Q = 2**255 - 19
_ED25519_L = 2**252 + 27742317777372353535851937790883648493
_ED25519_D = (-121665 * pow(121666, _ED25519_Q - 2, _ED25519_Q)) % _ED25519_Q
_ED25519_I = pow(2, (_ED25519_Q - 1) // 4, _ED25519_Q)

TD_REQUIRED_HOPS = (
    "google-identity-to-kube-apiserver",
    "kube-rbac-tokenrequest-to-named-client-ksa",
    "lumen-audience-ksa-token-to-lumen-tokenreview",
    "tokenreview-ksa-subject-to-subjectaccessreview",
)
TD_ISSUER_KINDS = ("google-user", "google-service-account")
TD_AUTHORIZATION_RESOURCES = ("lumencollections", "lumenadmin")
TD_FORBIDDEN_CREDENTIALS = (
    "google-access-token",
    "google-id-token",
    "adc",
    "gsa-credential",
    "metadata-server-token",
    "legacy-bearer",
)
TD_RETAINED_SECRETS = ("bearer-token", "kubeconfig-credential", "private-key")
TD_EVIDENCE_PATHS = (
    "run.json",
    "images.json",
    "cloud-build-submit.json",
    "cloud-build-source-archive.bin",
    "cloud-build-source-object.json",
    "cloud-build-final.json",
    "cloud-build-attestation.json",
    "kubernetes/lumen/deployed-lumen-cr.json",
    "kubernetes/auth/issuer-acquisitions.json",
    "kubernetes/auth/rendered-rbac.json",
    "kubernetes/auth/observations.json",
    "kubernetes/auth/cli-sibling-mint-failure.json",
    "kubernetes/auth/cli-controller-execution.json",
    "kubernetes/auth/unbound-rolebinding-deletion.json",
    "kubernetes/auth/revocation-observations.json",
    "kubernetes/auth/cleanup-observations.json",
    "kubernetes/auth/lumen-auth-live-redaction-scan.json",
    "kubernetes/auth/lumen-auth-redaction-audit.json",
)

# id: issuer kind, KSA name (or None), credential kind/audience, HTTP shape,
# resource class/name selector, expected status. These are EC-owned literals.
HTTP_EXPECTATIONS: dict[str, tuple[str, str | None, str, str | None, str, str, str, int]] = {
    "reader-search-granted": ("google-service-account", "auth-reader", "kubernetes-service-account", LUMEN_AUDIENCE, "POST", "collection:granted:search", "lumencollections", 200),
    "reader-stats-granted": ("google-service-account", "auth-reader", "kubernetes-service-account", LUMEN_AUDIENCE, "GET", "collection:granted:stats", "lumencollections", 200),
    "writer-index-granted": ("google-service-account", "auth-writer", "kubernetes-service-account", LUMEN_AUDIENCE, "POST", "collection:granted:index", "lumencollections", 200),
    "writer-search-granted": ("google-service-account", "auth-writer", "kubernetes-service-account", LUMEN_AUDIENCE, "POST", "collection:granted:search", "lumencollections", 200),
    "admin-create-granted": ("google-service-account", "auth-admin", "kubernetes-service-account", LUMEN_AUDIENCE, "PUT", "collection:granted:create", "lumencollections", 200),
    "operator-admin-granted": ("google-service-account", "auth-operator", "kubernetes-service-account", LUMEN_AUDIENCE, "GET", "admin", "lumenadmin", 200),
    "human-reader-search-granted": ("google-user", "auth-reader", "kubernetes-service-account", LUMEN_AUDIENCE, "POST", "collection:granted:search", "lumencollections", 200),
    "reader-index-denied": ("google-service-account", "auth-reader", "kubernetes-service-account", LUMEN_AUDIENCE, "POST", "collection:granted:index", "lumencollections", 403),
    "reader-create-denied": ("google-service-account", "auth-reader", "kubernetes-service-account", LUMEN_AUDIENCE, "PUT", "collection:granted:create", "lumencollections", 403),
    "reader-admin-denied": ("google-service-account", "auth-reader", "kubernetes-service-account", LUMEN_AUDIENCE, "GET", "admin", "lumenadmin", 403),
    "reader-other-collection": ("google-service-account", "auth-reader", "kubernetes-service-account", LUMEN_AUDIENCE, "POST", "collection:ungranted:search", "lumencollections", 403),
    "foreign-namespace-denied": ("google-service-account", "auth-foreign", "kubernetes-service-account", LUMEN_AUDIENCE, "POST", "collection:granted:search", "lumencollections", 403),
    "unbound-search-denied": ("google-service-account", "auth-unbound", "kubernetes-service-account", LUMEN_AUDIENCE, "POST", "collection:granted:search", "lumencollections", 403),
    "writer-create-denied": ("google-service-account", "auth-writer", "kubernetes-service-account", LUMEN_AUDIENCE, "PUT", "collection:granted:create", "lumencollections", 403),
    "writer-admin-denied": ("google-service-account", "auth-writer", "kubernetes-service-account", LUMEN_AUDIENCE, "GET", "admin", "lumenadmin", 403),
    "writer-other-collection": ("google-service-account", "auth-writer", "kubernetes-service-account", LUMEN_AUDIENCE, "POST", "collection:ungranted:index", "lumencollections", 403),
    "collection-admin-instance-denied": ("google-service-account", "auth-admin", "kubernetes-service-account", LUMEN_AUDIENCE, "GET", "admin", "lumenadmin", 403),
    "operator-search-denied": ("google-service-account", "auth-operator", "kubernetes-service-account", LUMEN_AUDIENCE, "POST", "collection:granted:search", "lumencollections", 403),
    "human-reader-admin-denied": ("google-user", "auth-reader", "kubernetes-service-account", LUMEN_AUDIENCE, "GET", "admin", "lumenadmin", 403),
    "human-unbound-denied": ("google-user", "auth-unbound", "kubernetes-service-account", LUMEN_AUDIENCE, "POST", "collection:granted:search", "lumencollections", 403),
    "google-access-token-refused": ("google-service-account", None, "google-access-token", None, "POST", "collection:granted:search", "lumencollections", 401),
    "google-id-token-refused": ("google-service-account", None, "google-id-token", LUMEN_AUDIENCE, "POST", "collection:granted:search", "lumencollections", 401),
    "human-google-token-refused": ("google-user", None, "google-access-token", None, "POST", "collection:granted:search", "lumencollections", 401),
    "wrong-audience-refused": ("google-service-account", "auth-admin", "kubernetes-service-account", KUBERNETES_AUDIENCE, "POST", "collection:granted:search", "lumencollections", 401),
    "anonymous-refused": ("anonymous", None, "anonymous", None, "POST", "collection:granted:search", "lumencollections", 401),
    "writer-search-before-revocation": ("google-service-account", "auth-writer", "kubernetes-service-account", LUMEN_AUDIENCE, "POST", "collection:granted:search", "lumencollections", 200),
}
BEHAVIOR_ROWS = (
    "reader-search-granted", "reader-stats-granted", "writer-index-granted",
    "writer-search-granted", "admin-create-granted", "operator-admin-granted",
    "human-reader-search-granted",
)
SECURITY_ROWS = tuple(row for row in HTTP_EXPECTATIONS if row not in BEHAVIOR_ROWS and row != "writer-search-before-revocation")
# The full retained TokenRequest corpus is a closed world. A later producer
# must name every mint, including the deliberately wrong-audience negative
# probe, rather than letting an unreviewed row hide behind a positive summary.
TOKEN_REQUEST_EXPECTATIONS = tuple(sorted({
    (issuer, ksa, audience, CLIENT_NAMESPACE if ksa == "auth-foreign" else AUTH_NAMESPACE, audience == KUBERNETES_AUDIENCE)
    for issuer, ksa, kind, audience, _, _, _, _ in HTTP_EXPECTATIONS.values()
    if kind == "kubernetes-service-account" and ksa is not None
}))

# These are the rendered Kubernetes objects that must be retained before the
# deliberate unbound and revocation deletions.  They are literal EC claims, not
# values copied from a product implementation or a producer's pass/fail line.
ACCESS_RULES: dict[str, tuple[dict[str, list[str]], ...]] = {
    "auth-reader": ({"apiGroups": [LUMEN_API_GROUP], "resources": ["lumencollections"], "resourceNames": [GRANTED_COLLECTION], "verbs": ["get"]},),
    "auth-writer": ({"apiGroups": [LUMEN_API_GROUP], "resources": ["lumencollections"], "resourceNames": [GRANTED_COLLECTION], "verbs": ["get", "update"]},),
    "auth-admin": ({"apiGroups": [LUMEN_API_GROUP], "resources": ["lumencollections"], "resourceNames": [GRANTED_COLLECTION, UNGRANTED_COLLECTION], "verbs": ["get", "update", "delete"]},),
    "auth-operator": ({"apiGroups": [LUMEN_API_GROUP], "resources": ["lumenadmin"], "resourceNames": [LUMEN_INSTANCE], "verbs": ["delete"]},),
    "auth-unbound": ({"apiGroups": [LUMEN_API_GROUP], "resources": ["lumencollections"], "resourceNames": [GRANTED_COLLECTION], "verbs": ["get"]},),
    "auth-foreign": ({"apiGroups": [LUMEN_API_GROUP], "resources": ["lumencollections"], "resourceNames": [GRANTED_COLLECTION], "verbs": ["get"]},),
}
ACCESS_NAMESPACES = {client: CLIENT_NAMESPACE if client == "auth-foreign" else AUTH_NAMESPACE for client in ACCESS_RULES}
ISSUER_RULE_TEMPLATE = {"apiGroups": [""], "resources": ["serviceaccounts/token"], "verbs": ["create"]}

# class: API, resource, namespace scope, exact retained operation identity.
CLEANUP_EXPECTATIONS: dict[str, tuple[str, str, str | None, tuple[str, ...], dict[str, str]]] = {
    "namespace-lumen": ("v1", "namespaces", None, ("kubectl", "get", "namespace", "lumen", "--no-headers"), {"name": "lumen"}),
    "namespace-lumen-system": ("v1", "namespaces", None, ("kubectl", "get", "namespace", "lumen-system", "--no-headers"), {"name": "lumen-system"}),
    "namespace-lumen-auth-client": ("v1", "namespaces", None, ("kubectl", "get", "namespace", "lumen-auth-client", "--no-headers"), {"name": "lumen-auth-client"}),
    "lumen-crd": ("apiextensions.k8s.io/v1", "customresourcedefinitions", None, ("kubectl", "get", "customresourcedefinition", "lumens.lumen.dev", "--no-headers"), {"name": "lumens.lumen.dev"}),
    "auth-delegator": ("rbac.authorization.k8s.io/v1", "clusterrolebindings", None, ("kubectl", "get", "clusterrolebinding", "-l", "app.kubernetes.io/component=auth-delegation,app.kubernetes.io/name=lumen", "--no-headers"), {"label_selector": "app.kubernetes.io/component=auth-delegation,app.kubernetes.io/name=lumen"}),
    "lumen-image-tag": ("artifactregistry.googleapis.com/v1", "dockerImages", None, ("gcloud", "artifacts", "docker", "images", "list", "--include-tags", "--format=json"), {"repository": "lumen"}),
    "node-service-account": ("iam.googleapis.com/v1", "serviceAccounts", None, ("gcloud", "iam", "service-accounts", "list", "--format=json"), {"email_prefix": "axo-{run_id}-node@"}),
    "persistent-disks": ("compute.googleapis.com/v1", "disks", None, ("gcloud", "compute", "disks", "list", "--format=json"), {"name_prefix": "axo-{run_id}-"}),
    "cloud-build-source-prefix": ("storage.googleapis.com/v1", "objects", None, ("gcloud", "storage", "ls", "--recursive"), {"prefix": "{gcs_prefix}"}),
}
CLEANUP_CLASSES = tuple(CLEANUP_EXPECTATIONS)
CORRELATION_RE = re.compile(r"^corr-[a-z0-9][a-z0-9-]{7,127}$")


class EvidenceError(ValueError):
    """Raised when #2879 evidence cannot independently prove its claim."""


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


def _digest(data: bytes) -> str:
    return f"sha256:{hashlib.sha256(data).hexdigest()}"


def _ed25519_decode(encoded: bytes) -> tuple[int, int, int, int] | None:
    if len(encoded) != 32:
        return None
    sign = encoded[31] >> 7
    y = int.from_bytes(encoded, "little") & ((1 << 255) - 1)
    if y >= _ED25519_Q:
        return None
    x_squared = ((y * y - 1) * pow((_ED25519_D * y * y + 1) % _ED25519_Q, _ED25519_Q - 2, _ED25519_Q)) % _ED25519_Q
    x = pow(x_squared, (_ED25519_Q + 3) // 8, _ED25519_Q)
    if (x * x - x_squared) % _ED25519_Q:
        x = (x * _ED25519_I) % _ED25519_Q
    if (x * x - x_squared) % _ED25519_Q:
        return None
    if (x & 1) != sign:
        x = _ED25519_Q - x
    if (x & 1) != sign:
        return None
    return x, y, 1, (x * y) % _ED25519_Q


def _ed25519_add(
    left: tuple[int, int, int, int], right: tuple[int, int, int, int],
) -> tuple[int, int, int, int]:
    x1, y1, z1, t1 = left
    x2, y2, z2, t2 = right
    a = ((y1 - x1) * (y2 - x2)) % _ED25519_Q
    b = ((y1 + x1) * (y2 + x2)) % _ED25519_Q
    c = (2 * _ED25519_D * t1 * t2) % _ED25519_Q
    d = (2 * z1 * z2) % _ED25519_Q
    e, f, g, h = (b - a) % _ED25519_Q, (d - c) % _ED25519_Q, (d + c) % _ED25519_Q, (b + a) % _ED25519_Q
    return (e * f) % _ED25519_Q, (g * h) % _ED25519_Q, (f * g) % _ED25519_Q, (e * h) % _ED25519_Q


def _ed25519_double(point: tuple[int, int, int, int]) -> tuple[int, int, int, int]:
    x, y, z, _ = point
    a, b, c, d = (x * x) % _ED25519_Q, (y * y) % _ED25519_Q, (2 * z * z) % _ED25519_Q, (-x * x) % _ED25519_Q
    e = ((x + y) * (x + y) - a - b) % _ED25519_Q
    g, f, h = (d + b) % _ED25519_Q, (d + b - c) % _ED25519_Q, (d - b) % _ED25519_Q
    return (e * f) % _ED25519_Q, (g * h) % _ED25519_Q, (f * g) % _ED25519_Q, (e * h) % _ED25519_Q


def _ed25519_scalar(point: tuple[int, int, int, int], scalar: int) -> tuple[int, int, int, int]:
    result = (0, 1, 1, 0)
    current = point
    while scalar:
        if scalar & 1:
            result = _ed25519_add(result, current)
        current = _ed25519_double(current)
        scalar >>= 1
    return result


def _ed25519_encode(point: tuple[int, int, int, int]) -> bytes:
    x, y, z, _ = point
    inverse = pow(z, _ED25519_Q - 2, _ED25519_Q)
    encoded = ((y * inverse) % _ED25519_Q).to_bytes(32, "little")
    return encoded[:31] + bytes([encoded[31] | ((((x * inverse) % _ED25519_Q) & 1) << 7)])


def _ed25519_same(left: tuple[int, int, int, int], right: tuple[int, int, int, int]) -> bool:
    return left[0] * right[2] % _ED25519_Q == right[0] * left[2] % _ED25519_Q and left[1] * right[2] % _ED25519_Q == right[1] * left[2] % _ED25519_Q


_ED25519_BASE = _ed25519_decode(bytes.fromhex("58" + "66" * 31))
assert _ED25519_BASE is not None


def _dsse_pae(payload_type: str, payload: bytes) -> bytes:
    encoded_type = payload_type.encode("utf-8")
    return b"DSSEv1 " + str(len(encoded_type)).encode("ascii") + b" " + encoded_type + b" " + str(len(payload)).encode("ascii") + b" " + payload


def _verify_ed25519(public_key: bytes, message: bytes, signature: bytes) -> bool:
    if len(public_key) != 32 or len(signature) != 64:
        return False
    point = _ed25519_decode(public_key)
    response = _ed25519_decode(signature[:32])
    scalar = int.from_bytes(signature[32:], "little")
    if point is None or response is None or scalar >= _ED25519_L:
        return False
    identity = (0, 1, 1, 0)
    if _ed25519_encode(point) != public_key or _ed25519_encode(response) != signature[:32]:
        return False
    if _ed25519_same(point, identity) or _ed25519_same(response, identity):
        return False
    if not _ed25519_same(_ed25519_scalar(point, _ED25519_L), identity) or not _ed25519_same(_ed25519_scalar(response, _ED25519_L), identity):
        return False
    challenge = int.from_bytes(hashlib.sha512(signature[:32] + public_key + message).digest(), "little") % _ED25519_L
    actual = _ed25519_scalar(_ED25519_BASE, scalar)
    expected = _ed25519_add(response, _ed25519_scalar(point, challenge))
    return _ed25519_same(actual, expected)


def _require_sha256(value: Any, label: str) -> str:
    if not isinstance(value, str) or not re.fullmatch(r"sha256:[0-9a-f]{64}", value):
        raise EvidenceError(f"{label} must be a sha256 digest")
    return value


def _decode_base64(value: Any, label: str, exact_bytes: int | None = None) -> bytes:
    if not isinstance(value, str) or not value:
        raise EvidenceError(f"{label} must be nonempty standard base64")
    try:
        decoded = base64.b64decode(value, validate=True)
    except ValueError as error:
        raise EvidenceError(f"{label} must be valid standard base64") from error
    if exact_bytes is not None and len(decoded) != exact_bytes:
        raise EvidenceError(f"{label} must encode exactly {exact_bytes} bytes")
    return decoded


def _crc32c(data: bytes) -> bytes:
    """Castagnoli CRC32C in GCS's big-endian raw checksum representation."""
    crc = 0xFFFFFFFF
    for byte in data:
        crc ^= byte
        for _ in range(8):
            crc = (crc >> 1) ^ (0x82F63B78 if crc & 1 else 0)
    return (crc ^ 0xFFFFFFFF).to_bytes(4, "big")


def _require_controller_ed25519_key(value: Any) -> bytes:
    return _decode_base64(value, "controller trusted Cloud Build Ed25519 public key", exact_bytes=32)


def _require_exact_strings(value: Any, expected: tuple[str, ...], label: str) -> None:
    if not isinstance(value, list) or tuple(value) != expected:
        raise EvidenceError(f"{label} does not equal the EC-owned literal contract")


def _require_exact_set(value: Any, expected: tuple[str, ...], label: str) -> None:
    if not isinstance(value, (list, tuple)) or len(value) != len(expected) or set(value) != set(expected):
        raise EvidenceError(f"{label} does not equal the complete EC-owned literal set")


def _parse_timestamp(value: Any, label: str) -> datetime:
    if not isinstance(value, str) or not value:
        raise EvidenceError(f"{label} is missing")
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as error:
        raise EvidenceError(f"{label} is not RFC3339") from error
    if parsed.tzinfo is None:
        raise EvidenceError(f"{label} must include a timezone")
    return parsed


def _digest_python_source_root(source_root: Path) -> str:
    if not source_root.is_dir():
        raise EvidenceError(f"TD source root is missing: {source_root}")
    files = sorted(path for path in source_root.rglob("*.py") if "__pycache__" not in path.parts)
    if not files:
        raise EvidenceError("TD source root has no Python files")
    hasher = hashlib.sha256()
    for path in files:
        data = path.read_bytes()
        hasher.update(path.relative_to(source_root).as_posix().encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(len(data).to_bytes(8, "big"))
        hasher.update(b"\0")
        hasher.update(data)
    return f"sha256:{hasher.hexdigest()}"


def _literal_assignments(path: Path) -> dict[str, Any]:
    try:
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    except (OSError, SyntaxError) as error:
        raise EvidenceError(f"could not parse TD artifact {path}") from error
    assignments: dict[str, Any] = {}
    for node in tree.body:
        if isinstance(node, ast.Assign):
            for target in node.targets:
                if isinstance(target, ast.Name):
                    try:
                        assignments[target.id] = ast.literal_eval(node.value)
                    except ValueError:
                        pass
    return assignments


def _read_td_contract(repo_root: Path, expected_digest: str) -> dict[str, Any]:
    _require_sha256(expected_digest, "expected TD source")
    source_root = repo_root / TD_SOURCE_ROOT
    if _digest_python_source_root(source_root) != expected_digest:
        raise EvidenceError("actual TD source-root digest does not match the controller binding")
    artifact = source_root / TD_ARTIFACT_PATH
    assignments = _literal_assignments(artifact)
    if assignments.get("__aw_work_item__") != "2879":
        raise EvidenceError("TD artifact is not bound to WI #2879")
    if assignments.get("__aw_artifact_id__") != TD_ARTIFACT_ID:
        raise EvidenceError("TD artifact identity does not match the #2879 public artifact")
    targets = assignments.get("__aw_native_handwrite_targets__")
    if not isinstance(targets, tuple) or len(targets) != len(TD_PUBLIC_BOUNDARIES) or set(targets) != set(TD_PUBLIC_BOUNDARIES):
        raise EvidenceError("TD artifact does not expose the complete #2879 public producer target set")
    contract = assignments.get(TD_CONTRACT_ASSIGNMENT)
    if not isinstance(contract, dict):
        raise EvidenceError("TD artifact lacks the literal #2879 external-contract declaration")
    return contract


def verify_td_behavior_source(repo_root: Path, expected_digest: str) -> None:
    contract = _read_td_contract(repo_root, expected_digest)
    if contract.get("schema") != TD_SCHEMA or contract.get("work_item") != "2879":
        raise EvidenceError("TD declaration has an unsupported schema or work item")
    if contract.get("artifact_id") != TD_ARTIFACT_ID:
        raise EvidenceError("TD declaration is not bound to the #2879 public artifact")
    _require_exact_set(contract.get("public_boundaries"), TD_PUBLIC_BOUNDARIES, "TD public producer targets")
    _require_exact_strings(contract.get("identity_hops"), TD_REQUIRED_HOPS, "TD identity hops")
    _require_exact_strings(contract.get("issuer_kinds"), TD_ISSUER_KINDS, "TD issuer kinds")
    if contract.get("audience") != LUMEN_AUDIENCE or contract.get("lumen_subject_kind") != "system:serviceaccount":
        raise EvidenceError("TD declaration does not bind the KSA-only Lumen identity")
    _require_exact_strings(contract.get("authorization_resources"), TD_AUTHORIZATION_RESOURCES, "TD authorization resources")
    _require_exact_strings(contract.get("retained_evidence_paths"), TD_EVIDENCE_PATHS, "TD retained evidence paths")


def verify_td_security_source(repo_root: Path, expected_digest: str) -> None:
    contract = _read_td_contract(repo_root, expected_digest)
    if contract.get("schema") != TD_SCHEMA or contract.get("work_item") != "2879":
        raise EvidenceError("TD declaration has an unsupported schema or work item")
    _require_exact_strings(contract.get("forbidden_direct_credentials"), TD_FORBIDDEN_CREDENTIALS, "TD forbidden credentials")
    _require_exact_strings(contract.get("retained_secret_kinds"), TD_RETAINED_SECRETS, "TD retained secrets")
    token_request = contract.get("token_request")
    if not isinstance(token_request, dict) or token_request.get("target_scope") != "one-named-client-serviceaccount":
        raise EvidenceError("TD declaration permits an overbroad TokenRequest target")
    if token_request.get("sibling_mint") != "denied" or token_request.get("max_lifetime_seconds") != MAX_KSA_TOKEN_LIFETIME_SECONDS:
        raise EvidenceError("TD declaration does not pin named-KSA mint denial and lifetime")
    revocation = contract.get("revocation")
    if not isinstance(revocation, dict):
        raise EvidenceError("TD declaration lacks revocation policy")
    if revocation.get("issuer_token_request_bound_seconds") != ISSUER_REVOCATION_BOUND_SECONDS:
        raise EvidenceError("TD issuer revocation bound is absent or unsafe")
    if revocation.get("lumen_authorization_bound_seconds") != LUMEN_REVOCATION_BOUND_SECONDS:
        raise EvidenceError("TD Lumen authorization revocation bound is absent or unsafe")
    if contract.get("cleanup_requirement") != "raw-exact-empty-residue-queries-for-every-auth-only-class":
        raise EvidenceError("TD declaration does not require exact raw cleanup residue evidence")
    if contract.get("redaction_requirement") != "controller-committed-live-scan-then-terminal-corpus-audit":
        raise EvidenceError("TD declaration does not require controller-committed terminal redaction")


def _snapshot_manifest(root: Path, excluded: set[Path] | None = None) -> tuple[list[dict[str, Any]], str]:
    excluded = {path.resolve() for path in excluded or set()}
    entries: list[dict[str, Any]] = []
    for path in sorted(root.rglob("*")):
        if not path.is_file() or path.resolve() in excluded:
            continue
        data = path.read_bytes()
        entries.append({"path": path.relative_to(root).as_posix(), "bytes": len(data), "sha256": _digest(data)})
    return entries, _digest(json.dumps(entries, separators=(",", ":"), sort_keys=True).encode("utf-8"))


def _redaction_commitment(
    run_id: str, credential_digests: dict[str, str], credential_bindings: list[dict[str, Any]],
) -> str:
    if not credential_digests or any(not isinstance(path, str) or not path or Path(path).is_absolute() or ".." in Path(path).parts or _require_sha256(digest, "credential digest") != digest for path, digest in credential_digests.items()):
        raise EvidenceError("redaction commitment has invalid complete credential paths or digests")
    if not isinstance(credential_bindings, list) or len(credential_bindings) != len(credential_digests):
        raise EvidenceError("redaction commitment lacks one binding for every complete credential")
    indexed: dict[str, dict[str, Any]] = {}
    for binding in credential_bindings:
        if not isinstance(binding, dict) or set(binding) != {"path", "class", "issuer_kind", "audience", "observation_id", "fingerprint"}:
            raise EvidenceError("redaction commitment binding has unknown or missing fields")
        path = binding.get("path")
        if not isinstance(path, str) or path in indexed or credential_digests.get(path) != binding.get("fingerprint"):
            raise EvidenceError("redaction commitment binding does not map the exact credential path and bytes")
        indexed[path] = binding
    payload = {
        "credential_bindings": [indexed[path] for path in sorted(indexed)],
        "credential_digests": dict(sorted(credential_digests.items())),
        "run_id": run_id,
    }
    return _digest(json.dumps(payload, separators=(",", ":"), sort_keys=True).encode("utf-8"))


def _require_project(value: Any, label: str) -> str:
    if not isinstance(value, str) or not re.fullmatch(r"[a-z][a-z0-9-]{4,62}", value):
        raise EvidenceError(f"{label} is not a valid expected GCP project")
    return value


def _require_correlation(value: Any, label: str) -> str:
    if not isinstance(value, str) or not CORRELATION_RE.fullmatch(value):
        raise EvidenceError(f"{label} is not a safe unique correlation identifier")
    return value


def _require_fingerprint(value: Any, label: str) -> str:
    return _require_sha256(value, label)


def _cleanup_argv(
    argv: tuple[str, ...], project: str, api: str, resource: str, namespace: str | None,
    run_id: str, image_tag: str, gcs_prefix: str, identity: dict[str, str],
) -> list[str]:
    """Return an exact, real CLI invocation for the retained residue query.

    The request object carries the cross-cloud correlation values.  The argv is
    deliberately command-specific: invented generic ``--api``/``--resource``
    switches could make a self-authored transcript look like a real query.
    """
    if argv[0] == "kubectl":
        return list(argv)
    if resource == "dockerImages":
        return ["gcloud", "artifacts", "docker", "images", "list", f"--project={project}", "--include-tags", "--format=json", f"--filter=tags:{image_tag}"]
    if resource == "serviceAccounts":
        return ["gcloud", "iam", "service-accounts", "list", f"--project={project}", "--format=json", f"--filter=email~^{identity['email_prefix']}"]
    if resource == "disks":
        return ["gcloud", "compute", "disks", "list", f"--project={project}", "--format=json", f"--filter=name~^{identity['name_prefix']}"]
    if resource == "objects":
        return ["gcloud", "storage", "ls", f"--project={project}", "--recursive", gcs_prefix]
    raise EvidenceError(f"no real per-tool cleanup argv is defined for {api}/{resource}")


def _require_google_principal(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value or value.startswith("system:serviceaccount:"):
        raise EvidenceError(f"{label} must be an expected non-KSA Google principal")
    return value


def _expected_gcs_prefix(project: str, run_id: str) -> str:
    return f"gs://{project}_cloudbuild/source/axiom-gcp-operator-{run_id}"


def _source_object_commitment(
    bucket: str, name: str, generation: str, expected_git_sha: str, archive_sha256: str, md5_hash: str, crc32c: str,
) -> str:
    """Controller-held binding for the raw staged object and clean source.

    Cloud Build's storage-source response identifies an object/generation but
    does not itself attest which repository revision was archived.  The
    controller therefore supplies a commitment over both identities; producer
    summaries cannot replace either component.
    """
    _require_sha256(archive_sha256, "controller archive content digest")
    _decode_base64(md5_hash, "controller immutable GCS MD5", exact_bytes=16)
    _decode_base64(crc32c, "controller immutable GCS CRC32C", exact_bytes=4)
    return _digest(json.dumps({
        "archive_sha256": archive_sha256,
        "bucket": bucket,
        "crc32c": crc32c,
        "expected_git_sha": expected_git_sha,
        "generation": generation,
        "md5Hash": md5_hash,
        "name": name,
    }, separators=(",", ":"), sort_keys=True).encode("utf-8"))


def _verify_cb_provenance(
    bundle_dir: Path,
    run_id: str,
    git_sha: str,
    not_before: datetime,
    expected_source_commitment: str,
    expected_project: str,
    expected_attestation_dsse_digest: str,
    expected_attestation_public_key: str,
) -> tuple[Path, dict[str, Any], tuple[datetime, ...]]:
    _require_sha256(expected_source_commitment, "controller source archive commitment")
    _require_sha256(expected_attestation_dsse_digest, "controller DSSE envelope digest")
    trusted_attestation_key = _require_controller_ed25519_key(expected_attestation_public_key)
    project = _require_project(expected_project, "controller project")
    run = _read_json(bundle_dir / "run.json")
    if run.get("schema") != RUN_SCHEMA or run.get("run_id") != run_id:
        raise EvidenceError("run provenance is not bound to the controller-generated run id")
    if run.get("git_sha") != git_sha or run.get("git_dirty") is not False or run.get("project") != project:
        raise EvidenceError("retained evidence does not match the expected clean source and project")
    started = _parse_timestamp(run.get("started_at"), "run start time")
    if run.get("image_provenance") != "cloud-build" or started < not_before:
        raise EvidenceError("run is not a fresh source-built GKE observation")
    build_id = run.get("cloud_build_id")
    image_tag = run.get("lumen_image_tag")
    prefix = _expected_gcs_prefix(project, run_id)
    if not isinstance(build_id, str) or not build_id or not isinstance(image_tag, str) or not image_tag.endswith(f":{git_sha}-{run_id}"):
        raise EvidenceError("run lacks its exact Cloud Build id or run-scoped Lumen image tag")
    archive_sha256 = run.get("source_archive_sha256")
    if not isinstance(archive_sha256, str):
        raise EvidenceError("run provenance lacks the controller-bound archive content digest")
    _require_sha256(archive_sha256, "run archive content digest")
    if run.get("source_archive_commitment") != expected_source_commitment or run.get("source_gcs_prefix") != prefix:
        raise EvidenceError("run is not bound to the controller source archive commitment and GCS prefix")

    images = _read_json(bundle_dir / "images.json")
    image_digest = images.get("lumen")
    if not isinstance(image_digest, str) or not re.fullmatch(r".+@sha256:[0-9a-f]{64}", image_digest):
        raise EvidenceError("retained image observation lacks the exact run image tag and immutable digest")
    if image_tag.rsplit(":", 1)[0] != image_digest.rsplit("@", 1)[0]:
        raise EvidenceError("run image tag and retained digest do not identify the same Lumen repository")
    digest_component = image_digest.rsplit("@", 1)[1]

    submit = _read_json(bundle_dir / "cloud-build-submit.json")
    storage = submit.get("source", {}).get("storageSource") if isinstance(submit.get("source"), dict) else None
    if submit.get("id") != build_id or submit.get("projectId") != project or submit.get("status") not in {"QUEUED", "PENDING", "WORKING", "SUCCESS"} or not isinstance(storage, dict):
        raise EvidenceError("raw Cloud Build submit response has no genuine id, project, status, and staged source shape")
    bucket, name, generation = storage.get("bucket"), storage.get("object"), storage.get("generation")
    if not all(isinstance(value, str) and value for value in (bucket, name, generation)) or not generation.isdigit() or int(generation) <= 0:
        raise EvidenceError("raw Cloud Build submit response lacks the exact storage source identity")
    submitted = _parse_timestamp(submit.get("createTime"), "Cloud Build submission time")

    source_object = _read_json(bundle_dir / "cloud-build-source-object.json")
    if source_object.get("bucket") != bucket or source_object.get("name") != name or str(source_object.get("generation")) != generation:
        raise EvidenceError("raw GCS object response contradicts Cloud Build staged source identity")
    if (
        not isinstance(source_object.get("size"), str)
        or not source_object["size"].isdigit()
        or int(source_object["size"]) <= 0
        or not isinstance(source_object.get("etag"), str)
        or not source_object["etag"]
        or not isinstance(source_object.get("md5Hash"), str)
        or not source_object["md5Hash"]
        or not isinstance(source_object.get("crc32c"), str)
        or not source_object["crc32c"]
    ):
        raise EvidenceError("raw immutable GCS source object response lacks size, etag, md5Hash, or crc32c")
    md5_hash, crc32c = source_object.get("md5Hash"), source_object.get("crc32c")
    raw_md5 = _decode_base64(md5_hash, "raw GCS source object MD5", exact_bytes=16)
    raw_crc32c = _decode_base64(crc32c, "raw GCS source object CRC32C", exact_bytes=4)
    metadata = source_object.get("metadata")
    if not isinstance(metadata, dict) or metadata.get("archive-sha256") != archive_sha256:
        raise EvidenceError("raw GCS source object does not expose the controller-bound archive content digest")
    if _source_object_commitment(bucket, name, generation, git_sha, archive_sha256, md5_hash, crc32c) != expected_source_commitment or run.get("source_archive_commitment") != expected_source_commitment:
        raise EvidenceError("controller source commitment does not bind the raw staged object and archive content")
    if not f"gs://{bucket}/{name}".startswith(prefix):
        raise EvidenceError("raw staged source object escapes the run-scoped GCS prefix")
    archive_path = bundle_dir / "cloud-build-source-archive.bin"
    try:
        archive_bytes = archive_path.read_bytes()
    except FileNotFoundError as error:
        raise EvidenceError("retained raw staged source archive bytes are missing") from error
    if not archive_bytes or str(len(archive_bytes)) != source_object["size"]:
        raise EvidenceError("retained raw staged source archive bytes contradict the GCS object size")
    if _digest(archive_bytes) != archive_sha256 or hashlib.md5(archive_bytes).digest() != raw_md5 or _crc32c(archive_bytes) != raw_crc32c:
        raise EvidenceError("retained raw staged source archive bytes contradict the controller-bound GCS content digests")
    staged_at = _parse_timestamp(source_object.get("timeCreated"), "staged source object creation time")

    final = _read_json(bundle_dir / "cloud-build-final.json")
    final_storage = final.get("source", {}).get("storageSource") if isinstance(final.get("source"), dict) else None
    results = final.get("results", {}).get("images") if isinstance(final.get("results"), dict) else None
    if final.get("id") != build_id or final.get("projectId") != project or final.get("status") != "SUCCESS" or final_storage != storage or not isinstance(results, list):
        raise EvidenceError("raw Cloud Build final response contradicts build identity or staged source")
    if not any(isinstance(item, dict) and item.get("name") == image_tag and item.get("digest") == digest_component for item in results):
        raise EvidenceError("raw Cloud Build final response lacks the exact Lumen result image digest")
    finished = _parse_timestamp(final.get("finishTime"), "Cloud Build final time")
    if submitted < started or staged_at < submitted or finished < staged_at:
        raise EvidenceError("raw Cloud Build timestamps are not an ordered source-build chain")

    attestation = _read_json(bundle_dir / "cloud-build-attestation.json")
    envelope = attestation.get("dsseEnvelope")
    if not isinstance(envelope, dict) or _digest(json.dumps(envelope, separators=(",", ":"), sort_keys=True).encode("utf-8")) != expected_attestation_dsse_digest:
        raise EvidenceError("Cloud Build provenance is not bound to the controller-captured DSSE envelope")
    if envelope.get("payloadType") != "application/vnd.in-toto+json" or not isinstance(envelope.get("payload"), str) or not isinstance(envelope.get("signatures"), list) or len(envelope["signatures"]) != 1:
        raise EvidenceError("raw Cloud Build provenance lacks a signed DSSE envelope")
    signature = envelope["signatures"][0]
    expected_key_id = "cloud-build-ed25519:" + _digest(trusted_attestation_key).removeprefix("sha256:")
    if not isinstance(signature, dict) or set(signature) != {"keyid", "sig"} or signature.get("keyid") != expected_key_id:
        raise EvidenceError("raw Cloud Build provenance lacks the controller-trusted Cloud Build DSSE key identity")
    try:
        payload_bytes = _decode_base64(envelope["payload"], "raw Cloud Build DSSE payload")
        dsse_signature = _decode_base64(signature.get("sig"), "raw Cloud Build DSSE signature", exact_bytes=64)
        payload = json.loads(payload_bytes)
    except (ValueError, json.JSONDecodeError, EvidenceError) as error:
        raise EvidenceError("raw Cloud Build DSSE payload is invalid") from error
    if not _verify_ed25519(trusted_attestation_key, _dsse_pae(envelope["payloadType"], payload_bytes), dsse_signature):
        raise EvidenceError("raw Cloud Build DSSE signature does not cryptographically verify against the controller-trusted identity")
    if not isinstance(payload, dict):
        raise EvidenceError("raw Cloud Build DSSE payload is not an in-toto statement")
    predicate = payload.get("predicate")
    subject = payload.get("subject")
    if payload.get("predicateType") != "https://slsa.dev/provenance/v1" or not isinstance(predicate, dict) or not isinstance(subject, list):
        raise EvidenceError("raw Cloud Build DSSE payload is missing its SLSA provenance shape")
    build_details = predicate.get("runDetails")
    dependencies = predicate.get("buildDefinition", {}).get("resolvedDependencies") if isinstance(predicate.get("buildDefinition"), dict) else None
    expected_subject = [{"name": image_digest.rsplit("@", 1)[0], "digest": {"sha256": digest_component.removeprefix("sha256:")}}]
    expected_dependency = [{"uri": f"gs://{bucket}/{name}#{generation}", "digest": {"sha256": archive_sha256.removeprefix("sha256:")}}]
    if subject != expected_subject or dependencies != expected_dependency or not isinstance(build_details, dict) or build_details.get("metadata", {}).get("invocationId") != build_id or build_details.get("builder", {}).get("id") != "https://cloudbuild.googleapis.com/GoogleHostedWorker":
        raise EvidenceError("raw Cloud Build provenance attestation contradicts the staged archive, image, or build id")

    deployed = _read_json(bundle_dir / "kubernetes" / "lumen" / "deployed-lumen-cr.json")
    metadata = deployed.get("metadata")
    if deployed.get("apiVersion") != "lumen.dev/v1alpha1" or deployed.get("kind") != "Lumen" or not isinstance(metadata, dict):
        raise EvidenceError("deployed evidence is not a genuine lumen.dev/v1alpha1 Lumen CR")
    if metadata.get("namespace") != "lumen" or metadata.get("name") != "lumen-auth" or not isinstance(metadata.get("uid"), str) or not metadata["uid"]:
        raise EvidenceError("deployed Lumen CR lacks the exact auth namespace/name/UID identity")
    if deployed.get("spec", {}).get("image") != image_digest or deployed.get("status", {}).get("phase") != "Ready":
        raise EvidenceError("deployed Lumen CR is not bound to the raw source-built image result")
    deployed_at = _parse_timestamp(metadata.get("creationTimestamp"), "deployed Lumen CR observation time")
    if deployed_at < finished:
        raise EvidenceError("deployed Lumen CR predates the Cloud Build final observation")
    auth_dir = bundle_dir / "kubernetes" / "auth"
    if not auth_dir.is_dir():
        raise EvidenceError("retained bundle has no auth evidence directory")
    return auth_dir, {"project": project, "git_sha": git_sha, "image_tag": image_tag, "image_digest": image_digest, "archive_sha256": archive_sha256, "gcs_prefix": prefix}, (finished, deployed_at)


def _verify_issuer_observations(
    bundle_dir: Path,
    auth_dir: Path,
    run_id: str,
    git_sha: str,
    not_before: datetime,
    expected_google_user: str,
    expected_google_service_account: str,
    expected_challenge: str,
) -> dict[str, dict[str, Any]]:
    """Bind both Google issuers to raw authenticated kube-apiserver replies.

    The acceptance summary remains retained context, but it is never a success
    oracle: each usable identity has to be present in the closed-world raw
    acquisition corpus with its own authenticated principal and WhoAmI result.
    """
    auth = _read_json(bundle_dir / "lumen-auth-acceptance.json")
    if auth.get("schema") != AUTH_SCHEMA or auth.get("run_id") != run_id:
        raise EvidenceError("auth summary is not bound to this run")
    if auth.get("audience") != LUMEN_AUDIENCE:
        raise EvidenceError("auth summary does not bind the Lumen audience")
    issuers = auth.get("issuers")
    if not isinstance(issuers, list) or len(issuers) != 2:
        raise EvidenceError("auth summary must name exactly the human and GSA issuers")
    by_kind = {item.get("kind"): item for item in issuers if isinstance(item, dict)}
    if set(by_kind) != set(TD_ISSUER_KINDS) or by_kind["google-service-account"].get("cluster_admin") is not False:
        raise EvidenceError("auth summary lacks the least-privilege GSA issuer")
    expected_principals = {
        "google-user": _require_google_principal(expected_google_user, "controller Google user principal"),
        "google-service-account": _require_google_principal(expected_google_service_account, "controller GSA principal"),
    }
    if len(set(expected_principals.values())) != len(expected_principals):
        raise EvidenceError("controller expected Google principals are not distinct")
    _require_sha256(expected_challenge, "controller issuer freshness challenge")
    acquisition = _read_json(auth_dir / "issuer-acquisitions.json")
    if acquisition.get("schema") != ISSUER_ACQUISITIONS_SCHEMA or acquisition.get("run_id") != run_id or acquisition.get("source_commit") != git_sha or acquisition.get("controller_challenge") != expected_challenge:
        raise EvidenceError("raw issuer acquisition corpus is missing its run/source binding")
    acquired_at = _parse_timestamp(acquisition.get("observed_at"), "issuer acquisition observation time")
    if acquired_at < not_before:
        raise EvidenceError("raw issuer acquisition corpus is stale")
    rows = acquisition.get("issuers")
    if not isinstance(rows, list) or len(rows) != len(TD_ISSUER_KINDS):
        raise EvidenceError("raw issuer acquisition corpus is not a closed two-issuer set")
    observed = {row.get("kind"): row for row in rows if isinstance(row, dict)}
    if len(observed) != len(rows) or set(observed) != set(TD_ISSUER_KINDS):
        raise EvidenceError("raw issuer acquisition corpus has an unknown, duplicate, or missing issuer")
    expected_acquisition_kind = {
        "google-user": "ambient-kubeconfig",
        "google-service-account": "gcloud-impersonated-service-account",
    }
    result: dict[str, dict[str, Any]] = {}
    filenames = {"google-user": "issuer-human-whoami.json", "google-service-account": "issuer-gsa-whoami.json"}
    for kind in TD_ISSUER_KINDS:
        row = observed[kind]
        if set(row) != {"kind", "acquisition_id", "kubernetes_username", "authenticated_principal", "acquisition", "command", "whoami", "observed_at"}:
            raise EvidenceError("raw issuer acquisition row contains unknown or missing evidence fields")
        username = row.get("kubernetes_username")
        principal = row.get("authenticated_principal")
        raw_whoami = row.get("whoami")
        whoami_status = raw_whoami.get("status") if isinstance(raw_whoami, dict) else None
        whoami_user = whoami_status.get("userInfo") if isinstance(whoami_status, dict) else None
        if (
            not isinstance(username, str)
            or not username
            or username.startswith("system:serviceaccount:")
            or row.get("acquisition_id") != f"{run_id}:{kind}"
            or principal != expected_principals[kind]
            or principal.startswith("system:serviceaccount:")
            or row.get("acquisition") != {"kind": expected_acquisition_kind[kind], "source_commit": git_sha, "controller_challenge": expected_challenge}
            or not isinstance(raw_whoami, dict)
            or raw_whoami.get("apiVersion") != "authentication.k8s.io/v1"
            or raw_whoami.get("kind") != "SelfSubjectReview"
            or not isinstance(whoami_user, dict)
            or whoami_user.get("username") != username
            or not isinstance(row.get("command"), dict)
            or row["command"].get("argv") != ["kubectl", "auth", "whoami", "-o", "json"]
            or row["command"].get("exit_code") != 0
            or row["command"].get("controller_challenge") != expected_challenge
            or _parse_timestamp(row["command"].get("finished_at"), f"{kind} issuer command finish") < not_before
            or row.get("observed_at") != row["command"].get("finished_at")
        ):
            raise EvidenceError(f"{kind} issuer acquisition is not a fresh controller-challenged raw authenticated WhoAmI observation")
        file_whoami = _read_json(auth_dir / filenames[kind])
        if file_whoami != raw_whoami or by_kind[kind].get("kubernetes_username") != username:
            raise EvidenceError(f"{kind} issuer is not cross-bound to retained raw WhoAmI evidence")
        result[kind] = {"kubernetes_username": username, "acquisition_id": row["acquisition_id"]}
    return result


def _load_observations(auth_dir: Path, run_id: str, issuer_summary: dict[str, dict[str, Any]]) -> dict[str, Any]:
    evidence = _read_json(auth_dir / "observations.json")
    if set(evidence) != {"schema", "run_id", "context", "issuers", "token_requests", "http_observations"}:
        raise EvidenceError("structured auth observations contain unknown or missing evidence sections")
    if evidence.get("schema") != OBSERVATIONS_SCHEMA or evidence.get("run_id") != run_id:
        raise EvidenceError("structured auth observations are missing or belong to another run")
    context = evidence.get("context")
    if not isinstance(context, dict) or context.get("audience") != LUMEN_AUDIENCE:
        raise EvidenceError("structured auth observations lack the Lumen audience context")
    if context.get("request_marker") != f"{REQUEST_MARKER_PREFIX}{run_id}":
        raise EvidenceError("structured auth observations lack the run-bound request marker")
    if context.get("namespace") != AUTH_NAMESPACE or context.get("client_namespace") != CLIENT_NAMESPACE:
        raise EvidenceError("structured auth observations do not name the real Lumen auth namespaces")
    resources = context.get("collections")
    if resources != {"granted": GRANTED_COLLECTION, "ungranted": UNGRANTED_COLLECTION}:
        raise EvidenceError("structured auth observations do not name the exact granted/ungranted collections")
    issuers = evidence.get("issuers")
    if not isinstance(issuers, list) or len(issuers) != len(TD_ISSUER_KINDS):
        raise EvidenceError("structured issuer observations are not the closed two-issuer set")
    observed_issuers = {row.get("kind"): row for row in issuers if isinstance(row, dict)}
    if len(observed_issuers) != len(issuers) or set(observed_issuers) != set(TD_ISSUER_KINDS):
        raise EvidenceError("structured issuer observations have an unknown, duplicate, or missing issuer")
    for kind, summary in issuer_summary.items():
        if set(observed_issuers[kind]) != {"kind", "kubernetes_username", "acquisition_id"}:
            raise EvidenceError("structured issuer observation contains unknown or missing fields")
        if (
            observed_issuers.get(kind, {}).get("kubernetes_username") != summary["kubernetes_username"]
            or observed_issuers.get(kind, {}).get("acquisition_id") != summary["acquisition_id"]
        ):
            raise EvidenceError("structured issuer observation does not match kube-apiserver evidence")
    return evidence


def _require_ksa(value: Any, namespace: str, name: str, label: str) -> None:
    if not isinstance(value, dict) or value.get("namespace") != namespace or value.get("name") != name:
        raise EvidenceError(f"{label} is not bound to the required named KSA")


def _object_identity(value: dict[str, Any]) -> tuple[str, str, str]:
    metadata = value.get("metadata")
    if not isinstance(metadata, dict):
        raise EvidenceError("rendered RBAC object lacks Kubernetes metadata")
    kind, namespace, name = value.get("kind"), metadata.get("namespace"), metadata.get("name")
    if not all(isinstance(item, str) and item for item in (kind, namespace, name)):
        raise EvidenceError("rendered RBAC object lacks kind/namespace/name identity")
    return kind, namespace, name


def _verify_rendered_rbac(auth_dir: Path, issuers: dict[str, dict[str, Any]], run_id: str) -> tuple[datetime, str]:
    rendered = _read_json(auth_dir / "rendered-rbac.json")
    if rendered.get("schema") != RENDERED_RBAC_SCHEMA or rendered.get("run_id") != run_id:
        raise EvidenceError("rendered RBAC corpus is missing or belongs to another run")
    rendered_at = _parse_timestamp(rendered.get("observed_at"), "rendered RBAC observation time")
    objects = rendered.get("objects")
    if not isinstance(objects, list) or not all(isinstance(item, dict) for item in objects):
        raise EvidenceError("rendered RBAC corpus lacks raw Kubernetes objects")
    expected_ids: set[tuple[str, str, str]] = set()
    for client, namespace in ACCESS_NAMESPACES.items():
        expected_ids.update({
            ("ServiceAccount", namespace, client),
            ("Role", namespace, f"{client}-token-issuer"),
            ("RoleBinding", namespace, f"{client}-token-issuer"),
            ("Role", namespace, f"{client}-lumen-access"),
            ("RoleBinding", namespace, f"{client}-lumen-access"),
        })
    indexed = {_object_identity(item): item for item in objects}
    if len(indexed) != len(objects) or set(indexed) != expected_ids:
        raise EvidenceError("rendered RBAC corpus is not the exact closed rendered object set")
    subjects = [
        {"apiGroup": "rbac.authorization.k8s.io", "kind": "User", "name": issuers[kind]["kubernetes_username"]}
        for kind in TD_ISSUER_KINDS
    ]
    for client, namespace in ACCESS_NAMESPACES.items():
        service_account = indexed[("ServiceAccount", namespace, client)]
        if service_account.get("apiVersion") != "v1" or service_account.get("automountServiceAccountToken") is not False:
            raise EvidenceError("rendered client ServiceAccount lacks the EC-owned hardened shape")
        issuer_role_name = f"{client}-token-issuer"
        issuer_role = indexed[("Role", namespace, issuer_role_name)]
        expected_issuer_rule = {**ISSUER_RULE_TEMPLATE, "resourceNames": [client]}
        if issuer_role.get("apiVersion") != "rbac.authorization.k8s.io/v1" or issuer_role.get("rules") != [expected_issuer_rule]:
            raise EvidenceError("rendered issuer Role does not bind create token to exactly its named ServiceAccount")
        issuer_binding = indexed[("RoleBinding", namespace, issuer_role_name)]
        expected_ref = {"apiGroup": "rbac.authorization.k8s.io", "kind": "Role", "name": issuer_role_name}
        if issuer_binding.get("apiVersion") != "rbac.authorization.k8s.io/v1" or issuer_binding.get("roleRef") != expected_ref or issuer_binding.get("subjects") != subjects:
            raise EvidenceError("rendered issuer RoleBinding does not bind exactly the observed Google issuers")
        access_role_name = f"{client}-lumen-access"
        access_role = indexed[("Role", namespace, access_role_name)]
        if access_role.get("apiVersion") != "rbac.authorization.k8s.io/v1" or access_role.get("rules") != list(ACCESS_RULES[client]):
            raise EvidenceError("rendered Lumen Role does not expose the exact EC-owned resources, names, and verbs")
        access_binding = indexed[("RoleBinding", namespace, access_role_name)]
        expected_access_ref = {"apiGroup": "rbac.authorization.k8s.io", "kind": "Role", "name": access_role_name}
        expected_sa_subject = [{"kind": "ServiceAccount", "name": client, "namespace": namespace}]
        if access_binding.get("apiVersion") != "rbac.authorization.k8s.io/v1" or access_binding.get("roleRef") != expected_access_ref or access_binding.get("subjects") != expected_sa_subject:
            raise EvidenceError("rendered Lumen RoleBinding does not bind exactly its named client ServiceAccount")
    unbound = indexed[("RoleBinding", AUTH_NAMESPACE, "auth-unbound-lumen-access")]
    uid = unbound.get("metadata", {}).get("uid") if isinstance(unbound.get("metadata"), dict) else None
    if not isinstance(uid, str) or not uid:
        raise EvidenceError("rendered auth-unbound Lumen RoleBinding lacks its observed UID")
    return rendered_at, uid


def _token_requests(
    observations: dict[str, Any], issuer_summary: dict[str, dict[str, Any]], canary_digests: dict[str, dict[str, Any]], run_id: str,
) -> list[dict[str, Any]]:
    rows = observations.get("token_requests")
    if not isinstance(rows, list) or not rows:
        raise EvidenceError("structured observations lack raw TokenRequest records")
    correlations: set[str] = set()
    fingerprints: set[str] = set()
    result: list[dict[str, Any]] = []
    for row in rows:
        if not isinstance(row, dict) or row.get("run_id") != run_id or row.get("outcome") != "issued":
            raise EvidenceError("TokenRequest record is malformed or not an issued raw observation")
        expected_keys = {"run_id", "outcome", "issuer_kind", "issuer_username", "issuer_acquisition_id", "audience", "negative_probe", "client_ksa", "correlation_id", "token_fingerprint", "credential_path", "request_uid", "issued_at", "expires_at", "lifetime_seconds", "request", "response"}
        if set(row) != expected_keys:
            raise EvidenceError("TokenRequest record contains unknown or missing raw evidence fields")
        issuer = row.get("issuer_kind")
        audience = row.get("audience")
        if issuer not in TD_ISSUER_KINDS or audience not in (LUMEN_AUDIENCE, KUBERNETES_AUDIENCE):
            raise EvidenceError("TokenRequest record has an unauthorized issuer or audience")
        if audience == KUBERNETES_AUDIENCE and row.get("negative_probe") is not True:
            raise EvidenceError("non-Lumen TokenRequest record is not an explicit negative audience probe")
        if audience == LUMEN_AUDIENCE and row.get("negative_probe") is not False:
            raise EvidenceError("Lumen TokenRequest record is not an explicit positive audience observation")
        if row.get("issuer_username") != issuer_summary[issuer]["kubernetes_username"]:
            raise EvidenceError("TokenRequest record is not tied to its observed issuer username")
        if row.get("issuer_acquisition_id") != issuer_summary[issuer]["acquisition_id"]:
            raise EvidenceError("TokenRequest record is not tied to its raw issuer acquisition")
        correlation = _require_correlation(row.get("correlation_id"), "TokenRequest")
        fingerprint = _require_fingerprint(row.get("token_fingerprint"), "TokenRequest token fingerprint")
        if fingerprint not in canary_digests:
            raise EvidenceError("TokenRequest credential fingerprint is not controller-committed as a live credential canary")
        binding = canary_digests[fingerprint]
        if binding != {"path": row.get("credential_path"), "class": "kubernetes-service-account", "issuer_kind": issuer, "audience": audience, "observation_id": correlation, "fingerprint": fingerprint}:
            raise EvidenceError("TokenRequest credential is not one-to-one bound to its controller path/class/issuer/audience observation")
        if correlation in correlations or fingerprint in fingerprints:
            raise EvidenceError("TokenRequest records reuse a supposedly unique correlation or token fingerprint")
        correlations.add(correlation)
        fingerprints.add(fingerprint)
        ksa = row.get("client_ksa")
        if not isinstance(ksa, dict) or not isinstance(ksa.get("namespace"), str) or not isinstance(ksa.get("name"), str):
            raise EvidenceError("TokenRequest record lacks its named KSA")
        issued = _parse_timestamp(row.get("issued_at"), "TokenRequest issue time")
        expires = _parse_timestamp(row.get("expires_at"), "TokenRequest expiry")
        lifetime = row.get("lifetime_seconds")
        if not isinstance(lifetime, int) or not 0 < lifetime <= MAX_KSA_TOKEN_LIFETIME_SECONDS:
            raise EvidenceError("TokenRequest lifetime is missing, expired, or overlong")
        if int((expires - issued).total_seconds()) != lifetime:
            raise EvidenceError("TokenRequest timestamps do not prove its bounded lifetime")
        request_uid = row.get("request_uid")
        if not isinstance(request_uid, str) or not request_uid:
            raise EvidenceError("TokenRequest record lacks raw request metadata UID")
        expected_request = {
            "apiVersion": "authentication.k8s.io/v1",
            "kind": "TokenRequest",
            "metadata": {"namespace": ksa["namespace"], "name": ksa["name"]},
            "spec": {"audiences": [audience], "expirationSeconds": lifetime},
        }
        expected_response = {
            "apiVersion": "authentication.k8s.io/v1",
            "kind": "TokenRequest",
            "metadata": {"namespace": ksa["namespace"], "name": ksa["name"], "uid": request_uid},
            "status": {"expirationTimestamp": row.get("expires_at"), "token_sha256": fingerprint},
        }
        if row.get("request") != expected_request or row.get("response") != expected_response:
            raise EvidenceError("TokenRequest record lacks exact raw request/response metadata")
        result.append(row)
    for issuer_kind in TD_ISSUER_KINDS:
        if not any(row["issuer_kind"] == issuer_kind and row["client_ksa"].get("name") == "auth-reader" for row in result):
            raise EvidenceError("both issuer kinds must mint the observed auth-reader KSA token")
    observed = {
        (row["issuer_kind"], row["client_ksa"]["name"], row["audience"], row["client_ksa"]["namespace"], row["negative_probe"])
        for row in result
    }
    if len(result) != len(observed) or observed != set(TOKEN_REQUEST_EXPECTATIONS):
        raise EvidenceError("TokenRequest observations are not the exact closed-world #2879 mint set")
    return result


def _path_for_shape(shape: str, collections: dict[str, str]) -> str:
    if shape == "admin":
        return "/admin/backup"
    _, collection_class, operation = shape.split(":")
    collection = collections[collection_class]
    return f"/collections/{collection}" if operation == "create" else f"/collections/{collection}/{operation}"


def _sar_verb_for_shape(shape: str) -> str:
    if shape == "admin" or shape.endswith(":create"):
        return "delete"
    if shape.endswith(":index"):
        return "update"
    return "get"


def _find_http_rows(observations: dict[str, Any]) -> dict[str, dict[str, Any]]:
    rows = observations.get("http_observations")
    if not isinstance(rows, list):
        raise EvidenceError("structured observations lack raw HTTP rows")
    indexed: dict[str, dict[str, Any]] = {}
    for row in rows:
        if not isinstance(row, dict) or not isinstance(row.get("id"), str) or row["id"] in indexed:
            raise EvidenceError("structured HTTP rows are malformed or duplicate")
        base_keys = {"id", "run_id", "issuer_kind", "observed_at", "credential", "request", "response"}
        ksa_keys = base_keys | {"token_review", "subject_access_review", "lumen_audit"}
        if set(row) not in (base_keys, ksa_keys):
            raise EvidenceError("HTTP row contains unknown or missing raw evidence fields")
        indexed[row["id"]] = row
    return indexed


def _validate_tokenreview_and_audit(
    row: dict[str, Any], credential: dict[str, Any], context: dict[str, Any], ksa_name: str,
    shape: str, resource_kind: str, status: int,
) -> None:
    ksa = credential["client_ksa"]
    subject = f"system:serviceaccount:{ksa['namespace']}:{ksa_name}"
    correlation = credential["correlation_id"]
    fingerprint = credential["token_fingerprint"]
    review = row.get("token_review")
    review_uid = review.get("metadata", {}).get("uid") if isinstance(review, dict) and isinstance(review.get("metadata"), dict) else None
    expected_review = {
        "apiVersion": "authentication.k8s.io/v1", "kind": "TokenReview", "metadata": {"uid": review_uid},
        "spec": {"token_sha256": fingerprint, "audiences": [credential["audience"]]},
        "status": {"authenticated": True, "user": {"username": subject}, "audiences": [credential["audience"]]},
        "correlation_id": correlation, "token_fingerprint": fingerprint,
    }
    if not isinstance(review_uid, str) or not review_uid or review != expected_review:
        raise EvidenceError("HTTP row lacks the exact raw TokenReview request/response metadata")
    resource_name = LUMEN_INSTANCE if resource_kind == "lumenadmin" else context["collections"][shape.split(":")[1]]
    sar = row.get("subject_access_review")
    sar_uid = sar.get("metadata", {}).get("uid") if isinstance(sar, dict) and isinstance(sar.get("metadata"), dict) else None
    expected_sar = {
        "apiVersion": "authorization.k8s.io/v1", "kind": "SubjectAccessReview", "metadata": {"uid": sar_uid},
        "spec": {"user": subject, "resourceAttributes": {"namespace": context["namespace"], "group": LUMEN_API_GROUP, "resource": resource_kind, "name": resource_name, "verb": _sar_verb_for_shape(shape)}},
        "status": {"allowed": status == 200},
    }
    if not isinstance(sar_uid, str) or not sar_uid or sar != expected_sar:
        raise EvidenceError("HTTP row lacks the exact raw SubjectAccessReview request/response metadata")
    audit = row.get("lumen_audit")
    expected = {
        "subject": subject, "audience": credential["audience"], "correlation_id": correlation,
        "token_fingerprint": fingerprint, "request_marker": context["request_marker"],
        "http_observation_id": row["id"],
    }
    if not isinstance(audit, dict) or any(audit.get(key) != value for key, value in expected.items()):
        raise EvidenceError("Lumen audit subject is not tied to the TokenReview and exact request")


def _validate_http_row(
    row: dict[str, Any], expectation: tuple[str, str | None, str, str | None, str, str, str, int],
    context: dict[str, Any], token_requests: list[dict[str, Any]], issuer_summary: dict[str, dict[str, Any]], canary_digests: dict[str, dict[str, Any]], run_id: str,
) -> None:
    issuer, ksa_name, credential_kind, audience, method, shape, resource_kind, status = expectation
    if row.get("run_id") != run_id or row.get("issuer_kind") != issuer:
        raise EvidenceError("HTTP row is not bound to this run and issuer kind")
    observed_at = _parse_timestamp(row.get("observed_at"), "HTTP observation time")
    credential = row.get("credential")
    if not isinstance(credential, dict) or credential.get("kind") != credential_kind or credential.get("audience") != audience:
        raise EvidenceError("HTTP row has an uncorrelated credential kind or audience")
    request = row.get("request")
    expected_resource = {"kind": resource_kind, "namespace": context["namespace"], "name": LUMEN_INSTANCE if resource_kind == "lumenadmin" else context["collections"][shape.split(":")[1]]}
    if not isinstance(request, dict) or request.get("marker") != context["request_marker"]:
        raise EvidenceError("HTTP row lacks the exact run-bound request marker")
    if request.get("method") != method or request.get("path") != _path_for_shape(shape, context["collections"]) or request.get("resource") != expected_resource or request.get("http_version") != "HTTP/1.1":
        raise EvidenceError("HTTP row does not prove the exact method, path, and resource")
    response = row.get("response")
    if not isinstance(response, dict) or response.get("status") != status:
        raise EvidenceError("HTTP row does not retain the actual expected response status")
    _require_sha256(response.get("body_sha256"), "HTTP response body")
    if response.get("http_version") != "HTTP/1.1" or response.get("request_marker") != context["request_marker"]:
        raise EvidenceError("HTTP row lacks exact raw response metadata")
    if issuer in TD_ISSUER_KINDS:
        if credential.get("issuer_username") != issuer_summary[issuer]["kubernetes_username"]:
            raise EvidenceError("HTTP credential is not tied to the observed issuer username")
        if credential.get("issuer_acquisition_id") != issuer_summary[issuer]["acquisition_id"]:
            raise EvidenceError("HTTP credential is not tied to the raw issuer acquisition")
        _require_correlation(credential.get("correlation_id"), "HTTP credential")
        fingerprint = _require_fingerprint(credential.get("token_fingerprint"), "HTTP credential token fingerprint")
        if fingerprint not in canary_digests:
            raise EvidenceError("HTTP credential fingerprint is not controller-committed as a live credential canary")
        binding = canary_digests[fingerprint]
        expected_class = credential_kind
        expected_observation = credential.get("correlation_id")
        if binding != {"path": credential.get("credential_path"), "class": expected_class, "issuer_kind": issuer, "audience": audience, "observation_id": expected_observation, "fingerprint": fingerprint}:
            raise EvidenceError("HTTP credential is not one-to-one bound to its controller path/class/issuer/audience observation")
    if ksa_name is None:
        if credential_kind in ("google-access-token", "google-id-token"):
            expected_direct_credential = {
                "kind": credential_kind,
                "audience": audience,
                "client_ksa": None,
                "issuer_username": issuer_summary[issuer]["kubernetes_username"],
                "issuer_acquisition_id": issuer_summary[issuer]["acquisition_id"],
                "correlation_id": credential.get("correlation_id"),
                "token_fingerprint": credential.get("token_fingerprint"),
                "credential_path": credential.get("credential_path"),
            }
            if credential != expected_direct_credential:
                raise EvidenceError("direct Google credential does not have the exact per-credential raw schema")
            if set(row) != {"id", "run_id", "issuer_kind", "observed_at", "credential", "request", "response"}:
                raise EvidenceError("direct Google rejection must not retain TokenReview, SubjectAccessReview, or an authenticated Lumen audit subject")
        if credential_kind == "anonymous":
            if credential != {"kind": "anonymous", "audience": None, "client_ksa": None} or set(row) != {"id", "run_id", "issuer_kind", "observed_at", "credential", "request", "response"}:
                raise EvidenceError("anonymous rejection does not have the exact unauthenticated negative schema")
        if credential.get("client_ksa") is not None:
            raise EvidenceError("non-KSA credential row falsely carries a client KSA")
        if credential_kind in ("google-access-token", "google-id-token") and issuer not in TD_ISSUER_KINDS:
            raise EvidenceError("direct Google rejection is not tied to an observed issuer")
        return
    namespace = context["client_namespace"] if ksa_name == "auth-foreign" else context["namespace"]
    _require_ksa(credential.get("client_ksa"), namespace, ksa_name, "HTTP credential")
    if credential_kind != "kubernetes-service-account":
        raise EvidenceError("a named KSA row used a non-KSA credential")
    matching = [
        token for token in token_requests
        if token["issuer_kind"] == issuer and token["issuer_username"] == credential["issuer_username"]
        and token["client_ksa"] == credential["client_ksa"] and token["correlation_id"] == credential["correlation_id"]
        and token["token_fingerprint"] == credential["token_fingerprint"] and token["audience"] == credential["audience"]
        and _parse_timestamp(token["issued_at"], "TokenRequest issue time") <= observed_at <= _parse_timestamp(token["expires_at"], "TokenRequest expiry")
    ]
    if not matching:
        raise EvidenceError("HTTP KSA row is not correlated with a bounded raw TokenRequest")
    if audience == KUBERNETES_AUDIENCE:
        if set(row) != {"id", "run_id", "issuer_kind", "observed_at", "credential", "request", "response"}:
            raise EvidenceError("wrong-audience rejection must not retain an authenticated TokenReview, allowed SubjectAccessReview, or Lumen audit subject")
        return
    _validate_tokenreview_and_audit(row, credential, context, ksa_name, shape, resource_kind, status)


def _verify_http_rows(observations: dict[str, Any], issuer_summary: dict[str, dict[str, Any]], canary_digests: dict[str, dict[str, Any]], required: tuple[str, ...], run_id: str) -> dict[str, dict[str, Any]]:
    tokens = _token_requests(observations, issuer_summary, canary_digests, run_id)
    rows = _find_http_rows(observations)
    if set(rows) != set(HTTP_EXPECTATIONS):
        raise EvidenceError("HTTP observations are not the exact closed-world #2879 request set")
    direct_pairs: set[tuple[str, str, str]] = set()
    for row_id in required:
        if row_id not in rows:
            raise EvidenceError(f"missing structured HTTP observation {row_id}")
        _validate_http_row(rows[row_id], HTTP_EXPECTATIONS[row_id], observations["context"], tokens, issuer_summary, canary_digests, run_id)
        issuer, ksa, kind, _, _, _, _, _ = HTTP_EXPECTATIONS[row_id]
        if ksa is None and kind in ("google-access-token", "google-id-token"):
            credential = rows[row_id]["credential"]
            pair = (credential["issuer_username"], credential["correlation_id"], credential["token_fingerprint"])
            if pair in direct_pairs:
                raise EvidenceError("direct Google rejection rows reuse a correlation fingerprint")
            direct_pairs.add(pair)
    return rows


def _controller_canary_digests(auth_dir: Path, run_id: str, expected_commitment: str) -> dict[str, dict[str, Any]]:
    _require_sha256(expected_commitment, "controller redaction commitment")
    live = _read_json(auth_dir / "lumen-auth-live-redaction-scan.json")
    if live.get("schema") != LIVE_REDACTION_SCHEMA or live.get("run_id") != run_id or live.get("controller_commitment") != expected_commitment:
        raise EvidenceError("live credential canary record is not bound to this run and controller commitment")
    bindings = live.get("credential_bindings")
    digests = live.get("credential_digests")
    paths = live.get("credential_paths")
    if not isinstance(digests, dict) or not isinstance(paths, list) or paths != sorted(digests) or not isinstance(bindings, list) or _redaction_commitment(run_id, digests, bindings) != expected_commitment:
        raise EvidenceError("live complete credential record does not match the controller commitment")
    if len(set(digests.values())) != len(digests):
        raise EvidenceError("controller credential records reuse a fingerprint across distinct paths")
    if len(bindings) != len(digests):
        raise EvidenceError("controller credential record lacks one-to-one path/class/issuer/audience/observation bindings")
    by_fingerprint: dict[str, dict[str, Any]] = {}
    for binding in bindings:
        if not isinstance(binding, dict) or set(binding) != {"path", "class", "issuer_kind", "audience", "observation_id", "fingerprint"}:
            raise EvidenceError("controller credential binding has unknown or missing fields")
        path, fingerprint = binding.get("path"), binding.get("fingerprint")
        if not isinstance(path, str) or digests.get(path) != fingerprint or fingerprint in by_fingerprint:
            raise EvidenceError("controller credential binding does not uniquely map a path to its complete credential")
        if binding.get("class") not in {"kubernetes-service-account", "google-access-token", "google-id-token"} or binding.get("issuer_kind") not in TD_ISSUER_KINDS or binding.get("audience") not in {LUMEN_AUDIENCE, KUBERNETES_AUDIENCE, None} or not isinstance(binding.get("observation_id"), str) or not binding["observation_id"]:
            raise EvidenceError("controller credential binding has an invalid class, issuer, audience, or observation")
        by_fingerprint[fingerprint] = binding
    return by_fingerprint


def _cli_binary_image_binding(image_digest: str, binary_digest: str) -> str:
    _require_sha256(binary_digest, "controller expected CLI binary digest")
    return _digest(json.dumps({"binary_sha256": binary_digest, "image_digest": image_digest}, separators=(",", ":"), sort_keys=True).encode("utf-8"))


def _verify_cli_sibling_failure(
    auth_dir: Path, run_id: str, context: dict[str, Any], issuer_summary: dict[str, dict[str, Any]], binding: dict[str, Any], expected_binary_digest: str, expected_controller_execution_public_key: str,
) -> None:
    artifact = _read_json(auth_dir / "cli-sibling-mint-failure.json")
    if artifact.get("schema") != "axiom.gcp.lumen.auth.cli-sibling-failure.v2" or artifact.get("run_id") != run_id:
        raise EvidenceError("public CLI sibling-mint failure artifact is missing or foreign")
    _require_ksa(artifact.get("client_ksa"), context["namespace"], "auth-sibling", "CLI sibling failure")
    if artifact.get("issuer_kind") != "google-service-account" or artifact.get("exit_code") == 0:
        raise EvidenceError("public CLI sibling mint did not fail nonzero for the GSA issuer")
    expected_argv = [
        "lumen", "query", "search", "--url", "http://127.0.0.1:17375",
        "--namespace", AUTH_NAMESPACE, "--client-sa", "auth-sibling",
        "--collection", GRANTED_COLLECTION, "--term", f"message={REQUEST_MARKER_PREFIX}{run_id}",
    ]
    binary = artifact.get("binary")
    if not isinstance(binary, dict) or binary.get("source_commit") != binding["git_sha"] or binary.get("image_digest") != binding["image_digest"]:
        raise EvidenceError("public CLI sibling-mint failure is not bound to the built source/image identity")
    expected_digest = _require_sha256(expected_binary_digest, "controller expected CLI binary digest")
    if _require_sha256(binary.get("sha256"), "public CLI binary digest") != expected_digest or binary.get("controller_image_binding") != _cli_binary_image_binding(binding["image_digest"], expected_digest):
        raise EvidenceError("public CLI binary digest is not the controller-extracted value bound to the retained image")
    if set(binary) != {"source_commit", "image_digest", "sha256", "controller_image_binding"}:
        raise EvidenceError("public CLI binary identity contains unknown or missing fields")
    if artifact.get("argv") != expected_argv:
        raise EvidenceError("public CLI sibling-mint failure does not retain the exact invocation")
    sibling = artifact.get("sibling_service_account")
    sibling_metadata = sibling.get("metadata") if isinstance(sibling, dict) else None
    sibling_uid = sibling_metadata.get("uid") if isinstance(sibling_metadata, dict) else None
    expected_sibling = {"apiVersion": "v1", "kind": "ServiceAccount", "metadata": {"namespace": AUTH_NAMESPACE, "name": "auth-sibling", "uid": sibling_uid}}
    if not isinstance(sibling_uid, str) or not re.fullmatch(r"[0-9a-f]{8}-[0-9a-f-]{27}", sibling_uid) or sibling != expected_sibling:
        raise EvidenceError("public CLI sibling failure lacks the retained existing ServiceAccount UID")
    expected_token_request = {
        "request": {"apiVersion": "authentication.k8s.io/v1", "kind": "TokenRequest", "metadata": {"namespace": AUTH_NAMESPACE, "name": "auth-sibling"}, "spec": {"audiences": [LUMEN_AUDIENCE], "expirationSeconds": MAX_KSA_TOKEN_LIFETIME_SECONDS}},
        "response": {"apiVersion": "v1", "kind": "Status", "status": "Failure", "reason": "Forbidden", "code": 403, "details": {"name": "auth-sibling", "kind": "serviceaccounts/token"}},
    }
    if artifact.get("sibling_token_request") != expected_token_request:
        raise EvidenceError("public CLI sibling failure lacks the exact raw TokenRequest and forbidden response")
    if artifact.get("issuer_username") != issuer_summary["google-service-account"]["kubernetes_username"] or artifact.get("issuer_acquisition_id") != issuer_summary["google-service-account"]["acquisition_id"]:
        raise EvidenceError("public CLI sibling failure is not tied to the observed GSA issuer acquisition")
    if set(artifact) != {"schema", "run_id", "issuer_kind", "issuer_username", "issuer_acquisition_id", "client_ksa", "sibling_service_account", "sibling_token_request", "exit_code", "argv", "binary", "raw_failure", "stderr"}:
        raise EvidenceError("public CLI sibling-mint failure contains unknown or missing producer fields")
    error = artifact.get("raw_failure")
    if error != expected_token_request["response"]:
        raise EvidenceError("public CLI sibling-mint failure lacks the raw Kubernetes forbidden error")
    if not isinstance(artifact.get("stderr"), str) or "--subresource=token" not in artifact["stderr"]:
        raise EvidenceError("public CLI sibling-mint failure does not retain its public token-subresource error")

    controller_key = _require_controller_ed25519_key(expected_controller_execution_public_key)
    capture = _read_json(auth_dir / "cli-controller-execution.json")
    transcript = capture.get("transcript")
    signature = capture.get("signature")
    expected_key_id = "controller-ed25519:" + _digest(controller_key).removeprefix("sha256:")
    if capture.get("schema") != "axiom.lumen.ec.cli-controller-execution.v1" or capture.get("run_id") != run_id or not isinstance(transcript, dict) or not isinstance(signature, dict) or set(signature) != {"keyid", "sig"} or signature.get("keyid") != expected_key_id:
        raise EvidenceError("controller-observed CLI execution transcript is missing, foreign, or unauthenticated")
    expected_transcript = {
        "argv": expected_argv,
        "executable": {"path": "/usr/local/bin/lumen", "sha256": expected_binary_digest, "image_digest": binding["image_digest"], "extracted_by": "controller-image-extraction"},
        "issuer": {"kind": "google-service-account", "kubernetes_username": issuer_summary["google-service-account"]["kubernetes_username"], "acquisition_id": issuer_summary["google-service-account"]["acquisition_id"]},
        "sibling_service_account_uid": sibling_uid,
        "sibling_token_request": expected_token_request,
        "exit_code": artifact["exit_code"],
        "stdout": "",
        "stderr": artifact["stderr"],
        "raw_failure": error,
    }
    canonical_capture = {"schema": capture.get("schema"), "run_id": capture.get("run_id"), "transcript": transcript}
    if transcript != expected_transcript:
        raise EvidenceError("controller-observed CLI execution transcript does not bind the extracted bytes, exact invocation, and raw failure")
    try:
        controller_signature = _decode_base64(signature.get("sig"), "controller-observed CLI execution signature", exact_bytes=64)
    except EvidenceError as error:
        raise EvidenceError("controller-observed CLI execution transcript has an invalid signature encoding") from error
    signed = json.dumps(canonical_capture, separators=(",", ":"), sort_keys=True).encode("utf-8")
    if not _verify_ed25519(controller_key, signed, controller_signature):
        raise EvidenceError("controller-observed CLI execution transcript signature does not cryptographically verify")


def _require_deletion(value: Any, run_id: str, namespace: str, name: str, label: str) -> datetime:
    if not isinstance(value, dict) or value.get("run_id") != run_id:
        raise EvidenceError(f"{label} deletion is missing or foreign")
    uid = value.get("uid")
    if not isinstance(uid, str) or not re.fullmatch(r"[0-9a-f]{8}-[0-9a-f-]{27}", uid):
        raise EvidenceError(f"{label} deletion lacks the observed RoleBinding UID")
    expected = {"apiVersion": "rbac.authorization.k8s.io/v1", "kind": "RoleBinding", "namespace": namespace, "name": name, "uid": uid, "verb": "delete"}
    if value.get("request") != expected:
        raise EvidenceError(f"{label} deletion does not name the exact RoleBinding")
    response = value.get("response")
    if not isinstance(response, dict) or response.get("apiVersion") != "v1" or response.get("kind") != "Status" or response.get("status") != "Success" or response.get("code") != 200 or response.get("details", {}).get("uid") != uid:
        raise EvidenceError(f"{label} deletion has no raw successful API response")
    audit = value.get("audit")
    if not isinstance(audit, dict) or audit.get("verb") != "delete" or audit.get("objectRef") != {"apiVersion": "rbac.authorization.k8s.io/v1", "resource": "rolebindings", "namespace": namespace, "name": name, "uid": uid}:
        raise EvidenceError(f"{label} deletion has no raw audit UID binding")
    return _parse_timestamp(value.get("deleted_at"), f"{label} deletion time")


def _verify_unbound_grant_deletion(
    auth_dir: Path, run_id: str, rendered_at: datetime, rendered_uid: str, http_rows: dict[str, dict[str, Any]],
) -> None:
    evidence = _read_json(auth_dir / "unbound-rolebinding-deletion.json")
    if evidence.get("schema") != "axiom.gcp.lumen.auth.unbound-deletion.v1" or evidence.get("run_id") != run_id:
        raise EvidenceError("auth-unbound grant deletion evidence is missing or foreign")
    if _parse_timestamp(evidence.get("rendered_at"), "auth-unbound render time") != rendered_at:
        raise EvidenceError("auth-unbound deletion is not tied to the retained pre-delete rendered snapshot")
    deletion = evidence.get("deletion")
    deleted_at = _require_deletion(deletion, run_id, AUTH_NAMESPACE, "auth-unbound-lumen-access", "auth-unbound grant")
    if not isinstance(deletion, dict) or deletion.get("uid") != rendered_uid:
        raise EvidenceError("auth-unbound deletion UID does not match the retained rendered RoleBinding")
    denied = http_rows.get("unbound-search-denied")
    if denied is None or _parse_timestamp(denied.get("observed_at"), "auth-unbound denial time") <= deleted_at or not rendered_at < deleted_at:
        raise EvidenceError("auth-unbound grant does not prove render then delete then denied request chronology")


def _verify_revocations(auth_dir: Path, observations: dict[str, Any], http_rows: dict[str, dict[str, Any]], run_id: str, binding: dict[str, Any]) -> None:
    evidence = _read_json(auth_dir / "revocation-observations.json")
    if evidence.get("schema") != REVOCATION_SCHEMA or evidence.get("run_id") != run_id:
        raise EvidenceError("raw revocation observations are missing or foreign")
    context = observations["context"]
    issuer = evidence.get("issuer_tokenrequest")
    if not isinstance(issuer, dict):
        raise EvidenceError("raw issuer TokenRequest revocation is missing")
    deleted = _require_deletion(issuer.get("deletion"), run_id, context["namespace"], "auth-reader-token-issuer", "issuer TokenRequest")
    issuer_pre = issuer.get("pre_allow")
    token_rows = observations.get("token_requests")
    issued = next((row.get("issued_at") for row in token_rows if isinstance(row, dict) and row.get("issuer_kind") == "google-service-account" and row.get("client_ksa", {}).get("name") == "auth-reader" and row.get("audience") == LUMEN_AUDIENCE), None) if isinstance(token_rows, list) else None
    expected_issuer_pre = {"controller_binding": {"run_id": run_id, "git_sha": binding["git_sha"], "image_digest": binding["image_digest"]}, "token_request": "google-service-account:auth-reader", "observed_at": issued, "status": "issued"}
    if issuer_pre != expected_issuer_pre or _parse_timestamp(issuer_pre["observed_at"], "issuer pre-allow time") >= deleted:
        raise EvidenceError("issuer revocation lacks a controller-bound pre-delete successful mint")
    polls = issuer.get("polls")
    if not isinstance(polls, list) or not polls:
        raise EvidenceError("issuer TokenRequest revocation contains no raw mint poll")
    denied = []
    for poll in polls:
        if not isinstance(poll, dict) or poll.get("run_id") != run_id or poll.get("issuer_kind") != "google-service-account":
            raise EvidenceError("issuer TokenRequest poll is malformed or foreign")
        _require_ksa(poll.get("client_ksa"), context["namespace"], "auth-reader", "issuer TokenRequest poll")
        if poll.get("audience") != LUMEN_AUDIENCE:
            raise EvidenceError("issuer TokenRequest poll lost the Lumen audience")
        if poll.get("request") != {"apiVersion": "authentication.k8s.io/v1", "kind": "TokenRequest", "namespace": context["namespace"], "serviceAccount": "auth-reader", "audience": LUMEN_AUDIENCE}:
            raise EvidenceError("issuer TokenRequest poll lacks the raw named TokenRequest request")
        response = poll.get("response")
        if not isinstance(response, dict) or response.get("kind") != "Status" or response.get("status") != "Failure" or response.get("reason") != "Forbidden" or response.get("code") != 403:
            raise EvidenceError("issuer TokenRequest poll has no raw forbidden response")
        denied.append(_parse_timestamp(poll.get("observed_at"), "issuer TokenRequest poll time"))
    if not denied or not deleted < min(denied) or not 0 <= (min(denied) - deleted).total_seconds() <= ISSUER_REVOCATION_BOUND_SECONDS:
        raise EvidenceError("issuer TokenRequest revocation is outside the EC-owned bound")
    lumen = evidence.get("lumen_authorization")
    if not isinstance(lumen, dict) or lumen.get("before_http_row") != "writer-search-before-revocation":
        raise EvidenceError("Lumen revocation has no correlated pre-revocation allow row")
    pre = http_rows.get("writer-search-before-revocation")
    if pre is None:
        raise EvidenceError("Lumen revocation has no retained pre-revocation HTTP allow row")
    deleted = _require_deletion(lumen.get("deletion"), run_id, context["namespace"], "auth-writer-lumen-access", "Lumen authorization")
    pre_allow = lumen.get("pre_allow")
    expected_pre_allow = {
        "controller_binding": {"run_id": run_id, "git_sha": binding["git_sha"], "image_digest": binding["image_digest"]},
        "http_row_id": "writer-search-before-revocation",
        "observed_at": pre.get("observed_at"),
        "status": 200,
    }
    pre_at = _parse_timestamp(pre.get("observed_at"), "Lumen pre-delete allow time")
    if pre_allow != expected_pre_allow or pre_at >= deleted:
        raise EvidenceError("Lumen revocation lacks a controller-bound pre-allow strictly before deletion")
    polls = lumen.get("polls")
    if not isinstance(polls, list) or not polls:
        raise EvidenceError("Lumen authorization revocation contains no raw authorization poll")
    denied = []
    credential = pre["credential"]
    subject = f"system:serviceaccount:{context['namespace']}:auth-writer"
    for poll in polls:
        if not isinstance(poll, dict) or poll.get("run_id") != run_id or poll.get("request_marker") != context["request_marker"]:
            raise EvidenceError("Lumen authorization poll is malformed or foreign")
        if poll.get("correlation_id") != credential["correlation_id"] or poll.get("token_fingerprint") != credential["token_fingerprint"]:
            raise EvidenceError("Lumen authorization poll is not tied to the pre-revocation KSA credential")
        response, review = poll.get("response"), poll.get("token_review")
        expected_request = {"method": "POST", "path": f"/collections/{GRANTED_COLLECTION}/search", "resource": {"kind": "lumencollections", "namespace": AUTH_NAMESPACE, "name": GRANTED_COLLECTION}}
        review_uid = review.get("metadata", {}).get("uid") if isinstance(review, dict) and isinstance(review.get("metadata"), dict) else None
        expected_review = {"apiVersion": "authentication.k8s.io/v1", "kind": "TokenReview", "metadata": {"uid": review_uid}, "spec": {"token_sha256": credential["token_fingerprint"], "audiences": [LUMEN_AUDIENCE]}, "status": {"authenticated": True, "user": {"username": subject}, "audiences": [LUMEN_AUDIENCE]}, "correlation_id": credential["correlation_id"], "token_fingerprint": credential["token_fingerprint"]}
        sar = poll.get("subject_access_review")
        sar_uid = sar.get("metadata", {}).get("uid") if isinstance(sar, dict) and isinstance(sar.get("metadata"), dict) else None
        expected_sar = {"apiVersion": "authorization.k8s.io/v1", "kind": "SubjectAccessReview", "metadata": {"uid": sar_uid}, "spec": {"user": subject, "resourceAttributes": {"namespace": AUTH_NAMESPACE, "group": LUMEN_API_GROUP, "resource": "lumencollections", "name": GRANTED_COLLECTION, "verb": "get"}}, "status": {"allowed": False}}
        if poll.get("request") != expected_request or not isinstance(response, dict) or response.get("status") != 403 or not isinstance(review_uid, str) or not review_uid or review != expected_review or not isinstance(sar_uid, str) or not sar_uid or sar != expected_sar:
            raise EvidenceError("Lumen authorization poll lacks exact raw TokenReview and SubjectAccessReview denial evidence")
        denied.append(_parse_timestamp(poll.get("observed_at"), "Lumen authorization poll time"))
    if not denied or not deleted < min(denied) or not 0 <= (min(denied) - deleted).total_seconds() <= LUMEN_REVOCATION_BOUND_SECONDS:
        raise EvidenceError("Lumen authorization revocation is outside the EC-owned bound")


def _verify_cleanup(auth_dir: Path, run_id: str, binding: dict[str, Any]) -> datetime:
    summary = _read_json(auth_dir.parent.parent / "cleanup.json")
    if (
        summary.get("schema") != "axiom.gcp.operator.cleanup.v1"
        or summary.get("project_id") != binding["project"]
        or summary.get("run_id") != run_id
        or summary.get("status") != "clean"
        or summary.get("preserved") != {"artifact_registry": True, "preexisting_apis": True}
    ):
        raise EvidenceError("verify-clean summary is not bound to the auth run and preserved infrastructure contract")
    verified = _parse_timestamp(summary.get("verified_at"), "terminal verify-clean summary time")
    evidence = _read_json(auth_dir / "cleanup-observations.json")
    if evidence.get("schema") != RESIDUE_SCHEMA or evidence.get("run_id") != run_id:
        raise EvidenceError("raw cleanup observations are missing or foreign")
    completed = _parse_timestamp(evidence.get("cleanup_completed_at"), "cleanup completion time")
    if completed >= verified:
        raise EvidenceError("terminal verify-clean summary does not follow cleanup completion")
    queries = evidence.get("queries")
    if not isinstance(queries, list) or len(queries) != len(CLEANUP_CLASSES) or not all(isinstance(query, dict) for query in queries):
        raise EvidenceError("raw cleanup observations must contain exactly one raw query per cleanup class")
    by_class = {query.get("class"): query for query in queries}
    if len(by_class) != len(queries) or set(by_class) != set(CLEANUP_CLASSES):
        raise EvidenceError("raw cleanup observations have a duplicate, unknown, or missing cleanup class")
    latest = completed
    for cleanup_class, (api, resource, namespace_scope, argv, identity) in CLEANUP_EXPECTATIONS.items():
        query = by_class[cleanup_class]
        if set(query) != {"class", "run_id", "request", "command", "observed_at"}:
            raise EvidenceError(f"cleanup residue query {cleanup_class} contains unknown or synthetic summary fields")
        namespace = AUTH_NAMESPACE if namespace_scope == "namespace" else None
        rendered_identity = {
            key: value.format(run_id=run_id, gcs_prefix=binding["gcs_prefix"])
            for key, value in identity.items()
        }
        expected_request = {
            "api": api, "resource": resource, "project": binding["project"], "namespace": namespace,
            "run_selector": f"{RUN_LABEL_KEY}={run_id}", "image_tag": binding["image_tag"],
            "gcs_prefix": binding["gcs_prefix"], "identity": rendered_identity,
        }
        if query.get("run_id") != run_id or query.get("request") != expected_request:
            raise EvidenceError(f"cleanup residue query {cleanup_class} lacks its exact EC-owned query identity")
        expected_argv = _cleanup_argv(argv, binding["project"], api, resource, namespace, run_id, binding["image_tag"], binding["gcs_prefix"], rendered_identity)
        command = query.get("command")
        if not isinstance(command, dict) or set(command) != {"argv", "context", "exit_code", "stdout", "stderr"} or command.get("argv") != expected_argv or command.get("context") != expected_request or not isinstance(command.get("exit_code"), int) or not isinstance(command.get("stdout"), str) or not isinstance(command.get("stderr"), str):
            raise EvidenceError(f"cleanup residue query {cleanup_class} has no genuine per-tool argv/exit/stdout/stderr transcript")
        # Each producer's native empty form is deliberately different.  Do not
        # accept a normalized API wrapper: it lets a producer manufacture a
        # clean answer without preserving what kubectl/gcloud actually said.
        if command["argv"][0] == "kubectl":
            if command["exit_code"] == 0 and command["stderr"] == "" and command["stdout"] == "No resources found\\n":
                pass
            elif command["exit_code"] == 1 and command["stdout"] == "" and "name" in rendered_identity:
                native_resource = {
                    "namespaces": "namespaces",
                    "customresourcedefinitions": "customresourcedefinitions.apiextensions.k8s.io",
                }.get(resource)
                expected_stderr = f'Error from server (NotFound): {native_resource} "{rendered_identity["name"]}" not found\\n' if native_resource else None
                if command["stderr"] != expected_stderr:
                    raise EvidenceError(f"cleanup residue query {cleanup_class} raw kubectl NotFound transcript is not exact")
            else:
                raise EvidenceError(f"cleanup residue query {cleanup_class} raw kubectl table/NotFound output found residue or is synthetic")
        elif resource == "objects":
            if command["exit_code"] != 0 or command["stderr"] or command["stdout"] != "":
                raise EvidenceError(f"cleanup residue query {cleanup_class} raw gcloud storage ls output found residue")
        else:
            if command["exit_code"] != 0 or command["stderr"]:
                raise EvidenceError(f"cleanup residue query {cleanup_class} did not retain a successful native gcloud transcript")
            try:
                output = json.loads(command["stdout"])
            except json.JSONDecodeError as error:
                raise EvidenceError(f"cleanup residue query {cleanup_class} raw gcloud list output is not JSON") from error
            if output != []:
                raise EvidenceError(f"cleanup residue query {cleanup_class} raw gcloud list output found residue or is synthetic")
        observed = _parse_timestamp(query.get("observed_at"), f"cleanup residue query {cleanup_class} time")
        if observed < completed or observed >= verified:
            raise EvidenceError(f"cleanup residue query {cleanup_class} predates cleanup completion")
        latest = max(latest, observed)
    return verified


def _verify_redaction_proof(evidence_root: Path, auth_dir: Path, run_id: str, expected_commitment: str, after: datetime) -> None:
    _require_sha256(expected_commitment, "controller redaction commitment")
    live_path = auth_dir / "lumen-auth-live-redaction-scan.json"
    proof_path = auth_dir / "lumen-auth-redaction-audit.json"
    live = _read_json(live_path)
    expected_source = _digest(Path(__file__).with_name("redaction_auditor.py").read_bytes())
    if live.get("schema") != LIVE_REDACTION_SCHEMA or live.get("run_id") != run_id or live.get("controller_commitment") != expected_commitment:
        raise EvidenceError("live redaction scan is not bound to the controller commitment and run")
    credential_digests = live.get("credential_digests")
    credential_paths = live.get("credential_paths")
    credential_bindings = live.get("credential_bindings")
    if not isinstance(credential_digests, dict) or not isinstance(credential_paths, list) or credential_paths != sorted(credential_digests) or not isinstance(credential_bindings, list) or _redaction_commitment(run_id, credential_digests, credential_bindings) != expected_commitment:
        raise EvidenceError("live redaction scan credentials do not match the controller commitment")
    if live.get("auditor_source_digest") != expected_source:
        raise EvidenceError("live redaction scan was not produced by the reviewed auditor source")
    actions = live.get("actions")
    if not isinstance(actions, list) or len(actions) != 2 or not all(isinstance(action, dict) for action in actions):
        raise EvidenceError("live redaction scan lacks observed scan and destruction actions")
    if actions[0].get("sequence") != 1 or actions[0].get("kind") != "live-credential-scan" or actions[0].get("credential_count") != len(credential_digests):
        raise EvidenceError("live redaction scan does not record the controller complete-credential scan action")
    if actions[1].get("sequence") != 2 or actions[1].get("kind") != "credential-directory-destroyed" or actions[1].get("directory_absent") is not True:
        raise EvidenceError("live redaction scan does not directly prove credential directory destruction")
    if not all(isinstance(action.get("observed_at"), str) for action in actions):
        raise EvidenceError("live redaction actions lack auditor-observed timestamps")
    scanned_at = _parse_timestamp(actions[0]["observed_at"], "live credential scan time")
    destroyed_at = _parse_timestamp(actions[1]["observed_at"], "credential destruction time")
    if not scanned_at <= destroyed_at or (destroyed_at - scanned_at).total_seconds() > MAX_CREDENTIAL_DESTRUCTION_SECONDS:
        raise EvidenceError("live credential scan was not destroyed within the EC-owned five-second bound")
    proof = _read_json(proof_path)
    if proof.get("schema") != REDACTION_SCHEMA or proof.get("status") != "passed" or proof.get("run_id") != run_id:
        raise EvidenceError("redaction proof is not a successful terminal audit")
    if proof.get("auditor_source_digest") != expected_source or proof.get("controller_commitment") != expected_commitment:
        raise EvidenceError("terminal redaction proof is not bound to the reviewed auditor and controller commitment")
    if proof.get("live_scan_digest") != _digest(live_path.read_bytes()):
        raise EvidenceError("terminal redaction proof is not tied to the committed live scan")
    terminal_actions = proof.get("actions")
    if not isinstance(terminal_actions, list) or len(terminal_actions) != 1 or not isinstance(terminal_actions[0], dict):
        raise EvidenceError("terminal redaction proof lacks its observed terminal corpus scan action")
    terminal = terminal_actions[0]
    if terminal.get("sequence") != 3 or terminal.get("kind") != "terminal-corpus-credential-scan" or terminal.get("credential_count") != len(credential_digests) or terminal.get("credential_directory_absent") is not True or proof.get("credential_paths") != credential_paths or proof.get("credential_bindings") != credential_bindings:
        raise EvidenceError("terminal redaction action does not follow the observed credential destruction")
    if not isinstance(terminal.get("observed_at"), str):
        raise EvidenceError("terminal redaction action lacks an auditor-observed timestamp")
    if _parse_timestamp(terminal["observed_at"], "terminal redaction audit time") <= after:
        raise EvidenceError("terminal redaction audit does not follow cleanup and terminal verify-clean summary")
    actual_manifest, actual_digest = _snapshot_manifest(evidence_root, {proof_path})
    if proof.get("snapshot_manifest") != actual_manifest or proof.get("snapshot_digest") != actual_digest:
        raise EvidenceError("redaction proof does not cover the immutable terminal evidence corpus")
    if proof.get("forbidden_credential_fields_absent") is not True:
        raise EvidenceError("terminal redaction auditor found a retained credential field")


def verify_cb_behavior_evidence(
    bundle_dir: Path, run_id: str, git_sha: str, not_before: datetime, expected_source_commitment: str, expected_project: str, expected_redaction_commitment: str,
    expected_google_user: str, expected_google_service_account: str, expected_issuer_challenge: str, expected_attestation_dsse_digest: str, expected_attestation_public_key: str,
) -> None:
    auth_dir, binding, _ = _verify_cb_provenance(bundle_dir, run_id, git_sha, not_before, expected_source_commitment, expected_project, expected_attestation_dsse_digest, expected_attestation_public_key)
    issuers = _verify_issuer_observations(bundle_dir, auth_dir, run_id, git_sha, not_before, expected_google_user, expected_google_service_account, expected_issuer_challenge)
    _verify_rendered_rbac(auth_dir, issuers, run_id)
    observations = _load_observations(auth_dir, run_id, issuers)
    canaries = _controller_canary_digests(auth_dir, run_id, expected_redaction_commitment)
    _verify_http_rows(observations, issuers, canaries, BEHAVIOR_ROWS, run_id)


def verify_cb_security_evidence(
    bundle_dir: Path, run_id: str, git_sha: str, not_before: datetime, expected_source_commitment: str, expected_project: str, expected_redaction_commitment: str,
    expected_google_user: str, expected_google_service_account: str, expected_issuer_challenge: str, expected_cli_binary_digest: str, expected_attestation_dsse_digest: str, expected_attestation_public_key: str, expected_controller_execution_public_key: str,
) -> None:
    auth_dir, binding, _ = _verify_cb_provenance(bundle_dir, run_id, git_sha, not_before, expected_source_commitment, expected_project, expected_attestation_dsse_digest, expected_attestation_public_key)
    issuers = _verify_issuer_observations(bundle_dir, auth_dir, run_id, git_sha, not_before, expected_google_user, expected_google_service_account, expected_issuer_challenge)
    rendered_at, unbound_uid = _verify_rendered_rbac(auth_dir, issuers, run_id)
    observations = _load_observations(auth_dir, run_id, issuers)
    canaries = _controller_canary_digests(auth_dir, run_id, expected_redaction_commitment)
    http_rows = _verify_http_rows(observations, issuers, canaries, SECURITY_ROWS + ("writer-search-before-revocation",), run_id)
    _verify_cli_sibling_failure(auth_dir, run_id, observations["context"], issuers, binding, expected_cli_binary_digest, expected_controller_execution_public_key)
    _verify_unbound_grant_deletion(auth_dir, run_id, rendered_at, unbound_uid, http_rows)
    _verify_revocations(auth_dir, observations, http_rows, run_id, binding)
    cleanup_at = _verify_cleanup(auth_dir, run_id, binding)
    _verify_redaction_proof(evidence_root=bundle_dir, auth_dir=auth_dir, run_id=run_id, expected_commitment=expected_redaction_commitment, after=cleanup_at)
