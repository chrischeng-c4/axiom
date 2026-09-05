"""Closed release-plan schema and resumable apply contract."""
from __future__ import annotations

import argparse
import copy
import hashlib
import json
import sys
from pathlib import Path

import pytest
from typer.testing import CliRunner

from aw import main as cli

sys.path.insert(0, str(cli._SCRIPTS))
import release_plan


BASE = "a" * 40
NEXT = "b" * 40
REPO = "owner/repo"


def sha(text: str = "") -> str:
    return hashlib.sha256(text.encode()).hexdigest()


VALID_BODY = """## Goal

Calling the public API changes the observed result from old to new.

## How

### Verified premises

- `apps/demo/README.md:1` names the current public result.

### Change points

- Update `apps/demo/src/lib.rs`.

### Frozen decisions

none

## Acceptance

| # | command | current | target | why it cannot hold by accident |
|---|---|---|---|---|
| 1 | `cargo test -p demo` | fails | passes | The public result is asserted. |

### Negative control

Change the expected result. The command must fail. Restore sha256 `""" + "c" * 64 + """`.

## Never

This addresses the worker implementing this work item, not the controller.

### Must not touch

- `apps/other/src/lib.rs`

### Must not do

- Do not publish a release.
"""


MILESTONE_DESCRIPTION = """## Goal

Ship the approved demo product promise with executable delivery issues.

## Development Order

{{development_order}}

## Acceptance

- Every assigned issue is complete and every declared gate passes.
"""


def tracker(milestones: list[dict] | None = None,
            issues: list[dict] | None = None) -> dict:
    value = {
        "milestones": milestones or [],
        "issues": issues or [],
    }
    return {"sha256": release_plan.digest(value), **value}


def document(after: str = (
    "## Promise (Milestone #{{milestone_number}})\n\n"
    "- Outcome: `promise`. Tracking: "
    "[Milestone #{{milestone_number}}]"
    "(https://github.com/owner/repo/milestone/{{milestone_number}})\n"
)) -> dict:
    return {"path": "README.md", "before_sha256": sha("old\n"), "after": after}


def product(path: str = "apps/demo") -> dict:
    return {
        "path": path,
        "mode": "product",
        "tracker_baseline": tracker(),
        "documents": [document("new\n")],
    }


def release(path: str = "apps/demo", *, issues: list[dict] | None = None,
            baseline: dict | None = None, milestone: dict | None = None) -> dict:
    planned = issues or [{
        "key": "one",
        "title": "One observable delivery",
        "type": "feat",
        "priority": "p2",
        "owner": "app:demo",
        "body": VALID_BODY,
    }]
    return {
        "path": path,
        "mode": "release",
        "tracker_baseline": baseline or tracker(),
        "documents": [document()],
        "milestone": milestone or {
            "title": "demo@1.2.3",
            "description": MILESTONE_DESCRIPTION,
        },
        "issues": planned,
        "development_order": [row["key"] for row in planned],
    }


def plan(*projects: dict) -> dict:
    value = {
        "schema": release_plan.SCHEMA,
        "repo": REPO,
        "base_commit": BASE,
        "projects": list(projects),
    }
    value["plan_sha256"] = release_plan.plan_digest(value)
    return value


def reseal(value: dict) -> dict:
    value["plan_sha256"] = release_plan.plan_digest(value)
    return value


def expect_refused(value: dict, text: str) -> None:
    with pytest.raises(release_plan.PlanError, match=text):
        release_plan.validate_plan(value)


def test_accepts_product_release_and_ordered_multi_project_plans() -> None:
    value = plan(product(), release("libs/tool", issues=[{
        "key": "docs",
        "title": "Document the tool",
        "type": "docs",
        "priority": "p1",
        "owner": "lib:tool",
        "body": VALID_BODY.replace("apps/demo", "libs/tool"),
    }], milestone={
        "title": "tool@0.6.0",
        "description": MILESTONE_DESCRIPTION.replace("demo", "tool"),
    }))
    assert release_plan.validate_plan(value) == value
    assert len(release_plan.digest(value)) == 64


@pytest.mark.parametrize(
    ("mutate", "message"),
    [
        (lambda value: value.update(extra=True), "unknown field"),
        (lambda value: value["projects"].append(copy.deepcopy(value["projects"][0])),
         "duplicate project"),
        (lambda value: value["projects"][0]["milestone"].update(title="demo@01.2.3"),
         "SemVer"),
        (lambda value: value["projects"][0]["issues"][0].update(type="change"),
         "allowed delivery type"),
        (lambda value: value["projects"][0]["issues"][0].update(priority="p6"),
         "allowed priority"),
        (lambda value: value["projects"][0]["issues"][0].pop("priority"),
         "missing field"),
        (lambda value: value["projects"][0]["issues"][0].update(owner="apps/demo"),
         "owner label"),
        (lambda value: value["projects"][0].update(development_order=[]),
         "every issue key exactly once"),
        (lambda value: value["projects"][0].update(development_order=["one", "one"]),
         "every issue key exactly once"),
        (lambda value: value["projects"][0]["tracker_baseline"].update(sha256="d" * 64),
         "tracker baseline digest"),
        (lambda value: value.update(plan_sha256="d" * 64),
         "plan_sha256 does not match"),
        (lambda value: value["projects"][0]["issues"][0].update(command="gh issue create"),
         "unknown field"),
        (lambda value: value["projects"][0]["documents"][0].update(
            path="docs//promise.md"),
         "canonical relative path"),
    ],
)
def test_schema_rejects_ambiguous_or_unsafe_plans(mutate, message: str) -> None:
    value = plan(release())
    mutate(value)
    expect_refused(value, message)


def test_release_requires_number_and_order_placeholders() -> None:
    value = plan(release())
    value["projects"][0]["documents"][0]["after"] = "no binding\n"
    expect_refused(value, "milestone_number")

    value = plan(release())
    value["projects"][0]["milestone"]["description"] = \
        MILESTONE_DESCRIPTION.replace("{{development_order}}", "1. #99")
    expect_refused(value, "development_order")


def test_release_rejects_a_milestone_marker_outside_a_promise_heading() -> None:
    value = plan(release())
    value["projects"][0]["documents"][0]["after"] = \
        "Tracking: Milestone #{{milestone_number}}\n"
    expect_refused(value, "promise heading marker")


