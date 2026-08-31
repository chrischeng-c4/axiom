#!/usr/bin/env python3
"""Pure checks for Milestone identity, description, ownership, and order."""

from __future__ import annotations

import argparse
import contextlib
import io
import json
import sys
from pathlib import Path


SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import milestone  # noqa: E402
import workitem  # noqa: E402
import change  # noqa: E402
import wis  # noqa: E402


failed: list[str] = []


def check(label: str, ok: bool, detail: str = "") -> None:
    suffix = f" -- {detail}" if detail else ""
    print(f"{'PASS' if ok else 'FAIL'} {label}{suffix}")
    if not ok:
        failed.append(label)


def description(order: str = "1. #101\n2. #102") -> str:
    return f"""## Goal

使用者可以從公開介面觀察到這個版本已交付。

## Development Order

{order}

## Acceptance

- The named release checks pass with exact evidence.
"""


def issue(number: int, *, state: str = "OPEN", labels: tuple[str, ...] = (
        "type:feat", "app:tape"), body: str = "") -> dict:
    return {
        "number": number,
        "title": f"change {number}",
        "state": state,
        "labels": list(labels),
        "url": f"https://example.invalid/issues/{number}",
        "body": body,
    }


def main() -> int:
    valid = milestone.release_identity("tape@0.4.63")
    check("release title parses", valid is not None and valid.version == (0, 4, 63))
    for title in ("tape@0.4.64", "tape@0.64.0", "tape@00.4.1", "tape@0.4", "Tape@0.4.1"):
        check(f"release title refuses {title}", milestone.release_identity(title) is None)

    ready = description()
    check("strict description accepts Chinese observable goal",
          milestone.validate_description(ready) == [],
          repr(milestone.validate_description(ready)))
    draft = description(milestone.DRAFT_LINE)
    check("draft line is refused by the strict gate",
          bool(milestone.validate_description(draft)))
    check("draft line is accepted only by the draft gate",
          milestone.validate_description(draft, allow_draft=True) == [])
    check("parenthesis ranks are refused",
          any("exactly" in error for error in milestone.validate_description(description("1) #101"))))
    check("trailing order prose is refused",
          any("exactly" in error for error in milestone.validate_description(description("1. #101 first"))))
    check("non-contiguous ranks are refused",
          any("contiguous" in error for error in milestone.validate_description(description("1. #101\n3. #102"))))
    check("duplicate issues are refused",
          any("duplicate" in error for error in milestone.validate_description(description("1. #101\n2. #101"))))

    release = {"number": 7, "title": "tape@0.4.63", "state": "open",
               "description": ready}
    issues = [issue(101), issue(102, state="CLOSED")]
    payload = milestone.order_payload(release, issues)
    check("valid ownership and order are orderable", payload["orderable"], repr(payload["errors"]))
    check("declared order is preserved",
          [row["number"] for row in payload["order"]] == [101, 102])
    open_payload = milestone.order_payload(release, issues, open_only=True)
    check("open-only filters after validation",
          [row["number"] for row in open_payload["order"]] == [101])

    missing = milestone.order_payload(release, [issue(101), issue(102), issue(103)])
    check("assigned issue missing from order is refused",
          any("#103" in error and "missing" in error for error in missing["errors"]))
    foreign_release = dict(release, description=description("1. #101\n2. #999"))
    foreign = milestone.order_payload(foreign_release, [issue(101)])
    check("ordered issue not assigned is refused",
          any("#999" in error and "not assigned" in error for error in foreign["errors"]))
    intake = milestone.order_payload(
        release, [issue(101), issue(102, labels=("type:report", "app:tape"))])
    check("intake child is refused",
          any("#102" in error and "intake `type:report`" in error
              for error in intake["errors"]), repr(intake["errors"]))
    legacy = milestone.order_payload(
        release, [issue(101), issue(102, labels=("type:change", "app:tape"))])
    check("legacy child is refused",
          any("#102" in error and "retired `type:change`" in error
              for error in legacy["errors"]), repr(legacy["errors"]))
    dual = milestone.order_payload(
        release, [issue(101), issue(102, labels=("type:feat", "type:fix", "app:tape"))])
    check("dual-type child is refused",
          any("#102" in error and "exactly one" in error for error in dual["errors"]),
          repr(dual["errors"]))
    missing_type = milestone.order_payload(
        release, [issue(101), issue(102, labels=("app:tape",))])
    check("untyped child is refused",
          any("#102" in error and "exactly one" in error
              for error in missing_type["errors"]), repr(missing_type["errors"]))
    unknown_type = milestone.order_payload(
        release, [issue(101), issue(102, labels=("type:security", "app:tape"))])
    check("unknown-type child is refused",
          any("#102" in error and "unknown `type:security`" in error
              for error in unknown_type["errors"]), repr(unknown_type["errors"]))
    wrong_project = milestone.order_payload(
        release, [issue(101), issue(102, labels=("type:feat", "app:lumen"))])
    check("wrong-project child is refused",
          any("#102" in error and "app:tape" in error for error in wrong_project["errors"]))

    mixed_release = dict(
        release,
        description=description("1. #101\n2. #102\n3. #103"),
    )
    mixed = [
        issue(101, labels=("type:docs", "app:tape")),
        issue(102, labels=("type:feat", "app:tape")),
        issue(103, labels=("type:chore", "app:tape")),
    ]
    check("mixed delivery flows share one global order",
          milestone.order_payload(mixed_release, mixed)["orderable"])

    original_next_resolve = milestone.resolve_milestone
    original_next_issues = milestone.milestone_issues
    original_next_fetch = workitem.fetch_issue
    current = {row["number"]: dict(row) for row in mixed}
    milestone.resolve_milestone = lambda _ref, _repo: dict(mixed_release)
    milestone.milestone_issues = lambda _target, _repo: [dict(row) for row in current.values()]
    workitem.fetch_issue = lambda iid, _repo: dict(current[int(iid)])

    def next_json() -> tuple[int, dict]:
        stream = io.StringIO()
        with contextlib.redirect_stdout(stream):
            code = milestone.cmd_next(argparse.Namespace(
                ref="milestone:7", repo="owner/repo", json=True,
            ))
        return code, json.loads(stream.getvalue()) if code == 0 else {}

    try:
        code, row = next_json()
        check("queue begins with docs maintenance",
              code == 0 and row.get("iid") == 101
              and row.get("type") == "docs" and row.get("flow") == "maintenance"
              and row.get("next_phase") == "maint", repr(row))
        current[101]["body"] = workitem.lifecycle_upsert(
            "", "maint", "a" * 40, "b" * 64
        )
        code, row = next_json()
        check("completed maint evidence makes close the only next phase",
              code == 0 and row.get("iid") == 101
              and row.get("next_phase") == "close", repr(row))
        current[101]["state"] = "CLOSED"
        code, row = next_json()
        check("closed docs advances to feat e2e",
              code == 0 and row.get("iid") == 102
              and row.get("flow") == "behavior"
              and row.get("next_phase") == "e2e", repr(row))
        current[102]["body"] = workitem.lifecycle_upsert(
            "", "e2e", "c" * 40, "d" * 64
        )
        code, row = next_json()
        check("feat e2e evidence advances only to impl",
              code == 0 and row.get("iid") == 102
              and row.get("next_phase") == "impl", repr(row))
        current[102]["state"] = "CLOSED"
        code, row = next_json()
        check("closed feat advances to chore maint",
              code == 0 and row.get("iid") == 103
              and row.get("type") == "chore"
              and row.get("next_phase") == "maint", repr(row))

        current[101]["state"] = "OPEN"
        current[101]["body"] = ""
        current[103]["labels"] = ["type:report", "app:tape"]
        stream = io.StringIO()
        with contextlib.redirect_stdout(stream):
            invalid_code = milestone.cmd_next(argparse.Namespace(
                ref="milestone:7", repo="owner/repo", json=True,
            ))
        check("next validates an illegal tail before returning the valid head",
              invalid_code == 1 and "#103" in stream.getvalue(), stream.getvalue())
    finally:
        milestone.resolve_milestone = original_next_resolve
        milestone.milestone_issues = original_next_issues
        workitem.fetch_issue = original_next_fetch

    try:
        milestone.resolve_milestone("7", "owner/repo")
    except workitem.GhError as exc:
        check("bare Milestone number is refused before network access",
              "ambiguous" in str(exc))
    else:
        check("bare Milestone number is refused before network access", False)

    # The project-labelled issue population must be complete and must exclude
    # pull requests returned by the REST issues endpoint.
    gh_calls: list[tuple[str, ...]] = []
    original_gh = workitem.gh
    workitem.gh = lambda *argv, **_kwargs: (
        gh_calls.append(argv) or json.dumps([[
            {
                "number": 1, "title": "one", "state": "open",
                "labels": [{"name": "app:tape"}],
                "html_url": "https://example.invalid/issues/1",
                "milestone": {"number": 7, "title": "tape@0.4.63"},
            },
            {
                "number": 2, "title": "pull", "state": "open",
                "labels": [], "html_url": "https://example.invalid/pull/2",
                "pull_request": {},
            },
        ], [
            {
                "number": 3, "title": "three", "state": "closed",
                "labels": [{"name": "type:feat"}],
                "html_url": "https://example.invalid/issues/3",
                "milestone": None,
            },
        ]])
    )
    try:
        population = workitem.fetch_issues_by_label("app:tape", "owner/repo")
    finally:
        workitem.gh = original_gh
    check("label population uses paginated REST",
          bool(gh_calls) and "--paginate" in gh_calls[0] and "--slurp" in gh_calls[0]
          and "labels=app%3Atape" in gh_calls[0][-1], repr(gh_calls))
    check("label population spans pages and removes pull requests",
          [row["number"] for row in population] == [1, 3])

    ledger = wis.Ledger()
    promise = wis.Promise(
        path="apps/tape/docs/area.md", title="Future", milestone_number=7,
        shape="future", outcome="surface", rows=(),
    )
    wis.g2_orphan_items(
        ledger,
        [{"number": 7, "title": "tape@0.4.63", "state": "OPEN"}],
        [{
            "number": 101, "title": "wrong owner", "state": "OPEN",
            "labels": ["type:feat", "app:tape"],
            "milestone": {"number": 8, "title": "lumen@0.4.63"},
        }],
        [promise],
        "app:tape",
    )
    check("G2 refuses a change assigned to another project's Milestone",
          any("not a release Milestone for this project" in gap.message
              for gap in ledger.gaps))

    original_list = milestone.list_milestones
    milestone.list_milestones = lambda _repo, _state="all": [
        {"number": 7, "title": "tape@0.4.63"}
    ]
    try:
        duplicate = milestone.duplicate_title("tape@0.4.63", "owner/repo")
        excluded = milestone.duplicate_title(
            "tape@0.4.63", "owner/repo", excluding=7
        )
    finally:
        milestone.list_milestones = original_list
    check("duplicate release identity is found", duplicate and duplicate["number"] == 7)
    check("the milestone being renamed is excluded from duplicate detection",
          excluded is None)

    # `update` must validate the proposed description against the assigned
    # issues before it can build a PATCH request.
    originals = (
        milestone.resolve_milestone,
        milestone.milestone_issues,
        milestone._description,
        milestone.duplicate_title,
        milestone._api,
    )
    api_calls: list[list[str]] = []
    proposed = ready
    assigned = [issue(101)]
    milestone.resolve_milestone = lambda _ref, _repo: dict(release)
    milestone.milestone_issues = lambda _target, _repo: list(assigned)
    milestone._description = lambda _path: proposed
    milestone.duplicate_title = lambda _title, _repo, excluding=None: None
    milestone._api = lambda argv, _dry: api_calls.append(argv) or ""
    args = argparse.Namespace(
        ref="milestone:7", repo="owner/repo", title=None,
        description_file="unused", due_on=None, clear_due_on=False,
        draft=False, dry_run=True,
    )
    try:
        invalid_update = milestone.cmd_update(args)
        check("update refuses an order that omits assigned membership",
              invalid_update == 1 and not api_calls)
        assigned = [issue(101)]
        proposed = draft
        args.draft = True
        draft_update = milestone.cmd_update(args)
        check("update refuses draft after assignment",
              draft_update == 1 and not api_calls)
        assigned = [issue(101), issue(102)]
        proposed = ready
        args.draft = False
        valid_update = milestone.cmd_update(args)
        check("update builds PATCH only for a reconciled order",
              valid_update == 0 and len(api_calls) == 1)
    finally:
        (
            milestone.resolve_milestone,
            milestone.milestone_issues,
            milestone._description,
            milestone.duplicate_title,
            milestone._api,
        ) = originals

    # Change assignment preflight and tracker readback are both load-bearing.
    original_resolve = change.milestone_surface.resolve_milestone
    original_project_label = workitem.project_label
    target = {"number": 7, "title": "tape@0.4.63", "state": "open"}
    change.milestone_surface.resolve_milestone = lambda _ref, _repo: dict(target)
    workitem.project_label = lambda project: project if ":" in project else f"app:{project}"
    try:
        resolved, expected_label = change.resolve_assignment("milestone:7", "owner/repo")
        check("assignment resolves one open release owner",
              resolved["number"] == 7 and expected_label == "app:tape")
        target["state"] = "closed"
        try:
            change.resolve_assignment("milestone:7", "owner/repo")
        except workitem.GhError:
            check("closed Milestone assignment is refused", True)
        else:
            check("closed Milestone assignment is refused", False)
        target["state"] = "open"
        try:
            change.require_assignment_labels(
                ["type:feat", "app:tape", "app:lumen"], "app:tape"
            )
        except workitem.GhError:
            check("cross-project assignment labels are refused", True)
        else:
            check("cross-project assignment labels are refused", False)
        try:
            change.verify_assignment(
                {"number": 101, "labels": ["type:feat", "app:tape"],
                 "milestone": {"number": 8}},
                7, "app:tape",
            )
        except workitem.GhError:
            check("wrong Milestone readback is refused", True)
        else:
            check("wrong Milestone readback is refused", False)
        change.verify_assignment(
            {"number": 101, "labels": ["type:feat", "app:tape"],
             "milestone": {"number": 7}},
            7, "app:tape",
        )
        check("matching Milestone readback is accepted", True)
    finally:
        change.milestone_surface.resolve_milestone = original_resolve
        workitem.project_label = original_project_label

    original_assignment = change.resolve_assignment
    original_change_fetch = change.fetch_issue
    original_change_create = workitem.cmd_create
    original_change_update = workitem.cmd_update
    writes: list[str] = []
    change.resolve_assignment = lambda _ref, _repo, **_kwargs: (dict(target), "app:tape")
    try:
        def fake_create(args) -> int:
            writes.append("create")
            args.created_iid = "101"
            return 0

        workitem.cmd_create = fake_create
        change.fetch_issue = lambda _iid, _repo: {
            "number": 101, "labels": ["type:feat", "app:tape"],
            "milestone": {"number": 7},
        }
        create_args = argparse.Namespace(
            milestone="milestone:7", repo="owner/repo", project=None,
            type="feat", dry_run=False,
        )
        create_result = change.cmd_create(create_args)
        check("change create derives owner and verifies tracker readback",
              create_result == 0 and create_args.project == "app:tape"
              and writes == ["create"])

        writes.clear()
        workitem.cmd_update = lambda _args: writes.append("update") or 0
        change.fetch_issue = lambda _iid, _repo: {
            "number": 101, "labels": ["type:feat", "app:lumen"],
            "milestone": None,
        }
        update_args = argparse.Namespace(
            iid="101", repo="owner/repo", milestone="milestone:7",
            remove_milestone=False, add_label=None, remove_label=None,
            dry_run=False,
        )
        try:
            change.cmd_update(update_args)
        except workitem.GhError:
            check("change update refuses wrong-project ownership before write",
                  not writes)
        else:
            check("change update refuses wrong-project ownership before write", False)

        reads = iter((
            {"number": 101, "labels": ["type:feat", "app:lumen"],
             "milestone": None},
            {"number": 101, "labels": ["type:feat", "app:tape"],
             "milestone": {"number": 7}},
        ))
        change.fetch_issue = lambda _iid, _repo: next(reads)
        update_args.add_label = ["app:tape"]
        update_args.remove_label = ["app:lumen"]
        update_result = change.cmd_update(update_args)
        check("change update verifies native ownership after write",
              update_result == 0 and writes == ["update"])

        writes.clear()
        change.fetch_issue = lambda _iid, _repo: {
            "number": 101, "labels": ["type:feat", "app:tape"],
            "milestone": {"number": 7},
        }
        update_args.milestone = None
        update_args.remove_milestone = False
        update_args.add_label = ["app:lumen"]
        update_args.remove_label = None
        try:
            change.cmd_update(update_args)
        except workitem.GhError:
            check("label-only update cannot corrupt existing Milestone ownership",
                  not writes)
        else:
            check("label-only update cannot corrupt existing Milestone ownership", False)

        reads = iter((
            {"number": 101, "labels": ["type:feat", "app:tape"],
             "milestone": {"number": 7}},
            {"number": 101, "labels": ["type:feat", "app:tape"],
             "milestone": None},
        ))
        change.fetch_issue = lambda _iid, _repo: next(reads)
        update_args.remove_milestone = True
        update_args.add_label = None
        update_args.remove_label = None
        remove_result = change.cmd_update(update_args)
        check("change update verifies Milestone removal after write",
              remove_result == 0 and writes == ["update"])
    finally:
        change.resolve_assignment = original_assignment
        change.fetch_issue = original_change_fetch
        workitem.cmd_create = original_change_create
        workitem.cmd_update = original_change_update

    if failed:
        print(f"\n=> RED: {len(failed)} failure(s)")
        return 1
    print("\n=> GREEN: Milestone identity, description, ownership, and order")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
