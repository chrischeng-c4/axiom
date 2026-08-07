from __future__ import annotations

import importlib.util
import json
import os
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


CASE_PATH = Path(__file__).parents[2] / "src" / "ec-2879.py"
SPEC = importlib.util.spec_from_file_location("lumen_ec_2879_test", CASE_PATH)
assert SPEC is not None and SPEC.loader is not None
CASE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CASE)
RUNNER_PATH = Path(__file__).parents[2] / "src" / "runner.py"
RUNNER_SPEC = importlib.util.spec_from_file_location("lumen_ec_2879_runner_test", RUNNER_PATH)
assert RUNNER_SPEC is not None and RUNNER_SPEC.loader is not None
RUNNER = importlib.util.module_from_spec(RUNNER_SPEC)
RUNNER_SPEC.loader.exec_module(RUNNER)
AUDITOR_PATH = Path(__file__).parents[2] / "src" / "redaction_auditor.py"
AUDITOR_SPEC = importlib.util.spec_from_file_location("lumen_redaction_auditor_test", AUDITOR_PATH)
assert AUDITOR_SPEC is not None and AUDITOR_SPEC.loader is not None
AUDITOR = importlib.util.module_from_spec(AUDITOR_SPEC)
AUDITOR_SPEC.loader.exec_module(AUDITOR)


def _write_status(path: Path, code: str) -> None:
    path.write_text(f"POST /proof -> {code} (expected {code})\n", encoding="utf-8")


def _audit(root: Path) -> None:
    credentials = root.parent / f"{root.name}-credentials"
    credentials.mkdir(exist_ok=True)
    (credentials / "reader.token").write_text("header.payload.signature-canary", encoding="utf-8")
    AUDITOR.audit(
        root,
        credentials,
        root / "kubernetes" / "auth" / "lumen-auth-redaction-audit.json",
    )


def _write_green_bundle(root: Path, namespace: str = "tenant-a") -> Path:
    auth_dir = root / "kubernetes" / "auth"
    auth_dir.mkdir(parents=True)
    (root / "run.log").write_text("acceptance auth phase started\n", encoding="utf-8")
    for row in CASE.GOOGLE_REJECTION_ROWS:
        _write_status(auth_dir / row, "401")
    for row, code in CASE.LEAST_PRIVILEGE_ROWS.items():
        _write_status(auth_dir / row, code)
    (auth_dir / "serving.log").write_text(
        "\n".join(
            [
                f"subject=system:serviceaccount:{namespace}:auth-reader",
                f"subject=system:serviceaccount:{namespace}:auth-operator",
                "request completed",
            ]
        ),
        encoding="utf-8",
    )
    (root / "lumen-auth-acceptance.json").write_text(
        json.dumps(
            {
                "schema": CASE.AUTH_SCHEMA,
                "status": "passed",
                "run_id": "ec-2879-green",
                "namespace": namespace,
                "audience": CASE.LUMEN_AUDIENCE,
                "issuers": [
                    {
                        "kind": "google-user",
                        "kubernetes_username": "human@example.test",
                        "cluster_admin": True,
                    },
                    {
                        "kind": "google-service-account",
                        "kubernetes_username": "gsa@example.test",
                        "cluster_admin": False,
                    },
                ],
                "sibling_mint_refusals": 1,
                "revocation": {
                    "issuer_token_request_seconds": 2,
                    "lumen_authorization_seconds": 12,
                    "documented_bound_seconds": CASE.LUMEN_REVOCATION_BOUND_SECONDS,
                },
            }
        ),
        encoding="utf-8",
    )
    (root / "cleanup.json").write_text(
        json.dumps(
            {
                "schema": CASE.CLEANUP_SCHEMA,
                "status": "clean",
                "run_id": "ec-2879-green",
            }
        ),
        encoding="utf-8",
    )
    _audit(root)
    return root


