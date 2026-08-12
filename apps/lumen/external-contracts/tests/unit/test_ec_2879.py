from __future__ import annotations

import hashlib
import base64
import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from datetime import datetime, timedelta, timezone
from pathlib import Path


ROOT = Path(__file__).parents[2]
CASE_PATH = ROOT / "src" / "ec-2879.py"
RUNNER_PATH = ROOT / "src" / "runner.py"
AUDITOR_PATH = ROOT / "src" / "support" / "redaction_auditor.py"
EXPECTED_RUN_ID = "ec2879synthetic"
EXPECTED_GIT_SHA = "1eec8d061998"
EXPECTED_PROJECT = "lumen-ec-project"
EXPECTED_GOOGLE_USER = "human@example.test"
EXPECTED_GOOGLE_SERVICE_ACCOUNT = "gsa@example.test"
EXPECTED_ISSUER_CHALLENGE = "sha256:" + "e" * 64
EXPECTED_CLI_BINARY_DIGEST = "sha256:" + "d" * 64
NOT_BEFORE = datetime(2026, 8, 1, tzinfo=timezone.utc)


def _load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


CASE = _load("lumen_ec_2879_test", CASE_PATH)
RUNNER = _load("lumen_ec_2879_runner_test", RUNNER_PATH)
AUDITOR = _load("lumen_redaction_auditor_test", AUDITOR_PATH)

_CLOUDBUILD_SEED = bytes.fromhex("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60")
_CONTROLLER_SEED = bytes.fromhex("4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb")


def _ed25519_public(seed: bytes) -> bytes:
    expanded = hashlib.sha512(seed).digest()
    scalar = int.from_bytes(expanded[:32], "little")
    scalar &= (1 << 254) - 8
    scalar |= 1 << 254
    return CASE._ed25519_encode(CASE._ed25519_scalar(CASE._ED25519_BASE, scalar))


def _ed25519_sign(seed: bytes, message: bytes) -> bytes:
    expanded = hashlib.sha512(seed).digest()
    scalar = int.from_bytes(expanded[:32], "little")
    scalar &= (1 << 254) - 8
    scalar |= 1 << 254
    public_key = _ed25519_public(seed)
    nonce = int.from_bytes(hashlib.sha512(expanded[32:] + message).digest(), "little") % CASE._ED25519_L
    response = CASE._ed25519_encode(CASE._ed25519_scalar(CASE._ED25519_BASE, nonce))
    challenge = int.from_bytes(hashlib.sha512(response + public_key + message).digest(), "little") % CASE._ED25519_L
    return response + ((nonce + challenge * scalar) % CASE._ED25519_L).to_bytes(32, "little")


EXPECTED_CLOUDBUILD_PUBLIC_KEY = base64.b64encode(_ed25519_public(_CLOUDBUILD_SEED)).decode("ascii")
EXPECTED_CONTROLLER_PUBLIC_KEY = base64.b64encode(_ed25519_public(_CONTROLLER_SEED)).decode("ascii")


def _json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, sort_keys=True) + "\n", encoding="utf-8")


def _digest(value: str) -> str:
    return "sha256:" + hashlib.sha256(value.encode("utf-8")).hexdigest()


def _digest_bytes(value: bytes) -> str:
    return "sha256:" + hashlib.sha256(value).hexdigest()


def _canary(label: str) -> bytes:
    return hashlib.sha256(label.encode("utf-8")).digest()


def _issuer_username(kind: str) -> str:
    return EXPECTED_GOOGLE_USER if kind == "google-user" else EXPECTED_GOOGLE_SERVICE_ACCOUNT


def _acquisition_id(kind: str) -> str:
    return f"{EXPECTED_RUN_ID}:{kind}"


def _td_contract() -> dict[str, object]:
    return {
        "schema": CASE.TD_SCHEMA,
        "work_item": "2879",
        "artifact_id": CASE.TD_ARTIFACT_ID,
        "public_boundaries": list(CASE.TD_PUBLIC_BOUNDARIES),
        "identity_hops": list(CASE.TD_REQUIRED_HOPS),
        "issuer_kinds": list(CASE.TD_ISSUER_KINDS),
        "audience": CASE.LUMEN_AUDIENCE,
        "lumen_subject_kind": "system:serviceaccount",
        "authorization_resources": list(CASE.TD_AUTHORIZATION_RESOURCES),
        "retained_evidence_paths": list(CASE.TD_EVIDENCE_PATHS),
        "forbidden_direct_credentials": list(CASE.TD_FORBIDDEN_CREDENTIALS),
        "retained_secret_kinds": list(CASE.TD_RETAINED_SECRETS),
        "token_request": {"target_scope": "one-named-client-serviceaccount", "sibling_mint": "denied", "max_lifetime_seconds": CASE.MAX_KSA_TOKEN_LIFETIME_SECONDS},
        "revocation": {"issuer_token_request_bound_seconds": CASE.ISSUER_REVOCATION_BOUND_SECONDS, "lumen_authorization_bound_seconds": CASE.LUMEN_REVOCATION_BOUND_SECONDS},
        "cleanup_requirement": "raw-exact-empty-residue-queries-for-every-auth-only-class",
        "redaction_requirement": "controller-committed-live-scan-then-terminal-corpus-audit",
    }


def _write_td_source(repo_root: Path, contract: dict[str, object] | None = None, targets: tuple[str, ...] | None = None) -> str:
    artifact = repo_root / CASE.TD_SOURCE_ROOT / CASE.TD_ARTIFACT_PATH
    artifact.parent.mkdir(parents=True, exist_ok=True)
    artifact.write_text("\n".join((
        '__aw_work_item__ = "2879"',
        f"__aw_artifact_id__ = {CASE.TD_ARTIFACT_ID!r}",
        f"__aw_native_handwrite_targets__ = {(targets if targets is not None else CASE.TD_PUBLIC_BOUNDARIES)!r}",
        f"{CASE.TD_CONTRACT_ASSIGNMENT} = {contract if contract is not None else _td_contract()!r}",
        "",
    )), encoding="utf-8")
    return CASE._digest_python_source_root(repo_root / CASE.TD_SOURCE_ROOT)


def _token_rows() -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for issuer, ksa_name, audience, namespace, negative_probe in CASE.TOKEN_REQUEST_EXPECTATIONS:
        canary = _canary(f"token:{issuer}:{ksa_name}:{audience}")
        rows.append({
            "run_id": EXPECTED_RUN_ID,
            "outcome": "issued",
            "issuer_kind": issuer,
            "issuer_username": _issuer_username(issuer),
            "issuer_acquisition_id": _acquisition_id(issuer),
            "audience": audience,
            "negative_probe": negative_probe,
            "client_ksa": {"namespace": namespace, "name": ksa_name},
            "correlation_id": f"corr-{issuer.replace('google-', '')}-{ksa_name}-{('kube' if negative_probe else 'lumen')}",
            "token_fingerprint": _digest_bytes(canary),
            "credential_path": f"tokens/{issuer}-{ksa_name}-{audience.replace(':', '_').replace('/', '_')}.token",
            "request_uid": f"00000000-0000-4000-8000-{len(rows)+31:012d}",
            "issued_at": "2026-08-02T00:00:05Z",
            "expires_at": "2026-08-02T00:45:05Z",
            "lifetime_seconds": CASE.MAX_KSA_TOKEN_LIFETIME_SECONDS,
        })
        rows[-1]["request"] = {"apiVersion": "authentication.k8s.io/v1", "kind": "TokenRequest", "metadata": {"namespace": namespace, "name": ksa_name}, "spec": {"audiences": [audience], "expirationSeconds": CASE.MAX_KSA_TOKEN_LIFETIME_SECONDS}}
        rows[-1]["response"] = {"apiVersion": "authentication.k8s.io/v1", "kind": "TokenRequest", "metadata": {"namespace": namespace, "name": ksa_name, "uid": rows[-1]["request_uid"]}, "status": {"expirationTimestamp": rows[-1]["expires_at"], "token_sha256": rows[-1]["token_fingerprint"]}}
    return rows


def _token_by_key(rows: list[dict[str, object]], issuer: str, ksa_name: str, audience: str) -> dict[str, object]:
    return next(row for row in rows if row["issuer_kind"] == issuer and row["client_ksa"]["name"] == ksa_name and row["audience"] == audience)


