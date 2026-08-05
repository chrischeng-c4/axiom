"""Black-box external contract for linked worktree lifecycle fixture (#3383)."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import linked_worktree_fixture, verify_case

CASE_ID = "ec-linked-worktree-lifecycle-fixture"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "reusable-ec-linked-worktree-lifecycle-fixture"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case ec-linked-worktree-lifecycle-fixture"
)
ASSERTIONS = (
    "linked_worktree_fixture builds a committed base main, bare origin, and clean linked worker worktree",
    "WI creation, validation, TD creation, and TD lock execute coherently in the linked worktree",
    "HEAD, branch, index tree, remote refs, and issue snapshots return deterministic state without product imports",
    "a clean-worktree helper, distinct base/linked branches, and unchanged origin/main SHA are verified",
    "authored TD source replaces AW_TD_FILL and return pending with executable declarations before apply",
    "aw td lock --project demo --json returns TdLockStatus with clean True, status locked, declared lock_path, and exact phase td_created",
    "issue snapshot preserves original semantic body fragments, required sorted labels, and deterministic consecutive reads",
    "a known absent public command fails after TD admission with an exact CLI unknown-subcommand diagnostic",
    "repeated fixture calls operate without branch, ref, tracker, or origin collision and verify isolated main SHAs",
)


def verify() -> list[str]:
    # Phase 1: setup change and TD in linked worktree 1
    with linked_worktree_fixture(branch_name="project-demo-1") as fixture1:
        assert fixture1.current_branch() == "project-demo-1"
        assert fixture1.base_branch() == "main"
        assert fixture1.base_branch() != fixture1.current_branch()
        initial_head = fixture1.head_sha()
        assert len(initial_head) == 40

        expected_body = (
            "## Problem\n\nProvide reusable EC linked-worktree lifecycle fixture.\n\n"
            "## Requirements\n\n"
            "- R1: Reusable fixture creates bare origin, committed base, and linked worker worktree.\n\n"
            "## Verification Inventory\n\n"
            "| Requirement | Gate | Oracle | Depends On |\n"
            "|-------------|------|--------|------------|\n"
            "| R1 | `aw td create` | Valid change WI and admitted TD created in linked worktree | - |\n"
        )

        slug1, snapshot1 = fixture1.setup_change_and_td(
            title="Linked WT Lifecycle Test Change",
            body=expected_body,
        )

        # 1. Assert exact clean-worktree, branch distinction, and unchanged origin/main
        assert fixture1.is_clean() is True, "Linked worktree must be clean after setup commit"
        assert fixture1.current_branch() == "project-demo-1"
        assert fixture1.base_branch() == "main"
        assert fixture1.base_branch() != fixture1.current_branch()
        assert "refs/heads/main" in fixture1.remote_refs()
        assert fixture1.remote_refs()["refs/heads/main"] == fixture1.initial_origin_main_sha

        # 2. Assert individual fields of deterministic issue snapshot
        assert snapshot1["slug"] == slug1
        # 2a. Assert snapshot body preserves each original semantic body fragment separately
        assert "Provide reusable EC linked-worktree lifecycle fixture." in snapshot1["body"]
        assert "- R1: Reusable fixture creates bare origin, committed base, and linked worker worktree." in snapshot1["body"]
        assert "## Verification Inventory" in snapshot1["body"]
        assert "aw td create" in snapshot1["body"]

        # 2b. Determinism assertion: two consecutive reads with no mutation must be equal
        snapshot1_again = fixture1.issue_snapshot(slug1)
        assert snapshot1 == snapshot1_again, f"Issue snapshot reads are not deterministic:\n{snapshot1}\nvs\n{snapshot1_again}"

        # 2c. Assert labels are sorted and contain required app:demo and type:change
        assert snapshot1["labels"] == sorted(snapshot1["labels"]), f"Labels are not sorted: {snapshot1['labels']}"
        assert "app:demo" in snapshot1["labels"]
        assert "type:change" in snapshot1["labels"]

        # 2d. Snapshot metadata assertions
        assert snapshot1["state"] == "open"
        assert snapshot1["head"] == fixture1.head_sha()
        assert snapshot1["head"] != initial_head, "HEAD should advance after setup commit"
        assert snapshot1["branch"] == "project-demo-1"
        assert isinstance(snapshot1["index_tree"], list)
        assert "aw.toml" in snapshot1["index_tree"]
        assert "tracked.txt" in snapshot1["index_tree"]
        assert any(p.startswith("tech-design/") for p in snapshot1["index_tree"])
        assert snapshot1["remote_refs"]["refs/heads/main"] == fixture1.initial_origin_main_sha

        # 3. Assert concrete admitted/locked TD snapshot, authored source transform, and TdLockStatus
        assert fixture1.td_path is not None
        assert fixture1.td_path.is_file(), f"TD source path does not exist: {fixture1.td_path}"
        td_content = fixture1.td_path.read_text(encoding="utf-8")
        assert "AW_TD_FILL" not in td_content, f"TD module still contains AW_TD_FILL: {td_content}"
        assert 'return "pending"' not in td_content, f"TD module still contains return 'pending': {td_content}"
        assert 'return "aw.python-td-ir.v1"' in td_content
        assert f'__aw_work_item__ = "{slug1}"' in td_content

        # TdLockStatus assertions from aw td lock --project demo --json
        assert fixture1.lock_res is not None, "aw td lock --project demo --json emitted no status"
        assert fixture1.lock_res.get("clean") is True, f"TdLockStatus.clean is not True: {fixture1.lock_res}"
        assert fixture1.lock_res.get("status") == "locked", f"TdLockStatus.status is not 'locked': {fixture1.lock_res}"
        declared_lock_path_str = fixture1.lock_res.get("lock_path")
        assert declared_lock_path_str is not None, f"TdLockStatus missing lock_path: {fixture1.lock_res}"
        resolved_lock_path = (
            Path(declared_lock_path_str)
            if Path(declared_lock_path_str).is_absolute()
            else fixture1.worktree_dir / declared_lock_path_str
        )
        assert resolved_lock_path.is_file(), f"Declared lock_path does not exist on disk: {resolved_lock_path}"

        # Assert exact issue snapshot phase is td_created
        assert snapshot1["phase"] == "td_created", f"Expected phase td_created, got {snapshot1['phase']!r}"

        # 4. Assert absent command fails AFTER fixture admission with exact CLI diagnostic wording only
        absent_result = fixture1.run_aw("cb", "materialize", expect_success=False)
        assert absent_result.returncode != 0
        error_msg = f"{absent_result.stdout}\n{absent_result.stderr}".lower()
        assert (
            "unrecognized subcommand" in error_msg
            or "unknown subcommand" in error_msg
            or "is not a valid subcommand" in error_msg
        ), f"Expected CLI unknown-subcommand wording, got stdout={absent_result.stdout!r}, stderr={absent_result.stderr!r}"

        # Capture all phase 1 values BEFORE exiting fixture1 context
        phase1_branch = fixture1.current_branch()
        phase1_worktree_dir = fixture1.worktree_dir
        phase1_raw_root = fixture1.raw_root
        phase1_origin_dir = fixture1.origin_dir
        phase1_slug = slug1
        phase1_snapshot = snapshot1

    # Phase 2: verify repeated fixture calls run cleanly without collision, using captured phase 1 values
    with linked_worktree_fixture(branch_name="project-demo-2") as fixture2:
        assert fixture2.current_branch() == "project-demo-2"
        assert fixture2.current_branch() != phase1_branch
        assert fixture2.worktree_dir != phase1_worktree_dir
        assert fixture2.raw_root != phase1_raw_root
        assert fixture2.origin_dir != phase1_origin_dir

        slug2, snapshot2 = fixture2.setup_change_and_td("Second Isolated Invocation")

        # Compare phase 2 identities directly with captured phase 1 values while open
        assert slug2 != phase1_slug, f"WorkItem identity collision: {slug2} == {phase1_slug}"
        assert snapshot2["slug"] != phase1_snapshot["slug"]
        assert snapshot2["branch"] == "project-demo-2"
        assert snapshot2["branch"] != phase1_snapshot["branch"]
        assert fixture2.is_clean() is True
        assert snapshot2["state"] == "open"
        assert snapshot2["phase"] == "td_created"
        assert fixture2.remote_refs()["refs/heads/main"] == fixture2.initial_origin_main_sha

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify_case(CASE_ID, verify)
