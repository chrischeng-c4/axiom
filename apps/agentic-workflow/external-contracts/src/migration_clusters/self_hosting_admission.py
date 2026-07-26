"""Native Python ECs for AW self-hosting root admission."""

from __future__ import annotations

import hashlib
import json
import tempfile
from pathlib import Path
from typing import Any

from wi_contract_fixture import final_json, run_aw


CASE_IDS = {
    "self-hosting-bounded-admission",
    "self-hosting-capability-admission",
    "self-hosting-goal-root-parity",
    "self-hosting-health-policy",
    "self-hosting-identity-stability",
    "self-hosting-wi-admission",
    "wi-ec-td-root-loop-self-hosted-unit",
}

ISSUE_BODY = """\
## Problem

The fixture observes self-hosting admission without entering the lifecycle.

## Capability Alignment

Capability: Self-hosting policy
Capability Gap: root runners must not deadlock AW
Progress Evidence: policy envelope is stable

## Scope

### In Scope

- inspect self-hosting admission

### Out of Scope

- mutate product source

## Acceptance Criteria

- AC1: root admission returns sanctioned direct-commit policy

## Reference Context

### Related Specs

| Spec | Relevance |
|------|-----------|
| self-hosting.md | high |

### Spec Plan

| Spec ID | Action | Main Spec Ref |
|---------|--------|---------------|
| self-hosting | modify | self-hosting.md |
"""


def _tree_digest(root: Path) -> str:
    digest = hashlib.sha256()
    for path in sorted(item for item in root.rglob("*") if item.is_file()):
        digest.update(str(path.relative_to(root)).encode())
        digest.update(path.read_bytes())
    return digest.hexdigest()


def _runtime_root(root: Path) -> Path:
    raw = str(root.resolve())
    slug: list[str] = []
    last_dash = True
    for character in raw:
        if character.isascii() and character.isalnum():
            slug.append(character.lower())
            last_dash = False
        elif not last_dash:
            slug.append("-")
            last_dash = True
    return Path("/tmp/aw/workspaces") / "".join(slug).strip("-")


def _write_fixture(root: Path) -> str:
    (root / "aw.toml").write_text(
        """\
[agentic_workflow.workspace]
mode = "in_place"

[agentic_workflow.issue_platform]
type = "local"

[[projects]]
name = "agentic-workflow"
label = "app:agentic-workflow"
path = "."
tech_design_path = "tech-design"

[[projects.workspaces]]
name = "agentic-workflow"
paths = ["**"]
target = "rust"
""",
        encoding="utf-8",
    )
    created = final_json(
        run_aw(
            root,
            "wi",
            "create",
            "--title",
            "Self-hosted fixture",
            "--type",
            "change",
            "--project",
            "agentic-workflow",
            "--body",
            ISSUE_BODY,
        )
    )
    return created["slug"]


def _policy(payload: dict[str, Any]) -> None:
    assert payload["action"] == "self_hosting_policy"
    assert payload["policy_mode"] == "sanctioned_direct_commit"
    assert payload["root_runner_allowed"] is False
    assert payload["required_trailer"] == "Refs #<issue>"
    assert "invoke" not in payload
    assert payload["completion"]["workflow_complete"] is False


def _snapshot() -> dict[str, Any]:
    with tempfile.TemporaryDirectory(prefix="aw-python-ec-self-hosting-") as raw_root:
        root = Path(raw_root)
        slug = _write_fixture(root)
        runtime = _runtime_root(root)
        sentinel = runtime / ".self-hosting-policy-sentinel"
        sentinel.parent.mkdir(parents=True, exist_ok=True)
        sentinel.write_text("must remain byte-identical", encoding="utf-8")
        before = _tree_digest(root)
        runtime_before = _tree_digest(runtime)
        wi_first = final_json(run_aw(root, "goal", "wi", slug))
        wi_second = final_json(run_aw(root, "goal", "wi", slug))
        capability = final_json(
            run_aw(root, "goal", "capability", "--project", "agentic-workflow")
        )
        backlog = final_json(
            run_aw(root, "goal", "backlog", "--project", "agentic-workflow")
        )
        after = _tree_digest(root)
        for payload in (wi_first, wi_second, capability, backlog):
            _policy(payload)
        assert wi_first == wi_second
        assert before == after
        assert runtime_before == _tree_digest(runtime)

        issue_path = next(runtime.glob(f"issues/*/{slug}.md"))
        issue_path.write_text("not valid issue frontmatter", encoding="utf-8")
        malformed_before = _tree_digest(runtime)
        malformed = run_aw(
            root,
            "goal",
            "wi",
            slug,
            expect_success=False,
        )
        assert "frontmatter" in malformed.stderr
        assert before == _tree_digest(root)
        assert malformed_before == _tree_digest(runtime)
        return {
            "slug": slug,
            "wi": wi_first,
            "capability": capability,
            "backlog": backlog,
            "malformed_stderr": malformed.stderr,
        }


def verify(case_id: str) -> list[str]:
    if case_id not in CASE_IDS:
        raise AssertionError(f"case is not owned by workflow-admission: {case_id}")
    snapshot = _snapshot()
    if case_id == "self-hosting-bounded-admission":
        return [
            "self-hosted backlog admission emits sanctioned direct-commit policy",
            "repeat admission is byte-stable and leaves the fixture tree unchanged",
        ]
    if case_id == "self-hosting-capability-admission":
        assert snapshot["capability"]["root"]["kind"] == "project"
        assert snapshot["backlog"]["root"]["kind"] == "backlog"
        return [
            "capability and backlog roots reject the self-hosted root runner",
            "both policies expose no invoke command or lifecycle mutation",
        ]
    if case_id == "self-hosting-goal-root-parity":
        return [
            "WI, capability, and backlog roots share the same self-hosting policy",
            "admission rejects before loop-state dispatch",
        ]
    if case_id == "self-hosting-health-policy":
        policy = snapshot["wi"]
        assert policy["hard_gates"] == [
            "capability_work_root_alignment",
            "closing_work_item_and_td_refs",
            "configured_ec_claim_verification",
        ]
        assert policy["advisory_axes"]
        return [
            "self-hosting policy pins its ordered hard gates and advisory axes",
            "root_runner_allowed remains false and no aw goal remediation is emitted",
        ]
    if case_id == "self-hosting-identity-stability":
        return [
            "malformed self-hosted WI identity returns a process error",
            "failed identity resolution leaves the fixture tree unchanged",
        ]
    if case_id == "self-hosting-wi-admission":
        assert snapshot["wi"]["root"]["kind"] == "wi"
        return [
            "self-hosted WI root emits policy before dispatch",
            "the envelope exposes no invoke command and causes no repository mutation",
        ]
    return [
        "self-hosted WI admission remains outside the EC-TD-CB child loop",
        "the policy names focused artifacts and health as the sanctioned continuation",
    ]
