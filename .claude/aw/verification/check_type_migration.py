#!/usr/bin/env python3
"""Exercise manifest preflight, atomic migration, and receipt resume."""

from __future__ import annotations

import copy
import json
import sys
import tempfile
from pathlib import Path


SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
VERSIONED_MANIFEST = (
    Path(__file__).resolve().parents[1]
    / "migrations"
    / "open-legacy-types-2026-08-31.json"
)
sys.path.insert(0, str(SCRIPTS))

import type_migration  # noqa: E402
import workitem  # noqa: E402


failed: list[str] = []


def check(label: str, ok: bool, detail: str = "") -> None:
    suffix = f" -- {detail}" if detail else ""
    print(f"{'PASS' if ok else 'FAIL'} {label}{suffix}")
    if not ok:
        failed.append(label)


def row(issue: dict, source: str, target: str) -> dict:
    return {
        "iid": str(issue["number"]),
        "source_type": source,
        "target_type": target,
        "reason": "classified by the frozen cutover rules",
        "evidence": "body=read; milestone=read; product-promise=read",
        "expected_labels_sha256": type_migration.labels_sha256(issue["labels"]),
        "updatedAt": issue["updated_at"],
        "milestone_number": (issue.get("milestone") or {}).get("number"),
        "state": "OPEN",
    }