def _http_rows(tokens: list[dict[str, object]]) -> list[dict[str, object]]:
    collections = {"granted": CASE.GRANTED_COLLECTION, "ungranted": CASE.UNGRANTED_COLLECTION}
    rows: list[dict[str, object]] = []
    for row_id, expectation in CASE.HTTP_EXPECTATIONS.items():
        issuer, ksa_name, credential_kind, audience, method, shape, resource_kind, status = expectation
        credential: dict[str, object] = {"kind": credential_kind, "audience": audience, "client_ksa": None}
        token: dict[str, object] | None = None
        if issuer in CASE.TD_ISSUER_KINDS:
            credential.update({"issuer_username": _issuer_username(issuer), "issuer_acquisition_id": _acquisition_id(issuer)})
        if ksa_name is not None:
            token = _token_by_key(tokens, issuer, ksa_name, audience or "")
            credential.update({"client_ksa": token["client_ksa"], "correlation_id": token["correlation_id"], "token_fingerprint": token["token_fingerprint"], "credential_path": token["credential_path"]})
        elif credential_kind in ("google-access-token", "google-id-token"):
            credential.update({"correlation_id": f"corr-direct-{row_id}", "token_fingerprint": _digest_bytes(_canary(f"direct:{row_id}")), "credential_path": f"direct/{row_id}.credential"})
        resource = {"kind": resource_kind, "namespace": CASE.AUTH_NAMESPACE, "name": CASE.LUMEN_INSTANCE if resource_kind == "lumenadmin" else collections[shape.split(":")[1]]}
        row: dict[str, object] = {
            "id": row_id, "run_id": EXPECTED_RUN_ID, "issuer_kind": issuer,
            "observed_at": "2026-08-02T00:10:00Z", "credential": credential,
            "request": {"marker": f"{CASE.REQUEST_MARKER_PREFIX}{EXPECTED_RUN_ID}", "method": method, "path": CASE._path_for_shape(shape, collections), "resource": resource, "http_version": "HTTP/1.1"},
            "response": {"status": status, "body_sha256": _digest(row_id), "http_version": "HTTP/1.1", "request_marker": f"{CASE.REQUEST_MARKER_PREFIX}{EXPECTED_RUN_ID}"},
        }
        if token is not None and audience == CASE.LUMEN_AUDIENCE:
            ksa = token["client_ksa"]
            subject = f"system:serviceaccount:{ksa['namespace']}:{ksa['name']}"
            review_uid = f"00000000-0000-4000-8001-{len(rows)+1:012d}"
            row["token_review"] = {"apiVersion": "authentication.k8s.io/v1", "kind": "TokenReview", "metadata": {"uid": review_uid}, "spec": {"token_sha256": token["token_fingerprint"], "audiences": [audience]}, "status": {"authenticated": True, "user": {"username": subject}, "audiences": [audience]}, "correlation_id": token["correlation_id"], "token_fingerprint": token["token_fingerprint"]}
            sar_uid = f"00000000-0000-4000-8002-{len(rows)+1:012d}"
            resource_name = CASE.LUMEN_INSTANCE if resource_kind == "lumenadmin" else collections[shape.split(":")[1]]
            row["subject_access_review"] = {"apiVersion": "authorization.k8s.io/v1", "kind": "SubjectAccessReview", "metadata": {"uid": sar_uid}, "spec": {"user": subject, "resourceAttributes": {"namespace": CASE.AUTH_NAMESPACE, "group": CASE.LUMEN_API_GROUP, "resource": resource_kind, "name": resource_name, "verb": CASE._sar_verb_for_shape(shape)}}, "status": {"allowed": status == 200}}
            row["lumen_audit"] = {"subject": subject, "audience": audience, "correlation_id": token["correlation_id"], "token_fingerprint": token["token_fingerprint"], "request_marker": f"{CASE.REQUEST_MARKER_PREFIX}{EXPECTED_RUN_ID}", "http_observation_id": row_id}
        rows.append(row)
    return rows


def _rendered_rbac() -> list[dict[str, object]]:
    subjects = [{"apiGroup": "rbac.authorization.k8s.io", "kind": "User", "name": _issuer_username(kind)} for kind in CASE.TD_ISSUER_KINDS]
    objects: list[dict[str, object]] = []
    for index, (client, namespace) in enumerate(CASE.ACCESS_NAMESPACES.items(), start=1):
        issuer_name = f"{client}-token-issuer"
        access_name = f"{client}-lumen-access"
        objects.extend([
            {"apiVersion": "v1", "kind": "ServiceAccount", "metadata": {"namespace": namespace, "name": client}, "automountServiceAccountToken": False},
            {"apiVersion": "rbac.authorization.k8s.io/v1", "kind": "Role", "metadata": {"namespace": namespace, "name": issuer_name}, "rules": [{**CASE.ISSUER_RULE_TEMPLATE, "resourceNames": [client]}]},
            {"apiVersion": "rbac.authorization.k8s.io/v1", "kind": "RoleBinding", "metadata": {"namespace": namespace, "name": issuer_name, "uid": f"00000000-0000-4000-8003-{index:012d}"}, "roleRef": {"apiGroup": "rbac.authorization.k8s.io", "kind": "Role", "name": issuer_name}, "subjects": subjects},
            {"apiVersion": "rbac.authorization.k8s.io/v1", "kind": "Role", "metadata": {"namespace": namespace, "name": access_name}, "rules": list(CASE.ACCESS_RULES[client])},
            {"apiVersion": "rbac.authorization.k8s.io/v1", "kind": "RoleBinding", "metadata": {"namespace": namespace, "name": access_name, "uid": f"00000000-0000-4000-8004-{index:012d}"}, "roleRef": {"apiGroup": "rbac.authorization.k8s.io", "kind": "Role", "name": access_name}, "subjects": [{"kind": "ServiceAccount", "name": client, "namespace": namespace}]},
        ])
    return objects


def _deletion(name: str, at: str, uid: str) -> dict[str, object]:
    request = {"apiVersion": "rbac.authorization.k8s.io/v1", "kind": "RoleBinding", "namespace": CASE.AUTH_NAMESPACE, "name": name, "uid": uid, "verb": "delete"}
    return {"run_id": EXPECTED_RUN_ID, "uid": uid, "request": request, "response": {"apiVersion": "v1", "kind": "Status", "status": "Success", "code": 200, "details": {"uid": uid}}, "audit": {"verb": "delete", "objectRef": {"apiVersion": "rbac.authorization.k8s.io/v1", "resource": "rolebindings", "namespace": CASE.AUTH_NAMESPACE, "name": name, "uid": uid}}, "deleted_at": at}


def _write_redaction(bundle: Path, credentials_by_path: dict[str, bytes], credential_bindings: list[dict[str, object]]) -> str:
    auth = bundle / "kubernetes" / "auth"
    credentials = bundle.parent / "credentials"
    credentials.mkdir(exist_ok=True)
    for path, value in credentials_by_path.items():
        target = credentials / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(value)
    live = auth / "lumen-auth-live-redaction-scan.json"
    AUDITOR.scan_live_credentials(credentials, EXPECTED_RUN_ID, credentials_by_path, credential_bindings, live)
    commitment = AUDITOR.controller_commitment(EXPECTED_RUN_ID, credentials_by_path, credential_bindings)
    AUDITOR.audit_terminal(bundle, credentials, live, EXPECTED_RUN_ID, credentials_by_path, credential_bindings, auth / "lumen-auth-redaction-audit.json")
    return commitment


def _attestation_digest(bundle: Path) -> str:
    envelope = json.loads((bundle / "cloud-build-attestation.json").read_text(encoding="utf-8"))["dsseEnvelope"]
    return _digest_bytes(json.dumps(envelope, separators=(",", ":"), sort_keys=True).encode("utf-8"))


