"""Black-box CB materialization and controller Git publication contract (#3382)."""

from __future__ import annotations

import hashlib
import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import final_json, linked_worktree_fixture, verify_case

CASE_ID = "aw-cb-materialization-controller-git-ownership"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "separate-cb-materialization-from-controller-git-ownership"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-cb-materialization-controller-git-ownership"
)
ASSERTIONS = (
    "setup_change_and_td in linked_worktree_fixture produces a clean worktree and phase td_created",
    "aw cb materialize produces candidate output while preserving HEAD, branch, index, remote refs, body, state, labels, and phase",
    "materialize envelope details action materialized, normalized candidate paths, SHA-256 digest bound to file contents, evidence mapping, exact publish command, and workflow_complete False",
    "publish without --candidate-digest returns action refused with typed missing digest code and preserves full snapshot",
    "publish with a stale candidate digest returns action refused with typed stale evidence code and preserves full snapshot",
    "publish with a mismatched candidate digest returns action refused with typed mismatched evidence code and preserves full snapshot",
    "controller publish validates candidate digest, advances HEAD and index, matches exact candidate paths to Git delta, and leaves issue open with workflow_complete False",
    "only a committed terminal CB completion path closes the tracker issue",
)


def _refusal_code(envelope: dict) -> str:
    assert envelope.get("action") == "refused", f"Expected action 'refused', got: {envelope}"
    refusal = envelope.get("refusal")
    if isinstance(refusal, dict):
        return str(refusal.get("code") or refusal.get("reason") or refusal.get("kind") or "")
    return str(envelope.get("code") or envelope.get("reason") or envelope.get("refusal_code") or "")