def test_release_requires_the_exact_repository_tracking_link() -> None:
    value = plan(release())
    value["projects"][0]["documents"][0]["after"] = \
        "## Promise (Milestone #{{milestone_number}})\n"
    expect_refused(value, "exact promise Tracking link")

    value = plan(release())
    value["projects"][0]["documents"][0]["after"] = (
        "## Promise (Milestone #{{milestone_number}})\n\n"
        "- Outcome: `promise`. Tracking: "
        "[Milestone #{{milestone_number}}]"
        "(https://github.com/other/repo/milestone/{{milestone_number}})\n"
    )
    expect_refused(value, "exact promise Tracking link")


def test_existing_records_bind_number_prior_digest_type_and_owner() -> None:
    milestone_row = {
        "number": 7,
        "title": "demo@1.2.3",
        "state": "OPEN",
        "description_sha256": sha("old milestone\n"),
    }
    issue_row = {
        "number": 10,
        "title": "Old title",
        "state": "OPEN",
        "labels": ["app:demo", "phase:created", "priority:p1", "type:feat"],
        "milestone": 7,
        "body_sha256": sha("old issue\n"),
    }
    existing_issue = {
        "key": "one",
        "number": 10,
        "prior_sha256": release_plan.digest(issue_row),
        "title": "New title",
        "type": "feat",
        "priority": "p1",
        "owner": "app:demo",
        "body": VALID_BODY,
    }
    existing_milestone = {
        "number": 7,
        "prior_sha256": release_plan.digest(milestone_row),
        "title": "demo@1.2.3",
        "description": MILESTONE_DESCRIPTION,
    }
    value = plan(release(
        baseline=tracker([milestone_row], [issue_row]),
        issues=[existing_issue],
        milestone=existing_milestone,
    ))
    assert release_plan.validate_plan(value) == value

    value["projects"][0]["issues"][0]["type"] = "fix"
    expect_refused(value, "existing issue type")


def test_existing_milestone_plan_must_cover_every_assigned_issue() -> None:
    milestone_row = {"number": 7, "title": "demo@1.2.3", "state": "OPEN",
                     "description_sha256": sha("old milestone\n")}
    issue_row = {"number": 10, "title": "Existing", "state": "OPEN",
                 "labels": ["app:demo", "phase:created", "priority:p1", "type:feat"],
                 "milestone": 7, "body_sha256": sha("old\n")}
    value = plan(release(
        baseline=tracker([milestone_row], [issue_row]),
        milestone={"number": 7, "prior_sha256": release_plan.digest(milestone_row),
                   "title": "demo@1.2.3", "description": MILESTONE_DESCRIPTION},
    ))
    expect_refused(value, "every baseline issue assigned")


def test_wis_evidence_records_delivery_gaps_but_blocks_planning_gaps(
        monkeypatch) -> None:
    population = {
        "project": "apps/demo", "label": "app:demo", "area_files": 1,
        "sections": 1, "milestones": 1, "work_items": 1,
    }
    delivery = release_plan.wis.Ledger()
    for rule in ("G1", "G2", "G3", "G4", "G5"):
        delivery.measured(rule, 1)
    delivery.cannot("G6", "apps/demo/e2e/ does not exist")
    delivery.measured("G7", 1)
    delivery.add("G7", "apps/demo/README.md", "gate target is missing")
    monkeypatch.setattr(
        release_plan.wis, "collect",
        lambda _root, _project, _repo: (delivery, population),
    )

    evidence = release_plan._wis_evidence(REPO, release())

    assert evidence["status"] == "DELIVERY_GAPS"
    assert evidence["rows"]["G6"]["blocked"] == \
        "apps/demo/e2e/ does not exist"
    assert evidence["rows"]["G7"]["gaps"] == 1

    planning = release_plan.wis.Ledger()
    for rule in release_plan.wis.GAPS:
        planning.measured(rule, 1)
    planning.add("G2", "Milestone #21", "no promise reaches it")
    monkeypatch.setattr(
        release_plan.wis, "collect",
        lambda _root, _project, _repo: (planning, population),
    )
    with pytest.raises(release_plan.PlanError, match="planning row.*G2"):
        release_plan._wis_evidence(REPO, release())


def test_validate_cli_delegates_without_writing(monkeypatch) -> None:
    seen = []
    monkeypatch.setattr(cli, "_delegate", lambda module, argv: seen.append((module, argv)))
    result = CliRunner().invoke(cli.app, ["release-plan", "validate", "--plan", "-"])
    assert result.exit_code == 0
    assert seen == [("release-plan", ["validate", "--plan", "-"])]


def test_validate_stdin_emits_one_sealed_canonical_plan(monkeypatch, capsys) -> None:
    value = plan(product())
    draft = copy.deepcopy(value)
    draft.pop("plan_sha256")
    monkeypatch.setattr(sys, "stdin", type("In", (), {
        "read": lambda self: json.dumps(draft),
    })())
    assert release_plan.main(["validate", "--plan", "-"]) == 0
    out = json.loads(capsys.readouterr().out)
    assert out == value
    assert out["plan_sha256"] == release_plan.plan_digest(out)


def test_plan_json_rejects_duplicate_fields(tmp_path) -> None:
    value = plan(product())
    raw = json.dumps(value).replace(
        f'"repo": "{REPO}"', f'"repo": "other/repo", "repo": "{REPO}"', 1,
    )
    path = tmp_path / "duplicate.json"
    path.write_text(raw, encoding="utf-8")
    with pytest.raises(release_plan.PlanError, match="duplicate field: repo"):
        release_plan.read_plan(str(path), stdin_ok=False)


