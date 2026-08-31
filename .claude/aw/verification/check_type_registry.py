#!/usr/bin/env python3
"""Check the closed AW type registry and delivery facade routing."""

from __future__ import annotations

import argparse
import contextlib
import io
import json
import sys
from pathlib import Path


SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import change  # noqa: E402
import wi_types  # noqa: E402
import workitem  # noqa: E402


failed: list[str] = []


def check(label: str, ok: bool, detail: str = "") -> None:
    suffix = f" -- {detail}" if detail else ""
    print(f"{'PASS' if ok else 'FAIL'} {label}{suffix}")
    if not ok:
        failed.append(label)


def refuses(call, phrase: str = "") -> bool:
    try:
        call()
    except (wi_types.TypeError, workitem.GhError) as exc:
        return not phrase or phrase in str(exc)
    return False


def issue(kind: str, *, state: str = "OPEN", body: str = "") -> dict:
    return {
        "number": 41,
        "title": "typed delivery",
        "body": body,
        "state": state,
        "labels": [f"type:{kind}", "app:tape", "phase:created", "priority:p2"],
        "milestone": {"number": 7, "title": "tape@0.4.1"},
        "url": "https://example.invalid/issues/41",
    }


def main() -> int:
    expected_delivery = {
        "feat": ("behavior", ("e2e", "impl")),
        "fix": ("behavior", ("e2e", "impl")),
        "refactor": ("maintenance", ("maint",)),
        "perf": ("behavior", ("e2e", "impl")),
        "test": ("maintenance", ("maint",)),
        "docs": ("maintenance", ("maint",)),
        "chore": ("maintenance", ("maint",)),
    }
    check("delivery registry is exact",
          tuple(expected_delivery) == wi_types.DELIVERY_TYPES,
          repr(wi_types.DELIVERY_TYPES))
    for kind, (flow, legs) in expected_delivery.items():
        labels = ["app:tape", f"type:{kind}"]
        check(f"type:{kind} resolves",
              wi_types.delivery_type(labels) == kind
              and wi_types.flow_for(kind) == flow
              and wi_types.required_legs(kind) == legs)

    check("intake registry is exact", wi_types.INTAKE_TYPES == ("spike", "report"))
    for kind in wi_types.INTAKE_TYPES:
        labels = [f"type:{kind}", "app:tape"]
        check(f"type:{kind} is canonical intake",
              wi_types.canonical_type(labels) == kind)
        check(f"type:{kind} cannot enter delivery",
              refuses(lambda labels=labels: wi_types.delivery_type(labels), "intake"))

    bad = {
        "zero type": ["app:tape"],
        "two types": ["type:feat", "type:fix", "app:tape"],
        "unknown type": ["type:security", "app:tape"],
    }
    for label, labels in bad.items():
        check(f"{label} is refused",
              refuses(lambda labels=labels: wi_types.canonical_type(labels)))
    for kind in wi_types.LEGACY_TYPES:
        check(f"legacy type:{kind} is refused",
              refuses(lambda kind=kind: wi_types.canonical_type([f"type:{kind}"]),
                      "retired"))

    parser = change.build_parser()
    for label, argv in (
        ("skeleton", ["skeleton"]),
        ("bodydir", ["bodydir"]),
        ("create", ["create", "--title", "x", "--body-file", "x.md"]),
    ):
        with contextlib.redirect_stderr(io.StringIO()):
            try:
                parser.parse_args(argv)
            except SystemExit as exc:
                check(f"{label} requires --type", exc.code == 2)
            else:
                check(f"{label} requires --type", False)
    body_args = parser.parse_args(["validate", "--body-file", "x.md"])
    check("body-file validation requires --type",
          refuses(lambda: change.cmd_validate(body_args), "needs --type"))
    check("normal update refuses type addition",
          refuses(lambda: workitem.reject_type_label_mutation(["type:fix"], None),
                  "cannot add"))
    check("normal update refuses type removal",
          refuses(lambda: workitem.reject_type_label_mutation(None, ["type:feat"]),
                  "cannot add"))

    original_fetch = change.fetch_issue
    original_refs = workitem.refs_commits
    original_replace = workitem.replace_issue_labels
    replacements: list[list[str]] = []
    before = issue("feat")
    after = issue("fix")
    reads = iter((before, after))
    change.fetch_issue = lambda _iid, _repo: next(reads)
    workitem.refs_commits = lambda _iid: []
    workitem.replace_issue_labels = (
        lambda _iid, _repo, labels, _dry: replacements.append(list(labels)) or ""
    )
    try:
        result = change.cmd_retype(argparse.Namespace(
            iid="41", repo="owner/repo", to="fix", dry_run=False,
        ))
        check("retype uses one complete label replacement",
              result == 0 and replacements == [[
                  "app:tape", "phase:created", "priority:p2", "type:fix"
              ]], repr(replacements))

        def retype_refused(candidate: dict, refs: list[str] | None = None) -> bool:
            replacements.clear()
            change.fetch_issue = lambda _iid, _repo: candidate
            workitem.refs_commits = lambda _iid: list(refs or [])
            return refuses(lambda: change.cmd_retype(argparse.Namespace(
                iid="41", repo="owner/repo", to="fix", dry_run=False,
            ))) and not replacements

        check("closed issue cannot retype", retype_refused(issue("feat", state="CLOSED")))
        no_created = issue("feat")
        no_created["labels"].remove("phase:created")
        check("non-created issue cannot retype", retype_refused(no_created))
        lifecycle = issue("feat", body=workitem.lifecycle_upsert(
            "", "e2e", "a" * 40, "b" * 64
        ))
        check("issue with lifecycle cannot retype", retype_refused(lifecycle))
        check("issue with delivery commit cannot retype",
              retype_refused(issue("feat"), ["c" * 40]))
        check("intake cannot use delivery retype", retype_refused(issue("report")))
    finally:
        change.fetch_issue = original_fetch
        workitem.refs_commits = original_refs
        workitem.replace_issue_labels = original_replace

    original_fetch = change.fetch_issue
    change.fetch_issue = lambda _iid, _repo: issue("perf")
    try:
        stream = io.StringIO()
        with contextlib.redirect_stdout(stream):
            shown = change.cmd_show(argparse.Namespace(
                iid="41", repo="owner/repo", json=True,
            ))
        payload = json.loads(stream.getvalue())
        check("show JSON reports type and flow",
              shown == 0 and payload["type"] == "perf"
              and payload["flow"] == "behavior")
    finally:
        change.fetch_issue = original_fetch

    original_fetch = change.fetch_issue
    original_refs = workitem.refs_commits
    original_message = workitem.commit_message
    original_run = workitem.run_or_show
    writes: list[list[str]] = []

    def close_case(kind: str, evidence: list[tuple[str, str, str]],
                   *, mismatch: bool = False) -> int:
        body = ""
        messages: dict[str, str] = {}
        trailers = {
            "e2e": "E2E-Change-Digest",
            "impl": "Impl-Change-Digest",
            "maint": "Maint-Change-Digest",
        }
        for leg, sha, digest in evidence:
            body = workitem.lifecycle_upsert(body, leg, sha, digest)
            recorded = "0" * 64 if mismatch and leg == evidence[-1][0] else digest
            messages[sha] = (
                f"{leg}(wi-41): evidence\n\nRefs #41\n"
                f"{trailers[leg]}: {recorded}\n"
            )
        opened = issue(kind, body=body)
        closed = dict(opened, state="CLOSED")
        reads = iter((opened, closed))
        change.fetch_issue = lambda _iid, _repo: next(reads)
        workitem.refs_commits = lambda _iid: [sha for _leg, sha, _digest in evidence]
        workitem.commit_message = lambda sha: messages[sha]
        workitem.run_or_show = lambda argv, _dry: writes.append(list(argv)) or ""
        return change.cmd_close(argparse.Namespace(
            iid="41", repo="owner/repo", dry_run=False,
        ))

    try:
        writes.clear()
        check("behavior close verifies e2e and impl evidence",
              close_case("feat", [
                  ("e2e", "a" * 40, "1" * 64),
                  ("impl", "b" * 40, "2" * 64),
              ]) == 0 and len(writes) == 1)
        writes.clear()
        check("maintenance close verifies maint evidence",
              close_case("docs", [("maint", "c" * 40, "3" * 64)]) == 0
              and len(writes) == 1)
        writes.clear()
        check("close refuses a mismatched commit digest",
              refuses(lambda: close_case(
                  "fix", [
                      ("e2e", "d" * 40, "4" * 64),
                      ("impl", "e" * 40, "5" * 64),
                  ], mismatch=True,
              ), "does not match") and not writes)
        missing_digest = issue("chore", body=workitem.lifecycle_upsert(
            "", "maint", "f" * 40, ""
        ))
        change.fetch_issue = lambda _iid, _repo: missing_digest
        check("close refuses missing lifecycle digest",
              refuses(lambda: change.cmd_close(argparse.Namespace(
                  iid="41", repo="owner/repo", dry_run=False,
              )), "missing or invalid digest"))
        wrong_flow_body = workitem.lifecycle_upsert(
            workitem.lifecycle_upsert("", "maint", "1" * 40, "6" * 64),
            "e2e", "2" * 40, "7" * 64,
        )
        change.fetch_issue = lambda _iid, _repo: issue("docs", body=wrong_flow_body)
        check("close refuses lifecycle evidence from another flow",
              refuses(lambda: change.cmd_close(argparse.Namespace(
                  iid="41", repo="owner/repo", dry_run=False,
              )), "outside this issue's required flow"))
    finally:
        change.fetch_issue = original_fetch
        workitem.refs_commits = original_refs
        workitem.commit_message = original_message
        workitem.run_or_show = original_run

    if failed:
        print(f"\n=> RED: {len(failed)} failure(s)")
        return 1
    print("\n=> GREEN: type registry and delivery facade")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
