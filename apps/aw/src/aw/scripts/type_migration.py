#!/usr/bin/env python3
"""One-time, manifest-led migration from retired open issue types."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import wi_types  # noqa: E402
import workitem  # noqa: E402


ROW_FIELDS = {
    "iid", "source_type", "target_type", "reason", "evidence",
    "expected_labels_sha256", "updatedAt", "milestone_number", "state",
}

FIXED_TARGETS = {"bug": "fix", "enhancement": "feat"}


def labels_sha256(labels: list[str]) -> str:
    wire = json.dumps(sorted(labels), separators=(",", ":"), ensure_ascii=False)
    return hashlib.sha256(wire.encode("utf-8")).hexdigest()


def load_manifest(path: str) -> list[dict]:
    loaded = json.loads(Path(path).read_text(encoding="utf-8"))
    rows = loaded.get("issues") if isinstance(loaded, dict) else loaded
    if not isinstance(rows, list) or not rows:
        raise workitem.GhError("manifest must contain a non-empty `issues` list")
    answer: list[dict] = []
    seen: set[str] = set()
    for raw in rows:
        if not isinstance(raw, dict) or set(raw) != ROW_FIELDS:
            raise workitem.GhError("each manifest row must contain the complete frozen migration fields")
        iid, source, target = str(raw["iid"]), str(raw["source_type"]), str(raw["target_type"])
        if not iid.isdigit() or int(iid) < 1 or iid in seen:
            raise workitem.GhError(f"manifest has invalid or duplicate iid `{iid}`")
        if source not in wi_types.MIGRATABLE_LEGACY_TYPES:
            raise workitem.GhError(f"manifest #{iid} source_type `{source}` is not migratable")
        if target not in wi_types.DELIVERY_TYPES:
            raise workitem.GhError(f"manifest #{iid} target_type `{target}` is not canonical")
        if not isinstance(raw["reason"], str) or not raw["reason"].strip():
            raise workitem.GhError(f"manifest #{iid} needs a non-empty reason")
        if not isinstance(raw["evidence"], str) or not raw["evidence"].strip():
            raise workitem.GhError(f"manifest #{iid} needs non-empty evidence")
        if not isinstance(raw["expected_labels_sha256"], str) or len(raw["expected_labels_sha256"]) != 64:
            raise workitem.GhError(f"manifest #{iid} has no labels sha256")
        fixed = FIXED_TARGETS.get(source)
        if fixed is not None and target != fixed:
            raise workitem.GhError(
                f"manifest #{iid} must map type:{source} to type:{fixed}, not type:{target}"
            )
        if not isinstance(raw["updatedAt"], str) or not raw["updatedAt"]:
            raise workitem.GhError(f"manifest #{iid} has no updatedAt")
        if raw["milestone_number"] is not None and not isinstance(raw["milestone_number"], int):
            raise workitem.GhError(f"manifest #{iid} has invalid milestone_number")
        if raw["state"] != "OPEN":
            raise workitem.GhError(f"manifest #{iid} state must be OPEN")
        seen.add(iid)
        answer.append({**raw, "iid": iid, "source_type": source, "target_type": target})
    return answer


def open_legacy(repo: str) -> set[str]:
    answer: set[str] = set()
    for kind in wi_types.LEGACY_TYPES:
        for row in workitem.fetch_issues_by_label(f"type:{kind}", repo):
            if row.get("state", "").upper() == "OPEN":
                answer.add(str(row["number"]))
    return answer


def milestone_number(issue: dict) -> int | None:
    return (issue.get("milestone") or {}).get("number")


def verify_expected(row: dict, issue: dict) -> None:
    iid = row["iid"]
    if issue.get("state", "").upper() != row["state"]:
        raise workitem.GhError(f"#{iid} state drifted")
    if labels_sha256(issue.get("labels", [])) != row["expected_labels_sha256"]:
        raise workitem.GhError(f"#{iid} label set drifted")
    if issue.get("updated_at") != row["updatedAt"]:
        raise workitem.GhError(f"#{iid} updatedAt drifted")
    if milestone_number(issue) != row["milestone_number"]:
        raise workitem.GhError(f"#{iid} Milestone relation drifted")
    try:
        source = wi_types.legacy_type(issue.get("labels", []), subject=f"#{iid}")
    except wi_types.TypeError as exc:
        raise workitem.GhError(str(exc)) from exc
    if source != row["source_type"]:
        raise workitem.GhError(f"#{iid} type drifted from `type:{row['source_type']}` to `type:{source}`")


def write_receipt(path: Path, receipt: dict) -> None:
    path.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def begin_receipt(repo: str, rows: list[dict], issues: dict[str, dict]) -> dict:
    return {"version": 1, "repo": repo, "status": "INCOMPLETE", "rows": [
        {"manifest": row, "status": "PENDING", "before": {
            "title": issues[row["iid"]]["title"], "body": issues[row["iid"]].get("body") or "",
            "labels": sorted(issues[row["iid"]].get("labels", [])),
            "state": issues[row["iid"]].get("state"), "milestone": milestone_number(issues[row["iid"]]),
        }} for row in rows
    ]}


def preflight(repo: str, rows: list[dict]) -> dict[str, dict]:
    cohort, named = open_legacy(repo), {row["iid"] for row in rows}
    if named != cohort:
        missing, extra = sorted(cohort - named, key=int), sorted(named - cohort, key=int)
        detail = (["missing " + ", ".join("#" + iid for iid in missing)] if missing else [])
        detail += (["not-open-migratable-legacy " + ", ".join("#" + iid for iid in extra)] if extra else [])
        raise workitem.GhError("manifest must cover every open legacy issue: " + "; ".join(detail))
    issues = {row["iid"]: workitem.fetch_issue(row["iid"], repo) for row in rows}
    for row in rows:
        verify_expected(row, issues[row["iid"]])
    return issues


def apply_pending(receipt: dict, path: Path) -> int:
    for entry in receipt["rows"]:
        if entry["status"] == "APPLIED":
            continue
        row, repo = entry["manifest"], receipt["repo"]
        before = workitem.fetch_issue(row["iid"], repo)
        verify_expected(row, before)
        final = sorted((set(before.get("labels", [])) - {f"type:{row['source_type']}"})
                       | {f"type:{row['target_type']}"})
        workitem.replace_issue_labels(row["iid"], repo, final, False)
        after = workitem.fetch_issue(row["iid"], repo)
        try:
            actual = wi_types.delivery_type(after.get("labels", []), subject=f"#{row['iid']}")
        except wi_types.TypeError as exc:
            raise workitem.GhError(f"migration readback failed: {exc}") from exc
        old = entry["before"]
        if actual != row["target_type"]:
            raise workitem.GhError(f"migration readback expected #{row['iid']} `type:{row['target_type']}`")
        if after["title"] != old["title"] or (after.get("body") or "") != old["body"]:
            raise workitem.GhError(f"migration readback changed title or body on #{row['iid']}")
        if after.get("state") != old["state"] or milestone_number(after) != old["milestone"]:
            raise workitem.GhError(f"migration readback changed state or Milestone on #{row['iid']}")
        if set(after.get("labels", [])) - {f"type:{actual}"} != set(old["labels"]) - {f"type:{row['source_type']}"}:
            raise workitem.GhError(f"migration readback changed non-type labels on #{row['iid']}")
        entry["status"] = "APPLIED"
        entry["readback"] = {"type": actual, "updated_at": after.get("updated_at")}
        write_receipt(path, receipt)
    receipt["status"] = "COMPLETE"
    write_receipt(path, receipt)
    print(json.dumps({"receipt": str(path), "status": receipt["status"]}, indent=2))
    return 0


def verify_applied(entry: dict, issue: dict) -> None:
    """Recover a write that succeeded before its durable receipt update."""
    row, old = entry["manifest"], entry["before"]
    try:
        actual = wi_types.delivery_type(issue.get("labels", []), subject=f"#{row['iid']}")
    except wi_types.TypeError as exc:
        raise workitem.GhError(f"receipt readback failed: {exc}") from exc
    if actual != row["target_type"]:
        raise workitem.GhError(f"receipt #{row['iid']} has unexpected canonical type")
    if issue["title"] != old["title"] or (issue.get("body") or "") != old["body"]:
        raise workitem.GhError(f"receipt #{row['iid']} changed title or body")
    if issue.get("state") != old["state"] or milestone_number(issue) != old["milestone"]:
        raise workitem.GhError(f"receipt #{row['iid']} changed state or Milestone")
    if set(issue.get("labels", [])) - {f"type:{actual}"} != set(old["labels"]) - {f"type:{row['source_type']}"}:
        raise workitem.GhError(f"receipt #{row['iid']} changed non-type labels")


def start_apply(args) -> int:
    rows = load_manifest(args.manifest)
    issues = preflight(args.repo, rows)
    path = Path(args.receipt)
    if path.exists():
        raise workitem.GhError(f"receipt already exists: {path}; use --resume to continue it")
    path.parent.mkdir(parents=True, exist_ok=True)
    receipt = begin_receipt(args.repo, rows, issues)
    write_receipt(path, receipt)
    return apply_pending(receipt, path)


def resume(path_text: str) -> int:
    path = Path(path_text)
    receipt = json.loads(path.read_text(encoding="utf-8"))
    if receipt.get("version") != 1 or receipt.get("status") not in {"INCOMPLETE", "COMPLETE"}:
        raise workitem.GhError("receipt is not a type migration receipt")
    if receipt["status"] == "COMPLETE":
        for entry in receipt["rows"]:
            issue = workitem.fetch_issue(entry["manifest"]["iid"], receipt["repo"])
            verify_applied(entry, issue)
        remaining = open_legacy(receipt["repo"])
        if remaining:
            raise workitem.GhError(
                "complete receipt readback found open legacy issues: "
                + ", ".join("#" + iid for iid in sorted(remaining, key=int))
            )
        print(json.dumps({"receipt": str(path), "status": "COMPLETE"}, indent=2))
        return 0
    # A process can stop after GitHub accepts a replacement and before the
    # next local receipt write.  Re-read each pending row to distinguish that
    # durable tracker state from a true unfinished legacy row.
    for entry in receipt["rows"]:
        issue = workitem.fetch_issue(entry["manifest"]["iid"], receipt["repo"])
        if entry["status"] == "APPLIED":
            verify_applied(entry, issue)
            continue
        try:
            actual = wi_types.delivery_type(issue.get("labels", []), subject=f"#{entry['manifest']['iid']}")
        except wi_types.TypeError:
            continue
        if actual != entry["manifest"]["target_type"]:
            raise workitem.GhError(f"receipt #{entry['manifest']['iid']} has conflicting canonical type")
        verify_applied(entry, issue)
        entry["status"] = "APPLIED"
        entry["readback"] = {"type": actual, "updated_at": issue.get("updated_at")}
        write_receipt(path, receipt)
    pending = {entry["manifest"]["iid"] for entry in receipt["rows"] if entry["status"] == "PENDING"}
    if open_legacy(receipt["repo"]) != pending:
        raise workitem.GhError("open legacy cohort drifted since the receipt was written")
    return apply_pending(receipt, path)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="type_migration.py")
    parser.add_argument("--repo")
    parser.add_argument("--manifest")
    parser.add_argument("--receipt")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--apply", action="store_true")
    mode.add_argument("--resume")
    args = parser.parse_args(argv)
    try:
        if args.resume:
            if args.repo or args.manifest or args.receipt:
                raise workitem.GhError("--resume <receipt> accepts no --repo, --manifest, or --receipt")
            return resume(args.resume)
        if args.apply:
            if not args.repo or not args.manifest or not args.receipt:
                raise workitem.GhError("--apply needs --repo, --manifest, and --receipt")
            return start_apply(args)
        if not args.repo or not args.manifest:
            raise workitem.GhError("dry-run needs --repo and --manifest")
        rows = load_manifest(args.manifest)
        preflight(args.repo, rows)
        print(json.dumps({"mode": "dry-run", "repo": args.repo, "issues": rows}, indent=2))
        return 0
    except (workitem.GhError, json.JSONDecodeError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