def test_issue_writer_uses_only_the_planned_priority(tmp_path, monkeypatch) -> None:
    created = release()["issues"][0]
    created["priority"] = "p4"
    created_target = release(issues=[created])
    created_row = release_plan.desired_issue_row(created, None, 101, 21)
    calls = []
    monkeypatch.setattr(release_plan.change, "main", lambda argv: calls.append(argv) or 0)
    monkeypatch.setattr(
        release_plan, "_tracker_snapshot",
        lambda _repo, _target: {"milestones": [], "issues": [created_row]},
    )
    assert release_plan._write_planned_issue(
        REPO, created_target, created, 21, tmp_path,
    ) == 101
    assert ["--priority", "p4"] == calls[-1][calls[-1].index("--priority"):][:2]

    before = {
        "number": 10,
        "title": "Old",
        "state": "OPEN",
        "labels": ["app:demo", "phase:created", "priority:p1", "type:feat"],
        "milestone": 21,
        "body_sha256": sha("old\n"),
    }
    updated = {
        **created,
        "number": 10,
        "prior_sha256": release_plan.digest(before),
        "priority": "p3",
    }
    updated_target = release(issues=[updated], baseline=tracker(issues=[before]))
    updated_row = release_plan.desired_issue_row(updated, before, 10, 21)
    monkeypatch.setattr(
        release_plan, "_tracker_snapshot",
        lambda _repo, _target: {"milestones": [], "issues": [updated_row]},
    )
    assert release_plan._write_planned_issue(
        REPO, updated_target, updated, 21, tmp_path,
    ) == 10
    assert ["--remove-label", "priority:p1"] == \
        calls[-1][calls[-1].index("--remove-label"):][:2]
    assert ["--add-label", "priority:p3"] == \
        calls[-1][calls[-1].index("--add-label"):][:2]


class World:
    """Small live system used to inject crashes after accepted writes."""

    def __init__(self, root: Path, value: dict):
        self.root = root
        self.plan = value
        self.target = value["projects"][0]
        self.snapshot = copy.deepcopy(self.target["tracker_baseline"])
        self.snapshot.pop("sha256")
        self.head = BASE
        self.next_milestone = 21
        self.next_issue = 101
        self.calls = {"milestone": 0, "meta_commit": 0, "issue": 0, "finalize": 0,
                      "release_gates": 0, "product_gates": 0}
        project = root / self.target["path"]
        project.mkdir(parents=True)
        (project / "README.md").write_text("old\n", encoding="utf-8")

    def install(self, monkeypatch) -> None:
        monkeypatch.setattr(release_plan, "ROOT", self.root)
        monkeypatch.setattr(release_plan, "_head", lambda: self.head)
        monkeypatch.setattr(release_plan, "_dirty", lambda: [])
        monkeypatch.setattr(release_plan, "_dirty_paths", lambda: [])
        monkeypatch.setattr(release_plan, "_configured_repo", lambda: REPO)
        monkeypatch.setattr(
            release_plan, "_preview_documents",
            lambda _target, _start, _repository: None,
        )
        monkeypatch.setattr(
            release_plan, "_tracker_snapshot",
            lambda _repo, _target: copy.deepcopy(self.snapshot),
        )
        monkeypatch.setattr(
            release_plan, "_milestone_issue_numbers",
            lambda _repo, number: sorted(
                row["number"] for row in self.snapshot["issues"]
                if row["milestone"] == number
            ),
        )
        monkeypatch.setattr(release_plan, "_create_draft_milestone", self.create_milestone)
        monkeypatch.setattr(release_plan, "_commit_documents", self.commit_documents)
        monkeypatch.setattr(release_plan, "_is_expected_meta_commit", self.expected_commit)
        monkeypatch.setattr(release_plan, "_write_planned_issue", self.write_issue)
        monkeypatch.setattr(release_plan, "_finalize_milestone", self.finalize)
        monkeypatch.setattr(release_plan, "_run_release_gates", self.release_gates)
        monkeypatch.setattr(release_plan, "_run_product_gates", self.product_gates)

    def create_milestone(self, _repo: str, title: str, description: str) -> int:
        path = release_plan.receipt_path(self.plan["plan_sha256"], self.target["path"])
        assert path.is_file(), "receipt must exist before the first tracker write"
        self.calls["milestone"] += 1
        number = self.next_milestone
        self.next_milestone += 1
        self.snapshot["milestones"].append({
            "number": number,
            "title": title,
            "state": "OPEN",
            "description_sha256": sha(description),
        })
        self.snapshot["milestones"].sort(key=lambda row: row["number"])
        return number

    def commit_documents(self, _target: dict, rendered: list[dict],
                         start: str, _scratch: Path, *,
                         milestone_number: int | None,
                         repository: str) -> str:
        assert repository == REPO
        expected = None
        if self.target["mode"] == "release":
            expected = self.target["milestone"].get("number") or 21
        assert milestone_number == expected
        self.calls["meta_commit"] += 1
        assert self.head == start
        for row in rendered:
            path = self.root / self.target["path"] / row["path"]
            if row["after"] is None:
                path.unlink()
            else:
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(row["after"], encoding="utf-8")
        self.head = NEXT
        return self.head

    def expected_commit(self, start: str, commit: str, _target: dict,
                        rendered: list[dict]) -> bool:
        if start != BASE or commit != NEXT:
            return False
        for row in rendered:
            path = self.root / self.target["path"] / row["path"]
            if row["after"] is None:
                if path.exists():
                    return False
            elif not path.is_file() or path.read_text(encoding="utf-8") != row["after"]:
                return False
        return True

    def write_issue(self, _repo: str, _target: dict, planned: dict,
                    milestone_number: int, _scratch: Path) -> int:
        self.calls["issue"] += 1
        number = planned.get("number")
        if number is None:
            number = self.next_issue
            self.next_issue += 1
            before = None
        else:
            before = next(row for row in self.snapshot["issues"]
                          if row["number"] == number)
            self.snapshot["issues"].remove(before)
        self.snapshot["issues"].append(release_plan.desired_issue_row(
            planned, before, number, milestone_number,
        ))
        self.snapshot["issues"].sort(key=lambda row: row["number"])
        return number

    def finalize(self, _repo: str, number: int, description: str,
                 _scratch: Path) -> None:
        self.calls["finalize"] += 1
        row = next(row for row in self.snapshot["milestones"]
                   if row["number"] == number)
        row["description_sha256"] = sha(description)

    def release_gates(self, _repo: str, _target: dict, _milestone: int,
                      numbers: list[int]) -> dict:
        self.calls["release_gates"] += 1
        return {
            "reconcile": "ALIGNED", "order": numbers,
            "wis_gap": {"status": "ALIGNED", "rows": {}, "gaps": []},
        }

    def product_gates(self, _repo: str, _target: dict) -> dict:
        self.calls["product_gates"] += 1
        return {"tracker": "UNCHANGED"}


