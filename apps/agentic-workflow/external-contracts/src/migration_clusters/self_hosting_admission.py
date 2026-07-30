"""Native Python ECs for Python-first AW self-hosting root admission."""

from __future__ import annotations

import hashlib
import json
import tempfile
from pathlib import Path
from typing import Any

from wi_contract_fixture import (
    final_json,
    run_aw,
    write_python_artifact_lock,
    write_python_artifact_unit_test,
)


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

The fixture observes self-hosting entering the normal Python-first lifecycle.

## Capability Alignment

Capability: Workflow root runner
Capability Gap: self-hosting roots must dogfood the Python lifecycle
Progress Evidence: normal root envelopes are machine-actionable

## Scope

### In Scope

- enter self-hosting admission

### Out of Scope

- mutate product source

## Acceptance Criteria

- AC1: root admission returns the normal lifecycle continuation

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


def _write_claim_linkage(root: Path) -> None:
    """Give the fixture capability claim its derived Python TD/EC linkage.

    A `python-v1` capability claim is linked when an EC case names
    `(capability_id, use_case_id, artifact_id)` and a public Python TD module
    exposes a public behavior of the same name, so a fixture that omits either
    side cannot reach production readiness on its capability root.
    """
    td_root = root / "tech-design"
    (td_root / "src").mkdir(parents=True, exist_ok=True)
    (td_root / "pyproject.toml").write_text(
        """\
[project]
name = "self-hosting-tech-design"
version = "0.1.0"
requires-python = ">=3.11"
""",
        encoding="utf-8",
    )
    write_python_artifact_lock(td_root, name="self-hosting-tech-design")
    (td_root / "src/workflow_root_runner.py").write_text(
        '''\
__aw_artifact_id__ = "artifact:agentic-workflow/workflow-root-runner"
__aw_public_contract__ = True


def self_hosted_fixture() -> str:
    return "Python-first lifecycle roots drive an executable continuation"
''',
        encoding="utf-8",
    )

    ec_root = root / "external-contracts"
    (ec_root / "src/cases").mkdir(parents=True, exist_ok=True)
    (ec_root / "evidence").mkdir(parents=True, exist_ok=True)
    (ec_root / "src/runner.py").write_text(
        'print("fixture runner")\n', encoding="utf-8"
    )
    (ec_root / "src/cases/self_hosted_fixture.py").write_text(
        "def verify_self_hosted_fixture() -> list[str]:\n"
        '    return ["the self-hosted fixture claim is externally observable"]\n',
        encoding="utf-8",
    )
    (ec_root / "evidence/self-hosted-fixture.json").write_text(
        '{"protocol":"aw.python-ec.evidence.v1","exit_code":0}\n', encoding="utf-8"
    )
    (ec_root / "pyproject.toml").write_text(
        """\
[project]
name = "self-hosting-external-contracts"
version = "0.1.0"
requires-python = ">=3.11"

[tool.aw.python-artifact]
protocol = "aw.python-artifact.v1"
entrypoint = "src/runner.py"
source_roots = ["src"]
dependency_files = ["pyproject.toml", "uv.lock"]
evidence_dir = "evidence"

[tool.aw.python-ec]
protocol = "aw.python-ec.v1"
author = "agent:fixture"
efficiency_policy = "not-applicable"

[[tool.aw.python-ec.cases]]
id = "self-hosted-fixture-behavior"
artifact_id = "artifact:agentic-workflow/workflow-root-runner"
capability_id = "workflow-root-runner"
use_case_id = "self-hosted-fixture"
dimension = "behavior"
applicability = "td"
test_path = "src/cases/self_hosted_fixture.py"
promise = "Python-first lifecycle roots drive an executable continuation."
oracle = "the outer EC independently inspects the real aw goal envelopes"
target = "rust"
command = "true"
evidence_paths = ["evidence/self-hosted-fixture.json"]
""",
        encoding="utf-8",
    )
    write_python_artifact_lock(ec_root, name="self-hosting-external-contracts")
    write_python_artifact_unit_test(ec_root, "self_hosted_fixture")