def main() -> int:
    frozen = json.loads(VERSIONED_MANIFEST.read_text(encoding="utf-8"))
    frozen_rows = type_migration.load_manifest(str(VERSIONED_MANIFEST))
    frozen_iids = [row["iid"] for row in frozen_rows]
    check(
        "versioned cutover manifest loads with exact rows and numeric order",
        frozen.get("repository") == "chrischeng-c4/axiom"
        and bool(frozen_rows)
        and frozen_iids == sorted(frozen_iids, key=int),
    )

    initial = {
        "11": {
            "number": 11, "title": "bug", "body": "repair the promise",
            "state": "OPEN", "labels": ["type:bug", "app:tape", "phase:created"],
            "milestone": {"number": 7}, "updated_at": "2026-08-31T00:00:11Z",
        },
        "12": {
            "number": 12, "title": "tooling", "body": "update build tooling",
            "state": "OPEN", "labels": ["type:change", "app:tape", "phase:created"],
            "milestone": None, "updated_at": "2026-08-31T00:00:12Z",
        },
        "13": {
            "number": 13, "title": "capability", "body": "add public command",
            "state": "OPEN", "labels": ["type:enhancement", "app:tape", "phase:created"],
            "milestone": {"number": 7}, "updated_at": "2026-08-31T00:00:13Z",
        },
    }
    rows = [
        row(initial["11"], "bug", "fix"),
        row(initial["12"], "change", "chore"),
        row(initial["13"], "enhancement", "feat"),
    ]

    with tempfile.TemporaryDirectory() as raw:
        root = Path(raw)
        manifest = root / "manifest.json"
        manifest.write_text(json.dumps({"issues": rows}), encoding="utf-8")
        loaded = type_migration.load_manifest(str(manifest))
        check("complete manifest loads", [entry["iid"] for entry in loaded] == ["11", "12", "13"])

        bad_fixed = copy.deepcopy(rows)
        bad_fixed[0]["target_type"] = "feat"
        wrong = root / "wrong-fixed.json"
        wrong.write_text(json.dumps({"issues": bad_fixed}), encoding="utf-8")
        try:
            type_migration.load_manifest(str(wrong))
        except workitem.GhError as exc:
            check("fixed bug mapping cannot drift", "must map type:bug to type:fix" in str(exc))
        else:
            check("fixed bug mapping cannot drift", False)

        duplicate = root / "duplicate.json"
        duplicate.write_text(json.dumps({"issues": [rows[0], rows[0]]}), encoding="utf-8")
        try:
            type_migration.load_manifest(str(duplicate))
        except workitem.GhError as exc:
            check("duplicate issue is refused", "duplicate" in str(exc))
        else:
            check("duplicate issue is refused", False)

        unknown = copy.deepcopy(rows)
        unknown[1]["target_type"] = "security"
        unknown_path = root / "unknown.json"
        unknown_path.write_text(json.dumps({"issues": unknown}), encoding="utf-8")
        try:
            type_migration.load_manifest(str(unknown_path))
        except workitem.GhError as exc:
            check("unknown target is refused", "not canonical" in str(exc))
        else:
            check("unknown target is refused", False)

        store = copy.deepcopy(initial)
        original_by_label = workitem.fetch_issues_by_label
        original_fetch = workitem.fetch_issue
        original_replace = workitem.replace_issue_labels
        replacements: list[tuple[str, list[str]]] = []

        def by_label(label: str, _repo: str) -> list[dict]:
            return [copy.deepcopy(value) for value in store.values()
                    if value["state"] == "OPEN" and label in value["labels"]]

        def fetch(iid: str, _repo: str) -> dict:
            return copy.deepcopy(store[str(iid)])

        fail_once = {"iid": "12"}

        def replace(iid: str, _repo: str, labels: list[str], _dry: bool) -> str:
            replacements.append((str(iid), list(labels)))
            if fail_once.get("iid") == str(iid):
                fail_once.clear()
                raise workitem.GhError("injected replacement failure")
            store[str(iid)]["labels"] = list(labels)
            store[str(iid)]["updated_at"] += "+write"
            return ""

        workitem.fetch_issues_by_label = by_label
        workitem.fetch_issue = fetch
        workitem.replace_issue_labels = replace
        try:
            check("preflight accepts the exact live cohort",
                  set(type_migration.preflight("owner/repo", rows)) == {"11", "12", "13"})
            try:
                type_migration.preflight("owner/repo", rows[:-1])
            except workitem.GhError as exc:
                check("incomplete manifest refuses before writes",
                      "#13" in str(exc) and replacements == [])
            else:
                check("incomplete manifest refuses before writes", False)

            drifted = copy.deepcopy(rows)
            drifted[1]["updatedAt"] = "old"
            try:
                type_migration.preflight("owner/repo", drifted)
            except workitem.GhError as exc:
                check("tracker updatedAt drift is refused", "updatedAt drifted" in str(exc))
            else:
                check("tracker updatedAt drift is refused", False)

            receipt = root / "receipt.json"
            args = type("Args", (), {
                "manifest": str(manifest), "repo": "owner/repo", "receipt": str(receipt),
            })()
            try:
                type_migration.start_apply(args)
            except workitem.GhError as exc:
                check("partial failure stays explicit", "injected" in str(exc))
            else:
                check("partial failure stays explicit", False)
            partial = json.loads(receipt.read_text(encoding="utf-8"))
            check("partial receipt stays INCOMPLETE",
                  partial["status"] == "INCOMPLETE"
                  and [entry["status"] for entry in partial["rows"]]
                  == ["APPLIED", "PENDING", "PENDING"], repr(partial))
            check("replacement sends the complete label set once",
                  replacements[0] == (
                      "11", ["app:tape", "phase:created", "type:fix"]
                  ), repr(replacements[:1]))

            resumed = type_migration.resume(str(receipt))
            complete = json.loads(receipt.read_text(encoding="utf-8"))
            check("same receipt resumes to COMPLETE",
                  resumed == 0 and complete["status"] == "COMPLETE"
                  and all(entry["status"] == "APPLIED" for entry in complete["rows"]))
            check("resume applied every fixed target",
                  {iid: next(label for label in data["labels"] if label.startswith("type:"))
                   for iid, data in store.items()}
                  == {"11": "type:fix", "12": "type:chore", "13": "type:feat"})
            check("non-type labels, body, title, state, and Milestone survive",
                  all(
                      set(store[iid]["labels"]) - {next(
                          label for label in store[iid]["labels"] if label.startswith("type:")
                      )}
                      == set(initial[iid]["labels"]) - {next(
                          label for label in initial[iid]["labels"] if label.startswith("type:")
                      )}
                      and store[iid]["body"] == initial[iid]["body"]
                      and store[iid]["title"] == initial[iid]["title"]
                      and store[iid]["state"] == initial[iid]["state"]
                      and store[iid]["milestone"] == initial[iid]["milestone"]
                      for iid in store
                  ))

            store["12"]["title"] = "drift after complete"
            try:
                type_migration.resume(str(receipt))
            except workitem.GhError as exc:
                check("complete receipt still detects readback drift",
                      "changed title or body" in str(exc))
            else:
                check("complete receipt still detects readback drift", False)
        finally:
            workitem.fetch_issues_by_label = original_by_label
            workitem.fetch_issue = original_fetch
            workitem.replace_issue_labels = original_replace

    if failed:
        print(f"\n=> RED: {len(failed)} failure(s)")
        return 1
    print("\n=> GREEN: migration manifest, replacement, and receipt resume")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