def write_plan(root: Path, value: dict) -> tuple[Path, str]:
    path = root / "plan.json"
    path.write_text(json.dumps(value), encoding="utf-8")
    return path, value["plan_sha256"]


def apply_args(path: Path, digest_: str, project: str = "apps/demo") -> argparse.Namespace:
    return argparse.Namespace(plan=str(path), project=project, approved_digest=digest_)


def test_product_mode_commits_documents_without_tracker_writes(tmp_path, monkeypatch) -> None:
    value = plan(product())
    world = World(tmp_path, value)
    world.install(monkeypatch)
    path, digest_ = write_plan(tmp_path, value)

    assert release_plan.cmd_apply(apply_args(path, digest_)) == 0
    receipt = release_plan.read_receipt(release_plan.receipt_path(digest_, "apps/demo"))
    assert receipt["state"] == "COMPLETE"
    assert world.calls == {
        "milestone": 0, "meta_commit": 1, "issue": 0, "finalize": 0,
        "release_gates": 0, "product_gates": 1,
    }


def test_product_mode_preserves_meta_create_and_delete(tmp_path, monkeypatch) -> None:
    target = product()
    target["documents"] = [
        {"path": "README.md", "before_sha256": sha("old\n"), "after": None},
        {"path": "docs/new.md", "before_sha256": None, "after": "new promise\n"},
    ]
    value = plan(target)
    world = World(tmp_path, value)
    world.install(monkeypatch)
    path, digest_ = write_plan(tmp_path, value)

    assert release_plan.cmd_apply(apply_args(path, digest_)) == 0
    assert not (tmp_path / "apps/demo/README.md").exists()
    assert (tmp_path / "apps/demo/docs/new.md").read_text() == "new promise\n"


def test_new_release_applies_draft_docs_issues_order_and_gates(tmp_path, monkeypatch) -> None:
    value = plan(release(issues=[
        release()["issues"][0],
        {"key": "two", "title": "Two", "type": "docs", "priority": "p0",
         "owner": "app:demo",
         "body": VALID_BODY},
    ]))
    world = World(tmp_path, value)
    world.install(monkeypatch)
    path, digest_ = write_plan(tmp_path, value)

    assert release_plan.cmd_apply(apply_args(path, digest_)) == 0
    receipt = release_plan.read_receipt(release_plan.receipt_path(digest_, "apps/demo"))
    assert receipt["state"] == "COMPLETE"
    assert receipt["milestone"] == {"status": "FINAL", "number": 21, "created": True}
    assert [row["number"] for row in receipt["issues"]] == [101, 102]
    assert receipt["evidence"]["order"] == [101, 102]
    assert (tmp_path / "apps/demo/README.md").read_text() == (
        "## Promise (Milestone #21)\n\n"
        "- Outcome: `promise`. Tracking: "
        "[Milestone #21](https://github.com/owner/repo/milestone/21)\n"
    )


def test_existing_milestone_and_issue_are_updated_not_created(tmp_path, monkeypatch) -> None:
    milestone_row = {"number": 7, "title": "demo@1.2.3", "state": "OPEN",
                     "description_sha256": sha("old milestone\n")}
    issue_row = {"number": 10, "title": "Old", "state": "OPEN",
                 "labels": ["app:demo", "phase:created", "priority:p1", "type:feat"],
                 "milestone": 7, "body_sha256": sha("old\n")}
    target = release(
        baseline=tracker([milestone_row], [issue_row]),
        milestone={"number": 7, "prior_sha256": release_plan.digest(milestone_row),
                   "title": "demo@1.2.3", "description": MILESTONE_DESCRIPTION},
        issues=[{"key": "one", "number": 10,
                 "prior_sha256": release_plan.digest(issue_row),
                 "title": "New", "type": "feat", "priority": "p3",
                 "owner": "app:demo",
                 "body": VALID_BODY}],
    )
    value = plan(target)
    world = World(tmp_path, value)
    world.install(monkeypatch)
    path, digest_ = write_plan(tmp_path, value)

    assert release_plan.cmd_apply(apply_args(path, digest_)) == 0
    assert world.calls["milestone"] == 0
    assert world.calls["issue"] == 1
    assert len(world.snapshot["issues"]) == 1
    assert world.snapshot["issues"][0]["number"] == 10
    assert "priority:p3" in world.snapshot["issues"][0]["labels"]
    assert "priority:p1" not in world.snapshot["issues"][0]["labels"]


@pytest.mark.parametrize("point", ["milestone", "meta_commit", "issue:one", "finalize"])
def test_resume_recovers_each_accepted_write_without_duplicates(
        tmp_path, monkeypatch, point: str) -> None:
    value = plan(release())
    world = World(tmp_path, value)
    world.install(monkeypatch)
    path, digest_ = write_plan(tmp_path, value)
    fired = False

    def fail_once(actual: str) -> None:
        nonlocal fired
        if actual == point and not fired:
            fired = True
            raise release_plan.PlanError(f"injected after {point}")

    monkeypatch.setattr(release_plan, "_after_write", fail_once)
    with pytest.raises(release_plan.PlanError, match="injected"):
        release_plan.cmd_apply(apply_args(path, digest_))

    receipt_path = release_plan.receipt_path(digest_, "apps/demo")
    assert release_plan.read_receipt(receipt_path)["state"] == "INCOMPLETE"
    counts = copy.deepcopy(world.calls)
    monkeypatch.setattr(release_plan, "_after_write", lambda _point: None)
    assert release_plan.cmd_resume(argparse.Namespace(receipt=str(receipt_path))) == 0
    assert release_plan.read_receipt(receipt_path)["state"] == "COMPLETE"

    if point == "milestone":
        assert world.calls["milestone"] == counts["milestone"]
    if point == "meta_commit":
        assert world.calls["meta_commit"] == counts["meta_commit"]
    if point == "issue:one":
        assert world.calls["issue"] == counts["issue"]
    if point == "finalize":
        assert world.calls["finalize"] == counts["finalize"]