def _write_fixture(root: Path, project: str = "agentic-workflow") -> str:
    (root / "aw.toml").write_text(
        f"""\
[agentic_workflow.workspace]
mode = "in_place"

[agentic_workflow.issue_platform]
type = "local"

[[projects]]
name = "{project}"
label = "app:{project}"
path = "."
tech_design_path = "tech-design"

[[projects.workspaces]]
name = "{project}"
paths = ["**"]
target = "rust"
""",
        encoding="utf-8",
    )
    (root / "CAPABILITIES.md").write_text(
        """\
# Agentic Workflow Fixture

## Brief

Self-hosting admission contract.

## Capabilities

### Capability Index

| Capability | ID | Status | Evidence |
|------------|----|--------|----------|
| Workflow root runner | workflow-root-runner | implemented | `true` |

### Workflow root runner

ID: workflow-root-runner
Type: AgentFirst
Root WI: self-hosted-fixture
Status: verified
Required Verification: smoke
Surfaces:
- CLI: `aw goal` - drive Python-first lifecycle roots.
EC Dimensions:
- behavior: `aw goal capability` - dispatch the next lifecycle worker.
Promise:
Drive Python-first lifecycle roots through an executable continuation.
Gate Inventory:
- `true`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Self-hosted fixture | change | self-hosted-fixture | implemented | verified | smoke | `true` |
""",
        encoding="utf-8",
    )
    _write_claim_linkage(root)
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
            project,
            "--body",
            ISSUE_BODY,
        )
    )
    return created["slug"]


def _normal_root(payload: dict[str, Any]) -> None:
    assert payload["action"] != "self_hosting_policy"
    assert payload.get("policy_mode") != "sanctioned_direct_commit"
    assert payload["next"]["kind"] != "policy"


def _health_policy_snapshot(project: str) -> dict[str, Any]:
    with tempfile.TemporaryDirectory(
        prefix=f"aw-python-ec-self-hosting-health-{project}-"
    ) as raw_root:
        root = Path(raw_root)
        _write_fixture(root, project)
        # Self-hosting policy metadata is orthogonal to the readiness verdict, so
        # the exit status must not be the oracle here. The case pins the policy
        # field set instead and asserts the two fixtures agree on readiness.
        return final_json(
            run_aw(
                root,
                "health",
                "--project",
                project,
                expect_success=None,
            )
        )


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
        runtime_after_first = _tree_digest(runtime)
        wi_second = final_json(run_aw(root, "goal", "wi", slug))
        capability = final_json(
            run_aw(
                root,
                "goal",
                "capability",
                "workflow-root-runner",
                "--project",
                "agentic-workflow",
            )
        )
        backlog = final_json(
            run_aw(root, "goal", "backlog", "--project", "agentic-workflow")
        )
        after = _tree_digest(root)
        for payload in (wi_first, wi_second, capability, backlog):
            _normal_root(payload)
        assert wi_first == wi_second
        assert before == after
        assert runtime_before == runtime_after_first
        assert runtime_after_first == _tree_digest(runtime)

        issue_path = next(runtime.glob(f"issues/*/{slug}.md"))
        issue_path.write_text("not valid issue frontmatter", encoding="utf-8")
        malformed_before = _tree_digest(runtime)
        malformed = final_json(run_aw(root, "goal", "wi", slug))
        assert malformed["action"] == "blocked"
        assert "inventory unavailable" in malformed["next"]["reason"]
        assert before == _tree_digest(root)
        assert malformed_before == _tree_digest(runtime)
        return {
            "slug": slug,
            "wi": wi_first,
            "capability": capability,
            "backlog": backlog,
            "malformed": malformed,
        }