def _git_diff_name_only(worktree_dir: Path, from_sha: str, to_sha: str) -> list[str]:
    completed = subprocess.run(
        ["git", "diff", "--name-only", from_sha, to_sha],
        cwd=worktree_dir,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        raise AssertionError(f"git diff failed:\nstdout={completed.stdout}\nstderr={completed.stderr}")
    return sorted([line.strip() for line in completed.stdout.splitlines() if line.strip()])


def _calculate_candidate_digest(worktree_dir: Path, candidate_paths: list[str]) -> str:
    hasher = hashlib.sha256()
    for p in sorted(candidate_paths):
        target_file = worktree_dir / p
        assert target_file.is_file(), f"Candidate file missing on disk: {p}"
        file_bytes = target_file.read_bytes()
        assert len(file_bytes) > 0, f"Candidate file is empty on disk: {p}"
        hasher.update(p.encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(file_bytes)
    return "sha256:" + hasher.hexdigest()


def _assert_valid_candidate_handoff(
    fixture: any,
    slug: str,
    mat_envelope: dict,
) -> tuple[list[str], str, dict]:
    assert mat_envelope.get("action") == "materialized", mat_envelope

    completion = mat_envelope.get("completion")
    assert isinstance(completion, dict), f"Materialize envelope missing completion mapping dict: {mat_envelope}"
    assert completion.get("workflow_complete") is False, f"Expected workflow_complete is False, got: {completion}"

    candidate = mat_envelope.get("candidate")
    assert isinstance(candidate, dict), f"Expected candidate object mapping, got: {mat_envelope}"
    candidate_paths = candidate.get("paths") or candidate.get("changed_paths")
    assert isinstance(candidate_paths, list) and len(candidate_paths) > 0, candidate
    for p in candidate_paths:
        assert isinstance(p, str) and len(p) > 0, candidate
        assert not p.startswith("/"), f"Candidate path must not be absolute: {p}"
        assert ".." not in p.split("/"), f"Candidate path must not contain '..' escape: {p}"
        assert not p.startswith("./"), f"Candidate path must not start with './': {p}"
        assert Path(p).as_posix() == p, f"Candidate path must be normalized posix: {p}"

    candidate_digest = candidate.get("digest") or candidate.get("candidate_digest")
    assert isinstance(candidate_digest, str), candidate
    assert re.match(r"^sha256:[0-9a-f]{64}$", candidate_digest), f"Invalid digest format: {candidate_digest}"

    candidate_evidence = candidate.get("evidence")
    assert isinstance(candidate_evidence, dict) and len(candidate_evidence) > 0, candidate
    assert candidate_evidence.get("candidate_digest") == candidate_digest, (
        f"Evidence candidate_digest must equal candidate_digest:\n"
        f"Evidence: {candidate_evidence}\nCandidate digest: {candidate_digest}"
    )
    ev_paths = candidate_evidence.get("paths") or candidate_evidence.get("changed_paths")
    if ev_paths is not None:
        assert sorted(ev_paths) == sorted(candidate_paths), (
            f"Evidence paths mismatch candidate paths:\nEvidence paths: {ev_paths}\nCandidate paths: {candidate_paths}"
        )

    publish_cmd_str = candidate.get("publish_command") or mat_envelope.get("publish_command")
    assert isinstance(publish_cmd_str, str), mat_envelope
    expected_tokens = ["aw", "cb", "publish", slug, "--candidate-digest", candidate_digest]
    assert publish_cmd_str.split() == expected_tokens, (
        f"Publish command tokens mismatch:\nExpected: {expected_tokens}\nGot: {publish_cmd_str.split()}"
    )

    expected_local_digest = _calculate_candidate_digest(fixture.worktree_dir, candidate_paths)
    assert candidate_digest == expected_local_digest, (
        f"Candidate digest mismatch with local calculation:\nReturned: {candidate_digest}\nCalculated: {expected_local_digest}"
    )
    return candidate_paths, candidate_digest, candidate_evidence


def verify() -> list[str]:
    with linked_worktree_fixture(branch_name="worker-cb-sep") as fixture:
        assert fixture.is_clean() is True, "Linked worktree must be clean before setup"
        slug, initial_snapshot = fixture.setup_change_and_td(
            title="Separate CB Materialization from Controller Git Ownership",
        )
        assert fixture.is_clean() is True, "Linked worktree must be clean after setup_change_and_td"
        assert initial_snapshot["phase"] == "td_created", (
            f"Expected phase td_created, got {initial_snapshot['phase']!r}"
        )
        assert initial_snapshot["state"] == "open"

        mat_res1 = fixture.run_aw("cb", "materialize", slug)
        assert mat_res1.returncode == 0, f"aw cb materialize failed:\nstdout={mat_res1.stdout}\nstderr={mat_res1.stderr}"
        mat_envelope1 = final_json(mat_res1)
        mat_snapshot1 = fixture.issue_snapshot(slug)
        assert mat_snapshot1 == initial_snapshot, (
            f"Materialize mutated snapshot:\nBefore: {initial_snapshot}\nAfter: {mat_snapshot1}"
        )
        candidate_paths1, stale_candidate_digest, candidate_evidence1 = _assert_valid_candidate_handoff(
            fixture, slug, mat_envelope1
        )

        fail_no_digest = fixture.run_aw("cb", "publish", slug, expect_success=False)
        assert fail_no_digest.returncode != 0, "publish without --candidate-digest unexpectedly succeeded"
        env_no_digest = final_json(fail_no_digest)
        code_no_digest = _refusal_code(env_no_digest)
        assert code_no_digest in {"missing_candidate_digest", "missing_digest", "candidate_digest_required"}, env_no_digest
        assert fixture.issue_snapshot(slug) == initial_snapshot, "Snapshot mutated on missing digest refusal"

        updated_td_source = (
            fixture.td_path.read_text(encoding="utf-8")
            + "\n\n"
            + "def evolved_candidate_v2_declaration() -> str:\n"
            + '    """Material semantic TD behavior addition for candidate 2 evolution."""\n'
            + '    return "v2_behavior_declaration"\n'
        )
        fixture.td_path.write_text(updated_td_source, encoding="utf-8")
        rel_td_path = str(fixture.td_path.relative_to(fixture.worktree_dir))
        fixture.run_aw(
            "td", "create", slug, "--project", fixture.project_name,
            "--apply", "--spec-path", rel_td_path,
        )
        fixture.run_aw("td", "lock", "--project", fixture.project_name)
        subprocess.run(["git", "add", "-A"], cwd=fixture.worktree_dir, check=True)
        subprocess.run(["git", "commit", "-m", "setup candidate 2 TD evolution"], cwd=fixture.worktree_dir, check=True)

        assert fixture.is_clean() is True, "Linked worktree must be clean after setup candidate 2 commit"
        baseline2 = fixture.issue_snapshot(slug)
        mat_res2 = fixture.run_aw("cb", "materialize", slug)
        assert mat_res2.returncode == 0, f"aw cb materialize (pass 2) failed:\nstdout={mat_res2.stdout}\nstderr={mat_res2.stderr}"
        mat_envelope2 = final_json(mat_res2)
        mat_snapshot2 = fixture.issue_snapshot(slug)
        assert mat_snapshot2 == baseline2, (
            f"Materialize pass 2 mutated snapshot:\nBefore: {baseline2}\nAfter: {mat_snapshot2}"
        )
        candidate_paths2, current_candidate_digest, candidate_evidence2 = _assert_valid_candidate_handoff(
            fixture, slug, mat_envelope2
        )
        assert current_candidate_digest != stale_candidate_digest, (current_candidate_digest, stale_candidate_digest)

        fail_stale = fixture.run_aw("cb", "publish", slug, "--candidate-digest", stale_candidate_digest, expect_success=False)
        assert fail_stale.returncode != 0, "publish with stale digest unexpectedly succeeded"
        env_stale = final_json(fail_stale)
        code_stale = _refusal_code(env_stale)
        assert code_stale in {"stale_candidate_digest", "stale_evidence", "stale_candidate"}, env_stale
        assert fixture.issue_snapshot(slug) == baseline2, "Snapshot mutated on stale digest refusal"

        mismatched_digest = "sha256:" + "1" * 64
        fail_mismatch = fixture.run_aw("cb", "publish", slug, "--candidate-digest", mismatched_digest, expect_success=False)
        assert fail_mismatch.returncode != 0, "publish with mismatched digest unexpectedly succeeded"
        env_mismatch = final_json(fail_mismatch)
        code_mismatch = _refusal_code(env_mismatch)
        assert code_mismatch in {"mismatched_candidate_digest", "mismatched_evidence", "mismatched_digest"}, env_mismatch
        assert fixture.issue_snapshot(slug) == baseline2, "Snapshot mutated on mismatched digest refusal"

        pub_res = fixture.run_aw("cb", "publish", slug, "--candidate-digest", current_candidate_digest)
        assert pub_res.returncode == 0, f"aw cb publish failed:\nstdout={pub_res.stdout}\nstderr={pub_res.stderr}"
        pub_envelope = final_json(pub_res)
        assert pub_envelope.get("action") == "published", pub_envelope
        pub_completion = pub_envelope.get("completion")
        assert isinstance(pub_completion, dict), f"Publish envelope missing completion mapping dict: {pub_envelope}"
        assert pub_completion.get("workflow_complete") is False, f"Expected workflow_complete is False in publish envelope, got: {pub_completion}"

        pub_snapshot = fixture.issue_snapshot(slug)
        assert pub_snapshot["head"] != baseline2["head"], "HEAD must advance on publish"
        assert pub_snapshot["index_tree"] != baseline2["index_tree"], "Index tree must change on publish"
        assert pub_snapshot["phase"] != baseline2["phase"], "Phase must advance on publish"
        actual_git_delta = _git_diff_name_only(fixture.worktree_dir, baseline2["head"], pub_snapshot["head"])
        assert sorted(candidate_paths2) == actual_git_delta, (
            f"Candidate paths mismatch post-publish Git delta:\nCandidate: {sorted(candidate_paths2)}\nGit diff: {actual_git_delta}"
        )
        for path_str in candidate_paths2:
            assert path_str in pub_snapshot["index_tree"], f"Candidate path {path_str} missing from post-publish index: {pub_snapshot['index_tree']}"
            target_file = fixture.worktree_dir / path_str
            assert target_file.is_file(), f"Candidate path {path_str} missing on disk after publish"
            assert len(target_file.read_text(encoding="utf-8").strip()) > 0, f"Candidate path {path_str} is empty on disk"

        tracker_changed = (
            pub_snapshot["body"] != baseline2["body"]
            or pub_snapshot["labels"] != baseline2["labels"]
        )
        assert tracker_changed, (
            f"Expected tracker projection (body or labels) to change on publish:\n"
            f"Initial: {baseline2}\nPost-pub: {pub_snapshot}"
        )
        assert pub_snapshot["state"] == "open", "Issue state must remain open on publish"

        close_res = fixture.run_aw("cb", "check", slug)
        assert close_res.returncode == 0, f"aw cb check failed:\nstdout={close_res.stdout}\nstderr={close_res.stderr}"
        close_snapshot = fixture.issue_snapshot(slug)
        assert close_snapshot["state"] == "closed", f"Expected issue state 'closed', got {close_snapshot['state']!r}"

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify_case(CASE_ID, verify)