def test_apply_failure_prints_the_incomplete_receipt_handoff(
        tmp_path, monkeypatch, capsys) -> None:
    value = plan(release())
    world = World(tmp_path, value)
    world.install(monkeypatch)
    path, digest_ = write_plan(tmp_path, value)
    monkeypatch.setattr(
        release_plan, "_after_write",
        lambda point: (_ for _ in ()).throw(release_plan.PlanError("stop"))
        if point == "milestone" else None,
    )

    assert release_plan.main([
        "apply", "--plan", str(path), "--project", "apps/demo",
        "--approved-digest", digest_,
    ]) == 1

    receipt_path = release_plan.receipt_path(digest_, "apps/demo")
    handoff = json.loads(capsys.readouterr().err.splitlines()[0])
    assert handoff == {
        "next_command": (
            "uv run --project apps/aw aw release-plan resume "
            f"--receipt {receipt_path.relative_to(tmp_path)}"
        ),
        "receipt": str(receipt_path.relative_to(tmp_path)),
        "state": "INCOMPLETE",
    }


def test_resume_failure_reprints_the_same_incomplete_receipt_handoff(
        tmp_path, monkeypatch, capsys) -> None:
    value = plan(release())
    world = World(tmp_path, value)
    world.install(monkeypatch)
    path, digest_ = write_plan(tmp_path, value)
    monkeypatch.setattr(
        release_plan, "_after_write",
        lambda point: (_ for _ in ()).throw(release_plan.PlanError("first stop"))
        if point == "milestone" else None,
    )
    with pytest.raises(release_plan.PlanError, match="first stop"):
        release_plan.cmd_apply(apply_args(path, digest_))
    receipt_path = release_plan.receipt_path(digest_, "apps/demo")
    monkeypatch.setattr(
        release_plan, "_after_write",
        lambda point: (_ for _ in ()).throw(release_plan.PlanError("second stop"))
        if point == "meta_commit" else None,
    )

    assert release_plan.main([
        "resume", "--receipt", str(receipt_path),
    ]) == 1

    handoff = json.loads(capsys.readouterr().err.splitlines()[0])
    assert handoff["receipt"] == str(receipt_path.relative_to(tmp_path))
    assert handoff["state"] == "INCOMPLETE"
    assert handoff["next_command"].endswith(
        f"release-plan resume --receipt {receipt_path.relative_to(tmp_path)}"
    )


def test_complete_resume_rejects_remote_drift(tmp_path, monkeypatch) -> None:
    value = plan(release())
    world = World(tmp_path, value)
    world.install(monkeypatch)
    path, digest_ = write_plan(tmp_path, value)
    assert release_plan.cmd_apply(apply_args(path, digest_)) == 0
    receipt_path = release_plan.receipt_path(digest_, "apps/demo")

    world.snapshot["issues"][0]["title"] = "external edit"
    with pytest.raises(release_plan.PlanError, match="tracker drift"):
        release_plan.cmd_resume(argparse.Namespace(receipt=str(receipt_path)))


def test_digest_or_project_refusal_happens_before_receipt_or_writes(
        tmp_path, monkeypatch) -> None:
    value = plan(release())
    world = World(tmp_path, value)
    world.install(monkeypatch)
    path, digest_ = write_plan(tmp_path, value)

    with pytest.raises(release_plan.PlanError, match="approved digest"):
        release_plan.cmd_apply(apply_args(path, "f" * 64))
    with pytest.raises(release_plan.PlanError, match="exactly one project"):
        release_plan.cmd_apply(apply_args(path, digest_, "apps/missing"))
    assert not release_plan.receipt_path(digest_, "apps/demo").exists()
    assert sum(world.calls.values()) == 0


def test_invalid_meta_preview_is_refused_before_receipt_or_remote_write(
        tmp_path, monkeypatch) -> None:
    value = plan(release())
    world = World(tmp_path, value)
    world.install(monkeypatch)
    path, digest_ = write_plan(tmp_path, value)
    monkeypatch.setattr(
        release_plan, "_preview_documents",
        lambda _target, _start, _repository: (_ for _ in ()).throw(
            release_plan.PlanError("planned META documents fail P10")
        ),
    )

    with pytest.raises(release_plan.PlanError, match="planned META documents fail"):
        release_plan.cmd_apply(apply_args(path, digest_))

    assert not release_plan.receipt_path(digest_, "apps/demo").exists()
    assert sum(world.calls.values()) == 0


def test_meta_preview_uses_a_disposable_clone(tmp_path, monkeypatch) -> None:
    root = tmp_path / "source"
    project = root / "apps/demo"
    project.mkdir(parents=True)
    (project / "README.md").write_text("old\n", encoding="utf-8")
    for argv in (
        ["git", "init", "--quiet"],
        ["git", "config", "user.name", "AW Test"],
        ["git", "config", "user.email", "aw@example.invalid"],
        ["git", "add", "apps/demo/README.md"],
        ["git", "commit", "--quiet", "-m", "baseline"],
    ):
        release_plan.subprocess.run(argv, cwd=root, check=True)
    start = release_plan.subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=root, check=True,
        capture_output=True, text=True,
    ).stdout.strip()
    observed: list[Path] = []

    def inspect_preview(preview: Path, _target: dict,
                        milestone_number: int | None,
                        repository: str) -> None:
        assert milestone_number is None
        assert repository == REPO
        observed.append(preview)
        assert preview != root
        assert (preview / "apps/demo/README.md").read_text(encoding="utf-8") == "new\n"
        staged = release_plan.subprocess.run(
            ["git", "diff", "--cached", "--name-only"], cwd=preview,
            check=True, capture_output=True, text=True,
        )
        assert staged.stdout.strip() == "apps/demo/README.md"

    monkeypatch.setattr(release_plan, "ROOT", root)
    monkeypatch.setattr(release_plan, "_check_rendered_meta", inspect_preview)
    release_plan._preview_documents(product(), start, REPO)

    assert observed
    assert not observed[0].exists()
    assert (project / "README.md").read_text(encoding="utf-8") == "old\n"