def _write_green_bundle(root: Path) -> tuple[Path, str, str]:
    bundle = root / "bundle"
    auth = bundle / "kubernetes" / "auth"
    bucket = f"{EXPECTED_PROJECT}_cloudbuild"
    name = f"source/axiom-gcp-operator-{EXPECTED_RUN_ID}/source.tgz"
    generation = "73"
    archive_bytes = b"canonical-source-archive"
    archive_sha256 = _digest_bytes(archive_bytes)
    gcs_md5 = base64.b64encode(hashlib.md5(archive_bytes).digest()).decode("ascii")
    gcs_crc32c = base64.b64encode(CASE._crc32c(archive_bytes)).decode("ascii")
    source_commitment = CASE._source_object_commitment(bucket, name, generation, EXPECTED_GIT_SHA, archive_sha256, gcs_md5, gcs_crc32c)
    image_tag = f"example.test/lumen:{EXPECTED_GIT_SHA}-{EXPECTED_RUN_ID}"
    image_digest = "example.test/lumen@sha256:" + "b" * 64
    prefix = CASE._expected_gcs_prefix(EXPECTED_PROJECT, EXPECTED_RUN_ID)
    storage = {"bucket": bucket, "object": name, "generation": generation}
    _json(bundle / "run.json", {"schema": CASE.RUN_SCHEMA, "run_id": EXPECTED_RUN_ID, "git_sha": EXPECTED_GIT_SHA, "git_dirty": False, "project": EXPECTED_PROJECT, "image_provenance": "cloud-build", "cloud_build_id": "build-ec2879", "lumen_image_tag": image_tag, "source_archive_commitment": source_commitment, "source_archive_sha256": archive_sha256, "source_gcs_prefix": prefix, "started_at": "2026-08-02T00:00:00Z"})
    _json(bundle / "images.json", {"lumen": image_digest})
    _json(bundle / "cloud-build-submit.json", {"id": "build-ec2879", "projectId": EXPECTED_PROJECT, "status": "QUEUED", "source": {"storageSource": storage}, "createTime": "2026-08-02T00:00:01Z"})
    (bundle / "cloud-build-source-archive.bin").parent.mkdir(parents=True, exist_ok=True)
    (bundle / "cloud-build-source-archive.bin").write_bytes(archive_bytes)
    _json(bundle / "cloud-build-source-object.json", {"bucket": bucket, "name": name, "generation": generation, "size": str(len(archive_bytes)), "etag": "object-etag", "md5Hash": gcs_md5, "crc32c": gcs_crc32c, "metadata": {"archive-sha256": archive_sha256}, "timeCreated": "2026-08-02T00:00:02Z"})
    _json(bundle / "cloud-build-final.json", {"id": "build-ec2879", "projectId": EXPECTED_PROJECT, "status": "SUCCESS", "source": {"storageSource": storage}, "results": {"images": [{"name": image_tag, "digest": image_digest.rsplit("@", 1)[1]}]}, "finishTime": "2026-08-02T00:00:03Z"})
    attestation_payload = {
        "predicateType": "https://slsa.dev/provenance/v1",
        "subject": [{"name": image_digest.rsplit("@", 1)[0], "digest": {"sha256": image_digest.rsplit("@", 1)[1].removeprefix("sha256:")}}],
        "predicate": {
            "buildDefinition": {"resolvedDependencies": [{"uri": f"gs://{bucket}/{name}#{generation}", "digest": {"sha256": archive_sha256.removeprefix("sha256:")}}]},
            "runDetails": {"metadata": {"invocationId": "build-ec2879"}, "builder": {"id": "https://cloudbuild.googleapis.com/GoogleHostedWorker"}},
        },
    }
    attestation_payload_bytes = json.dumps(attestation_payload, separators=(",", ":"), sort_keys=True).encode("utf-8")
    payload_type = "application/vnd.in-toto+json"
    cloudbuild_key = _ed25519_public(_CLOUDBUILD_SEED)
    envelope = {
        "payloadType": payload_type,
        "payload": base64.b64encode(attestation_payload_bytes).decode("ascii"),
        "signatures": [{
            "keyid": "cloud-build-ed25519:" + _digest_bytes(cloudbuild_key).removeprefix("sha256:"),
            "sig": base64.b64encode(_ed25519_sign(_CLOUDBUILD_SEED, CASE._dsse_pae(payload_type, attestation_payload_bytes))).decode("ascii"),
        }],
    }
    attestation_digest = _digest_bytes(json.dumps(envelope, separators=(",", ":"), sort_keys=True).encode("utf-8"))
    _json(bundle / "cloud-build-attestation.json", {"dsseEnvelope": envelope})
    _json(bundle / "kubernetes" / "lumen" / "deployed-lumen-cr.json", {"apiVersion": "lumen.dev/v1alpha1", "kind": "Lumen", "metadata": {"namespace": CASE.AUTH_NAMESPACE, "name": CASE.LUMEN_INSTANCE, "uid": "00000000-0000-4000-8000-000000000001", "creationTimestamp": "2026-08-02T00:00:04Z"}, "spec": {"image": image_digest}, "status": {"phase": "Ready"}})
    _json(bundle / "lumen-auth-acceptance.json", {"schema": CASE.AUTH_SCHEMA, "status": "passed", "run_id": EXPECTED_RUN_ID, "audience": CASE.LUMEN_AUDIENCE, "issuers": [{"kind": "google-user", "kubernetes_username": _issuer_username("google-user"), "cluster_admin": True}, {"kind": "google-service-account", "kubernetes_username": _issuer_username("google-service-account"), "cluster_admin": False}]})
    acquisition_rows = []
    for kind, filename, acquisition_kind in (("google-user", "issuer-human-whoami.json", "ambient-kubeconfig"), ("google-service-account", "issuer-gsa-whoami.json", "gcloud-impersonated-service-account")):
        whoami = {"apiVersion": "authentication.k8s.io/v1", "kind": "SelfSubjectReview", "status": {"userInfo": {"username": _issuer_username(kind)}}}
        _json(auth / filename, whoami)
        acquisition_rows.append({"kind": kind, "acquisition_id": _acquisition_id(kind), "kubernetes_username": _issuer_username(kind), "authenticated_principal": _issuer_username(kind), "acquisition": {"kind": acquisition_kind, "source_commit": EXPECTED_GIT_SHA, "controller_challenge": EXPECTED_ISSUER_CHALLENGE}, "command": {"argv": ["kubectl", "auth", "whoami", "-o", "json"], "exit_code": 0, "controller_challenge": EXPECTED_ISSUER_CHALLENGE, "finished_at": "2026-08-02T00:00:05Z"}, "whoami": whoami, "observed_at": "2026-08-02T00:00:05Z"})
    _json(auth / "issuer-acquisitions.json", {"schema": CASE.ISSUER_ACQUISITIONS_SCHEMA, "run_id": EXPECTED_RUN_ID, "source_commit": EXPECTED_GIT_SHA, "controller_challenge": EXPECTED_ISSUER_CHALLENGE, "observed_at": "2026-08-02T00:00:05Z", "issuers": acquisition_rows})
    rendered_at = "2026-08-02T00:00:06Z"
    rendered_objects = _rendered_rbac()
    _json(auth / "rendered-rbac.json", {"schema": CASE.RENDERED_RBAC_SCHEMA, "run_id": EXPECTED_RUN_ID, "observed_at": rendered_at, "objects": rendered_objects})
    tokens = _token_rows()
    rows = _http_rows(tokens)
    _json(auth / "observations.json", {"schema": CASE.OBSERVATIONS_SCHEMA, "run_id": EXPECTED_RUN_ID, "context": {"namespace": CASE.AUTH_NAMESPACE, "client_namespace": CASE.CLIENT_NAMESPACE, "audience": CASE.LUMEN_AUDIENCE, "request_marker": f"{CASE.REQUEST_MARKER_PREFIX}{EXPECTED_RUN_ID}", "collections": {"granted": CASE.GRANTED_COLLECTION, "ungranted": CASE.UNGRANTED_COLLECTION}}, "issuers": [{"kind": kind, "kubernetes_username": _issuer_username(kind), "acquisition_id": _acquisition_id(kind)} for kind in CASE.TD_ISSUER_KINDS], "token_requests": tokens, "http_observations": rows})
    cli_argv = ["lumen", "query", "search", "--url", "http://127.0.0.1:17375", "--namespace", CASE.AUTH_NAMESPACE, "--client-sa", "auth-sibling", "--collection", CASE.GRANTED_COLLECTION, "--term", f"message={CASE.REQUEST_MARKER_PREFIX}{EXPECTED_RUN_ID}"]
    sibling_uid = "00000000-0000-4000-8007-000000000001"
    sibling_service_account = {"apiVersion": "v1", "kind": "ServiceAccount", "metadata": {"namespace": CASE.AUTH_NAMESPACE, "name": "auth-sibling", "uid": sibling_uid}}
    sibling_token_request = {
        "request": {"apiVersion": "authentication.k8s.io/v1", "kind": "TokenRequest", "metadata": {"namespace": CASE.AUTH_NAMESPACE, "name": "auth-sibling"}, "spec": {"audiences": [CASE.LUMEN_AUDIENCE], "expirationSeconds": CASE.MAX_KSA_TOKEN_LIFETIME_SECONDS}},
        "response": {"apiVersion": "v1", "kind": "Status", "status": "Failure", "reason": "Forbidden", "code": 403, "details": {"name": "auth-sibling", "kind": "serviceaccounts/token"}},
    }
    cli_failure = sibling_token_request["response"]
    cli_stderr = "forbidden: --subresource=token"
    _json(auth / "cli-sibling-mint-failure.json", {"schema": "axiom.gcp.lumen.auth.cli-sibling-failure.v2", "run_id": EXPECTED_RUN_ID, "issuer_kind": "google-service-account", "issuer_username": _issuer_username("google-service-account"), "issuer_acquisition_id": _acquisition_id("google-service-account"), "client_ksa": {"namespace": CASE.AUTH_NAMESPACE, "name": "auth-sibling"}, "sibling_service_account": sibling_service_account, "sibling_token_request": sibling_token_request, "exit_code": 1, "argv": cli_argv, "binary": {"source_commit": EXPECTED_GIT_SHA, "image_digest": image_digest, "sha256": EXPECTED_CLI_BINARY_DIGEST, "controller_image_binding": CASE._cli_binary_image_binding(image_digest, EXPECTED_CLI_BINARY_DIGEST)}, "raw_failure": cli_failure, "stderr": cli_stderr})
    controller_transcript = {"argv": cli_argv, "executable": {"path": "/usr/local/bin/lumen", "sha256": EXPECTED_CLI_BINARY_DIGEST, "image_digest": image_digest, "extracted_by": "controller-image-extraction"}, "issuer": {"kind": "google-service-account", "kubernetes_username": _issuer_username("google-service-account"), "acquisition_id": _acquisition_id("google-service-account")}, "sibling_service_account_uid": sibling_uid, "sibling_token_request": sibling_token_request, "exit_code": 1, "stdout": "", "stderr": cli_stderr, "raw_failure": cli_failure}
    controller_capture = {"schema": "axiom.lumen.ec.cli-controller-execution.v1", "run_id": EXPECTED_RUN_ID, "transcript": controller_transcript}
    controller_capture["signature"] = {"keyid": "controller-ed25519:" + _digest_bytes(_ed25519_public(_CONTROLLER_SEED)).removeprefix("sha256:"), "sig": base64.b64encode(_ed25519_sign(_CONTROLLER_SEED, json.dumps(controller_capture, separators=(",", ":"), sort_keys=True).encode("utf-8"))).decode("ascii")}
    _json(auth / "cli-controller-execution.json", controller_capture)
    writer = next(row for row in rows if row["id"] == "writer-search-before-revocation")
    writer_credential = writer["credential"]
    controller_binding = {"run_id": EXPECTED_RUN_ID, "git_sha": EXPECTED_GIT_SHA, "image_digest": image_digest}
    unbound_uid = next(item["metadata"]["uid"] for item in rendered_objects if item["kind"] == "RoleBinding" and item["metadata"]["name"] == "auth-unbound-lumen-access")
    _json(auth / "unbound-rolebinding-deletion.json", {"schema": "axiom.gcp.lumen.auth.unbound-deletion.v1", "run_id": EXPECTED_RUN_ID, "rendered_at": rendered_at, "deletion": _deletion("auth-unbound-lumen-access", "2026-08-02T00:00:07Z", unbound_uid)})
    revocation_review_uid = "00000000-0000-4000-8005-000000000001"
    revocation_sar_uid = "00000000-0000-4000-8006-000000000001"
    writer_subject = f"system:serviceaccount:{CASE.AUTH_NAMESPACE}:auth-writer"
    revocation_poll = {"run_id": EXPECTED_RUN_ID, "request_marker": f"{CASE.REQUEST_MARKER_PREFIX}{EXPECTED_RUN_ID}", "correlation_id": writer_credential["correlation_id"], "token_fingerprint": writer_credential["token_fingerprint"], "request": {"method": "POST", "path": f"/collections/{CASE.GRANTED_COLLECTION}/search", "resource": {"kind": "lumencollections", "namespace": CASE.AUTH_NAMESPACE, "name": CASE.GRANTED_COLLECTION}}, "observed_at": "2026-08-02T00:25:12Z", "response": {"status": 403, "body_sha256": _digest("revoked-writer")}, "token_review": {"apiVersion": "authentication.k8s.io/v1", "kind": "TokenReview", "metadata": {"uid": revocation_review_uid}, "spec": {"token_sha256": writer_credential["token_fingerprint"], "audiences": [CASE.LUMEN_AUDIENCE]}, "status": {"authenticated": True, "user": {"username": writer_subject}, "audiences": [CASE.LUMEN_AUDIENCE]}, "correlation_id": writer_credential["correlation_id"], "token_fingerprint": writer_credential["token_fingerprint"]}, "subject_access_review": {"apiVersion": "authorization.k8s.io/v1", "kind": "SubjectAccessReview", "metadata": {"uid": revocation_sar_uid}, "spec": {"user": writer_subject, "resourceAttributes": {"namespace": CASE.AUTH_NAMESPACE, "group": CASE.LUMEN_API_GROUP, "resource": "lumencollections", "name": CASE.GRANTED_COLLECTION, "verb": "get"}}, "status": {"allowed": False}}}
    _json(auth / "revocation-observations.json", {"schema": CASE.REVOCATION_SCHEMA, "run_id": EXPECTED_RUN_ID, "issuer_tokenrequest": {"pre_allow": {"controller_binding": controller_binding, "token_request": "google-service-account:auth-reader", "observed_at": next(row["issued_at"] for row in tokens if row["issuer_kind"] == "google-service-account" and row["client_ksa"]["name"] == "auth-reader" and row["audience"] == CASE.LUMEN_AUDIENCE), "status": "issued"}, "deletion": _deletion("auth-reader-token-issuer", "2026-08-02T00:20:00Z", "00000000-0000-4000-8000-000000000011"), "polls": [{"run_id": EXPECTED_RUN_ID, "issuer_kind": "google-service-account", "client_ksa": {"namespace": CASE.AUTH_NAMESPACE, "name": "auth-reader"}, "audience": CASE.LUMEN_AUDIENCE, "request": {"apiVersion": "authentication.k8s.io/v1", "kind": "TokenRequest", "namespace": CASE.AUTH_NAMESPACE, "serviceAccount": "auth-reader", "audience": CASE.LUMEN_AUDIENCE}, "observed_at": "2026-08-02T00:20:02Z", "response": {"kind": "Status", "status": "Failure", "reason": "Forbidden", "code": 403}}]}, "lumen_authorization": {"before_http_row": "writer-search-before-revocation", "pre_allow": {"controller_binding": controller_binding, "http_row_id": "writer-search-before-revocation", "observed_at": writer["observed_at"], "status": 200}, "deletion": _deletion("auth-writer-lumen-access", "2026-08-02T00:25:00Z", "00000000-0000-4000-8000-000000000012"), "polls": [revocation_poll]}})
    queries = []
    for cleanup_class, (api, resource, scope, argv, identity) in CASE.CLEANUP_EXPECTATIONS.items():
        identity = {key: value.format(run_id=EXPECTED_RUN_ID, gcs_prefix=prefix) for key, value in identity.items()}
        request = {"api": api, "resource": resource, "project": EXPECTED_PROJECT, "namespace": CASE.AUTH_NAMESPACE if scope == "namespace" else None, "run_selector": f"{CASE.RUN_LABEL_KEY}={EXPECTED_RUN_ID}", "image_tag": image_tag, "gcs_prefix": prefix, "identity": identity}
        command_argv = CASE._cleanup_argv(argv, EXPECTED_PROJECT, api, resource, request["namespace"], EXPECTED_RUN_ID, image_tag, prefix, identity)
        if command_argv[0] == "kubectl" and "name" in identity:
            native_resource = "namespaces" if resource == "namespaces" else "customresourcedefinitions.apiextensions.k8s.io"
            exit_code, stdout, stderr = 1, "", f'Error from server (NotFound): {native_resource} "{identity["name"]}" not found\\n'
        else:
            exit_code, stdout, stderr = 0, ("No resources found\\n" if command_argv[0] == "kubectl" else "" if resource == "objects" else "[]"), ""
        queries.append({"class": cleanup_class, "run_id": EXPECTED_RUN_ID, "request": request, "command": {"argv": command_argv, "context": request, "exit_code": exit_code, "stdout": stdout, "stderr": stderr}, "observed_at": "2026-08-02T00:30:02Z"})
    _json(auth / "cleanup-observations.json", {"schema": CASE.RESIDUE_SCHEMA, "run_id": EXPECTED_RUN_ID, "cleanup_completed_at": "2026-08-02T00:30:00Z", "queries": queries})
    _json(bundle / "cleanup.json", {"schema": "axiom.gcp.operator.cleanup.v1", "project_id": EXPECTED_PROJECT, "run_id": EXPECTED_RUN_ID, "verified_at": "2026-08-02T00:30:03Z", "status": "clean", "preserved": {"artifact_registry": True, "preexisting_apis": True}})
    credentials = {str(row["credential_path"]): _canary(f"token:{row['issuer_kind']}:{row['client_ksa']['name']}:{row['audience']}") for row in tokens}
    credentials.update({str(row["credential"]["credential_path"]): _canary(f"direct:{row['id']}") for row in rows if row["credential"]["kind"] in ("google-access-token", "google-id-token")})
    bindings = [
        {"path": row["credential_path"], "class": "kubernetes-service-account", "issuer_kind": row["issuer_kind"], "audience": row["audience"], "observation_id": row["correlation_id"], "fingerprint": row["token_fingerprint"]}
        for row in tokens
    ]
    bindings.extend(
        {"path": row["credential"]["credential_path"], "class": row["credential"]["kind"], "issuer_kind": row["issuer_kind"], "audience": row["credential"]["audience"], "observation_id": row["credential"]["correlation_id"], "fingerprint": row["credential"]["token_fingerprint"]}
        for row in rows if row["credential"]["kind"] in ("google-access-token", "google-id-token")
    )
    assert _attestation_digest(bundle) == attestation_digest
    return bundle, source_commitment, _write_redaction(bundle, credentials, bindings)


