#!/usr/bin/env python3
"""Create a public, redacted Tape GKE receipt from retained harness evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path
from typing import Any

CANDIDATE_SCHEMA = "cclab.tape.candidate-manifest.v1"
RECEIPT_SCHEMA = "tape.gke-release-receipt/v1"
REPOSITORY = "chrischeng-c4/axiom"
IMAGE_REPOSITORY = "ghcr.io/chrischeng-c4/tape"
WORKFLOW_REF = (
    "chrischeng-c4/axiom/.github/workflows/"
    "tape-release-candidate.yml@refs/heads/main"
)
FINAL_JOBS = {
    "identity": "success",
    "build": "success",
    "tape-release-gates": "success",
    "manifest": "success",
    "ghcr-image-and-attest": "success",
    "verify-candidate": "success",
    "verify-libraries": "success",
    "kind-amd64": "success",
    "kind-arm64": "success",
    "result": "success",
}
PASS_FIELDS = (
    "operator_reconcile_1x1",
    "append_replay_lifecycle",
    "subscription_pull_ack_cursor",
    "subscription_lag_gauge",
    "pod_restart_data_retention",
    "gcs_backup",
    "cold_restore_from_backup",
    "bootstrap_seed_uri_restart",
    "seed_cleared_rolling_restart_retention",
    "post_failover_write_committed",
)
TAPE_KEYS = set(PASS_FIELDS) | {
    "schema",
    "gcs_object",
    "gcs_object_bytes",
    "topology_1_to_3",
    "raft_failover",
}


def fail(message: str) -> None:
    raise SystemExit(f"GKE receipt refused: {message}")


def sha256(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def load_json(path: Path, label: str) -> tuple[bytes, dict[str, Any]]:
    if not path.is_file() or path.is_symlink():
        fail(f"{label} must be a regular file")
    raw = path.read_bytes()
    try:
        value = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        fail(f"{label} is not valid JSON: {error}")
    if not isinstance(value, dict):
        fail(f"{label} must be a JSON object")
    return raw, value


def require_digest(value: object, label: str) -> str:
    if not isinstance(value, str) or not re.fullmatch(r"sha256:[0-9a-f]{64}", value):
        fail(f"{label} is not an immutable SHA-256 digest")
    return value


def validate_candidate(raw: bytes, candidate: dict[str, Any]) -> dict[str, Any]:
    expected_keys = {
        "schema",
        "repository",
        "workflow_path",
        "workflow_id",
        "run_id",
        "run_attempt",
        "run_url",
        "source_ref",
        "workflow_ref",
        "commit",
        "version",
        "tag",
        "candidate_tag",
        "pr",
        "image",
        "artifacts",
        "sboms",
        "jobs",
    }
    if set(candidate) != expected_keys:
        fail("candidate manifest keys changed")
    if candidate.get("schema") != CANDIDATE_SCHEMA:
        fail("candidate manifest schema changed")
    if candidate.get("repository") != REPOSITORY:
        fail("candidate repository changed")
    version = candidate.get("version")
    commit = candidate.get("commit")
    run_id = candidate.get("run_id")
    run_attempt = candidate.get("run_attempt")
    if not isinstance(version, str) or not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version):
        fail("candidate version is invalid")
    if not isinstance(commit, str) or not re.fullmatch(r"[0-9a-f]{40}", commit):
        fail("candidate commit is invalid")
    if not isinstance(run_id, str) or not run_id.isdigit():
        fail("candidate run ID is invalid")
    if not isinstance(run_attempt, str) or not run_attempt.isdigit():
        fail("candidate run attempt is invalid")
    if candidate.get("tag") != f"tape@{version}":
        fail("candidate release tag changed")
    if candidate.get("candidate_tag") != f"release-candidate-{run_id}-{run_attempt}":
        fail("candidate image tag is not run scoped")
    if candidate.get("source_ref") != "refs/heads/main":
        fail("candidate was not built from main")
    if candidate.get("workflow_ref") != WORKFLOW_REF:
        fail("candidate workflow identity changed")
    if candidate.get("jobs") != FINAL_JOBS:
        fail("candidate does not bind every required successful job")
    image = candidate.get("image")
    if not isinstance(image, dict) or set(image) != {
        "repository",
        "root_digest",
        "amd64_digest",
        "arm64_digest",
    }:
        fail("candidate image shape changed")
    if image.get("repository") != IMAGE_REPOSITORY:
        fail("candidate image repository changed")
    root = require_digest(image.get("root_digest"), "candidate root digest")
    amd64 = require_digest(image.get("amd64_digest"), "candidate amd64 digest")
    arm64 = require_digest(image.get("arm64_digest"), "candidate arm64 digest")
    if len({root, amd64, arm64}) != 3:
        fail("candidate root and child digests must be distinct")
    return {
        "repository": REPOSITORY,
        "version": version,
        "commit": commit,
        "run_id": run_id,
        "run_attempt": run_attempt,
        "workflow_ref": WORKFLOW_REF,
        "manifest_sha256": sha256(raw),
        "root_digest": root,
        "amd64_digest": amd64,
        "arm64_digest": arm64,
    }


def validate_evidence(
    candidate: dict[str, Any],
    run: dict[str, Any],
    images: dict[str, Any],
    acceptance: dict[str, Any],
    cleanup: dict[str, Any],
) -> tuple[str, dict[str, str]]:
    run_id = run.get("run_id")
    if run.get("schema") != "axiom.gcp.operator.run.v1":
        fail("GKE run schema changed")
    project_id = run.get("project_id")
    region = run.get("region")
    zone = run.get("gke_zone")
    if not isinstance(project_id, str) or not re.fullmatch(
        r"[a-z][a-z0-9-]{4,28}[a-z0-9]", project_id
    ):
        fail("GKE project identity is absent or invalid")
    if not isinstance(region, str) or not re.fullmatch(
        r"[a-z]+-[a-z]+[0-9]", region
    ):
        fail("GKE region is absent or invalid")
    if not isinstance(zone, str) or not re.fullmatch(
        re.escape(region) + r"-[a-z]", zone
    ):
        fail("GKE zone is absent or does not belong to the run region")
    if run.get("git_sha") != candidate["commit"][:12] or run.get("git_dirty") is not False:
        fail("GKE run does not bind the clean candidate commit")
    if run.get("image_provenance") != "prebuilt":
        fail("GKE run rebuilt the candidate image")
    if not isinstance(run_id, str) or not re.fullmatch(r"[a-z0-9][a-z0-9-]{0,17}", run_id):
        fail("GKE run ID is invalid")
    image = f'{IMAGE_REPOSITORY}@{candidate["root_digest"]}'
    if images != {"tape": image}:
        fail("GKE runtime image is not the candidate root digest")
    if acceptance.get("schema") != "axiom.gcp.operator.acceptance.v1":
        fail("GKE functional receipt schema changed")
    if acceptance.get("run_id") != run_id:
        fail("GKE functional receipt belongs to a different run")
    if acceptance.get("project_id") != project_id or acceptance.get("region") != region:
        fail("GKE functional receipt identity changed")
    acceptance_map = acceptance.get("acceptance")
    if not isinstance(acceptance_map, dict) or set(acceptance_map) != {"tape"}:
        fail("GKE functional receipt is not Tape-only")
    tape = acceptance_map.get("tape")
    if not isinstance(tape, dict) or set(tape) != TAPE_KEYS:
        fail("Tape GKE functional result is incomplete")
    if tape.get("schema") != "axiom.gcp.tape.acceptance.v1":
        fail("Tape GKE acceptance schema changed")
    if any(tape.get(field) != "passed" for field in PASS_FIELDS):
        fail("Tape GKE functional result is not fully passed")
    if not isinstance(tape.get("gcs_object"), str) or not tape["gcs_object"]:
        fail("Tape GKE backup object evidence is absent")
    if not isinstance(tape.get("gcs_object_bytes"), int) or tape["gcs_object_bytes"] <= 0:
        fail("Tape GKE backup object is empty")
    topology = tape.get("topology_1_to_3")
    if topology != {"from": 1, "to": 3, "ready_pods": 3}:
        fail("Tape GKE 1-to-3 topology result is incomplete")
    failover = tape.get("raft_failover")
    if not isinstance(failover, dict) or set(failover) != {
        "leader_before",
        "leader_after",
        "leader_pod_uid_before",
        "leader_pod_uid_after",
        "distinct",
        "term_before",
        "term_after",
        "leader_pod_replaced",
    }:
        fail("Tape GKE failover result is incomplete")
    before = failover.get("term_before")
    after = failover.get("term_after")
    if (
        failover.get("distinct") is not True
        or failover.get("leader_before") == failover.get("leader_after")
        or not isinstance(failover.get("leader_pod_uid_before"), str)
        or not failover.get("leader_pod_uid_before")
        or not isinstance(failover.get("leader_pod_uid_after"), str)
        or not failover.get("leader_pod_uid_after")
        or failover.get("leader_pod_uid_before") == failover.get("leader_pod_uid_after")
        or failover.get("leader_pod_replaced") != "passed"
        or not isinstance(before, int)
        or isinstance(before, bool)
        or not isinstance(after, int)
        or isinstance(after, bool)
        or before < 1
        or after <= before
    ):
        fail("Tape GKE failover did not prove a new committed term")
    if cleanup.get("schema") != "axiom.gcp.operator.cleanup.v1":
        fail("GKE cleanup schema changed")
    if cleanup.get("run_id") != run_id or cleanup.get("status") != "clean":
        fail("GKE cleanup is not clean for this run")
    for field, expected in (
        ("project_id", project_id),
        ("region", region),
        ("gke_zone", zone),
    ):
        if cleanup.get(field) != expected:
            fail(f"GKE cleanup {field} belongs to a different run")
    if cleanup.get("preserved") != {
        "artifact_registry": True,
        "preexisting_apis": True,
    }:
        fail("GKE cleanup did not preserve pre-existing resources")
    functional = {field: "passed" for field in PASS_FIELDS}
    functional["topology_1_to_3"] = "passed"
    functional["raft_failover"] = "passed"
    return run_id, functional


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--candidate-manifest", required=True, type=Path)
    parser.add_argument("--run", required=True, type=Path)
    parser.add_argument("--images", required=True, type=Path)
    parser.add_argument("--acceptance", required=True, type=Path)
    parser.add_argument("--cleanup", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()

    candidate_raw, candidate_manifest = load_json(args.candidate_manifest, "candidate manifest")
    run_raw, run = load_json(args.run, "GKE run evidence")
    images_raw, images = load_json(args.images, "GKE image evidence")
    acceptance_raw, acceptance = load_json(args.acceptance, "GKE functional evidence")
    cleanup_raw, cleanup = load_json(args.cleanup, "GKE cleanup evidence")
    candidate = validate_candidate(candidate_raw, candidate_manifest)
    run_id, functional = validate_evidence(candidate, run, images, acceptance, cleanup)

    receipt = {
        "schema": RECEIPT_SCHEMA,
        "complete": True,
        "result": "passed",
        "candidate": candidate,
        "gke": {
            "run_id": run_id,
            "image": f'{IMAGE_REPOSITORY}@{candidate["root_digest"]}',
            "image_provenance": "prebuilt",
            "functional": functional,
            "cleanup": {
                "schema": "axiom.gcp.operator.cleanup.v1",
                "status": "clean",
                "preserved": {
                    "artifact_registry": True,
                    "preexisting_apis": True,
                },
            },
        },
        "evidence": {
            "run_sha256": sha256(run_raw),
            "images_sha256": sha256(images_raw),
            "acceptance_sha256": sha256(acceptance_raw),
            "cleanup_sha256": sha256(cleanup_raw),
        },
        "redaction": {
            "kubeconfig_retained": False,
            "token_retained": False,
            "secret_retained": False,
            "cluster_identity_retained": False,
            "command_output_retained": False,
        },
    }
    data = (json.dumps(receipt, sort_keys=True, separators=(",", ":")) + "\n").encode()
    if args.output.is_symlink():
        fail("output must not be a symlink")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(data)
    sidecar = args.output.with_name(args.output.name + ".sha256")
    if sidecar.is_symlink():
        fail("sidecar output must not be a symlink")
    sidecar.write_text(f"{sha256(data)}  {args.output.name}\n")


if __name__ == "__main__":
    main()