def _meta_preview_repo(
        tmp_path, first_title: str = "First",
        first_tracking: str = "not assigned.") -> tuple[Path, Path, str, str]:
    root = tmp_path / "source"
    project = root / "apps/demo"
    area = project / "docs/product/area.md"
    area.parent.mkdir(parents=True)
    first = (
        "- Problem: first problem.\n- Who: callers.\n- Promise: first promise.\n"
        "- Non-goals: none.\n- Open: none.\n- Neighbours: none.\n"
        f"- Outcome: `first`. Tracking: {first_tracking}\n"
    )
    second = (
        "- Problem: second problem.\n- Who: callers.\n- Promise: second promise.\n"
        "- Non-goals: none.\n- Open: none.\n- Neighbours: none.\n"
        "- Outcome: `second`. Tracking: not assigned.\n"
    )
    current = (
        "- Problem: current problem.\n- Who: callers.\n- Promise: current promise.\n"
        "- Non-goals: none.\n- Neighbours: none.\n"
        "- Status rows: `current`. Tracking: not assigned.\n"
    )
    before = (
        f"## {first_title}\n\n" + first + "\n## Second\n\n" + second
        + "\n## Current\n\n" + current + "\n## Non-goals in this area\n"
    )
    files = {
        "README.md": "# Demo\n",
        "STATUS.md": (
            "# Status\n\n## Support matrix\n\n"
            "| Surface | ID |\n|---|---|\n| Current | `current` |\n"
        ),
        "ROADMAP.md": (
            "# Roadmap\n\n## Near-term outcomes\n\n"
            "- ID: `first`\n- ID: `second`\n"
        ),
        "docs/product/README.md": (
            "## Section index\n\n| Section | File |\n|---|---|\n"
            "| First | area.md |\n| Second | area.md |\n"
            "| Current | area.md |\n"
        ),
        "docs/product/area.md": before,
    }
    for name, text in files.items():
        path = project / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
    validator = root / release_plan.metadoc.VALIDATOR
    validator.parent.mkdir(parents=True)
    validator.write_text(
        "import json\nprint(json.dumps({'reports': [{'ok': True}]}))\n",
        encoding="utf-8",
    )
    for argv in (
        ["git", "init", "--quiet"],
        ["git", "config", "user.name", "AW Test"],
        ["git", "config", "user.email", "aw@example.invalid"],
        ["git", "add", "."],
        ["git", "commit", "--quiet", "-m", "baseline"],
    ):
        release_plan.subprocess.run(argv, cwd=root, check=True)
    start = release_plan.subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=root, check=True,
        capture_output=True, text=True,
    ).stdout.strip()
    return root, area, before, start


def _install_meta_preview_mocks(monkeypatch, root: Path) -> None:
    monkeypatch.setattr(release_plan, "ROOT", root)
    monkeypatch.setattr(
        release_plan.metadoc, "pinned_launcher", lambda: [sys.executable],
    )
    monkeypatch.setattr(
        release_plan.meta, "collect", lambda _repo, rules, paths: (
            [], {"rules": list(rules), "paths": list(paths)},
        ),
    )


def _bind_future_promises(
        text: str, repository: str = REPO,
        number: int | str = release_plan.MILESTONE_NUMBER) -> str:
    link = release_plan.milestone_tracking_link(repository, number)
    for title in ("First", "Second"):
        text = text.replace(
            f"## {title}\n", f"## {title} (Milestone #{number})\n", 1,
        )
        text = text.replace(
            "Tracking: not assigned.", f"Tracking: {link}", 1,
        )
    return text


def test_new_milestone_preview_rejects_a_literal_binding_matching_first_probe(
        tmp_path, monkeypatch) -> None:
    root, area, before, start = _meta_preview_repo(tmp_path)
    target = release()
    after = _bind_future_promises(before).replace(
        "## Current", "Note: Milestone #1\n\n## Current", 1,
    )
    target["documents"] = [{
        "path": "docs/product/area.md", "before_sha256": sha(before), "after": after,
    }]
    _install_meta_preview_mocks(monkeypatch, root)

    with pytest.raises(release_plan.PlanError, match="unexpected Milestone binding"):
        release_plan._preview_documents(target, start, REPO)

    assert area.read_text(encoding="utf-8") == before


def test_new_milestone_preview_rejects_an_unbound_future_promise(
        tmp_path, monkeypatch) -> None:
    root, area, before, start = _meta_preview_repo(tmp_path)
    link = release_plan.milestone_tracking_link(
        REPO, release_plan.MILESTONE_NUMBER,
    )
    after = before.replace(
        "## First", "## First (Milestone #{{milestone_number}})", 1,
    ).replace("Tracking: not assigned.", f"Tracking: {link}", 1)
    target = release()
    target["documents"] = [{
        "path": "docs/product/area.md", "before_sha256": sha(before), "after": after,
    }]
    _install_meta_preview_mocks(monkeypatch, root)

    with pytest.raises(release_plan.PlanError, match="WIS.*G1"):
        release_plan._preview_documents(target, start, REPO)

    assert area.read_text(encoding="utf-8") == before


def test_new_milestone_preview_rejects_a_baseline_orphan_issue(
        tmp_path, monkeypatch) -> None:
    root, area, before, start = _meta_preview_repo(tmp_path)
    orphan = {
        "number": 88, "title": "Orphan", "state": "OPEN",
        "labels": ["app:demo", "phase:created", "priority:p2", "type:docs"],
        "milestone": None, "body_sha256": sha("orphan\n"),
    }
    target = release(baseline=tracker(issues=[orphan]))
    target["documents"] = [{
        "path": "docs/product/area.md", "before_sha256": sha(before),
        "after": _bind_future_promises(before),
    }]
    _install_meta_preview_mocks(monkeypatch, root)

    with pytest.raises(release_plan.PlanError, match="WIS.*G2"):
        release_plan._preview_documents(target, start, REPO)

    assert area.read_text(encoding="utf-8") == before


def test_new_milestone_preview_rejects_a_marker_only_in_tracking_prose(
        tmp_path, monkeypatch) -> None:
    root, area, before, start = _meta_preview_repo(tmp_path)
    after = before.replace(
        "## First\n\n",
        "## First\n\nTracking: (Milestone #{{milestone_number}})\n\n",
        1,
    )
    target = release()
    target["documents"] = [{
        "path": "docs/product/area.md", "before_sha256": sha(before), "after": after,
    }]
    _install_meta_preview_mocks(monkeypatch, root)

    with pytest.raises(release_plan.PlanError, match="exact-bound promise heading"):
        release_plan._preview_documents(target, start, REPO)

    assert area.read_text(encoding="utf-8") == before


