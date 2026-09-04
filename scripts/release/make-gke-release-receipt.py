#!/usr/bin/env python3
"""Create a public, redacted GKE release receipt for one shared-script app.

The receipt binds one verified release candidate (its final manifest bytes,
commit, run, and image digests) to one digest-pinned GKE acceptance run. Two
evidence backends exist, selected per app by scripts/release/apps.sh:

  --backend gcp             acceptance/gcp evidence (run.json, images.json,
                            acceptance.json, cleanup.json). Used by sift.
  --backend gke-acceptance  one .github/workflows/gke-acceptance.yml run that
                            was dispatched with the candidate image: the
                            `gh run view --json ...` record plus the downloaded
                            evidence bundle. Used by keep, relay, and defer.

The evidence bundle carries a kubeconfig; this script never reads, hashes,
or copies it. `--self-test` exercises both backends on synthetic evidence,
including the negative controls, without any network access.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import shutil
import tempfile
from pathlib import Path
from typing import Any, Callable, Dict, NoReturn, Optional, Tuple

REPOSITORY = "chrischeng-c4/axiom"
IMAGE_OWNER = "ghcr.io/chrischeng-c4"
ACCEPTANCE_JOB = "deploy + verify on GKE"
VERIFY_IMAGE_STEP = "Verify prebuilt image input"
HARNESS_STEP = "Run acceptance harness"
PARK_STEP = "Park node pool (belt and suspenders)"
HARNESS_FIELDS = ("readyz", "round_trip", "durability")
SIFT_FIELDS = (
    "operator_reconcile_1x1",
    "standard_gke_cri_collector",
    "lumen_structured_stdout_materialized",
    "scheduled_backup",
    "gcs_backup",
)
FIVE_TARGETS = (
    "aarch64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-musl",
)
TWO_TARGETS = ("x86_64-unknown-linux-musl", "aarch64-unknown-linux-musl")

# Mirrors scripts/release/apps.sh; verify-release-contract.py cross-checks the two.
APPS: Dict[str, Dict[str, Any]] = {
    "sift": {
        "backend": "gcp",
        "targets": FIVE_TARGETS,
        "functional": SIFT_FIELDS,
        "acceptance_schema": "axiom.gcp.sift.acceptance.v1",
        "image_companions": ("lumen",),
        "acceptance_companions": ("lumen", "auth"),
        "extra_keys": {"schema", "gcs_object", "gcs_object_bytes", "topology_beyond_1x1"},
    },
    "keep": {"backend": "gke-acceptance", "targets": TWO_TARGETS, "functional": HARNESS_FIELDS},
    "relay": {"backend": "gke-acceptance", "targets": TWO_TARGETS, "functional": HARNESS_FIELDS},
    "defer": {"backend": "gke-acceptance", "targets": TWO_TARGETS, "functional": HARNESS_FIELDS},
}


def candidate_schema(app: str) -> str:
    return f"cclab.{app}.candidate-manifest.v1"


def receipt_schema(app: str) -> str:
    return f"{app}.gke-release-receipt/v1"


def workflow_ref(app: str) -> str:
    return f"{REPOSITORY}/.github/workflows/{app}-release-candidate.yml@refs/heads/main"


def final_jobs(app: str) -> Dict[str, str]:
    return {
        job: "success"
        for job in (
            "identity",
            "build",
            f"{app}-release-gates",
            "ghcr-image-and-attest",
            "manifest",
            "verify-candidate",
            "result",
        )
    }


def fail(message: str) -> NoReturn:
    raise SystemExit(f"GKE receipt refused: {message}")


def sha256(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def read_regular(path: Path, label: str) -> bytes:
    if path.is_symlink() or not path.is_file():
        fail(f"{label} must be a regular file: {path}")
    return path.read_bytes()


def load_json(path: Path, label: str) -> Tuple[bytes, Dict[str, Any]]:
    raw = read_regular(path, label)
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


def is_int(value: object) -> bool:
    return isinstance(value, int) and not isinstance(value, bool)


def validate_candidate(app: str, raw: bytes, candidate: Dict[str, Any]) -> Dict[str, Any]:
    expected_keys = {
        "schema", "repository", "workflow_path", "workflow_id", "run_id", "run_attempt",
        "run_url", "source_ref", "workflow_ref", "commit", "version", "tag",
        "candidate_tag", "pr", "image", "artifacts", "sboms", "jobs",
    }
    if set(candidate) != expected_keys:
        fail("candidate manifest keys changed")
    if candidate.get("schema") != candidate_schema(app):
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
    if candidate.get("tag") != f"{app}@{version}":
        fail("candidate release tag changed")
    if candidate.get("candidate_tag") != f"release-candidate-{run_id}-{run_attempt}":
        fail("candidate image tag is not run scoped")
    if candidate.get("source_ref") != "refs/heads/main":
        fail("candidate was not built from main")
    if candidate.get("workflow_path") != f".github/workflows/{app}-release-candidate.yml":
        fail("candidate workflow path changed")
    if candidate.get("workflow_ref") != workflow_ref(app):
        fail("candidate workflow identity changed")
    if candidate.get("jobs") != final_jobs(app):
        fail("candidate does not bind every required successful job")
    artifacts = candidate.get("artifacts")
    targets = APPS[app]["targets"]
    if not isinstance(artifacts, list) or [a.get("target") if isinstance(a, dict) else None for a in artifacts] != list(targets):
        fail("candidate artifact targets changed")
    image = candidate.get("image")
    if not isinstance(image, dict) or set(image) != {"repository", "root_digest", "amd64_digest", "arm64_digest"}:
        fail("candidate image shape changed")
    if image.get("repository") != f"{IMAGE_OWNER}/{app}":
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
        "workflow_ref": workflow_ref(app),
        "manifest_sha256": sha256(raw),
        "root_digest": root,
        "amd64_digest": amd64,
        "arm64_digest": arm64,
    }


def validate_gcp(
    app: str,
    candidate: Dict[str, Any],
    run: Dict[str, Any],
    images: Dict[str, Any],
    acceptance: Dict[str, Any],
    cleanup: Dict[str, Any],
) -> Tuple[str, Dict[str, str], Dict[str, Any]]:
    table = APPS[app]
    fields = table["functional"]
    run_id = run.get("run_id")
    if run.get("schema") != "axiom.gcp.operator.run.v1":
        fail("GKE run schema changed")
    project_id = run.get("project_id")
    region = run.get("region")
    zone = run.get("gke_zone")
    if not isinstance(project_id, str) or not re.fullmatch(r"[a-z][a-z0-9-]{4,28}[a-z0-9]", project_id):
        fail("GKE project identity is absent or invalid")
    if not isinstance(region, str) or not re.fullmatch(r"[a-z]+-[a-z]+[0-9]", region):
        fail("GKE region is absent or invalid")
    if not isinstance(zone, str) or not re.fullmatch(re.escape(region) + r"-[a-z]", zone):
        fail("GKE zone is absent or does not belong to the run region")
    if run.get("git_sha") != candidate["commit"][:12] or run.get("git_dirty") is not False:
        fail("GKE run does not bind the clean candidate commit")
    if run.get("image_provenance") != "prebuilt":
        fail("GKE run rebuilt the candidate image")
    if not isinstance(run_id, str) or not re.fullmatch(r"[a-z0-9][a-z0-9-]{0,39}", run_id):
        fail("GKE run ID is invalid")
    image = f"{IMAGE_OWNER}/{app}@{candidate['root_digest']}"
    expected_images = {app} | set(table.get("image_companions", ()))
    if set(images) != expected_images:
        fail("GKE runtime image set is not the candidate plus its companions")
    if images.get(app) != image:
        fail("GKE runtime image is not the candidate root digest")
    for companion in table.get("image_companions", ()):
        if not isinstance(images.get(companion), str) or not images[companion]:
            fail(f"GKE companion image {companion} is absent")
    if acceptance.get("schema") != "axiom.gcp.operator.acceptance.v1":
        fail("GKE functional receipt schema changed")
    if acceptance.get("run_id") != run_id:
        fail("GKE functional receipt belongs to a different run")
    if acceptance.get("project_id") != project_id or acceptance.get("region") != region:
        fail("GKE functional receipt identity changed")
    acceptance_map = acceptance.get("acceptance")
    expected_map = {app} | set(table.get("acceptance_companions", ()))
    if not isinstance(acceptance_map, dict) or set(acceptance_map) != expected_map:
        fail("GKE functional receipt does not carry the expected acceptance set")
    block = acceptance_map.get(app)
    if not isinstance(block, dict) or set(block) != set(fields) | table["extra_keys"]:
        fail(f"{app} GKE functional result is incomplete")
    if block.get("schema") != table["acceptance_schema"]:
        fail(f"{app} GKE acceptance schema changed")
    if any(block.get(field) != "passed" for field in fields):
        fail(f"{app} GKE functional result is not fully passed")
    if not isinstance(block.get("gcs_object"), str) or not block["gcs_object"]:
        fail(f"{app} GKE backup object evidence is absent")
    if not is_int(block.get("gcs_object_bytes")) or block["gcs_object_bytes"] <= 0:
        fail(f"{app} GKE backup object is empty")
    if block.get("topology_beyond_1x1") != "not_claimed":
        fail(f"{app} GKE receipt claims a topology the harness does not prove")
    if cleanup.get("schema") != "axiom.gcp.operator.cleanup.v1":
        fail("GKE cleanup schema changed")
    if cleanup.get("run_id") != run_id or cleanup.get("status") != "clean":
        fail("GKE cleanup is not clean for this run")
    for field, expected in (("project_id", project_id), ("region", region), ("gke_zone", zone)):
        if cleanup.get(field) != expected:
            fail(f"GKE cleanup {field} belongs to a different run")
    if cleanup.get("preserved") != {"artifact_registry": True, "preexisting_apis": True}:
        fail("GKE cleanup did not preserve pre-existing resources")
    functional = {field: "passed" for field in fields}
    public_cleanup = {
        "schema": "axiom.gcp.operator.cleanup.v1",
        "status": "clean",
        "preserved": {"artifact_registry": True, "preexisting_apis": True},
    }
    return run_id, functional, public_cleanup


def validate_gke_acceptance(
    app: str,
    candidate: Dict[str, Any],
    gh_run: Dict[str, Any],
    evidence_dir: Path,
) -> Tuple[str, str, Dict[str, str], Dict[str, Any], Dict[str, str]]:
    image = f"{IMAGE_OWNER}/{app}@{candidate['root_digest']}"
    commit = candidate["commit"]
    required = {"databaseId", "attempt", "conclusion", "status", "url", "headSha", "event", "workflowName", "displayTitle", "jobs"}
    if not required <= set(gh_run):
        fail("gh run record is missing fields; use --json databaseId,attempt,conclusion,status,url,headSha,event,workflowName,displayTitle,jobs")
    run_number = gh_run["databaseId"]
    attempt = gh_run["attempt"]
    if not is_int(run_number) or run_number <= 0 or not is_int(attempt) or attempt <= 0:
        fail("gh run identity is invalid")
    if gh_run["workflowName"] != "gke-acceptance":
        fail("run is not a gke-acceptance workflow run")
    if gh_run["event"] != "workflow_dispatch":
        fail("run was not dispatched by hand")
    if gh_run["status"] != "completed" or gh_run["conclusion"] != "success":
        fail("run did not conclude success")
    if gh_run["headSha"] != commit:
        fail("run head commit is not the candidate commit")
    run_url = f"https://github.com/{REPOSITORY}/actions/runs/{run_number}"
    if gh_run["url"] != run_url:
        fail("run URL does not belong to this repository and run")
    if gh_run["displayTitle"] != f"gke-acceptance {app} @ {commit} image={image}":
        fail("run was not dispatched for this app with the candidate image")
    jobs = gh_run["jobs"]
    if not isinstance(jobs, list) or not jobs:
        fail("run has no job inventory")
    acceptance_jobs = [job for job in jobs if isinstance(job, dict) and job.get("name") == ACCEPTANCE_JOB]
    if len(acceptance_jobs) != 1:
        fail("run does not contain exactly one acceptance job")
    job = acceptance_jobs[0]
    if job.get("status") != "completed" or job.get("conclusion") != "success":
        fail("acceptance job did not conclude success")
    steps = {step.get("name"): step for step in job.get("steps", []) if isinstance(step, dict)}
    for name in (VERIFY_IMAGE_STEP, HARNESS_STEP, PARK_STEP):
        if steps.get(name, {}).get("conclusion") != "success":
            fail(f"acceptance step did not conclude success: {name}")
    for other in jobs:
        if other is job:
            continue
        if not isinstance(other, dict) or other.get("conclusion") != "skipped":
            fail("a build job ran; the candidate image must be the only image in play")
    if evidence_dir.is_symlink() or not evidence_dir.is_dir():
        fail("evidence directory is invalid")
    results = read_regular(evidence_dir / "results.txt", "harness results")
    if results != f"{app} PASS\n".encode():
        fail("harness results do not record exactly one PASS for this app")
    verdict = read_regular(evidence_dir / app / "verdict.txt", "harness verdict")
    if verdict != f"[{app}] PASS: readyz + round-trip + durability\n".encode():
        fail("harness verdict is not the exact PASS line")
    manifests = read_regular(evidence_dir / app / "manifests.yaml", "rendered manifests")
    try:
        lines = manifests.decode().splitlines()
    except UnicodeDecodeError:
        fail("rendered manifests are not UTF-8")
    if not any(line.strip() == f"image: {image}" for line in lines):
        fail("rendered manifests do not deploy the candidate image")
    read_regular(evidence_dir / app / "teardown.log", "teardown log")
    run_id = f"gha-{run_number}-{attempt}"
    functional = {field: "passed" for field in HARNESS_FIELDS}
    cleanup = {
        "schema": "axiom.gke-harness.cleanup.v1",
        "status": "clean",
        "namespace_deleted": True,
        "node_pool_parked": True,
    }
    evidence = {
        "manifests_sha256": sha256(manifests),
        "results_sha256": sha256(results),
        "verdict_sha256": sha256(verdict),
    }
    return run_id, run_url, functional, cleanup, evidence


def make_receipt(
    app: str,
    backend: str,
    candidate_manifest: Path,
    output: Path,
    *,
    run: Optional[Path] = None,
    images: Optional[Path] = None,
    acceptance: Optional[Path] = None,
    cleanup: Optional[Path] = None,
    gh_run: Optional[Path] = None,
    evidence_dir: Optional[Path] = None,
) -> Dict[str, Any]:
    if app not in APPS:
        fail(f"unknown shared-script app: {app} (known: {' '.join(sorted(APPS))})")
    if backend != APPS[app]["backend"]:
        fail(f"{app} proves GKE acceptance with backend {APPS[app]['backend']}, not {backend}")
    candidate_raw, candidate_manifest_json = load_json(candidate_manifest, "candidate manifest")
    candidate = validate_candidate(app, candidate_raw, candidate_manifest_json)
    image = f"{IMAGE_OWNER}/{app}@{candidate['root_digest']}"
    if backend == "gcp":
        if run is None or images is None or acceptance is None or cleanup is None:
            fail("backend gcp needs --run, --images, --acceptance, and --cleanup")
        run_raw, run_json = load_json(run, "GKE run evidence")
        images_raw, images_json = load_json(images, "GKE image evidence")
        acceptance_raw, acceptance_json = load_json(acceptance, "GKE functional evidence")
        cleanup_raw, cleanup_json = load_json(cleanup, "GKE cleanup evidence")
        run_id, functional, public_cleanup = validate_gcp(app, candidate, run_json, images_json, acceptance_json, cleanup_json)
        gke = {
            "backend": "gcp",
            "run_id": run_id,
            "image": image,
            "image_provenance": "prebuilt",
            "functional": functional,
            "cleanup": public_cleanup,
        }
        evidence = {
            "run_sha256": sha256(run_raw),
            "images_sha256": sha256(images_raw),
            "acceptance_sha256": sha256(acceptance_raw),
            "cleanup_sha256": sha256(cleanup_raw),
        }
    elif backend == "gke-acceptance":
        if gh_run is None or evidence_dir is None:
            fail("backend gke-acceptance needs --gh-run and --evidence-dir")
        gh_raw, gh_json = load_json(gh_run, "gh run record")
        run_id, run_url, functional, public_cleanup, evidence = validate_gke_acceptance(app, candidate, gh_json, evidence_dir)
        evidence["gh_run_sha256"] = sha256(gh_raw)
        gke = {
            "backend": "gke-acceptance",
            "run_id": run_id,
            "run_url": run_url,
            "image": image,
            "image_provenance": "prebuilt",
            "functional": functional,
            "cleanup": public_cleanup,
        }
    else:
        fail(f"unknown backend: {backend}")
    receipt = {
        "schema": receipt_schema(app),
        "complete": True,
        "result": "passed",
        "candidate": candidate,
        "gke": gke,
        "evidence": evidence,
        "redaction": {
            "kubeconfig_retained": False,
            "token_retained": False,
            "secret_retained": False,
            "cluster_identity_retained": False,
            "command_output_retained": False,
        },
    }
    data = (json.dumps(receipt, sort_keys=True, separators=(",", ":")) + "\n").encode()
    if output.is_symlink():
        fail("output must not be a symlink")
    if output.name != f"{app}-gke-receipt.json":
        fail(f"output must be named {app}-gke-receipt.json")
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(data)
    sidecar = output.with_name(output.name + ".sha256")
    if sidecar.is_symlink():
        fail("sidecar output must not be a symlink")
    sidecar.write_text(f"{sha256(data)}  {output.name}\n")
    return receipt


# --- self-test -------------------------------------------------------------

def _write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, sort_keys=True, indent=1) + "\n")


def fixture_candidate(app: str, version: str, commit: str, run_id: str = "12345", attempt: str = "1") -> Dict[str, Any]:
    def digest(seed: str) -> str:
        return "sha256:" + hashlib.sha256(f"{app}-{seed}".encode()).hexdigest()

    return {
        "schema": candidate_schema(app),
        "repository": REPOSITORY,
        "workflow_path": f".github/workflows/{app}-release-candidate.yml",
        "workflow_id": 42,
        "run_id": run_id,
        "run_attempt": attempt,
        "run_url": f"https://github.com/{REPOSITORY}/actions/runs/{run_id}/attempts/{attempt}",
        "source_ref": "refs/heads/main",
        "workflow_ref": workflow_ref(app),
        "commit": commit,
        "version": version,
        "tag": f"{app}@{version}",
        "candidate_tag": f"release-candidate-{run_id}-{attempt}",
        "pr": {"number": 7, "url": f"https://github.com/{REPOSITORY}/pull/7"},
        "image": {
            "repository": f"{IMAGE_OWNER}/{app}",
            "root_digest": digest("root"),
            "amd64_digest": digest("amd64"),
            "arm64_digest": digest("arm64"),
        },
        "artifacts": [
            {
                "target": target,
                "archive": f"{app}-{target}.tar.gz",
                "archive_sha256": "0" * 64,
                "sidecar": f"{app}-{target}.tar.gz.sha256",
                "sidecar_sha256": "1" * 64,
            }
            for target in APPS[app]["targets"]
        ],
        "sboms": {
            "amd64": {"file": "spdx-amd64.json", "sha256": "2" * 64},
            "arm64": {"file": "spdx-arm64.json", "sha256": "3" * 64},
        },
        "jobs": final_jobs(app),
    }


def fixture_gke_acceptance(app: str, commit: str, image: str) -> Dict[str, Any]:
    def step(name: str, conclusion: str = "success") -> Dict[str, Any]:
        return {"name": name, "status": "completed", "conclusion": conclusion, "number": 1}

    return {
        "databaseId": 99001,
        "attempt": 1,
        "conclusion": "success",
        "status": "completed",
        "event": "workflow_dispatch",
        "workflowName": "gke-acceptance",
        "url": f"https://github.com/{REPOSITORY}/actions/runs/99001",
        "headSha": commit,
        "displayTitle": f"gke-acceptance {app} @ {commit} image={image}",
        "jobs": [
            {"name": "build-keep", "status": "completed", "conclusion": "skipped", "steps": []},
            {"name": "build-defer", "status": "completed", "conclusion": "skipped", "steps": []},
            {"name": "build-relay", "status": "completed", "conclusion": "skipped", "steps": []},
            {"name": "build-loom", "status": "completed", "conclusion": "skipped", "steps": []},
            {
                "name": ACCEPTANCE_JOB,
                "status": "completed",
                "conclusion": "success",
                "steps": [
                    step("Set up job"),
                    step("Run actions/checkout@v4"),
                    step(VERIFY_IMAGE_STEP),
                    step("Prepare evidence directory"),
                    step("Authenticate to GCP (Workload Identity Federation)"),
                    step("Set up gcloud + GKE auth plugin"),
                    step(HARNESS_STEP),
                    step("Upload evidence bundle"),
                    step(PARK_STEP),
                ],
            },
        ],
    }


def write_harness_evidence(app: str, evidence_dir: Path, image: str) -> None:
    (evidence_dir / app).mkdir(parents=True, exist_ok=True)
    (evidence_dir / "kubeconfig").write_text("apiVersion: v1\nkind: Config\nusers: [{name: redacted}]\n")
    (evidence_dir / "cluster-name.txt").write_text("axiom-acceptance\n")
    (evidence_dir / "results.txt").write_text(f"{app} PASS\n")
    (evidence_dir / app / "verdict.txt").write_text(f"[{app}] PASS: readyz + round-trip + durability\n")
    (evidence_dir / app / "manifests.yaml").write_text(
        f"apiVersion: apps/v1\nkind: StatefulSet\nspec:\n  template:\n    spec:\n      containers:\n      - name: {app}\n        image: {image}\n"
    )
    (evidence_dir / app / "teardown.log").write_text("namespace deleted\n")


def fixture_gcp(app: str, commit: str, image: str) -> Dict[str, Dict[str, Any]]:
    run_id = "rc-99001-1"
    project = "axiom-accept-1"
    region = "asia-east1"
    zone = "asia-east1-a"
    sift = {
        "schema": APPS[app]["acceptance_schema"],
        "gcs_object": "gs://axiom-accept-1-backups/sift/2026-09-04/backup.tar",
        "gcs_object_bytes": 4096,
        "topology_beyond_1x1": "not_claimed",
    }
    sift.update({field: "passed" for field in APPS[app]["functional"]})
    return {
        "run": {
            "schema": "axiom.gcp.operator.run.v1",
            "project_id": project,
            "region": region,
            "gke_zone": zone,
            "run_id": run_id,
            "git_sha": commit[:12],
            "git_dirty": False,
            "image_provenance": "prebuilt",
        },
        "images": {"lumen": f"{IMAGE_OWNER}/lumen@sha256:{'a' * 64}", app: image},
        "acceptance": {
            "schema": "axiom.gcp.operator.acceptance.v1",
            "project_id": project,
            "region": region,
            "gke_zone": zone,
            "run_id": run_id,
            "backup_bucket": "axiom-accept-1-backups",
            "lumen_evidence": {},
            "lumen_provenance": {},
            "acceptance": {"lumen": {"status": "passed"}, "auth": {"status": "passed"}, app: sift},
        },
        "cleanup": {
            "schema": "axiom.gcp.operator.cleanup.v1",
            "project_id": project,
            "region": region,
            "gke_zone": zone,
            "run_id": run_id,
            "status": "clean",
            "preserved": {"artifact_registry": True, "preexisting_apis": True},
        },
    }


def expect_refusal(label: str, action: Callable[[], Any]) -> None:
    try:
        action()
    except SystemExit as error:
        if str(error).startswith("GKE receipt refused: "):
            return
        raise AssertionError(f"{label}: unexpected exit {error}")
    raise AssertionError(f"{label}: negative control was accepted")


def self_test() -> None:
    commit = ("0123456789abcdef" * 3)[:40]
    positive = 0
    negative = 0
    with tempfile.TemporaryDirectory() as tmp_name:
        tmp = Path(tmp_name)

        # --- gke-acceptance backend (keep) ---------------------------------
        app = "keep"
        base = tmp / app
        candidate = fixture_candidate(app, "0.4.13", commit)
        image = f"{IMAGE_OWNER}/{app}@{candidate['image']['root_digest']}"
        manifest_path = base / "final-candidate-manifest.json"
        _write_json(manifest_path, candidate)
        gh_run_path = base / "gh-run.json"
        _write_json(gh_run_path, fixture_gke_acceptance(app, commit, image))
        evidence = base / "evidence"
        write_harness_evidence(app, evidence, image)
        output = base / f"{app}-gke-receipt.json"

        def run_keep(manifest=manifest_path, gh=gh_run_path, ev=evidence, out=output):
            return make_receipt(app, "gke-acceptance", manifest, out, gh_run=gh, evidence_dir=ev)

        receipt = run_keep()
        positive += 1
        assert receipt["gke"]["run_id"] == "gha-99001-1", receipt["gke"]
        assert receipt["gke"]["backend"] == "gke-acceptance"
        assert set(receipt["evidence"]) == {"gh_run_sha256", "manifests_sha256", "results_sha256", "verdict_sha256"}
        kube_bytes = (evidence / "kubeconfig").read_bytes()
        rendered = json.dumps(receipt)
        assert sha256(kube_bytes) not in rendered
        assert "apiVersion: v1" not in rendered and "kind: Config" not in rendered
        assert not any("kubeconfig" in key for key in receipt["evidence"])
        assert receipt["redaction"]["kubeconfig_retained"] is False
        sidecar = output.with_name(output.name + ".sha256")
        assert sidecar.read_text() == f"{sha256(output.read_bytes())}  {output.name}\n"
        expect_refusal("wrong backend for keep", lambda: make_receipt(app, "gcp", manifest_path, output, run=gh_run_path, images=gh_run_path, acceptance=gh_run_path, cleanup=gh_run_path))
        negative += 1
        expect_refusal("wrong output name", lambda: make_receipt(app, "gke-acceptance", manifest_path, base / "receipt.json", gh_run=gh_run_path, evidence_dir=evidence))
        negative += 1

        def mutated_gh(label: str, mutate: Callable[[Dict[str, Any]], None]) -> None:
            nonlocal negative
            record = copy.deepcopy(json.loads(gh_run_path.read_text()))
            mutate(record)
            path = base / f"gh-{negative}.json"
            _write_json(path, record)
            expect_refusal(label, lambda: run_keep(gh=path))
            negative += 1

        def acceptance_job(record: Dict[str, Any]) -> Dict[str, Any]:
            return next(job for job in record["jobs"] if job["name"] == ACCEPTANCE_JOB)

        def find_step(record: Dict[str, Any], name: str) -> Dict[str, Any]:
            return next(step for step in acceptance_job(record)["steps"] if step["name"] == name)

        mutated_gh("run conclusion failure", lambda r: r.update(conclusion="failure"))
        mutated_gh("run not completed", lambda r: r.update(status="in_progress"))
        mutated_gh("headSha mismatch", lambda r: r.update(headSha="f" * 40))
        mutated_gh("displayTitle without image", lambda r: r.update(displayTitle=f"gke-acceptance {app} @ {commit}"))
        mutated_gh("displayTitle other app", lambda r: r.update(displayTitle=f"gke-acceptance relay @ {commit} image={image}"))
        mutated_gh("other workflow", lambda r: r.update(workflowName="keep-test-image"))
        mutated_gh("push event", lambda r: r.update(event="push"))
        mutated_gh("foreign url", lambda r: r.update(url="https://github.com/someone/else/actions/runs/99001"))
        mutated_gh("build job ran", lambda r: r["jobs"][0].update(conclusion="success"))
        mutated_gh("park step failed", lambda r: find_step(r, PARK_STEP).update(conclusion="failure"))
        mutated_gh("verify image step skipped", lambda r: find_step(r, VERIFY_IMAGE_STEP).update(conclusion="skipped"))
        mutated_gh("harness step absent", lambda r: acceptance_job(r)["steps"].remove(find_step(r, HARNESS_STEP)))
        mutated_gh("acceptance job failed", lambda r: acceptance_job(r).update(conclusion="failure"))
        mutated_gh("two acceptance jobs", lambda r: r["jobs"].append(copy.deepcopy(acceptance_job(r))))

        def mutated_evidence(label: str, mutate: Callable[[Path], None]) -> None:
            nonlocal negative
            path = base / f"evidence-{negative}"
            shutil.copytree(evidence, path)
            mutate(path)
            expect_refusal(label, lambda: run_keep(ev=path))
            negative += 1

        def drop(path: Path) -> None:
            path.rename(path.with_name(path.name + ".absent"))

        mutated_evidence("verdict FAIL", lambda p: (p / app / "verdict.txt").write_text(f"[{app}] FAIL: readyz\n"))
        mutated_evidence("results FAIL", lambda p: (p / "results.txt").write_text(f"{app} FAIL\n"))
        mutated_evidence("results extra app", lambda p: (p / "results.txt").write_text(f"{app} PASS\nrelay PASS\n"))
        mutated_evidence("manifests other image", lambda p: (p / app / "manifests.yaml").write_text(f"image: {IMAGE_OWNER}/{app}:latest\n"))
        mutated_evidence("teardown missing", lambda p: drop(p / app / "teardown.log"))
        mutated_evidence("verdict missing", lambda p: drop(p / app / "verdict.txt"))

        def mutated_manifest(label: str, mutate: Callable[[Dict[str, Any]], None]) -> None:
            nonlocal negative
            record = copy.deepcopy(candidate)
            mutate(record)
            path = base / f"manifest-{negative}.json"
            _write_json(path, record)
            expect_refusal(label, lambda: run_keep(manifest=path))
            negative += 1

        mutated_manifest("candidate job failed", lambda c: c["jobs"].update({"verify-candidate": "failure"}))
        mutated_manifest("candidate job missing", lambda c: c["jobs"].pop("keep-release-gates"))
        mutated_manifest("root equals amd64", lambda c: c["image"].update(root_digest=c["image"]["amd64_digest"]))
        mutated_manifest("foreign image repository", lambda c: c["image"].update(repository=f"{IMAGE_OWNER}/relay"))
        mutated_manifest("tag mismatch", lambda c: c.update(tag="keep@0.4.14"))
        mutated_manifest("built from a branch", lambda c: c.update(source_ref="refs/heads/feature"))
        mutated_manifest("candidate tag not run scoped", lambda c: c.update(candidate_tag="release-candidate-1-1"))
        mutated_manifest("extra target", lambda c: c["artifacts"].append(dict(c["artifacts"][0], target="aarch64-apple-darwin")))

        # --- gcp backend (sift) -------------------------------------------
        app = "sift"
        base = tmp / app
        candidate = fixture_candidate(app, "0.1.2", commit)
        image = f"{IMAGE_OWNER}/{app}@{candidate['image']['root_digest']}"
        manifest_path = base / "final-candidate-manifest.json"
        _write_json(manifest_path, candidate)
        gcp = fixture_gcp(app, commit, image)
        paths = {}
        for name, value in gcp.items():
            paths[name] = base / f"{name}.json"
            _write_json(paths[name], value)
        output = base / f"{app}-gke-receipt.json"

        def run_sift(manifest=manifest_path, **overrides):
            files = dict(paths)
            files.update(overrides)
            return make_receipt(app, "gcp", manifest, output, run=files["run"], images=files["images"], acceptance=files["acceptance"], cleanup=files["cleanup"])

        receipt = run_sift()
        positive += 1
        assert receipt["gke"]["backend"] == "gcp"
        assert receipt["gke"]["run_id"] == "rc-99001-1"
        assert set(receipt["gke"]["functional"]) == set(SIFT_FIELDS)
        assert set(receipt["evidence"]) == {"run_sha256", "images_sha256", "acceptance_sha256", "cleanup_sha256"}
        expect_refusal("wrong backend for sift", lambda: make_receipt(app, "gke-acceptance", manifest_path, output, gh_run=paths["run"], evidence_dir=base))
        negative += 1

        def mutated_gcp(label: str, name: str, mutate: Callable[[Dict[str, Any]], None]) -> None:
            nonlocal negative
            record = copy.deepcopy(gcp[name])
            mutate(record)
            path = base / f"{name}-{negative}.json"
            _write_json(path, record)
            expect_refusal(label, lambda: run_sift(**{name: path}))
            negative += 1

        mutated_gcp("functional failed", "acceptance", lambda a: a["acceptance"][app].update(scheduled_backup="failed"))
        mutated_gcp("topology claimed", "acceptance", lambda a: a["acceptance"][app].update(topology_beyond_1x1="passed"))
        mutated_gcp("auth block missing", "acceptance", lambda a: a["acceptance"].pop("auth"))
        mutated_gcp("acceptance run mismatch", "acceptance", lambda a: a.update(run_id="rc-other"))
        mutated_gcp("empty backup object", "acceptance", lambda a: a["acceptance"][app].update(gcs_object_bytes=0))
        mutated_gcp("images without lumen", "images", lambda i: i.pop("lumen"))
        mutated_gcp("images other sift digest", "images", lambda i: i.update({app: f"{IMAGE_OWNER}/{app}@sha256:{'b' * 64}"}))
        mutated_gcp("images tag not digest", "images", lambda i: i.update({app: f"{IMAGE_OWNER}/{app}:0.1.2"}))
        mutated_gcp("dirty run", "run", lambda r: r.update(git_dirty=True))
        mutated_gcp("run other commit", "run", lambda r: r.update(git_sha="f" * 12))
        mutated_gcp("run rebuilt image", "run", lambda r: r.update(image_provenance="built"))
        mutated_gcp("cleanup not clean", "cleanup", lambda c: c.update(status="leaked"))
        mutated_gcp("cleanup other zone", "cleanup", lambda c: c.update(gke_zone="asia-east1-b"))
        mutated_gcp("cleanup dropped preservation", "cleanup", lambda c: c["preserved"].update(artifact_registry=False))

        expect_refusal("unknown app", lambda: make_receipt("loom", "gke-acceptance", manifest_path, base / "loom-gke-receipt.json", gh_run=paths["run"], evidence_dir=base))
        negative += 1
    print(f"make-gke-release-receipt self-test PASS: {positive} positive, {negative} negative controls refused")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--app", choices=sorted(APPS))
    parser.add_argument("--backend", choices=("gcp", "gke-acceptance"))
    parser.add_argument("--candidate-manifest", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--run", type=Path, help="gcp: acceptance/gcp run.json")
    parser.add_argument("--images", type=Path, help="gcp: acceptance/gcp images.json")
    parser.add_argument("--acceptance", type=Path, help="gcp: acceptance/gcp acceptance.json")
    parser.add_argument("--cleanup", type=Path, help="gcp: acceptance/gcp cleanup.json")
    parser.add_argument("--gh-run", type=Path, help="gke-acceptance: gh run view <id> --json databaseId,attempt,conclusion,status,url,headSha,event,workflowName,displayTitle,jobs")
    parser.add_argument("--evidence-dir", type=Path, help="gke-acceptance: downloaded gke-acceptance-evidence-<run>-<attempt> bundle")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return
    for name in ("app", "backend", "candidate_manifest", "output"):
        if getattr(args, name) is None:
            parser.error(f"--{name.replace('_', '-')} is required")
    make_receipt(
        args.app,
        args.backend,
        args.candidate_manifest,
        args.output,
        run=args.run,
        images=args.images,
        acceptance=args.acceptance,
        cleanup=args.cleanup,
        gh_run=args.gh_run,
        evidence_dir=args.evidence_dir,
    )
    print(f"GKE receipt written: {args.output}")


if __name__ == "__main__":
    main()