class RetainedGkeEvidenceTests(unittest.TestCase):
    def test_green_bundle_passes(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            CASE.verify_retained_gke_evidence(_write_green_bundle(Path(temp)))

    def test_forged_redaction_flags_and_proof_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = _write_green_bundle(Path(temp))
            summary_path = root / "lumen-auth-acceptance.json"
            summary = json.loads(summary_path.read_text(encoding="utf-8"))
            summary["redaction"] = {
                "credential_fields_absent": True,
                "token_canary_absent": True,
            }
            summary_path.write_text(json.dumps(summary), encoding="utf-8")
            (root / "kubernetes" / "auth" / "lumen-auth-redaction-audit.json").write_text(
                json.dumps(
                    {
                        "schema": "axiom.lumen.ec.redaction-audit.v1",
                        "status": "passed",
                        "credential_canary_digests": ["sha256:forged"],
                        "forbidden_token_fields_absent": True,
                        "snapshot_manifest": [],
                        "snapshot_digest": "sha256:forged",
                    }
                ),
                encoding="utf-8",
            )
            with self.assertRaisesRegex(CASE.EvidenceError, "reviewed auditor source"):
                CASE.verify_retained_gke_evidence(root)

    def test_ksa_log_uses_retained_namespace(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = _write_green_bundle(Path(temp), namespace="tenant-z")
            serving_log = root / "kubernetes" / "auth" / "serving.log"
            serving_log.write_text(
                "subject=system:serviceaccount:lumen:auth-reader\n"
                "subject=system:serviceaccount:lumen:auth-operator\n",
                encoding="utf-8",
            )
            _audit(root)
            with self.assertRaisesRegex(CASE.EvidenceError, "tenant-z"):
                CASE.verify_retained_gke_evidence(root)

    def test_later_non_auth_run_artifacts_do_not_change_auth_audit_corpus(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = _write_green_bundle(Path(temp))
            with (root / "run.log").open("a", encoding="utf-8") as handle:
                handle.write("later sift phase appended\n")
            (root / "sift-later-phase.json").write_text("{}", encoding="utf-8")
            CASE.verify_retained_gke_evidence(root)

    def test_redaction_auditor_rejects_a_live_canary_in_retained_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / "bundle"
            root.mkdir()
            credentials = Path(temp) / "credentials"
            credentials.mkdir()
            token = "header.payload.leaked-canary"
            (credentials / "reader.token").write_text(token, encoding="utf-8")
            (root / "run.log").write_text(token, encoding="utf-8")
            with self.assertRaisesRegex(AUDITOR.RedactionAuditError, "canary leaked"):
                AUDITOR.audit(root, credentials, root / "lumen-auth-redaction-audit.json")

    def test_mutated_live_run_log_prefix_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = _write_green_bundle(Path(temp))
            (root / "run.log").write_text("rewritten after auth audit\n", encoding="utf-8")
            with self.assertRaisesRegex(CASE.EvidenceError, "snapshot file was truncated: run.log"):
                CASE.verify_retained_gke_evidence(root)

    def test_foreign_source_provenance_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "run.json").write_text(
                json.dumps(
                    {
                        "run_id": "ec2879abcdefgh123456",
                        "git_sha": "foreign-source",
                        "git_dirty": False,
                        "started_at": "2026-08-01T00:00:00Z",
                        "image_provenance": "cloud-build",
                    }
                ),
                encoding="utf-8",
            )
            (root / "images.json").write_text(
                json.dumps({"lumen": "example.test/lumen@sha256:abc"}),
                encoding="utf-8",
            )
            with self.assertRaisesRegex(ValueError, "current clean checkout"):
                RUNNER._verify_provenance(root, "ec2879abcdefgh123456", "current-source")

    def test_prebuilt_lumen_image_provenance_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "run.json").write_text(
                json.dumps(
                    {
                        "run_id": "ec2879abcdefgh123456",
                        "git_sha": "current-source",
                        "git_dirty": False,
                        "started_at": "2026-08-01T00:00:00Z",
                        "image_provenance": "prebuilt",
                    }
                ),
                encoding="utf-8",
            )
            (root / "images.json").write_text(
                json.dumps({"lumen": "example.test/lumen@sha256:abc"}),
                encoding="utf-8",
            )
            with self.assertRaisesRegex(ValueError, "built from the reviewed checkout"):
                RUNNER._verify_provenance(root, "ec2879abcdefgh123456", "current-source")

    def test_cloud_build_environment_removes_service_overrides(self) -> None:
        inherited = {
            "LUMEN_IMAGE": "old@sha256:aaa",
            "SIFT_IMAGE": "old@sha256:bbb",
            "TAPE_IMAGE": "old@sha256:ccc",
            "LUMEN_PRIOR_ACCEPTANCE": "/tmp/prior.json",
            "LUMEN_CLI": "/tmp/old-lumen",
        }
        previous = {key: os.environ.get(key) for key in inherited}
        try:
            os.environ.update(inherited)
            environment = RUNNER._cloud_build_environment(
                "test-project",
                "ec2879abcdefgh123456",
                Path("/tmp/run"),
                Path("/tmp/run/kubernetes/auth/lumen-auth-redaction-audit.json"),
            )
        finally:
            for key, value in previous.items():
                if value is None:
                    os.environ.pop(key, None)
                else:
                    os.environ[key] = value
        self.assertEqual(environment["ACCEPTANCE_APPS"], "lumen auth")
        self.assertTrue(environment["LUMEN_AUTH_REDACTION_AUDITOR"].endswith("redaction_auditor.py"))
        self.assertTrue(environment["LUMEN_AUTH_REDACTION_AUDIT_PATH"].endswith("lumen-auth-redaction-audit.json"))
        for key in inherited:
            self.assertNotIn(key, environment)

    def test_evidence_directory_git_ignore_rules(self) -> None:
        json_path = f"apps/lumen/external-contracts/evidence/{RUNNER.EVIDENCE_NAME}"
        log_path = f"apps/lumen/external-contracts/evidence/{RUNNER.RUNS_DIR}/example/run.log"
        readme_path = "apps/lumen/external-contracts/evidence/README.md"

        completed_json = subprocess.run(
            ["git", "check-ignore", "--no-index", json_path],
            check=False,
            capture_output=True,
            encoding="utf-8",
        )
        self.assertEqual(completed_json.returncode, 0, completed_json.stderr)

        completed_log = subprocess.run(
            ["git", "check-ignore", "--no-index", log_path],
            check=False,
            capture_output=True,
            encoding="utf-8",
        )
        self.assertEqual(completed_log.returncode, 0, completed_log.stderr)

        completed_readme = subprocess.run(
            ["git", "check-ignore", "--no-index", readme_path],
            check=False,
            capture_output=True,
            encoding="utf-8",
        )
        self.assertEqual(completed_readme.returncode, 1, completed_readme.stdout)

        completed_ls_files = subprocess.run(
            ["git", "ls-files", "--error-unmatch", readme_path],
            check=False,
            capture_output=True,
            encoding="utf-8",
        )
        self.assertEqual(completed_ls_files.returncode, 0, completed_ls_files.stderr)

    def test_acceptance_gcp_readme_closed_mode_set_matches_run_sh(self) -> None:
        repo_root = Path(__file__).resolve().parents[5]
        run_sh_path = repo_root / "acceptance" / "gcp" / "scripts" / "run.sh"
        readme_path = repo_root / "acceptance" / "gcp" / "README.md"

        run_sh_content = run_sh_path.read_text(encoding="utf-8")
        readme_content = readme_path.read_text(encoding="utf-8")

        case_start = run_sh_content.find('case "$ACCEPTANCE_APPS" in')
        self.assertNotEqual(case_start, -1, 'case "$ACCEPTANCE_APPS" in not found in run.sh')
        case_end = run_sh_content.find("esac", case_start)
        self.assertNotEqual(case_end, -1, "esac not found after case in run.sh")
        case_block = run_sh_content[case_start:case_end]

        run_sh_modes = set(re.findall(r'^\s*"([^"]+)"\)', case_block, re.MULTILINE))
        self.assertTrue(run_sh_modes, "No mode arms extracted from run.sh")

        readme_match = re.search(r'`ACCEPTANCE_APPS`[\s\S]*?closed:[\s\S]*?\.', readme_content)
        self.assertIsNotNone(readme_match, "Closed mode set statement for ACCEPTANCE_APPS not found in README.md")

        readme_modes = {
            m for m in re.findall(r'`([^`]+)`', readme_match.group(0))
            if m != "ACCEPTANCE_APPS"
        }
        self.assertEqual(readme_modes, run_sh_modes)


    def test_harness_callback_without_immediate_credential_destroy_is_rejected(self) -> None:
        incomplete = """lumen_auth_redaction_audit_and_destroy() {
  "${LUMEN_AUTH_REDACTION_AUDITOR:?required}" \\
    --evidence-root "$EVIDENCE_DIR" \\
    --credential-dir "$SECRET_DIR" \\
    --output "${LUMEN_AUTH_REDACTION_AUDIT_PATH:?required}"
  echo "unsafe output while credentials remain live"
  rm -rf "$SECRET_DIR"
  SECRET_DIR=""
}
if [[ -n "${LUMEN_AUTH_REDACTION_AUDITOR:-}" || -n "${LUMEN_AUTH_REDACTION_AUDIT_PATH:-}" ]]; then
  [[ -n "${LUMEN_AUTH_REDACTION_AUDITOR:-}" && -n "${LUMEN_AUTH_REDACTION_AUDIT_PATH:-}" ]] || {
    echo "LUMEN_AUTH_REDACTION_AUDITOR and LUMEN_AUTH_REDACTION_AUDIT_PATH must be set together" >&2
    exit 1
  }
  lumen_auth_redaction_audit_and_destroy
fi
"""
        self.assertFalse(RUNNER._has_ordered_harness_redaction_lifecycle(incomplete))

    def test_pair_guard_rejects_half_configured_callback(self) -> None:
        half_configured = """lumen_auth_redaction_audit_and_destroy() {
  "${LUMEN_AUTH_REDACTION_AUDITOR:?required}" \\
    --evidence-root "$EVIDENCE_DIR" \\
    --credential-dir "$SECRET_DIR" \\
    --output "${LUMEN_AUTH_REDACTION_AUDIT_PATH:?required}"
  rm -rf "$SECRET_DIR"
  SECRET_DIR=""
}
if [[ -n "${LUMEN_AUTH_REDACTION_AUDITOR:-}" || -n "${LUMEN_AUTH_REDACTION_AUDIT_PATH:-}" ]]; then
  lumen_auth_redaction_audit_and_destroy
fi
"""
        self.assertFalse(RUNNER._has_ordered_harness_redaction_lifecycle(half_configured))

    def test_ordered_pair_guarded_harness_lifecycle_is_accepted(self) -> None:
        approved = """lumen_auth_redaction_audit_and_destroy() {
  "${LUMEN_AUTH_REDACTION_AUDITOR:?required}" \\
    --evidence-root "$EVIDENCE_DIR" \\
    --credential-dir "$SECRET_DIR" \\
    --output "${LUMEN_AUTH_REDACTION_AUDIT_PATH:?required}"
  rm -rf "$SECRET_DIR"
  SECRET_DIR=""
}
if [[ -n "${LUMEN_AUTH_REDACTION_AUDITOR:-}" || -n "${LUMEN_AUTH_REDACTION_AUDIT_PATH:-}" ]]; then
  [[ -n "${LUMEN_AUTH_REDACTION_AUDITOR:-}" && -n "${LUMEN_AUTH_REDACTION_AUDIT_PATH:-}" ]] || {
    echo "LUMEN_AUTH_REDACTION_AUDITOR and LUMEN_AUTH_REDACTION_AUDIT_PATH must be set together" >&2
    exit 1
  }
  lumen_auth_redaction_audit_and_destroy
fi
"""
        self.assertTrue(RUNNER._has_ordered_harness_redaction_lifecycle(approved))

    def test_comment_only_redaction_lifecycle_is_rejected(self) -> None:
        comment_only = """# lumen_auth_redaction_audit_and_destroy() {
#   \"${LUMEN_AUTH_REDACTION_AUDITOR:?required}\" --evidence-root \"$EVIDENCE_DIR\"
#   rm -rf \"$SECRET_DIR\"
# }
"""
        self.assertFalse(RUNNER._has_ordered_harness_redaction_lifecycle(comment_only))

    def test_redaction_auditor_is_directly_executable(self) -> None:
        completed = subprocess.run(
            [str(AUDITOR_PATH), "--help"],
            check=False,
            capture_output=True,
            encoding="utf-8",
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("--evidence-root", completed.stdout)

    def test_artifact_runner_rejects_evidence_only_bypass_without_harness_seam(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            environment = {
                **os.environ,
                "AW_PYTHON_ARTIFACT_PROTOCOL": "aw.python-artifact.v1",
                "AW_PYTHON_ARTIFACT_SOURCE_DIGEST": "sha256:source",
                "AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST": "sha256:lock",
                "AW_PYTHON_ARTIFACT_EVIDENCE_DIR": str(Path(temp) / "result"),
                "PROJECT_ID": "test-project",
            }
            completed = subprocess.run(
                [sys.executable, "-I", str(RUNNER_PATH), "gke-ksa-rbac-authorization"],
                check=False,
                capture_output=True,
                encoding="utf-8",
                env=environment,
            )
            self.assertEqual(completed.returncode, 1)
            result = json.loads(completed.stdout)
            self.assertEqual(result["status"], "failed")
            self.assertEqual(completed.stderr, "")