def test_existing_milestone_preview_accepts_an_unchanged_exact_heading_binding(
        tmp_path, monkeypatch) -> None:
    root, area, before, start = _meta_preview_repo(
        tmp_path, "First (Milestone #7)",
        release_plan.milestone_tracking_link(REPO, 7),
    )
    after = before.replace(
        "First (Milestone #7)",
        "First (Milestone #{{milestone_number}})",
        1,
    ).replace(
        release_plan.milestone_tracking_link(REPO, 7),
        release_plan.milestone_tracking_link(
            REPO, release_plan.MILESTONE_NUMBER,
        ),
        1,
    ).replace(
        "## Second", "## Second (Milestone #{{milestone_number}})", 1,
    ).replace(
        "Tracking: not assigned.",
        "Tracking: " + release_plan.milestone_tracking_link(
            REPO, release_plan.MILESTONE_NUMBER,
        ),
        1,
    ).replace("first promise", "clarified first promise", 1)
    target = release()
    target["milestone"]["number"] = 7
    target["documents"] = [{
        "path": "docs/product/area.md", "before_sha256": sha(before), "after": after,
    }]
    _install_meta_preview_mocks(monkeypatch, root)

    release_plan._preview_documents(target, start, REPO)

    assert area.read_text(encoding="utf-8") == before


def test_preview_rejects_exact_and_wrong_duplicate_tracking_links(
        tmp_path, monkeypatch) -> None:
    root, area, before, start = _meta_preview_repo(tmp_path)
    exact = release_plan.milestone_tracking_link(
        REPO, release_plan.MILESTONE_NUMBER,
    )
    wrong = release_plan.milestone_tracking_link(
        "other/repo", release_plan.MILESTONE_NUMBER,
    )
    after = _bind_future_promises(before).replace(
        f"Tracking: {exact}", f"Tracking: {exact} and {wrong}", 1,
    )
    target = release()
    target["documents"] = [{
        "path": "docs/product/area.md", "before_sha256": sha(before), "after": after,
    }]
    _install_meta_preview_mocks(monkeypatch, root)

    with pytest.raises(release_plan.PlanError, match="exact Milestone Tracking link"):
        release_plan._preview_documents(target, start, REPO)

    assert area.read_text(encoding="utf-8") == before


def test_preview_rejects_a_tracking_binding_in_an_unbound_section(
        tmp_path, monkeypatch) -> None:
    root, area, before, start = _meta_preview_repo(tmp_path)
    wrong = release_plan.milestone_tracking_link(
        "other/repo", release_plan.MILESTONE_NUMBER,
    )
    after = _bind_future_promises(before).replace(
        "Tracking: not assigned.", f"Tracking: {wrong}", 1,
    )
    target = release()
    target["documents"] = [{
        "path": "docs/product/area.md", "before_sha256": sha(before), "after": after,
    }]
    _install_meta_preview_mocks(monkeypatch, root)

    with pytest.raises(release_plan.PlanError, match="outside an exact-bound"):
        release_plan._preview_documents(target, start, REPO)

    assert area.read_text(encoding="utf-8") == before


def test_new_milestone_preview_rejects_a_tracking_link_to_another_repo(
        tmp_path, monkeypatch) -> None:
    root, area, before, start = _meta_preview_repo(tmp_path)
    after = before.replace(
        "## First", "## First (Milestone #{{milestone_number}})", 1,
    ).replace(
        "Tracking: not assigned.",
        "Tracking: " + release_plan.milestone_tracking_link(
            "other/repo", release_plan.MILESTONE_NUMBER,
        ),
        1,
    )
    target = release()
    target["documents"] = [{
        "path": "docs/product/area.md", "before_sha256": sha(before), "after": after,
    }]
    _install_meta_preview_mocks(monkeypatch, root)

    with pytest.raises(release_plan.PlanError, match="exact Milestone Tracking link"):
        release_plan._preview_documents(target, start, REPO)

    assert area.read_text(encoding="utf-8") == before


def test_document_symlink_escape_is_refused_before_receipt(
        tmp_path, monkeypatch) -> None:
    target = product()
    target["documents"] = [{
        "path": "docs/link/out.md", "before_sha256": None, "after": "escape\n",
    }]
    value = plan(target)
    world = World(tmp_path, value)
    world.install(monkeypatch)
    outside = tmp_path / "outside"
    outside.mkdir()
    docs = tmp_path / "apps/demo/docs"
    docs.mkdir()
    (docs / "link").symlink_to(outside, target_is_directory=True)
    path, digest_ = write_plan(tmp_path, value)

    with pytest.raises(release_plan.PlanError, match="symbolic link"):
        release_plan.cmd_apply(apply_args(path, digest_))
    assert not (outside / "out.md").exists()
    assert not release_plan.receipt_path(digest_, "apps/demo").exists()
    assert sum(world.calls.values()) == 0


def test_existing_milestone_unowned_issue_is_refused_before_receipt(
        tmp_path, monkeypatch) -> None:
    milestone_row = {"number": 7, "title": "demo@1.2.3", "state": "OPEN",
                     "description_sha256": sha("old milestone\n")}
    issue_row = {"number": 10, "title": "Owned", "state": "OPEN",
                 "labels": ["app:demo", "phase:created", "priority:p1", "type:feat"],
                 "milestone": 7, "body_sha256": sha("old\n")}
    target = release(
        baseline=tracker([milestone_row], [issue_row]),
        milestone={"number": 7, "prior_sha256": release_plan.digest(milestone_row),
                   "title": "demo@1.2.3", "description": MILESTONE_DESCRIPTION},
        issues=[{"key": "one", "number": 10,
                 "prior_sha256": release_plan.digest(issue_row),
                 "title": "Owned", "type": "feat", "priority": "p1",
                 "owner": "app:demo", "body": VALID_BODY}],
    )
    value = plan(target)
    world = World(tmp_path, value)
    world.install(monkeypatch)
    monkeypatch.setattr(
        release_plan, "_milestone_issue_numbers", lambda _repo, _number: [10, 99],
    )
    path, digest_ = write_plan(tmp_path, value)

    with pytest.raises(release_plan.PlanError, match="unplanned or unowned"):
        release_plan.cmd_apply(apply_args(path, digest_))
    assert not release_plan.receipt_path(digest_, "apps/demo").exists()
    assert sum(world.calls.values()) == 0