def verify(case_id: str) -> list[str]:
    if case_id not in CASE_IDS:
        raise AssertionError(f"case is not owned by workflow-admission: {case_id}")
    if case_id == "self-hosting-health-policy":
        self_hosted = _health_policy_snapshot("agentic-workflow")
        control = _health_policy_snapshot("demo")
        assert self_hosted["action"] == "health"
        assert self_hosted["policy_mode"] == "python_first_lifecycle"
        assert self_hosted["root_runner_allowed"] is True
        assert self_hosted["direct_repair_default"] is False
        assert self_hosted["direct_repair_fallback"] == "bounded_direct_repair"
        assert self_hosted["fallback_trigger"] == "current_worker_verb_broken"
        assert self_hosted["required_trailer"] == "Refs #<issue>"
        for field in (
            "policy_mode",
            "root_runner_allowed",
            "direct_repair_default",
            "direct_repair_fallback",
            "fallback_trigger",
            "required_trailer",
        ):
            assert field not in control
        # The two fixtures differ only in which project the policy applies to, so
        # the policy fields must not be a side effect of a different readiness
        # verdict.
        assert self_hosted["status"] == control["status"], (
            self_hosted["status"],
            control["status"],
        )
        assert (
            self_hosted["readiness"]["production_ready"]
            == control["readiness"]["production_ready"]
        ), (self_hosted["readiness"], control["readiness"])
        return [
            "public AW health enables Python-first self-hosting roots",
            "bounded direct repair is conditional self-hosting fallback metadata",
            "the self-hosting policy field set is absent for a non-self-hosting project at the same readiness verdict",
        ]
    snapshot = _snapshot()
    if case_id == "self-hosting-bounded-admission":
        assert snapshot["backlog"]["action"] == "blocked"
        assert snapshot["backlog"]["next"]["command"].startswith("aw wi plan ")
        return [
            "self-hosted backlog admission reaches reviewed-graph preflight",
            "repeat WI admission is byte-stable and does not mutate lifecycle state",
        ]
    if case_id == "self-hosting-capability-admission":
        capability = snapshot["capability"]
        assert capability["status"] == "continue"
        assert capability["action"] == "done"
        assert capability["root"] == {
            "kind": "capability",
            "id": "workflow-root-runner",
        }
        assert capability["completion"]["root_complete"] is True
        assert capability["completion"]["workflow_complete"] is False
        assert (
            capability["next"]["command"]
            == "aw goal capability --project agentic-workflow --non-interactive --max-ticks 1"
        )
        assert (
            capability["next"]["reason"]
            == "capability is production ready; inspect project root for rollup"
        )
        backlog = snapshot["backlog"]
        assert backlog["status"] == "blocked"
        assert backlog["action"] == "blocked"
        assert backlog["root"] == {
            "kind": "backlog",
            "id": "agentic-workflow",
        }
        assert (
            backlog["next"]["command"]
            == "aw wi plan --project agentic-workflow --json"
        )
        assert backlog["next"]["reason"].startswith(
            "current reviewed project graph is unavailable:"
        )
        assert "project-plan.json cannot be read" in backlog["next"]["reason"]
        return [
            "scoped capability admission completes without unrelated project lifecycle gates",
            "backlog fail-closes on the missing reviewed graph with exact planning remediation",
        ]
    if case_id == "self-hosting-goal-root-parity":
        return [
            "WI, capability, and backlog roots share normal lifecycle admission",
            "no root returns the retired self-hosting policy envelope",
        ]
    if case_id == "self-hosting-identity-stability":
        return [
            "malformed self-hosted WI state returns a normal blocked envelope",
            "failed identity resolution leaves repository and runtime state unchanged",
        ]
    if case_id == "self-hosting-wi-admission":
        wi = snapshot["wi"]
        assert wi["status"] == "continue"
        assert wi["action"] == "dispatch"
        assert wi["root"] == {
            "kind": "change",
            "id": snapshot["slug"],
        }
        assert wi["prompt_contract"]["state"] == "ec.authoring"
        assert (
            wi["next"]["command"]
            == f"aw ec check --project agentic-workflow --wi {snapshot['slug']}"
        )
        assert (
            wi["next"]["reason"]
            == "the Python artifact lifecycle starts EC-first: author external-contracts Python source, then structurally check its contract"
        )
        return [
            "self-hosted WI root enters the normal EC-first lifecycle",
            "the envelope dispatches the exact project-and-WI-scoped EC check",
        ]
    return [
        "self-hosted WI admission enters the EC-TD-CB child loop",
        "normal lifecycle gates remain fail-closed",
    ]