class TdSourceTests(unittest.TestCase):
    def test_canonical_td_and_complete_public_producer_set(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            digest = _write_td_source(root)
            CASE.verify_td_behavior_source(root, digest)
            CASE.verify_td_security_source(root, digest)
            incomplete = _write_td_source(root, targets=(CASE.TD_PUBLIC_BOUNDARIES[1],))
            with self.assertRaisesRegex(CASE.EvidenceError, "complete #2879 public producer target set"):
                CASE.verify_td_behavior_source(root, incomplete)

    def test_current_td_is_intentional_negative_baseline_and_env_cannot_redirect_root(self) -> None:
        repo_root = ROOT.parents[2]
        digest = CASE._digest_python_source_root(repo_root / CASE.TD_SOURCE_ROOT)
        with self.assertRaises(CASE.EvidenceError):
            CASE.verify_td_security_source(repo_root, digest)
        old = os.environ.get("LUMEN_EC_REPO_ROOT")
        os.environ["LUMEN_EC_REPO_ROOT"] = "/tmp/forged-td-root"
        try:
            self.assertEqual(RUNNER._repo_root(), ROOT.parents[2])
        finally:
            if old is None:
                os.environ.pop("LUMEN_EC_REPO_ROOT", None)
            else:
                os.environ["LUMEN_EC_REPO_ROOT"] = old


class RetainedGkeEvidenceTests(unittest.TestCase):
    def _behavior(self, bundle: Path, source: str, commitment: str, attestation_digest: str | None = None) -> None:
        CASE.verify_cb_behavior_evidence(bundle, EXPECTED_RUN_ID, EXPECTED_GIT_SHA, NOT_BEFORE, source, EXPECTED_PROJECT, commitment, EXPECTED_GOOGLE_USER, EXPECTED_GOOGLE_SERVICE_ACCOUNT, EXPECTED_ISSUER_CHALLENGE, attestation_digest or _attestation_digest(bundle), EXPECTED_CLOUDBUILD_PUBLIC_KEY)

    def _security(self, bundle: Path, source: str, commitment: str) -> None:
        CASE.verify_cb_security_evidence(bundle, EXPECTED_RUN_ID, EXPECTED_GIT_SHA, NOT_BEFORE, source, EXPECTED_PROJECT, commitment, EXPECTED_GOOGLE_USER, EXPECTED_GOOGLE_SERVICE_ACCOUNT, EXPECTED_ISSUER_CHALLENGE, EXPECTED_CLI_BINARY_DIGEST, _attestation_digest(bundle), EXPECTED_CLOUDBUILD_PUBLIC_KEY, EXPECTED_CONTROLLER_PUBLIC_KEY)

    def test_canonical_raw_bundle_passes_all_cb_cases(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle, source, commitment = _write_green_bundle(Path(temp))
            self._behavior(bundle, source, commitment)
            self._security(bundle, source, commitment)

    def test_raw_build_chain_and_deployed_cr_contradictions_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle, source, commitment = _write_green_bundle(Path(temp))
            final_path = bundle / "cloud-build-final.json"
            final = json.loads(final_path.read_text(encoding="utf-8"))
            final["results"]["images"][0]["digest"] = "sha256:" + "c" * 64
            _json(final_path, final)
            with self.assertRaisesRegex(CASE.EvidenceError, "exact Lumen result image digest"):
                self._behavior(bundle, source, commitment)
            bundle, source, commitment = _write_green_bundle(Path(temp) / "cr")
            cr_path = bundle / "kubernetes" / "lumen" / "deployed-lumen-cr.json"
            cr = json.loads(cr_path.read_text(encoding="utf-8"))
            cr["metadata"]["namespace"] = "forged"
            _json(cr_path, cr)
            with self.assertRaisesRegex(CASE.EvidenceError, "exact auth namespace/name/UID"):
                self._behavior(bundle, source, commitment)

    def test_closed_world_token_http_and_rendered_rbac_attacks_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle, source, commitment = _write_green_bundle(Path(temp))
            obs_path = bundle / "kubernetes" / "auth" / "observations.json"
            obs = json.loads(obs_path.read_text(encoding="utf-8"))
            obs["token_requests"].append(dict(obs["token_requests"][0]))
            _json(obs_path, obs)
            with self.assertRaisesRegex(CASE.EvidenceError, "unique correlation or token fingerprint"):
                self._behavior(bundle, source, commitment)
            bundle, source, commitment = _write_green_bundle(Path(temp) / "rbac")
            rbac_path = bundle / "kubernetes" / "auth" / "rendered-rbac.json"
            rbac = json.loads(rbac_path.read_text(encoding="utf-8"))
            issuer_role = next(item for item in rbac["objects"] if item["kind"] == "Role" and item["metadata"]["name"] == "auth-reader-token-issuer")
            issuer_role["rules"][0]["resourceNames"] = ["auth-sibling"]
            _json(rbac_path, rbac)
            with self.assertRaisesRegex(CASE.EvidenceError, "exactly its named ServiceAccount"):
                self._behavior(bundle, source, commitment)
            bundle, source, commitment = _write_green_bundle(Path(temp) / "unknown-http")
            obs_path = bundle / "kubernetes" / "auth" / "observations.json"
            obs = json.loads(obs_path.read_text(encoding="utf-8"))
            extra = dict(obs["http_observations"][0])
            extra["id"] = "unexpected-request"
            obs["http_observations"].append(extra)
            _json(obs_path, obs)
            with self.assertRaisesRegex(CASE.EvidenceError, "exact closed-world"):
                self._behavior(bundle, source, commitment)

    def test_credential_provenance_direct_google_and_tokenreview_attacks_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle, source, commitment = _write_green_bundle(Path(temp))
            path = bundle / "kubernetes" / "auth" / "observations.json"
            obs = json.loads(path.read_text(encoding="utf-8"))
            row = next(item for item in obs["http_observations"] if item["id"] == "reader-search-granted")
            row["credential"]["issuer_acquisition_id"] = "forged"
            _json(path, obs)
            with self.assertRaisesRegex(CASE.EvidenceError, "raw issuer acquisition"):
                self._behavior(bundle, source, commitment)
            bundle, source, commitment = _write_green_bundle(Path(temp) / "direct")
            path = bundle / "kubernetes" / "auth" / "observations.json"
            obs = json.loads(path.read_text(encoding="utf-8"))
            row = next(item for item in obs["http_observations"] if item["id"] == "google-access-token-refused")
            row["response"]["status"] = 200
            _json(path, obs)
            with self.assertRaisesRegex(CASE.EvidenceError, "actual expected response status"):
                self._security(bundle, source, commitment)
            bundle, source, commitment = _write_green_bundle(Path(temp) / "tokenreview")
            path = bundle / "kubernetes" / "auth" / "observations.json"
            obs = json.loads(path.read_text(encoding="utf-8"))
            row = next(item for item in obs["http_observations"] if item["id"] == "reader-search-granted")
            row["token_review"]["username"] = "system:serviceaccount:lumen:forged"
            _json(path, obs)
            with self.assertRaisesRegex(CASE.EvidenceError, "TokenReview request/response metadata"):
                self._behavior(bundle, source, commitment)

    def test_raw_cli_revocation_and_cleanup_identity_attacks_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle, source, commitment = _write_green_bundle(Path(temp))
            cli_path = bundle / "kubernetes" / "auth" / "cli-sibling-mint-failure.json"
            cli = json.loads(cli_path.read_text(encoding="utf-8"))
            cli["binary"]["sha256"] = "not-a-digest"
            _json(cli_path, cli)
            with self.assertRaisesRegex(CASE.EvidenceError, "CLI binary digest"):
                self._security(bundle, source, commitment)
            bundle, source, commitment = _write_green_bundle(Path(temp) / "revocation")
            revocation_path = bundle / "kubernetes" / "auth" / "revocation-observations.json"
            revocation = json.loads(revocation_path.read_text(encoding="utf-8"))
            revocation["issuer_tokenrequest"]["deletion"]["audit"]["objectRef"]["uid"] = "forged"
            _json(revocation_path, revocation)
            with self.assertRaisesRegex(CASE.EvidenceError, "raw audit UID"):
                self._security(bundle, source, commitment)
            bundle, source, commitment = _write_green_bundle(Path(temp) / "cleanup")
            cleanup_path = bundle / "kubernetes" / "auth" / "cleanup-observations.json"
            cleanup = json.loads(cleanup_path.read_text(encoding="utf-8"))
            cleanup["queries"][1]["request"] = cleanup["queries"][0]["request"]
            _json(cleanup_path, cleanup)
            with self.assertRaisesRegex(CASE.EvidenceError, "exact EC-owned query identity"):
                self._security(bundle, source, commitment)

    def test_controller_committed_redaction_destruction_and_terminal_corpus_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle, source, commitment = _write_green_bundle(Path(temp))
            live_path = bundle / "kubernetes" / "auth" / "lumen-auth-live-redaction-scan.json"
            live = json.loads(live_path.read_text(encoding="utf-8"))
            live["credential_digests"] = {"tokens/forged.token": "sha256:" + "f" * 64}
            live["credential_paths"] = ["tokens/forged.token"]
            _json(live_path, live)
            with self.assertRaisesRegex(CASE.EvidenceError, "redaction commitment"):
                self._security(bundle, source, commitment)
            bundle, source, commitment = _write_green_bundle(Path(temp) / "ordering")
            live_path = bundle / "kubernetes" / "auth" / "lumen-auth-live-redaction-scan.json"
            live = json.loads(live_path.read_text(encoding="utf-8"))
            live["actions"][1]["sequence"] = 1
            _json(live_path, live)
            with self.assertRaisesRegex(CASE.EvidenceError, "credential directory destruction"):
                self._security(bundle, source, commitment)
            bundle, source, commitment = _write_green_bundle(Path(temp) / "append")
            (bundle / "after-audit.txt").write_text("post-audit addition", encoding="utf-8")
            with self.assertRaisesRegex(CASE.EvidenceError, "immutable terminal evidence corpus"):
                self._security(bundle, source, commitment)

    def test_r4_raw_binding_false_greens_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle, source, commitment = _write_green_bundle(Path(temp))
            gcs_path = bundle / "cloud-build-source-object.json"
            gcs = json.loads(gcs_path.read_text(encoding="utf-8"))
            gcs["md5Hash"] = ""
            _json(gcs_path, gcs)
            with self.assertRaisesRegex(CASE.EvidenceError, "md5Hash"):
                self._behavior(bundle, source, commitment)
            bundle, source, commitment = _write_green_bundle(Path(temp) / "issuer")
            issuer_path = bundle / "kubernetes" / "auth" / "issuer-acquisitions.json"
            issuers = json.loads(issuer_path.read_text(encoding="utf-8"))
            issuers["issuers"][1]["authenticated_principal"] = "system:serviceaccount:lumen:forged"
            _json(issuer_path, issuers)
            with self.assertRaisesRegex(CASE.EvidenceError, "controller-challenged"):
                self._behavior(bundle, source, commitment)
            bundle, source, commitment = _write_green_bundle(Path(temp) / "token")
            observation_path = bundle / "kubernetes" / "auth" / "observations.json"
            observations = json.loads(observation_path.read_text(encoding="utf-8"))
            observations["token_requests"][0]["response"]["metadata"]["uid"] = "contradictory"
            _json(observation_path, observations)
            with self.assertRaisesRegex(CASE.EvidenceError, "TokenRequest record lacks exact raw"):
                self._behavior(bundle, source, commitment)
            bundle, source, commitment = _write_green_bundle(Path(temp) / "unknown")
            observation_path = bundle / "kubernetes" / "auth" / "observations.json"
            observations = json.loads(observation_path.read_text(encoding="utf-8"))
            observations["unreviewed"] = {"status": "passed"}
            _json(observation_path, observations)
            with self.assertRaisesRegex(CASE.EvidenceError, "unknown or missing evidence sections"):
                self._behavior(bundle, source, commitment)
            bundle, source, commitment = _write_green_bundle(Path(temp) / "cli")
            cli_path = bundle / "kubernetes" / "auth" / "cli-sibling-mint-failure.json"
            cli = json.loads(cli_path.read_text(encoding="utf-8"))
            cli["binary"]["sha256"] = "sha256:" + "a" * 64
            _json(cli_path, cli)
            with self.assertRaisesRegex(CASE.EvidenceError, "controller-extracted"):
                self._security(bundle, source, commitment)
            bundle, source, commitment = _write_green_bundle(Path(temp) / "revocation")
            revocation_path = bundle / "kubernetes" / "auth" / "revocation-observations.json"
            revocation = json.loads(revocation_path.read_text(encoding="utf-8"))
            revocation["lumen_authorization"]["pre_allow"]["observed_at"] = "2026-08-02T00:26:00Z"
            _json(revocation_path, revocation)
            with self.assertRaisesRegex(CASE.EvidenceError, "strictly before deletion"):
                self._security(bundle, source, commitment)
            bundle, source, commitment = _write_green_bundle(Path(temp) / "cleanup")
            cleanup_path = bundle / "cleanup.json"
            cleanup = json.loads(cleanup_path.read_text(encoding="utf-8"))
            cleanup["verified_at"] = "2026-08-02T00:30:01Z"
            _json(cleanup_path, cleanup)
            with self.assertRaisesRegex(CASE.EvidenceError, "cleanup residue query"):
                self._security(bundle, source, commitment)

    def test_r5_controller_bound_raw_evidence_false_greens_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle, source, commitment = _write_green_bundle(Path(temp))
            gcs_path = bundle / "cloud-build-source-object.json"
            gcs = json.loads(gcs_path.read_text(encoding="utf-8"))
            gcs["md5Hash"] = "Zm9yZ2Vk"
            _json(gcs_path, gcs)
            with self.assertRaisesRegex(CASE.EvidenceError, "GCS source object MD5"):
                self._behavior(bundle, source, commitment)

            bundle, source, commitment = _write_green_bundle(Path(temp) / "unsigned")
            attestation_path = bundle / "cloud-build-attestation.json"
            attestation = json.loads(attestation_path.read_text(encoding="utf-8"))
            attestation["dsseEnvelope"]["signatures"] = []
            _json(attestation_path, attestation)
            with self.assertRaisesRegex(CASE.EvidenceError, "signed DSSE"):
                self._behavior(bundle, source, commitment, _attestation_digest(bundle))

            bundle, source, commitment = _write_green_bundle(Path(temp) / "duplicate-issuer")
            issuer_path = bundle / "kubernetes" / "auth" / "issuer-acquisitions.json"
            issuer_evidence = json.loads(issuer_path.read_text(encoding="utf-8"))
            issuer_evidence["issuers"].append(dict(issuer_evidence["issuers"][0]))
            _json(issuer_path, issuer_evidence)
            with self.assertRaisesRegex(CASE.EvidenceError, "closed two-issuer"):
                self._behavior(bundle, source, commitment)

            bundle, source, commitment = _write_green_bundle(Path(temp) / "direct-as-ksa")
            live_path = bundle / "kubernetes" / "auth" / "lumen-auth-live-redaction-scan.json"
            live = json.loads(live_path.read_text(encoding="utf-8"))
            direct = next(item for item in live["credential_bindings"] if item["class"] == "google-access-token")
            direct["class"] = "kubernetes-service-account"
            forged_commitment = CASE._redaction_commitment(EXPECTED_RUN_ID, live["credential_digests"], live["credential_bindings"])
            live["controller_commitment"] = forged_commitment
            _json(live_path, live)
            with self.assertRaisesRegex(CASE.EvidenceError, "one-to-one bound"):
                self._security(bundle, source, forged_commitment)

            bundle, source, commitment = _write_green_bundle(Path(temp) / "unbound")
            (bundle / "kubernetes" / "auth" / "unbound-rolebinding-deletion.json").unlink()
            with self.assertRaisesRegex(CASE.EvidenceError, "unbound"):
                self._security(bundle, source, commitment)

            bundle, source, commitment = _write_green_bundle(Path(temp) / "sar")
            revocation_path = bundle / "kubernetes" / "auth" / "revocation-observations.json"
            revocations = json.loads(revocation_path.read_text(encoding="utf-8"))
            revocations["lumen_authorization"]["polls"][0].pop("subject_access_review")
            _json(revocation_path, revocations)
            with self.assertRaisesRegex(CASE.EvidenceError, "TokenReview and SubjectAccessReview"):
                self._security(bundle, source, commitment)

            bundle, source, commitment = _write_green_bundle(Path(temp) / "redaction-bound")
            live_path = bundle / "kubernetes" / "auth" / "lumen-auth-live-redaction-scan.json"
            live = json.loads(live_path.read_text(encoding="utf-8"))
            scanned = datetime.fromisoformat(live["actions"][0]["observed_at"].replace("Z", "+00:00"))
            live["actions"][1]["observed_at"] = (scanned + timedelta(seconds=6)).isoformat().replace("+00:00", "Z")
            _json(live_path, live)
            with self.assertRaisesRegex(CASE.EvidenceError, "five-second"):
                self._security(bundle, source, commitment)

            bundle, source, commitment = _write_green_bundle(Path(temp) / "terminal-order")
            proof_path = bundle / "kubernetes" / "auth" / "lumen-auth-redaction-audit.json"
            proof = json.loads(proof_path.read_text(encoding="utf-8"))
            proof["actions"][0]["observed_at"] = "2026-08-02T00:30:03Z"
            _json(proof_path, proof)
            with self.assertRaisesRegex(CASE.EvidenceError, "does not follow cleanup"):
                self._security(bundle, source, commitment)

            bundle, source, commitment = _write_green_bundle(Path(temp) / "cli-executable")
            capture_path = bundle / "kubernetes" / "auth" / "cli-controller-execution.json"
            capture = json.loads(capture_path.read_text(encoding="utf-8"))
            capture["transcript"]["executable"]["path"] = "/tmp/arbitrary-lumen"
            unsigned = {"schema": capture["schema"], "run_id": capture["run_id"], "transcript": capture["transcript"]}
            capture["signature"]["sig"] = base64.b64encode(_ed25519_sign(_CONTROLLER_SEED, json.dumps(unsigned, separators=(",", ":"), sort_keys=True).encode("utf-8"))).decode("ascii")
            _json(capture_path, capture)
            with self.assertRaisesRegex(CASE.EvidenceError, "does not bind the extracted bytes"):
                self._security(bundle, source, commitment)

            bundle, source, commitment = _write_green_bundle(Path(temp) / "cleanup-argv")
            cleanup_path = bundle / "kubernetes" / "auth" / "cleanup-observations.json"
            cleanup = json.loads(cleanup_path.read_text(encoding="utf-8"))
            cleanup["queries"][0]["command"]["argv"].append("--api")
            _json(cleanup_path, cleanup)
            with self.assertRaisesRegex(CASE.EvidenceError, "per-tool argv/exit/stdout/stderr"):
                self._security(bundle, source, commitment)

    def test_r6_authenticated_dsse_direct_google_cleanup_and_cli_false_greens_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle, source, commitment = _write_green_bundle(Path(temp))
            attestation_path = bundle / "cloud-build-attestation.json"
            attestation = json.loads(attestation_path.read_text(encoding="utf-8"))
            attestation["dsseEnvelope"]["signatures"][0]["sig"] = base64.b64encode(b"x" * 64).decode("ascii")
            _json(attestation_path, attestation)
            with self.assertRaisesRegex(CASE.EvidenceError, "cryptographically verify"):
                self._behavior(bundle, source, commitment, _attestation_digest(bundle))

            bundle, source, commitment = _write_green_bundle(Path(temp) / "archive")
            archive_path = bundle / "cloud-build-source-archive.bin"
            archive_path.write_bytes(b"forged-controller-archive")
            with self.assertRaisesRegex(CASE.EvidenceError, "archive bytes contradict"):
                self._behavior(bundle, source, commitment)

            bundle, source, commitment = _write_green_bundle(Path(temp) / "crc")
            gcs_path = bundle / "cloud-build-source-object.json"
            gcs = json.loads(gcs_path.read_text(encoding="utf-8"))
            gcs["crc32c"] = "not-base64"
            _json(gcs_path, gcs)
            with self.assertRaisesRegex(CASE.EvidenceError, "CRC32C"):
                self._behavior(bundle, source, commitment)

            bundle, source, commitment = _write_green_bundle(Path(temp) / "direct-subject")
            observations_path = bundle / "kubernetes" / "auth" / "observations.json"
            observations = json.loads(observations_path.read_text(encoding="utf-8"))
            direct = next(row for row in observations["http_observations"] if row["id"] == "google-access-token-refused")
            direct.update({
                "token_review": {"status": {"authenticated": True}},
                "subject_access_review": {"status": {"allowed": True}},
                "lumen_audit": {"subject": "system:serviceaccount:lumen:auth-reader"},
            })
            _json(observations_path, observations)
            with self.assertRaisesRegex(CASE.EvidenceError, "direct Google rejection must not retain"):
                self._security(bundle, source, commitment)

            bundle, source, commitment = _write_green_bundle(Path(temp) / "duplicate-cleanup")
            cleanup_path = bundle / "kubernetes" / "auth" / "cleanup-observations.json"
            cleanup = json.loads(cleanup_path.read_text(encoding="utf-8"))
            cleanup["queries"].append(dict(cleanup["queries"][0]))
            _json(cleanup_path, cleanup)
            with self.assertRaisesRegex(CASE.EvidenceError, "exactly one raw query"):
                self._security(bundle, source, commitment)

            bundle, source, commitment = _write_green_bundle(Path(temp) / "synthetic-cleanup")
            cleanup_path = bundle / "kubernetes" / "auth" / "cleanup-observations.json"
            cleanup = json.loads(cleanup_path.read_text(encoding="utf-8"))
            cleanup["queries"][0]["response"] = {"items": []}
            _json(cleanup_path, cleanup)
            with self.assertRaisesRegex(CASE.EvidenceError, "synthetic summary"):
                self._security(bundle, source, commitment)

            bundle, source, commitment = _write_green_bundle(Path(temp) / "residue-stdout")
            cleanup_path = bundle / "kubernetes" / "auth" / "cleanup-observations.json"
            cleanup = json.loads(cleanup_path.read_text(encoding="utf-8"))
            command = cleanup["queries"][0]["command"]
            command["stdout"] = json.dumps({"apiVersion": cleanup["queries"][0]["request"]["api"], "kind": "NamespaceList", "items": [{"metadata": {"name": "lumen"}}]}, separators=(",", ":"), sort_keys=True)
            _json(cleanup_path, cleanup)
            with self.assertRaisesRegex(CASE.EvidenceError, "raw kubectl table/NotFound output found residue"):
                self._security(bundle, source, commitment)

            bundle, source, commitment = _write_green_bundle(Path(temp) / "self-assert-process")
            cli_path = bundle / "kubernetes" / "auth" / "cli-sibling-mint-failure.json"
            cli = json.loads(cli_path.read_text(encoding="utf-8"))
            cli["process"] = {"producer": "self-asserted"}
            _json(cli_path, cli)
            with self.assertRaisesRegex(CASE.EvidenceError, "producer fields"):
                self._security(bundle, source, commitment)

            bundle, source, commitment = _write_green_bundle(Path(temp) / "forged-cli-capture")
            capture_path = bundle / "kubernetes" / "auth" / "cli-controller-execution.json"
            capture = json.loads(capture_path.read_text(encoding="utf-8"))
            capture["signature"]["sig"] = base64.b64encode(b"y" * 64).decode("ascii")
            _json(capture_path, capture)
            with self.assertRaisesRegex(CASE.EvidenceError, "cryptographically verify"):
                self._security(bundle, source, commitment)

    def test_ed25519_verifier_accepts_rfc8032_vector_and_rejects_tampering(self) -> None:
        public_key = bytes.fromhex("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")
        signature = bytes.fromhex("e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e06522490155" "5fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b")
        self.assertTrue(CASE._verify_ed25519(public_key, b"", signature))
        self.assertFalse(CASE._verify_ed25519(public_key, b"forged", signature))
        identity = b"\x01" + b"\0" * 31
        self.assertFalse(CASE._verify_ed25519(identity, b"", identity + b"\0" * 32))
        self.assertFalse(CASE._verify_ed25519(public_key, b"", identity + b"\0" * 32))
        # x=0 must be encoded with sign=0; x=0/sign=1 is a non-canonical
        # alternate encoding, not a valid verifier input.
        noncanonical_x_zero = b"\x01" + b"\0" * 30 + b"\x80"
        self.assertFalse(CASE._verify_ed25519(noncanonical_x_zero, b"", identity + b"\0" * 32))
        # y=-1 is a canonical order-two point, but it is outside the prime
        # Ed25519 subgroup and cannot authenticate a controller transcript.
        small_order = (CASE._ED25519_Q - 1).to_bytes(32, "little")
        self.assertFalse(CASE._verify_ed25519(small_order, b"", identity + b"\0" * 32))

    def test_r7_native_cleanup_negative_auth_and_sibling_probes_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            bundle, source, commitment = _write_green_bundle(Path(temp))
            cleanup_path = bundle / "kubernetes" / "auth" / "cleanup-observations.json"
            cleanup = json.loads(cleanup_path.read_text(encoding="utf-8"))
            gcloud_query = next(query for query in cleanup["queries"] if query["class"] == "lumen-image-tag")
            gcloud_query["command"]["argv"] = [arg for arg in gcloud_query["command"]["argv"] if not arg.startswith("--filter=tags:")]
            _json(cleanup_path, cleanup)
            with self.assertRaisesRegex(CASE.EvidenceError, "per-tool argv/exit/stdout/stderr"):
                self._security(bundle, source, commitment)

            bundle, source, commitment = _write_green_bundle(Path(temp) / "wrong-audience")
            observations_path = bundle / "kubernetes" / "auth" / "observations.json"
            observations = json.loads(observations_path.read_text(encoding="utf-8"))
            wrong_audience = next(row for row in observations["http_observations"] if row["id"] == "wrong-audience-refused")
            wrong_audience["token_review"] = {"status": {"authenticated": True}}
            _json(observations_path, observations)
            with self.assertRaisesRegex(CASE.EvidenceError, "HTTP row contains unknown"):
                self._security(bundle, source, commitment)

            bundle, source, commitment = _write_green_bundle(Path(temp) / "anonymous")
            observations_path = bundle / "kubernetes" / "auth" / "observations.json"
            observations = json.loads(observations_path.read_text(encoding="utf-8"))
            anonymous = next(row for row in observations["http_observations"] if row["id"] == "anonymous-refused")
            anonymous["lumen_audit"] = {"subject": "system:serviceaccount:lumen:auth-reader"}
            _json(observations_path, observations)
            with self.assertRaisesRegex(CASE.EvidenceError, "HTTP row contains unknown"):
                self._security(bundle, source, commitment)

            bundle, source, commitment = _write_green_bundle(Path(temp) / "sibling")
            sibling_path = bundle / "kubernetes" / "auth" / "cli-sibling-mint-failure.json"
            sibling = json.loads(sibling_path.read_text(encoding="utf-8"))
            sibling["sibling_service_account"]["metadata"].pop("uid")
            _json(sibling_path, sibling)
            with self.assertRaisesRegex(CASE.EvidenceError, "existing ServiceAccount UID"):
                self._security(bundle, source, commitment)


class RedactionAuditorTests(unittest.TestCase):
    def test_recreated_directory_and_base64_credential_leak_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            bundle = root / "bundle"
            bundle.mkdir()
            credentials = root / "credentials"
            credentials.mkdir()
            value = b"controller-held-complete-credential-bytes"
            expected = {"gsa.token": value}
            bindings = [{"path": "gsa.token", "class": "google-access-token", "issuer_kind": "google-service-account", "audience": None, "observation_id": "corr-redaction-audit", "fingerprint": _digest_bytes(value)}]
            (credentials / "gsa.token").write_bytes(value)
            live = bundle / "live.json"
            AUDITOR.scan_live_credentials(credentials, EXPECTED_RUN_ID, expected, bindings, live)
            credentials.mkdir()
            (credentials / "gsa.token").write_bytes(value)
            with self.assertRaisesRegex(AUDITOR.RedactionAuditError, "recreated"):
                AUDITOR.audit_terminal(bundle, credentials, live, EXPECTED_RUN_ID, expected, bindings, bundle / "audit.json")
            shutil.rmtree(credentials)
            (bundle / "encoded.json").write_text(json.dumps({"leak": base64.b64encode(value).decode()}), encoding="utf-8")
            with self.assertRaisesRegex(AUDITOR.RedactionAuditError, "credential or encoding"):
                AUDITOR.audit_terminal(bundle, credentials, live, EXPECTED_RUN_ID, expected, bindings, bundle / "audit.json")
            (bundle / "encoded.json").unlink()
            (bundle / "access.json").write_text(json.dumps({"access_token": base64.b64encode(value).decode()}), encoding="utf-8")
            with self.assertRaisesRegex(AUDITOR.RedactionAuditError, "credential"):
                AUDITOR.audit_terminal(bundle, credentials, live, EXPECTED_RUN_ID, expected, bindings, bundle / "audit.json")
            (bundle / "access.json").unlink()
            (bundle / "url-safe.json").write_text(json.dumps({"leak": base64.urlsafe_b64encode(value).decode().rstrip("=")}), encoding="utf-8")
            with self.assertRaisesRegex(AUDITOR.RedactionAuditError, "credential or encoding"):
                AUDITOR.audit_terminal(bundle, credentials, live, EXPECTED_RUN_ID, expected, bindings, bundle / "audit.json")
            (bundle / "url-safe.json").unlink()
            (bundle / "controller-canary-retained.json").write_bytes(value)
            with self.assertRaisesRegex(AUDITOR.RedactionAuditError, "credential or encoding"):
                AUDITOR.audit_terminal(bundle, credentials, live, EXPECTED_RUN_ID, expected, bindings, bundle / "audit.json")

    def test_controller_canary_input_is_fd_only_and_never_a_retained_path(self) -> None:
        value = b"controller-fd-only-credential"
        payload = json.dumps({
            "credentials": {"tokens/fd.token": base64.b64encode(value).decode("ascii")},
            "bindings": [{"path": "tokens/fd.token", "class": "google-access-token", "issuer_kind": "google-service-account", "audience": None, "observation_id": "corr-fd-only-input", "fingerprint": _digest_bytes(value)}],
        })
        read_fd, write_fd = os.pipe()
        try:
            os.write(write_fd, payload.encode("utf-8"))
        finally:
            os.close(write_fd)
        try:
            credentials, bindings = AUDITOR._read_controller_credentials_fd(read_fd)
        finally:
            os.close(read_fd)
        self.assertEqual(credentials, {"tokens/fd.token": value})
        self.assertEqual(bindings[0]["fingerprint"], _digest_bytes(value))
        self.assertNotIn("--controller-canary-file", AUDITOR_PATH.read_text(encoding="utf-8"))


class RunnerProtocolTests(unittest.TestCase):
    def test_manifest_has_the_four_stable_td_cb_dimensions(self) -> None:
        manifest = (ROOT / "pyproject.toml").read_text(encoding="utf-8")
        sections = re.findall(r"\[\[tool\.aw\.python-ec\.cases\]\](.*?)(?=\n\[\[|\Z)", manifest, flags=re.DOTALL)
        observed = {(re.search(r'^id = "([^"]+)"$', section, flags=re.MULTILINE).group(1), re.search(r'^applicability = "([^"]+)"$', section, flags=re.MULTILINE).group(1), re.search(r'^dimension = "([^"]+)"$', section, flags=re.MULTILINE).group(1)) for section in sections}
        # Scoped to the #2879 rows. This manifest is the whole project's case
        # inventory, so asserting equality against it made every unrelated
        # contract landing in apps/lumen fail here. Restricting to the
        # gke-ksa-rbac family still catches a missing, retyped, or extra #2879
        # case; whether some *other* case is declared is the inventory gate's
        # question, not this file's.
        self.assertEqual({row for row in observed if row[0].startswith("gke-ksa-rbac-")}, {("gke-ksa-rbac-td-behavior", "td", "behavior"), ("gke-ksa-rbac-td-security", "td", "security"), ("gke-ksa-rbac-cb-behavior", "cb", "behavior"), ("gke-ksa-rbac-cb-security", "cb", "security")})

    def test_list_and_canonical_case_envelopes_are_machine_readable(self) -> None:
        listed = subprocess.run([sys.executable, "-I", str(RUNNER_PATH), "--list"], check=False, capture_output=True, encoding="utf-8")
        self.assertEqual(listed.returncode, 0, listed.stderr)
        self.assertEqual([case["id"] for case in json.loads(listed.stdout)["cases"]], list(RUNNER.CASE_IDS))
        with tempfile.TemporaryDirectory() as temp:
            work = Path(temp)
            repo = work / "repo"
            copied_src = repo / "apps" / "lumen" / "external-contracts" / "src"
            shutil.copytree(ROOT / "src", copied_src)
            td_digest = _write_td_source(repo)
            bundle, source, commitment = _write_green_bundle(work / "cb")
            base = {**os.environ, "AW_PYTHON_ARTIFACT_PROTOCOL": RUNNER.PROTOCOL, "AW_PYTHON_ARTIFACT_SOURCE_DIGEST": "sha256:" + "1" * 64, "AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST": "sha256:" + "2" * 64}
            runner = copied_src / "runner.py"
            cb = {"LUMEN_EC_RETAINED_BUNDLE": str(bundle), "LUMEN_EC_EXPECTED_RUN_ID": EXPECTED_RUN_ID, "LUMEN_EC_EXPECTED_GIT_SHA": EXPECTED_GIT_SHA, "LUMEN_EC_NOT_BEFORE": "2026-08-01T00:00:00Z", "LUMEN_EC_EXPECTED_SOURCE_ARCHIVE_COMMITMENT": source, "LUMEN_EC_EXPECTED_GCP_PROJECT": EXPECTED_PROJECT, "LUMEN_EC_EXPECTED_REDACTION_COMMITMENT": commitment, "LUMEN_EC_EXPECTED_GOOGLE_USER_PRINCIPAL": EXPECTED_GOOGLE_USER, "LUMEN_EC_EXPECTED_GOOGLE_SERVICE_ACCOUNT_PRINCIPAL": EXPECTED_GOOGLE_SERVICE_ACCOUNT, "LUMEN_EC_EXPECTED_ISSUER_CHALLENGE": EXPECTED_ISSUER_CHALLENGE, "LUMEN_EC_EXPECTED_CLI_BINARY_DIGEST": EXPECTED_CLI_BINARY_DIGEST, "LUMEN_EC_EXPECTED_ATTESTATION_DSSE_DIGEST": _attestation_digest(bundle), "LUMEN_EC_TRUSTED_CLOUDBUILD_ED25519_PUBLIC_KEY": EXPECTED_CLOUDBUILD_PUBLIC_KEY, "LUMEN_EC_TRUSTED_CONTROLLER_ED25519_PUBLIC_KEY": EXPECTED_CONTROLLER_PUBLIC_KEY}
            commands = [("gke-ksa-rbac-td-behavior", {"LUMEN_EC_EXPECTED_TD_SOURCE_DIGEST": td_digest}), ("gke-ksa-rbac-td-security", {"LUMEN_EC_EXPECTED_TD_SOURCE_DIGEST": td_digest}), ("gke-ksa-rbac-cb-behavior", cb), ("gke-ksa-rbac-cb-security", cb)]
            for index, (case_id, extra) in enumerate(commands):
                completed = subprocess.run([sys.executable, "-I", str(runner), case_id], check=False, capture_output=True, encoding="utf-8", env={**base, **extra, "AW_PYTHON_ARTIFACT_EVIDENCE_DIR": str(work / f"result-{index}")})
                self.assertEqual(completed.returncode, 0, completed.stderr)
                envelope = json.loads(completed.stdout)
                self.assertEqual(envelope["status"], "passed")
                self.assertEqual(envelope["cases"][0]["id"], case_id)

    def test_missing_cb_context_is_machine_readable_failure(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            completed = subprocess.run([sys.executable, "-I", str(RUNNER_PATH), "gke-ksa-rbac-cb-security"], check=False, capture_output=True, encoding="utf-8", env={**os.environ, "AW_PYTHON_ARTIFACT_PROTOCOL": RUNNER.PROTOCOL, "AW_PYTHON_ARTIFACT_SOURCE_DIGEST": "sha256:" + "1" * 64, "AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST": "sha256:" + "2" * 64, "AW_PYTHON_ARTIFACT_EVIDENCE_DIR": str(Path(temp) / "result")})
            self.assertEqual(completed.returncode, 1)
            self.assertEqual(json.loads(completed.stdout)["status"], "failed")


if __name__ == "__main__":
    unittest.main()