def test_resume_rechecks_unowned_milestone_members_before_next_write(
        tmp_path, monkeypatch) -> None:
    value = plan(release())
    world = World(tmp_path, value)
    world.install(monkeypatch)
    path, digest_ = write_plan(tmp_path, value)
    monkeypatch.setattr(
        release_plan, "_after_write",
        lambda point: (_ for _ in ()).throw(release_plan.PlanError("stop"))
        if point == "milestone" else None,
    )
    with pytest.raises(release_plan.PlanError, match="stop"):
        release_plan.cmd_apply(apply_args(path, digest_))

    receipt_path = release_plan.receipt_path(digest_, "apps/demo")
    monkeypatch.setattr(release_plan, "_after_write", lambda _point: None)
    monkeypatch.setattr(
        release_plan, "_milestone_issue_numbers", lambda _repo, _number: [999],
    )
    with pytest.raises(release_plan.PlanError, match="unplanned or unowned"):
        release_plan.cmd_resume(argparse.Namespace(receipt=str(receipt_path)))

    assert release_plan.read_receipt(receipt_path)["state"] == "INCOMPLETE"
    assert world.calls["meta_commit"] == 0
    assert world.calls["issue"] == 0


def test_resume_refuses_unrelated_dirty_work_before_next_tracker_write(
        tmp_path, monkeypatch) -> None:
    value = plan(release())
    world = World(tmp_path, value)
    world.install(monkeypatch)
    path, digest_ = write_plan(tmp_path, value)
    monkeypatch.setattr(
        release_plan, "_after_write",
        lambda point: (_ for _ in ()).throw(release_plan.PlanError("stop"))
        if point == "meta_commit" else None,
    )
    with pytest.raises(release_plan.PlanError, match="stop"):
        release_plan.cmd_apply(apply_args(path, digest_))

    receipt_path = release_plan.receipt_path(digest_, "apps/demo")
    monkeypatch.setattr(release_plan, "_after_write", lambda _point: None)
    monkeypatch.setattr(
        release_plan, "_dirty_paths", lambda: ["apps/other/README.md"],
    )
    with pytest.raises(release_plan.PlanError, match="outside planned META"):
        release_plan.cmd_resume(argparse.Namespace(receipt=str(receipt_path)))

    assert release_plan.read_receipt(receipt_path)["state"] == "INCOMPLETE"
    assert world.calls["issue"] == 0


def test_apply_and_resume_share_one_exclusive_process_lock(
        tmp_path, monkeypatch) -> None:
    monkeypatch.setattr(release_plan, "ROOT", tmp_path)
    with release_plan._release_plan_lock():
        with pytest.raises(release_plan.PlanError, match="another release-plan"):
            with release_plan._release_plan_lock():
                pytest.fail("a second process lock was acquired")


def test_multi_project_apply_requires_each_prior_project_receipt(
        tmp_path, monkeypatch) -> None:
    value = plan(product(), product("libs/tool"))
    value["projects"][1]["documents"][0]["before_sha256"] = sha("tool old\n")
    value["projects"][1]["documents"][0]["after"] = "tool new\n"
    reseal(value)
    world = World(tmp_path, value)
    world.install(monkeypatch)
    tool = tmp_path / "libs/tool"
    tool.mkdir(parents=True)
    (tool / "README.md").write_text("tool old\n", encoding="utf-8")
    path, digest_ = write_plan(tmp_path, value)

    with pytest.raises(release_plan.PlanError, match="project order"):
        release_plan.cmd_apply(apply_args(path, digest_, "libs/tool"))
    assert not release_plan.receipt_path(digest_, "libs/tool").exists()


def test_complete_earlier_receipt_accepts_exact_later_project_chain(
        tmp_path, monkeypatch) -> None:
    value = plan(product(), product("libs/tool"))
    plan_sha = value["plan_sha256"]
    first = release_plan._new_receipt(value, plan_sha, value["projects"][0], BASE)
    first.update(state="COMPLETE", evidence={"tracker": "UNCHANGED"})
    first["meta"] = {"status": "APPLIED", "commit": NEXT}
    last_commit = "c" * 40
    second = release_plan._new_receipt(
        value, plan_sha, value["projects"][1], NEXT,
    )
    second.update(state="COMPLETE", evidence={"tracker": "UNCHANGED"})
    second["meta"] = {"status": "APPLIED", "commit": last_commit}
    monkeypatch.setattr(release_plan, "ROOT", tmp_path)
    first_path = release_plan.receipt_path(plan_sha, "apps/demo")
    second_path = release_plan.receipt_path(plan_sha, "libs/tool")
    release_plan._write_receipt(first_path, first)
    release_plan._write_receipt(second_path, second)
    monkeypatch.setattr(release_plan, "_head", lambda: last_commit)
    monkeypatch.setattr(release_plan, "_configured_repo", lambda: REPO)
    monkeypatch.setattr(
        release_plan, "_is_expected_meta_commit",
        lambda _start, _commit, _target, _rendered: True,
    )
    monkeypatch.setattr(
        release_plan, "_assert_tracker",
        lambda _repo, _target, expected, _where: expected,
    )
    monkeypatch.setattr(
        release_plan, "_run_product_gates",
        lambda _repo, _target: {"tracker": "UNCHANGED"},
    )

    assert release_plan.cmd_resume(argparse.Namespace(receipt=str(first_path))) == 0


def test_incomplete_receipt_refuses_a_second_apply(tmp_path, monkeypatch) -> None:
    value = plan(release())
    world = World(tmp_path, value)
    world.install(monkeypatch)
    path, digest_ = write_plan(tmp_path, value)
    monkeypatch.setattr(
        release_plan, "_after_write",
        lambda point: (_ for _ in ()).throw(release_plan.PlanError("stop"))
        if point == "milestone" else None,
    )
    with pytest.raises(release_plan.PlanError, match="stop"):
        release_plan.cmd_apply(apply_args(path, digest_))
    with pytest.raises(release_plan.PlanError, match="receipt already exists"):
        release_plan.cmd_apply(apply_args(path, digest_))
