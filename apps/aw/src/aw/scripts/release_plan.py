#!/usr/bin/env python3
"""Validate and apply one approved, resumable release plan.

The plan is closed data. It cannot carry a command. ``validate`` is the only
stdin-capable verb and is read-only. ``apply`` writes one project after every
baseline check succeeds. Every later write is recovered through the same
durable receipt.
"""
from __future__ import annotations

import argparse
import copy
import fcntl
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
from collections import Counter
from contextlib import contextmanager
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import change  # noqa: E402
import meta  # noqa: E402
import metadoc  # noqa: E402
import milestone  # noqa: E402
import wi_types  # noqa: E402
import wis  # noqa: E402
import workitem  # noqa: E402


SCHEMA = "release-plan-v1"
RECEIPT_SCHEMA = "release-plan-receipt-v1"
MILESTONE_NUMBER = "{{milestone_number}}"
MILESTONE_HEADING_MARKER = f"(Milestone #{MILESTONE_NUMBER})"
DEVELOPMENT_ORDER = "{{development_order}}"
SHA256 = re.compile(r"^[0-9a-f]{64}$")
PROJECT = re.compile(r"^(apps|libs)/[a-z0-9][a-z0-9-]*$")
KEY = re.compile(r"^[a-z0-9][a-z0-9-]*$")
COMMIT = re.compile(r"^[0-9a-f]{40}$")
DOCUMENT_MILESTONE = re.compile(r"\bMilestone[ \t]+#(?P<number>\d+)\b")
ROOT = workitem.REPO_ROOT


class PlanError(ValueError):
    """The plan, receipt, checkout, or tracker no longer has one safe reading."""


def canonical_bytes(value: object) -> bytes:
    return (json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":"),
    ) + "\n").encode()


def digest(value: object) -> str:
    return hashlib.sha256(canonical_bytes(value)).hexdigest()


def plan_digest(value: object) -> str:
    """Hash a release plan with its self-describing digest field omitted."""
    if not isinstance(value, dict):
        return digest(value)
    payload = {key: item for key, item in value.items() if key != "plan_sha256"}
    return digest(payload)


def _load_json(raw: str, subject: str) -> object:
    def closed_object(pairs: list[tuple[str, object]]) -> dict:
        value = {}
        for key, item in pairs:
            if key in value:
                raise PlanError(f"{subject} has duplicate field: {key}")
            value[key] = item
        return value

    try:
        return json.loads(raw, object_pairs_hook=closed_object)
    except json.JSONDecodeError as error:
        raise PlanError(f"{subject} is not JSON: {error.msg}") from error


def _text_sha(text: str) -> str:
    return hashlib.sha256(text.encode()).hexdigest()


def _keys(value: object, allowed: set[str], where: str,
          required: set[str] | None = None) -> dict:
    if not isinstance(value, dict):
        raise PlanError(f"{where} must be an object")
    unknown = sorted(set(value) - allowed)
    if unknown:
        raise PlanError(f"{where} has unknown field(s): {', '.join(unknown)}")
    missing = sorted((required if required is not None else allowed) - set(value))
    if missing:
        raise PlanError(f"{where} is missing field(s): {', '.join(missing)}")
    return value


def _sha(value: object, where: str) -> str:
    if not isinstance(value, str) or not SHA256.fullmatch(value):
        raise PlanError(f"{where} must be a lowercase SHA-256")
    return value


def _positive(value: object, where: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value < 1:
        raise PlanError(f"{where} must be a positive integer")
    return value


def owner_label(project: str) -> str:
    prefix, name = project.split("/", 1)
    return f"{'app' if prefix == 'apps' else 'lib'}:{name}"


def milestone_tracking_link(repository: str, number: int | str) -> str:
    return (
        f"[Milestone #{number}]"
        f"(https://github.com/{repository}/milestone/{number})"
    )


def _document(value: object, where: str) -> dict:
    row = _keys(value, {"path", "before_sha256", "after"}, where)
    path = row["path"]
    if not isinstance(path, str) or not path or path.startswith("/") \
            or ".." in Path(path).parts:
        raise PlanError(f"{where}.path must be a safe relative path")
    if "\\" in path or Path(path).as_posix() != path:
        raise PlanError(f"{where}.path must be one canonical relative path")
    if not re.fullmatch(r"(?:README\.md|STATUS\.md|ROADMAP\.md|docs/.+\.md)", path):
        raise PlanError(f"{where}.path is not an allowed META document")
    before = row["before_sha256"]
    after = row["after"]
    if before is not None:
        _sha(before, f"{where}.before_sha256")
    if after is not None and not isinstance(after, str):
        raise PlanError(f"{where}.after must be exact UTF-8 text or null")
    if before is None and after is None:
        raise PlanError(f"{where} cannot be absent both before and after")
    if isinstance(after, str) and _text_sha(after) == before:
        raise PlanError(f"{where}.after must change the approved document bytes")
    return row


def _baseline_milestone(value: object, where: str) -> dict:
    row = _keys(value, {
        "number", "title", "state", "description_sha256",
    }, where)
    _positive(row["number"], f"{where}.number")
    if not isinstance(row["title"], str) or milestone.release_identity(row["title"]) is None:
        raise PlanError(f"{where}.title must be a SemVer milestone title")
    if row["state"] not in {"OPEN", "CLOSED"}:
        raise PlanError(f"{where}.state must be OPEN or CLOSED")
    _sha(row["description_sha256"], f"{where}.description_sha256")
    return row


def _baseline_issue(value: object, where: str, expected_owner: str) -> dict:
    row = _keys(value, {
        "number", "title", "state", "labels", "milestone", "body_sha256",
    }, where)
    _positive(row["number"], f"{where}.number")
    if not isinstance(row["title"], str) or not row["title"].strip():
        raise PlanError(f"{where}.title must be nonempty text")
    if row["state"] not in {"OPEN", "CLOSED"}:
        raise PlanError(f"{where}.state must be OPEN or CLOSED")
    labels = row["labels"]
    if not isinstance(labels, list) or any(not isinstance(label, str) or not label
                                           for label in labels):
        raise PlanError(f"{where}.labels must be a text list")
    if labels != sorted(set(labels)):
        raise PlanError(f"{where}.labels must be sorted and unique")
    owners = [label for label in labels if label.startswith(("app:", "lib:"))]
    if owners != [expected_owner]:
        raise PlanError(f"{where}.labels must contain only owner label {expected_owner}")
    if row["milestone"] is not None:
        _positive(row["milestone"], f"{where}.milestone")
    _sha(row["body_sha256"], f"{where}.body_sha256")
    return row


def _tracker_baseline(value: object, where: str, project: str) -> dict:
    row = _keys(value, {"sha256", "milestones", "issues"}, where)
    _sha(row["sha256"], f"{where}.sha256")
    if not isinstance(row["milestones"], list) or not isinstance(row["issues"], list):
        raise PlanError(f"{where}.milestones and .issues must be lists")
    milestones = [
        _baseline_milestone(item, f"{where}.milestones[{index}]")
        for index, item in enumerate(row["milestones"])
    ]
    issues = [
        _baseline_issue(item, f"{where}.issues[{index}]", owner_label(project))
        for index, item in enumerate(row["issues"])
    ]
    for name, items in (("milestones", milestones), ("issues", issues)):
        numbers = [item["number"] for item in items]
        if numbers != sorted(numbers) or len(numbers) != len(set(numbers)):
            raise PlanError(f"{where}.{name} must be sorted by unique number")
    summary = {"milestones": milestones, "issues": issues}
    if digest(summary) != row["sha256"]:
        raise PlanError(f"{where} tracker baseline digest does not match its summary")
    return row


def _planned_milestone(value: object, where: str, project: str,
                       baseline: dict, order_length: int) -> dict:
    row = _keys(
        value,
        {"number", "prior_sha256", "title", "description"},
        where,
        required={"title", "description"},
    )
    identity = milestone.release_identity(row["title"] if isinstance(row["title"], str) else "")
    if identity is None:
        raise PlanError(f"{where}.title must be a SemVer milestone title")
    if identity.project != project.rsplit("/", 1)[1]:
        raise PlanError(f"{where}.title must name project {project.rsplit('/', 1)[1]}")
    description = row["description"]
    if not isinstance(description, str) or description.count(DEVELOPMENT_ORDER) != 1:
        raise PlanError(f"{where}.description must contain one {DEVELOPMENT_ORDER}")
    sample_order = "\n".join(f"{index}. #{index}" for index in range(1, order_length + 1))
    errors = milestone.validate_description(description.replace(DEVELOPMENT_ORDER, sample_order))
    if errors:
        raise PlanError(f"{where}.description is invalid: {'; '.join(errors)}")
    number = row.get("number")
    prior = row.get("prior_sha256")
    matches = [item for item in baseline["milestones"] if item["title"] == row["title"]]
    if number is None:
        if "prior_sha256" in row:
            raise PlanError(f"{where}.prior_sha256 is only valid for an existing milestone")
        if matches:
            raise PlanError(f"{where}.number is required for the existing milestone title")
        return row
    _positive(number, f"{where}.number")
    _sha(prior, f"{where}.prior_sha256")
    found = [item for item in baseline["milestones"] if item["number"] == number]
    if len(found) != 1 or found[0]["title"] != row["title"]:
        raise PlanError(f"{where} does not match one baseline milestone")
    if found[0]["state"] != "OPEN":
        raise PlanError(f"{where} existing milestone must be OPEN")
    if digest(found[0]) != prior:
        raise PlanError(f"{where}.prior_sha256 does not match the baseline milestone")
    return row


def _planned_issue(value: object, where: str, project: str,
                   baseline: dict) -> dict:
    row = _keys(
        value,
        {"key", "number", "prior_sha256", "title", "type", "priority", "owner", "body"},
        where,
        required={"key", "title", "type", "priority", "owner", "body"},
    )
    if not isinstance(row["key"], str) or not KEY.fullmatch(row["key"]):
        raise PlanError(f"{where}.key must be a stable lowercase key")
    if not isinstance(row["title"], str) or not row["title"].strip():
        raise PlanError(f"{where}.title must be nonempty text")
    if row["type"] not in wi_types.DELIVERY_TYPES:
        raise PlanError(f"{where}.type must be one allowed delivery type")
    if row["priority"] not in workitem.PRIORITIES:
        raise PlanError(f"{where}.priority must be one allowed priority")
    expected_owner = owner_label(project)
    if row["owner"] != expected_owner:
        raise PlanError(f"{where}.owner must be the exact owner label {expected_owner}")
    if not isinstance(row["body"], str):
        raise PlanError(f"{where}.body must be exact UTF-8 text")
    body_errors = [error for error in change.validate_body(
        row["body"], change.CHANGE_TYPES[row["type"]],
    ) if not error.startswith("note:")]
    if body_errors:
        raise PlanError(f"{where}.body is invalid: {'; '.join(body_errors)}")
    number = row.get("number")
    if number is None:
        if "prior_sha256" in row:
            raise PlanError(f"{where}.prior_sha256 is only valid for an existing issue")
        if any(item["title"] == row["title"] for item in baseline["issues"]):
            raise PlanError(f"{where}.number is required for an existing issue title")
        return row
    _positive(number, f"{where}.number")
    _sha(row.get("prior_sha256"), f"{where}.prior_sha256")
    found = [item for item in baseline["issues"] if item["number"] == number]
    if len(found) != 1 or digest(found[0]) != row["prior_sha256"]:
        raise PlanError(f"{where}.prior_sha256 does not match one baseline issue")
    before = found[0]
    if before["state"] != "OPEN":
        raise PlanError(f"{where} existing issue must be OPEN")
    try:
        live_type = wi_types.delivery_type(before["labels"], subject=where)
    except wi_types.TypeError as error:
        raise PlanError(str(error)) from error
    if live_type != row["type"]:
        raise PlanError(f"{where} existing issue type does not match {row['type']}")
    if expected_owner not in before["labels"]:
        raise PlanError(f"{where} existing issue owner does not match {expected_owner}")
    return row


def validate_plan(value: object) -> dict:
    plan = _keys(
        value,
        {"schema", "repo", "base_commit", "plan_sha256", "projects"},
        "plan",
    )
    if plan["schema"] != SCHEMA:
        raise PlanError(f"plan.schema must be {SCHEMA}")
    if not isinstance(plan["repo"], str) \
            or not re.fullmatch(r"[^/\s]+/[^/\s]+", plan["repo"]):
        raise PlanError("plan.repo must be owner/repo")
    if not isinstance(plan["base_commit"], str) or not COMMIT.fullmatch(plan["base_commit"]):
        raise PlanError("plan.base_commit must be a full lowercase Git commit")
    _sha(plan["plan_sha256"], "plan.plan_sha256")
    projects = plan["projects"]
    if not isinstance(projects, list) or not projects:
        raise PlanError("plan.projects must be a nonempty ordered list")
    seen: set[str] = set()
    for index, item in enumerate(projects):
        where = f"plan.projects[{index}]"
        row = _keys(
            item,
            {"path", "mode", "documents", "tracker_baseline", "milestone",
             "issues", "development_order"},
            where,
            required={"path", "mode", "documents", "tracker_baseline"},
        )
        path = row["path"]
        if not isinstance(path, str) or not PROJECT.fullmatch(path):
            raise PlanError(f"{where}.path must be apps/name or libs/name")
        if path in seen:
            raise PlanError(f"duplicate project {path}")
        seen.add(path)
        if row["mode"] not in {"product", "release"}:
            raise PlanError(f"{where}.mode must be product or release")
        if not isinstance(row["documents"], list) or not row["documents"]:
            raise PlanError(f"{where}.documents must be nonempty")
        documents = [
            _document(doc, f"{where}.documents[{doc_index}]")
            for doc_index, doc in enumerate(row["documents"])
        ]
        paths = [doc["path"] for doc in documents]
        if len(paths) != len(set(paths)):
            raise PlanError(f"{where}.documents has duplicate paths")
        baseline = _tracker_baseline(row["tracker_baseline"],
                                     f"{where}.tracker_baseline", path)
        project_name = path.rsplit("/", 1)[1]
        if any(milestone.release_identity(item["title"]).project != project_name
               for item in baseline["milestones"]):
            raise PlanError(f"{where}.tracker_baseline has a foreign Milestone")
        if row["mode"] == "product":
            if any(name in row for name in ("milestone", "issues", "development_order")):
                raise PlanError(f"{where}: product mode has documents only")
            if any(isinstance(doc["after"], str)
                   and MILESTONE_NUMBER in doc["after"] for doc in documents):
                raise PlanError(f"{where}: product mode cannot bind a milestone number")
            continue
        if not {"milestone", "issues", "development_order"} <= set(row):
            raise PlanError(
                f"{where}: release mode requires milestone, issues, and development_order"
            )
        if not isinstance(row["issues"], list) or not row["issues"]:
            raise PlanError(f"{where}.issues must be nonempty")
        issues = [
            _planned_issue(issue, f"{where}.issues[{issue_index}]", path, baseline)
            for issue_index, issue in enumerate(row["issues"])
        ]
        keys = [issue["key"] for issue in issues]
        numbers = [issue.get("number") for issue in issues if issue.get("number") is not None]
        if len(keys) != len(set(keys)):
            raise PlanError(f"{where}.issues has duplicate keys")
        if len(numbers) != len(set(numbers)):
            raise PlanError(f"{where}.issues has duplicate existing numbers")
        order = row["development_order"]
        if not isinstance(order, list) or any(not isinstance(key, str) for key in order) \
                or len(order) != len(keys) or set(order) != set(keys):
            raise PlanError(
                f"{where}.development_order must contain every issue key exactly once"
            )
        planned_milestone = _planned_milestone(
            row["milestone"], f"{where}.milestone", path, baseline, len(order),
        )
        if planned_milestone.get("number") is not None:
            assigned = {
                item["number"] for item in baseline["issues"]
                if item["milestone"] == planned_milestone["number"]
            }
            covered = {
                issue["number"] for issue in issues if issue.get("number") is not None
            }
            if assigned != covered:
                raise PlanError(
                    f"{where}.issues must contain every baseline issue assigned "
                    "to the existing milestone"
                )
        if not any(isinstance(doc["after"], str)
                   and MILESTONE_HEADING_MARKER in doc["after"]
                   for doc in documents):
            raise PlanError(
                f"{where}.documents must contain the promise heading marker "
                f"{MILESTONE_HEADING_MARKER} for the facade to bind"
            )
        tracking_marker = milestone_tracking_link(plan["repo"], MILESTONE_NUMBER)
        if not any(isinstance(doc["after"], str)
                   and tracking_marker in doc["after"]
                   for doc in documents):
            raise PlanError(
                f"{where}.documents must contain the exact promise Tracking link "
                f"{tracking_marker}"
            )
    if plan_digest(plan) != plan["plan_sha256"]:
        raise PlanError("plan.plan_sha256 does not match the canonical plan payload")
    return plan


def read_plan(path: str, *, stdin_ok: bool,
              seal_missing: bool = False) -> tuple[dict, str]:
    if path == "-":
        if not stdin_ok:
            raise PlanError("apply requires a real --plan file")
        raw = sys.stdin.read()
    else:
        try:
            raw = Path(path).read_text(encoding="utf-8")
        except OSError as error:
            raise PlanError(f"cannot read plan: {error}") from error
    value = _load_json(raw, "plan")
    if seal_missing and isinstance(value, dict) and "plan_sha256" not in value:
        value = copy.deepcopy(value)
        value["plan_sha256"] = plan_digest(value)
    plan = validate_plan(value)
    return plan, plan["plan_sha256"]


def _git(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", "-c", "core.fsmonitor=false", *args], cwd=ROOT,
        capture_output=True, text=True, check=False,
    )


def _head() -> str:
    result = _git("rev-parse", "HEAD")
    if result.returncode or not COMMIT.fullmatch(result.stdout.strip()):
        raise PlanError("cannot resolve current Git HEAD")
    return result.stdout.strip()


def _dirty() -> list[str]:
    result = _git("status", "--porcelain=v1", "-uall")
    if result.returncode:
        raise PlanError("cannot inspect the working tree")
    return [line for line in result.stdout.splitlines() if line]


def _dirty_paths() -> list[str]:
    try:
        return metadoc.leg.dirty_set(ROOT)
    except SystemExit as error:
        raise PlanError(f"cannot inspect working-tree paths: {error}") from error


def _configured_repo() -> str:
    try:
        return workitem.default_repo()
    except workitem.GhError as error:
        raise PlanError(str(error)) from error


def _normal_milestone(raw: dict) -> dict:
    return {
        "number": int(raw["number"]),
        "title": raw.get("title") or "",
        "state": (raw.get("state") or "").upper(),
        "description_sha256": _text_sha(raw.get("description") or ""),
    }


def _normal_issue(raw: dict) -> dict:
    milestone_number = (raw.get("milestone") or {}).get("number")
    return {
        "number": int(raw["number"]),
        "title": raw.get("title") or "",
        "state": (raw.get("state") or "").upper(),
        "labels": sorted(raw.get("labels") or []),
        "milestone": milestone_number,
        "body_sha256": _text_sha(raw.get("body") or ""),
    }


def _tracker_snapshot(repo: str, target: dict) -> dict:
    project_name = target["path"].rsplit("/", 1)[1]
    milestones = []
    for raw in milestone.list_milestones(repo, "all"):
        identity = milestone.release_identity(raw.get("title") or "")
        if identity is not None and identity.project == project_name:
            milestones.append(_normal_milestone(raw))
    issues = []
    for listed in workitem.fetch_issues_by_label(owner_label(target["path"]), repo):
        issues.append(_normal_issue(workitem.fetch_issue(str(listed["number"]), repo)))
    milestones.sort(key=lambda row: row["number"])
    issues.sort(key=lambda row: row["number"])
    return {"milestones": milestones, "issues": issues}


def _milestone_issue_numbers(repo: str, number: int) -> list[int]:
    live = milestone.resolve_milestone(f"milestone:{number}", repo)
    return sorted(int(row["number"]) for row in milestone.milestone_issues(live, repo))


def _assert_existing_milestone_membership(repo: str, target: dict) -> None:
    if target["mode"] != "release" or target["milestone"].get("number") is None:
        return
    number = target["milestone"]["number"]
    expected = sorted(
        row["number"] for row in target["issues"] if row.get("number") is not None
    )
    if _milestone_issue_numbers(repo, number) != expected:
        raise PlanError(
            "existing milestone membership includes an unplanned or unowned issue"
        )


def _expected_milestone_membership(target: dict, receipt: dict,
                                   pending_number: int | None = None) -> list[int]:
    """Numbers that may be assigned at one exact receipt checkpoint."""
    state = receipt["milestone"]
    if state is None or state["number"] is None:
        return []
    expected = {
        row["number"] for row in target["tracker_baseline"]["issues"]
        if row["milestone"] == state["number"]
    }
    expected.update(
        row["number"] for row in receipt["issues"]
        if row["status"] == "APPLIED" and row["number"] is not None
    )
    if pending_number is not None:
        expected.add(pending_number)
    return sorted(expected)


def _assert_milestone_membership(repo: str, target: dict, receipt: dict,
                                 where: str, *,
                                 pending_number: int | None = None) -> None:
    """Refuse an issue that the owner-label snapshot cannot see."""
    state = receipt["milestone"]
    if target["mode"] != "release" or state["status"] == "PENDING" \
            or state["number"] is None:
        return
    expected = _expected_milestone_membership(target, receipt, pending_number)
    if _milestone_issue_numbers(repo, state["number"]) != expected:
        raise PlanError(
            f"milestone membership has an unplanned or unowned issue {where}"
        )


def _baseline_snapshot(target: dict) -> dict:
    return {
        "milestones": copy.deepcopy(target["tracker_baseline"]["milestones"]),
        "issues": copy.deepcopy(target["tracker_baseline"]["issues"]),
    }


def desired_milestone_row(target: dict, number: int, description: str) -> dict:
    return {
        "number": number,
        "title": target["milestone"]["title"],
        "state": "OPEN",
        "description_sha256": _text_sha(description),
    }


def desired_issue_row(planned: dict, before: dict | None, number: int,
                      milestone_number: int) -> dict:
    if before is None:
        labels = {
            planned["owner"], f"type:{planned['type']}", "phase:created",
        }
    else:
        labels = {
            label for label in before["labels"] if not label.startswith("priority:")
        }
    labels.add(f"priority:{planned['priority']}")
    return {
        "number": number,
        "title": planned["title"],
        "state": (before or {}).get("state", "OPEN"),
        "labels": sorted(labels),
        "milestone": milestone_number,
        "body_sha256": _text_sha(planned["body"]),
    }


def render_milestone_description(target: dict, numbers: list[int],
                                 *, draft: bool = False) -> str:
    replacement = milestone.DRAFT_LINE if draft else "\n".join(
        f"{rank}. #{number}" for rank, number in enumerate(numbers, 1)
    )
    return target["milestone"]["description"].replace(DEVELOPMENT_ORDER, replacement)


def render_documents(target: dict, milestone_number: int | None) -> list[dict]:
    rendered = copy.deepcopy(target["documents"])
    for row in rendered:
        if milestone_number is not None and isinstance(row["after"], str):
            row["after"] = row["after"].replace(MILESTONE_NUMBER, str(milestone_number))
    return rendered


def receipt_path(plan_sha: str, project: str) -> Path:
    return ROOT / ".aw" / "release-plans" / plan_sha / f"{project.replace('/', '__')}.json"


@contextmanager
def _release_plan_lock():
    """Serialize every Apply and Resume process for this checkout."""
    path = ROOT / ".aw" / "release-plans" / ".lock"
    path.parent.mkdir(parents=True, exist_ok=True)
    try:
        descriptor = os.open(path, os.O_CREAT | os.O_RDWR, 0o600)
    except OSError as error:
        raise PlanError(f"cannot open the release-plan lock: {error}") from error
    locked = False
    try:
        try:
            fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise PlanError("another release-plan apply or resume is active") from error
        locked = True
        yield
    finally:
        if locked:
            fcntl.flock(descriptor, fcntl.LOCK_UN)
        os.close(descriptor)


def _atomic_write_bytes(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    try:
        with os.fdopen(descriptor, "wb") as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
        directory = os.open(path.parent, os.O_RDONLY)
        try:
            os.fsync(directory)
        finally:
            os.close(directory)
    finally:
        try:
            os.unlink(temporary)
        except FileNotFoundError:
            pass


def _write_receipt(path: Path, receipt: dict) -> None:
    _atomic_write_bytes(path, canonical_bytes(receipt))


def _receipt_target(receipt: dict) -> dict:
    targets = [row for row in receipt["plan"]["projects"]
               if row["path"] == receipt["project"]]
    if len(targets) != 1:
        raise PlanError("receipt project is not unique in its plan")
    return targets[0]


def _validate_receipt(value: object) -> dict:
    receipt = _keys(value, {
        "schema", "state", "plan_sha256", "plan", "project", "start_commit",
        "milestone", "meta", "issues", "evidence",
    }, "receipt")
    if receipt["schema"] != RECEIPT_SCHEMA:
        raise PlanError("receipt has an unknown schema")
    if receipt["state"] not in {"INCOMPLETE", "COMPLETE"}:
        raise PlanError("receipt has an unknown state")
    _sha(receipt["plan_sha256"], "receipt.plan_sha256")
    plan = validate_plan(receipt["plan"])
    if plan["plan_sha256"] != receipt["plan_sha256"]:
        raise PlanError("receipt plan digest does not match its plan")
    if not isinstance(receipt["project"], str):
        raise PlanError("receipt.project must be text")
    if not isinstance(receipt["start_commit"], str) \
            or not COMMIT.fullmatch(receipt["start_commit"]):
        raise PlanError("receipt.start_commit must be a full Git commit")
    target = _receipt_target(receipt)
    meta_state = _keys(receipt["meta"], {"status", "commit"}, "receipt.meta")
    if meta_state["status"] not in {"PENDING", "APPLIED"}:
        raise PlanError("receipt.meta.status is invalid")
    if meta_state["status"] == "PENDING" and meta_state["commit"] is not None:
        raise PlanError("pending receipt meta cannot carry a commit")
    if meta_state["status"] == "APPLIED" \
            and (not isinstance(meta_state["commit"], str)
                 or not COMMIT.fullmatch(meta_state["commit"])):
        raise PlanError("applied receipt meta needs a full commit")
    if not isinstance(receipt["issues"], list):
        raise PlanError("receipt.issues must be a list")
    if target["mode"] == "product":
        if receipt["milestone"] is not None or receipt["issues"]:
            raise PlanError("product receipt cannot carry tracker write state")
    else:
        milestone_state = _keys(receipt["milestone"], {
            "status", "number", "created",
        }, "receipt.milestone")
        if milestone_state["status"] not in {"PENDING", "READY", "FINAL"}:
            raise PlanError("receipt.milestone.status is invalid")
        if not isinstance(milestone_state["created"], bool):
            raise PlanError("receipt.milestone.created must be boolean")
        if milestone_state["number"] is not None:
            _positive(milestone_state["number"], "receipt.milestone.number")
        elif milestone_state["status"] != "PENDING":
            raise PlanError("ready receipt milestone needs a number")
        if len(receipt["issues"]) != len(target["issues"]):
            raise PlanError("receipt issue state does not match the plan")
        for index, state in enumerate(receipt["issues"]):
            state = _keys(state, {"key", "status", "number"},
                          f"receipt.issues[{index}]")
            if state["key"] != target["issues"][index]["key"]:
                raise PlanError("receipt issue keys do not match the plan")
            if state["status"] not in {"PENDING", "APPLIED"}:
                raise PlanError("receipt issue status is invalid")
            if state["number"] is not None:
                _positive(state["number"], f"receipt.issues[{index}].number")
            elif state["status"] == "APPLIED":
                raise PlanError("applied receipt issue needs a number")
    if receipt["evidence"] is not None and not isinstance(receipt["evidence"], dict):
        raise PlanError("receipt.evidence must be an object or null")
    if receipt["state"] == "COMPLETE":
        if receipt["meta"]["status"] != "APPLIED" or receipt["evidence"] is None:
            raise PlanError("complete receipt lacks META commit or evidence")
        if target["mode"] == "release" and (
            receipt["milestone"]["status"] != "FINAL"
            or any(row["status"] != "APPLIED" for row in receipt["issues"])
        ):
            raise PlanError("complete release receipt has unfinished tracker writes")
    return receipt


def read_receipt(path: Path) -> dict:
    try:
        raw = path.read_text(encoding="utf-8")
    except OSError as error:
        raise PlanError(f"invalid receipt: {error}") from error
    value = _load_json(raw, "receipt")
    return _validate_receipt(value)


def _new_receipt(plan: dict, plan_sha: str, target: dict, start: str) -> dict:
    release = target["mode"] == "release"
    return {
        "schema": RECEIPT_SCHEMA,
        "state": "INCOMPLETE",
        "plan_sha256": plan_sha,
        "plan": plan,
        "project": target["path"],
        "start_commit": start,
        "milestone": None if not release else {
            "status": "PENDING",
            "number": target["milestone"].get("number"),
            "created": target["milestone"].get("number") is None,
        },
        "meta": {"status": "PENDING", "commit": None},
        "issues": [] if not release else [
            {"key": row["key"], "status": "PENDING", "number": row.get("number")}
            for row in target["issues"]
        ],
        "evidence": None,
    }


def _replace_numbered(rows: list[dict], desired: dict) -> None:
    rows[:] = [row for row in rows if row["number"] != desired["number"]]
    rows.append(desired)
    rows.sort(key=lambda row: row["number"])


def _expected_tracker(target: dict, receipt: dict,
                      *, pending_issue: tuple[dict, int] | None = None,
                      final_milestone: bool | None = None) -> dict:
    expected = _baseline_snapshot(target)
    if target["mode"] == "product":
        return expected
    milestone_state = receipt["milestone"]
    if milestone_state["status"] != "PENDING" and milestone_state["number"] is not None:
        final = milestone_state["status"] == "FINAL"
        if final_milestone is not None:
            final = final_milestone
        # A new Milestone exists as the generated draft until finalize. An
        # existing Milestone keeps its approved baseline description until
        # that same finalize write; it is never forced back to draft.
        if final or milestone_state["created"]:
            numbers = [row["number"] for row in receipt["issues"]
                       if row["status"] == "APPLIED"]
            description = render_milestone_description(
                target, numbers if final else [], draft=not final,
            )
            _replace_numbered(expected["milestones"], desired_milestone_row(
                target, milestone_state["number"], description,
            ))
    for planned, state in zip(target["issues"], receipt["issues"], strict=True):
        number = state["number"]
        applied = state["status"] == "APPLIED"
        if pending_issue is not None and pending_issue[0]["key"] == planned["key"]:
            applied = True
            number = pending_issue[1]
        if not applied or number is None:
            continue
        before = next((row for row in target["tracker_baseline"]["issues"]
                       if row["number"] == planned.get("number")), None)
        _replace_numbered(expected["issues"], desired_issue_row(
            planned, before, number, milestone_state["number"],
        ))
    return expected


def _assert_tracker(repo: str, target: dict, expected: dict, where: str) -> dict:
    actual = _tracker_snapshot(repo, target)
    if actual != expected:
        raise PlanError(f"tracker drift {where}")
    return actual


def _document_path(target: dict, row: dict, *, root: Path | None = None) -> Path:
    root = ROOT if root is None else root
    project = root / target["path"]
    cursor = root
    for part in (*Path(target["path"]).parts, *Path(row["path"]).parts):
        cursor /= part
        if cursor.is_symlink():
            raise PlanError(
                f"document path crosses a symbolic link: "
                f"{target['path']}/{row['path']}"
            )
    try:
        project_root = project.resolve(strict=True)
        candidate = (project / row["path"]).resolve(strict=False)
        candidate.relative_to(project_root)
        project_root.relative_to(root.resolve(strict=True))
    except (FileNotFoundError, ValueError) as error:
        raise PlanError(
            f"document path escapes its project: {target['path']}/{row['path']}"
        ) from error
    return project / row["path"]


def _documents_are_after(target: dict, rendered: list[dict]) -> bool:
    for row in rendered:
        path = _document_path(target, row)
        if row["after"] is None:
            if path.exists():
                return False
        elif not path.is_file() or path.read_bytes() != row["after"].encode():
            return False
    return True


def _check_documents_before(target: dict) -> None:
    for row in target["documents"]:
        path = _document_path(target, row)
        if row["before_sha256"] is None:
            valid = not path.exists()
        else:
            valid = (path.is_file()
                     and hashlib.sha256(path.read_bytes()).hexdigest()
                     == row["before_sha256"])
        if not valid:
            raise PlanError(f"document drift: {target['path']}/{row['path']}")


def _assert_resume_worktree(receipt: dict, target: dict) -> None:
    """Admit only exact partial META bytes before any resumed remote write."""
    dirty = _dirty_paths()
    if not dirty:
        return
    if receipt["meta"]["status"] == "APPLIED":
        raise PlanError(f"working tree drift after META commit: {dirty[0]}")
    milestone_state = receipt["milestone"]
    if target["mode"] == "release" and milestone_state["status"] == "PENDING":
        raise PlanError(f"working tree drift before Milestone recovery: {dirty[0]}")
    allowed = {
        f"{target['path']}/{row['path']}" for row in target["documents"]
    }
    outside = sorted(set(dirty) - allowed)
    if outside:
        raise PlanError(f"working tree drift outside planned META documents: {outside[0]}")
    number = None
    if target["mode"] == "release":
        number = milestone_state["number"]
    rendered = render_documents(target, number)
    for original, row in zip(target["documents"], rendered, strict=True):
        name = f"{target['path']}/{row['path']}"
        if name not in dirty:
            continue
        path = _document_path(target, row)
        current = path.read_bytes() if path.is_file() else None
        before_ok = (
            (original["before_sha256"] is None and not path.exists())
            or (
                original["before_sha256"] is not None
                and current is not None
                and hashlib.sha256(current).hexdigest() == original["before_sha256"]
            )
        )
        after = None if row["after"] is None else row["after"].encode()
        after_ok = (
            (after is None and not path.exists())
            or (after is not None and current == after)
        )
        if not before_ok and not after_ok:
            raise PlanError(f"working tree has non-plan META bytes: {name}")


def _write_rendered_documents(target: dict, rendered: list[dict], *,
                              root: Path | None = None) -> None:
    for original, row in zip(target["documents"], rendered, strict=True):
        path = _document_path(target, row, root=root)
        current = path.read_bytes() if path.is_file() else None
        after = None if row["after"] is None else row["after"].encode()
        if current == after:
            continue
        if original["before_sha256"] is None:
            valid_before = current is None and not path.exists()
        else:
            valid_before = (current is not None
                            and hashlib.sha256(current).hexdigest()
                            == original["before_sha256"])
        if not valid_before:
            raise PlanError(f"document drift: {target['path']}/{row['path']}")
        if after is None:
            path.unlink()
            directory = os.open(path.parent, os.O_RDONLY)
            try:
                os.fsync(directory)
            finally:
                os.close(directory)
        else:
            _atomic_write_bytes(path, after)


def _check_exact_document_bindings(repo: Path, target: dict,
                                   milestone_number: int | None,
                                   repository: str) -> None:
    if target["mode"] != "release":
        return
    if milestone_number is None:
        raise PlanError("release META validation needs the exact Milestone number")
    gained: Counter[int] = Counter()
    exact_promise_headings = 0
    gained_exact_promise_headings = 0
    existing_milestone = target["milestone"].get("number") is not None
    expected_tracking = milestone_tracking_link(repository, milestone_number)
    for row in target["documents"]:
        name = f"{target['path']}/{row['path']}"
        before = metadoc.git_show(repo, name)
        path = _document_path(target, row, root=repo)
        after = path.read_text(encoding="utf-8") if path.is_file() else ""
        before_numbers = Counter(
            int(match.group("number")) for match in DOCUMENT_MILESTONE.finditer(before)
        )
        after_numbers = Counter(
            int(match.group("number")) for match in DOCUMENT_MILESTONE.finditer(after)
        )
        gained += after_numbers - before_numbers
        if not metadoc.is_area(repo, target["path"], name):
            continue
        before_bindings: dict[str, int | None] = {}
        before_bodies: dict[str, str] = {}
        for raw, body in metadoc.sections(before):
            match = metadoc.MILESTONE.search(raw)
            title = metadoc.bare(raw)
            before_bindings[title] = (
                int(match.group(1)) if match else None
            )
            before_bodies[title] = body
        for raw, body in metadoc.sections(after):
            title = metadoc.bare(raw)
            match = metadoc.MILESTONE.search(raw)
            exact_heading = (
                match is not None and int(match.group(1)) == milestone_number
            )
            gained_heading = (
                exact_heading and before_bindings.get(title) != milestone_number
            )
            if exact_heading:
                exact_promise_headings += 1
            if gained_heading:
                gained_exact_promise_headings += 1
            check_owner = exact_heading and (existing_milestone or gained_heading)
            before_links = Counter(
                (link.group("kind"), int(link.group("number")))
                for link in metadoc.TRACKING_LINK.finditer(
                    before_bodies.get(title, "")
                )
            )
            after_links = Counter(
                (link.group("kind"), int(link.group("number")))
                for link in metadoc.TRACKING_LINK.finditer(body)
            )
            if after_links - before_links and not check_owner:
                raise PlanError(
                    f"section `{title}` adds a Tracking binding outside an "
                    "exact-bound promise heading"
                )
            if not check_owner:
                continue
            owners = [
                rest for key, rest in metadoc.bullets(body)
                if key in metadoc.OWNERS
            ]
            tracking_value = ""
            if len(owners) == 1 and owners[0].count("Tracking:") == 1 \
                    and body.count("Tracking:") == 1:
                tracking_value = owners[0].split("Tracking:", 1)[1].strip()
                if tracking_value.endswith("."):
                    tracking_value = tracking_value[:-1].rstrip()
            if tracking_value != expected_tracking:
                raise PlanError(
                    f"promise heading `{title}` must carry its exact Milestone "
                    "Tracking link"
                )
    unexpected = sorted(number for number in gained if number != milestone_number)
    if unexpected:
        raise PlanError(
            "planned META documents add an unexpected Milestone binding: "
            + ", ".join(f"#{number}" for number in unexpected)
        )
    if existing_milestone and exact_promise_headings < 1:
        raise PlanError(
            "planned META documents do not contain an exact-bound promise heading"
        )
    if not existing_milestone and gained_exact_promise_headings < 1:
        raise PlanError(
            "planned META documents do not add an exact-bound promise heading"
        )


def _check_rendered_meta(repo: Path, target: dict,
                         milestone_number: int | None,
                         repository: str) -> None:
    """Validate planned document bytes without granting the public P4 bypass."""
    if target["mode"] == "release" and milestone_number is None:
        raise PlanError("release META validation needs the exact Milestone number")
    try:
        findings, _population = metadoc.collect(
            repo, target["path"],
            release_milestone_number=milestone_number,
        )
    except SystemExit as error:
        raise PlanError(f"planned META document check failed: {error}") from error
    if findings:
        first = findings[0]
        raise PlanError(
            "planned META documents fail metadoc "
            f"{first.rule} at {first.path}: {first.message}"
        )
    _check_exact_document_bindings(repo, target, milestone_number, repository)
    try:
        findings, _population = meta.collect(
            repo, tuple(meta.RULES), (target["path"],),
        )
    except SystemExit as error:
        raise PlanError(f"planned META document check failed: {error}") from error
    if findings:
        first = findings[0]
        line = f":{first.line}" if first.line else ""
        raise PlanError(
            "planned META documents fail META "
            f"{first.rule} at {first.path}{line}: {first.message}"
        )


def _planned_tracker_for_wis(target: dict, milestone_number: int) -> tuple[
        list[dict], list[dict]]:
    """Render the final tracker shape with harmless stand-in issue numbers."""
    expected = _baseline_snapshot(target)
    used = {row["number"] for row in expected["issues"]}
    next_number = max(used, default=0) + 1
    numbers: dict[str, int] = {}
    for planned in target["issues"]:
        number = planned.get("number")
        if number is None:
            while next_number in used:
                next_number += 1
            number = next_number
            used.add(number)
            next_number += 1
        numbers[planned["key"]] = number
        before = next(
            (row for row in target["tracker_baseline"]["issues"]
             if row["number"] == planned.get("number")),
            None,
        )
        _replace_numbered(expected["issues"], desired_issue_row(
            planned, before, number, milestone_number,
        ))
    ordered = [numbers[key] for key in target["development_order"]]
    description = render_milestone_description(target, ordered)
    _replace_numbered(expected["milestones"], desired_milestone_row(
        target, milestone_number, description,
    ))
    milestones = [
        {"number": row["number"], "title": row["title"], "state": row["state"]}
        for row in expected["milestones"]
    ]
    issues = [
        {
            "number": row["number"], "title": row["title"],
            "state": row["state"], "labels": row["labels"],
            "milestone": (
                None if row["milestone"] is None
                else {"number": row["milestone"]}
            ),
        }
        for row in expected["issues"]
    ]
    return milestones, issues


def _check_planned_wis(repo: Path, target: dict,
                       milestone_number: int | None) -> None:
    """Prove the release plan's final G1-G5 state before any real write."""
    if target["mode"] != "release":
        return
    if milestone_number is None:
        raise PlanError("release WIS preview needs the exact Milestone number")
    texts = wis.area_texts(repo, target["path"])
    if not texts:
        raise PlanError("planned release has no indexed promise area for WIS")
    found = wis.promises(texts)
    milestones, issues = _planned_tracker_for_wis(target, milestone_number)
    ledger = wis.Ledger()
    wis.g1_unbound_promises(ledger, found)
    wis.g2_orphan_items(
        ledger, milestones, issues, found, owner_label(target["path"]),
    )
    wis.g3_stale_bindings(
        ledger, milestones, found, owner_label(target["path"]),
    )
    wis.g4_uncovered_outcomes(
        ledger, wis.roadmap_ids(repo, target["path"]), found, target["path"],
    )
    wis.g5_unpromised_surfaces(
        ledger, wis.status_ids(repo, target["path"]), found, target["path"],
    )
    rules = {"G1", "G2", "G3", "G4", "G5"}
    failed = sorted(
        set(rules.intersection(ledger.blocked))
        | {gap.rule for gap in ledger.gaps if gap.rule in rules}
    )
    if failed:
        raise PlanError(
            "planned final WIS state has unresolved row(s): "
            + ", ".join(failed)
        )


def _preview_documents(target: dict, start: str, repository: str) -> None:
    """Check the approved bytes in a disposable clone before any remote write."""
    if target["mode"] == "product":
        numbers: list[int | None] = [None]
    elif target["milestone"].get("number") is not None:
        numbers = [target["milestone"]["number"]]
    else:
        # A new Milestone has no number yet. Two distinct substitutions prove
        # that a literal binding matching one preview value cannot hide beside
        # the approved marker and fail only after GitHub creates the Milestone.
        numbers = [1, 2_147_483_647]
    with tempfile.TemporaryDirectory(prefix="aw-release-plan-preview-") as temporary:
        for index, number in enumerate(numbers):
            preview = Path(temporary) / f"repo-{index}"
            cloned = subprocess.run(
                ["git", "-c", "core.fsmonitor=false", "clone", "--quiet", "--shared",
                 "--no-checkout", str(ROOT), str(preview)],
                cwd=ROOT, capture_output=True, text=True,
            )
            if cloned.returncode != 0:
                raise PlanError(
                    "cannot create META preview clone: "
                    + (cloned.stderr.strip() or cloned.stdout.strip()
                       or "git clone failed")
                )
            checked_out = subprocess.run(
                ["git", "-c", "core.fsmonitor=false", "checkout", "--quiet",
                 "--detach", start],
                cwd=preview, capture_output=True, text=True,
            )
            if checked_out.returncode != 0:
                raise PlanError(
                    "cannot check out META preview baseline: "
                    + (checked_out.stderr.strip() or "git checkout failed")
                )
            rendered = render_documents(target, number)
            _write_rendered_documents(target, rendered, root=preview)
            staged = subprocess.run(
                ["git", "-c", "core.fsmonitor=false", "add", "--all", "--",
                 target["path"]],
                cwd=preview, capture_output=True, text=True,
            )
            if staged.returncode != 0:
                raise PlanError(
                    "cannot stage disposable META preview: "
                    + (staged.stderr.strip() or "git add failed")
                )
            _check_rendered_meta(preview, target, number, repository)
            _check_planned_wis(preview, target, number)


def _is_expected_meta_commit(start: str, commit: str, target: dict,
                             rendered: list[dict]) -> bool:
    parent = _git("rev-parse", f"{commit}^")
    paths = _git("diff-tree", "--no-commit-id", "--name-only", "-r", commit)
    message = _git("show", "-s", "--format=%B", commit)
    if parent.returncode or paths.returncode or message.returncode:
        return False
    expected_paths = sorted(f"{target['path']}/{row['path']}" for row in rendered)
    actual_paths = sorted(line for line in paths.stdout.splitlines() if line)
    committed = True
    for row in rendered:
        name = f"{target['path']}/{row['path']}"
        blob = subprocess.run(
            ["git", "-c", "core.fsmonitor=false", "show", f"{commit}:{name}"],
            cwd=ROOT, capture_output=True, check=False,
        )
        if row["after"] is None:
            committed = committed and blob.returncode != 0
        else:
            committed = (
                committed and blob.returncode == 0
                and blob.stdout == row["after"].encode()
            )
    return (
        parent.stdout.strip() == start
        and actual_paths == expected_paths
        and f"Meta-Project: {target['path']}" in message.stdout
        and committed
        and _documents_are_after(target, rendered)
    )


def _commit_documents(target: dict, rendered: list[dict], start: str,
                      scratch: Path, *, milestone_number: int | None,
                      repository: str) -> str:
    _write_rendered_documents(target, rendered)
    _check_rendered_meta(ROOT, target, milestone_number, repository)
    name = target["path"].rsplit("/", 1)[1]
    why = scratch / "metadoc-why.md"
    _atomic_write_bytes(
        why,
        (f"docs({name}): apply approved release plan\n\n"
         "Apply the exact human-approved META document bytes.\n").encode(),
    )
    try:
        status = metadoc.cmd_commit(argparse.Namespace(
            project=target["path"], why=str(why), dry_run=False,
            release_milestone_number=milestone_number,
        ))
    except SystemExit as error:
        raise PlanError(f"metadoc commit failed: {error}") from error
    if status != 0:
        raise PlanError("metadoc commit failed")
    commit = _head()
    if commit == start or not _is_expected_meta_commit(start, commit, target, rendered):
        raise PlanError("META commit readback did not match the approved documents")
    return commit


def _create_draft_milestone(repo: str, title: str, description: str) -> int:
    if milestone.validate_description(description, allow_draft=True) \
            or not milestone.is_draft_description(description):
        raise PlanError("generated draft milestone description is invalid")
    if milestone.duplicate_title(title, repo) is not None:
        raise PlanError(f"milestone title already exists: {title}")
    try:
        raw = workitem.gh(
            "api", "--method", "POST", f"repos/{repo}/milestones",
            "-f", f"title={title}", "-f", "state=open",
            "-f", f"description={description}",
        )
        created = json.loads(raw)
    except (workitem.GhError, json.JSONDecodeError, KeyError) as error:
        raise PlanError(f"milestone creation failed: {error}") from error
    return _positive(created.get("number"), "created milestone number")


def _issue_match(row: dict, planned: dict, milestone_number: int) -> bool:
    return (
        row["title"] == planned["title"]
        and row["body_sha256"] == _text_sha(planned["body"])
        and row["milestone"] == milestone_number
        and planned["owner"] in row["labels"]
        and f"type:{planned['type']}" in row["labels"]
        and f"priority:{planned['priority']}" in row["labels"]
    )


def _write_planned_issue(repo: str, target: dict, planned: dict,
                         milestone_number: int, scratch: Path) -> int:
    body = scratch / f"issue-{planned['key']}.md"
    _atomic_write_bytes(body, planned["body"].encode())
    argv = ["--repo", repo]
    if planned.get("number") is None:
        argv += [
            "create", "--title", planned["title"], "--body-file", str(body),
            "--type", planned["type"], "--milestone", f"milestone:{milestone_number}",
            "--priority", planned["priority"], "--project", planned["owner"],
        ]
    else:
        argv += [
            "update", str(planned["number"]), "--title", planned["title"],
            "--body-file", str(body), "--milestone", f"milestone:{milestone_number}",
        ]
        before = next(
            row for row in target["tracker_baseline"]["issues"]
            if row["number"] == planned["number"]
        )
        wanted = f"priority:{planned['priority']}"
        current = [label for label in before["labels"] if label.startswith("priority:")]
        for label in current:
            if label != wanted:
                argv += ["--remove-label", label]
        if wanted not in current:
            argv += ["--add-label", wanted]
    try:
        status = change.main(argv)
    except SystemExit as error:
        raise PlanError(f"issue write failed: {error}") from error
    if status != 0:
        raise PlanError(f"issue write failed for key {planned['key']}")
    snapshot = _tracker_snapshot(repo, target)
    if planned.get("number") is not None:
        candidates = [row for row in snapshot["issues"]
                      if row["number"] == planned["number"]
                      and _issue_match(row, planned, milestone_number)]
    else:
        candidates = [row for row in snapshot["issues"]
                      if _issue_match(row, planned, milestone_number)]
    if len(candidates) != 1:
        raise PlanError(
            f"issue readback for {planned['key']} found {len(candidates)} exact matches"
        )
    return candidates[0]["number"]


def _finalize_milestone(repo: str, number: int, description: str,
                        scratch: Path) -> None:
    description_path = scratch / "milestone-final.md"
    _atomic_write_bytes(description_path, description.encode())
    argv = [
        "--repo", repo, "update", f"milestone:{number}",
        "--description-file", str(description_path),
    ]
    try:
        status = milestone.main(argv)
    except SystemExit as error:
        raise PlanError(f"milestone finalize failed: {error}") from error
    if status != 0:
        raise PlanError("milestone finalize failed")


def _wis_evidence(repo: str, target: dict) -> dict:
    """Run every gap row, but block only release-planning ownership drift.

    G6 and G7 describe delivery work. A release plan may create the issues
    that will close those rows, so keeping its receipt incomplete until the
    later e2e and implementation phases run would deadlock that queue.
    """
    try:
        ledger, population = wis.collect(ROOT, target["path"], repo)
        status = wis.report(ledger, population, "json", "")
    except SystemExit as error:
        raise PlanError(f"wis gap failed: {error}") from error
    planning = {"G1", "G2", "G3", "G4", "G5"}
    blocked = sorted(planning.intersection(ledger.blocked))
    gaps = sorted({gap.rule for gap in ledger.gaps if gap.rule in planning})
    failed = sorted(set(blocked + gaps))
    if failed:
        raise PlanError(
            "wis gap has unresolved release-planning row(s): "
            + ", ".join(failed)
        )
    if status not in {0, 1}:
        raise PlanError("wis gap returned an invalid status")
    rows = {
        rule: {
            "population": ledger.population.get(rule),
            "blocked": ledger.blocked.get(rule),
            "gaps": sum(1 for gap in ledger.gaps if gap.rule == rule),
        }
        for rule in wis.GAPS
    }
    return {
        "status": "ALIGNED" if status == 0 else "DELIVERY_GAPS",
        "population": population,
        "rows": rows,
        "gaps": [gap.as_dict() for gap in ledger.gaps],
    }


def _run_release_gates(repo: str, target: dict, milestone_number: int,
                       numbers: list[int]) -> dict:
    ref = f"milestone:{milestone_number}"
    reconcile = argparse.Namespace(ref=ref, repo=repo, json=True)
    order = argparse.Namespace(ref=ref, repo=repo, json=True, open_only=False)
    if milestone.cmd_reconcile(reconcile) != 0:
        raise PlanError("milestone reconcile failed")
    if milestone.cmd_order(order) != 0:
        raise PlanError("milestone order failed")
    live = milestone.resolve_milestone(ref, repo)
    payload = milestone.order_payload(live, milestone.milestone_issues(live, repo))
    actual = [row["number"] for row in payload["order"]]
    if payload["errors"] or actual != numbers:
        raise PlanError("milestone Development Order readback drifted")
    gap_evidence = _wis_evidence(repo, target)
    return {"reconcile": "ALIGNED", "order": actual, "wis_gap": gap_evidence}


def _run_product_gates(repo: str, target: dict) -> dict:
    _assert_tracker(repo, target, _baseline_snapshot(target), "after product apply")
    return {"tracker": "UNCHANGED"}


def _after_write(_point: str) -> None:
    """Fault-injection seam. Production execution does nothing here."""


def _ensure_milestone(receipt: dict, path: Path, target: dict) -> None:
    state = receipt["milestone"]
    if state["status"] != "PENDING":
        # A later tracker write may have succeeded just before a crash while
        # its receipt row is still PENDING. The owning stage performs that
        # exact recovery. Requiring the earlier snapshot here would reject the
        # recoverable state before that stage can inspect it.
        return
    repo = receipt["plan"]["repo"]
    before = _expected_tracker(target, receipt)
    actual = _tracker_snapshot(repo, target)
    planned = target["milestone"]
    if planned.get("number") is not None:
        _assert_existing_milestone_membership(repo, target)
        if actual != before:
            raise PlanError("tracker drift before existing milestone binding")
        state["status"] = "READY"
        _assert_milestone_membership(
            repo, target, receipt, "before META commit",
        )
        _write_receipt(path, receipt)
        return
    draft = render_milestone_description(target, [], draft=True)
    candidates = [row for row in actual["milestones"]
                  if row["title"] == planned["title"]
                  and row["state"] == "OPEN"
                  and row["description_sha256"] == _text_sha(draft)]
    recoverable = []
    for candidate in candidates:
        probe = copy.deepcopy(before)
        _replace_numbered(probe["milestones"], candidate)
        if probe == actual:
            recoverable.append(candidate)
    if len(recoverable) == 1:
        state["number"] = recoverable[0]["number"]
    elif recoverable:
        raise PlanError("milestone recovery found multiple exact matches")
    elif actual == before:
        _assert_resume_worktree(receipt, target)
        state["number"] = _create_draft_milestone(repo, planned["title"], draft)
        _after_write("milestone")
    else:
        raise PlanError("tracker drift before milestone creation")
    state["status"] = "READY"
    _assert_tracker(repo, target, _expected_tracker(target, receipt),
                    "after milestone creation")
    _assert_milestone_membership(
        repo, target, receipt, "after milestone creation",
    )
    _write_receipt(path, receipt)


def _ensure_meta(receipt: dict, path: Path, target: dict) -> None:
    number = None if receipt["milestone"] is None else receipt["milestone"]["number"]
    rendered = render_documents(target, number)
    state = receipt["meta"]
    if state["status"] == "APPLIED":
        if _head() != state["commit"] or not _documents_are_after(target, rendered):
            raise PlanError("META commit or document drift")
        return
    start = receipt["start_commit"]
    current = _head()
    if current == start:
        commit = _commit_documents(
            target, rendered, start, path.parent, milestone_number=number,
            repository=receipt["plan"]["repo"],
        )
        _after_write("meta_commit")
    elif _is_expected_meta_commit(start, current, target, rendered):
        commit = current
    else:
        raise PlanError("Git drift before META commit recovery")
    state.update(status="APPLIED", commit=commit)
    _write_receipt(path, receipt)


def _recover_pending_issue(actual: dict, before: dict, target: dict,
                           receipt: dict, planned: dict) -> int | None:
    state = next(row for row in receipt["issues"] if row["key"] == planned["key"])
    if planned.get("number") is not None:
        candidate_numbers = [planned["number"]]
    else:
        before_numbers = {row["number"] for row in before["issues"]}
        candidate_numbers = [row["number"] for row in actual["issues"]
                             if row["number"] not in before_numbers]
    matches = []
    for number in candidate_numbers:
        probe = _expected_tracker(target, receipt, pending_issue=(planned, number))
        if probe == actual:
            matches.append(number)
    if len(matches) > 1:
        raise PlanError(f"issue recovery for {planned['key']} found multiple exact matches")
    return matches[0] if matches else None


def _ensure_issues(receipt: dict, path: Path, target: dict) -> None:
    repo = receipt["plan"]["repo"]
    milestone_number = receipt["milestone"]["number"]
    for planned, state in zip(target["issues"], receipt["issues"], strict=True):
        if state["status"] == "APPLIED":
            # A later issue or the final Milestone update may already be live
            # while its receipt row is still PENDING. Its own stage compares
            # the complete before and after snapshots.
            continue
        _assert_resume_worktree(receipt, target)
        before = _expected_tracker(target, receipt)
        actual = _tracker_snapshot(repo, target)
        recovered = _recover_pending_issue(actual, before, target, receipt, planned)
        _assert_milestone_membership(
            repo, target, receipt, f"before issue {planned['key']}",
            pending_number=recovered,
        )
        if recovered is not None:
            number = recovered
        elif actual == before:
            number = _write_planned_issue(
                repo, target, planned, milestone_number, path.parent,
            )
            _after_write(f"issue:{planned['key']}")
        else:
            raise PlanError(f"tracker drift before issue {planned['key']}")
        state.update(status="APPLIED", number=number)
        _assert_tracker(
            repo, target, _expected_tracker(target, receipt),
            f"after issue {planned['key']}",
        )
        _assert_milestone_membership(
            repo, target, receipt, f"after issue {planned['key']}",
        )
        _write_receipt(path, receipt)


def _ensure_final_milestone(receipt: dict, path: Path, target: dict) -> None:
    state = receipt["milestone"]
    repo = receipt["plan"]["repo"]
    _assert_resume_worktree(receipt, target)
    if state["status"] == "FINAL":
        _assert_tracker(repo, target, _expected_tracker(target, receipt),
                        "after milestone finalize")
        _assert_milestone_membership(
            repo, target, receipt, "after milestone finalize",
        )
        return
    numbers_by_key = {row["key"]: row["number"] for row in receipt["issues"]}
    numbers = [numbers_by_key[key] for key in target["development_order"]]
    description = render_milestone_description(target, numbers)
    before = _expected_tracker(target, receipt)
    after = _expected_tracker(target, receipt, final_milestone=True)
    actual = _tracker_snapshot(repo, target)
    _assert_milestone_membership(
        repo, target, receipt, "before milestone finalize",
    )
    if actual == after:
        pass
    elif actual == before:
        _finalize_milestone(repo, state["number"], description, path.parent)
        _after_write("finalize")
    else:
        raise PlanError("tracker drift before milestone finalize")
    state["status"] = "FINAL"
    _assert_tracker(repo, target, _expected_tracker(target, receipt),
                    "after milestone finalize")
    _assert_milestone_membership(
        repo, target, receipt, "after milestone finalize",
    )
    _write_receipt(path, receipt)


def _finish(receipt: dict, path: Path, target: dict) -> None:
    repo = receipt["plan"]["repo"]
    _assert_resume_worktree(receipt, target)
    if target["mode"] == "product":
        evidence = _run_product_gates(repo, target)
    else:
        _assert_tracker(repo, target, _expected_tracker(target, receipt),
                        "before final gates")
        _assert_milestone_membership(
            repo, target, receipt, "before final gates",
        )
        numbers_by_key = {row["key"]: row["number"] for row in receipt["issues"]}
        numbers = [numbers_by_key[key] for key in target["development_order"]]
        evidence = _run_release_gates(
            repo, target, receipt["milestone"]["number"], numbers,
        )
        _assert_milestone_membership(
            repo, target, receipt, "after final gates",
        )
    live = _tracker_snapshot(repo, target)
    rendered = render_documents(
        target, None if receipt["milestone"] is None else receipt["milestone"]["number"],
    )
    evidence.update({
        "tracker_sha256": digest(live),
        "documents": [
            {"path": row["path"],
             "after_sha256": None if row["after"] is None else _text_sha(row["after"])}
            for row in rendered
        ],
    })
    receipt["evidence"] = evidence
    receipt["state"] = "COMPLETE"
    _write_receipt(path, receipt)


def _execute(receipt: dict, path: Path) -> int:
    target = _receipt_target(receipt)
    if _configured_repo() != receipt["plan"]["repo"]:
        raise PlanError("configured repository drift")
    _assert_resume_worktree(receipt, target)
    if target["mode"] == "release":
        _ensure_milestone(receipt, path, target)
    if receipt["meta"]["status"] == "PENDING":
        _assert_tracker(
            receipt["plan"]["repo"], target,
            _expected_tracker(target, receipt), "before META commit",
        )
        if target["mode"] == "release":
            _assert_milestone_membership(
                receipt["plan"]["repo"], target, receipt, "before META commit",
            )
    _ensure_meta(receipt, path, target)
    _assert_resume_worktree(receipt, target)
    if target["mode"] == "release":
        _ensure_issues(receipt, path, target)
        _ensure_final_milestone(receipt, path, target)
    _finish(receipt, path, target)
    print(json.dumps({
        "receipt": str(path.relative_to(ROOT)),
        "state": receipt["state"],
        "project": receipt["project"],
        "meta_commit": receipt["meta"]["commit"],
        "milestone": receipt["milestone"],
        "issues": receipt["issues"],
        "evidence": receipt["evidence"],
    }, ensure_ascii=False, sort_keys=True))
    return 0


def _verify_complete(receipt: dict, path: Path, *, run_gates: bool = True,
                     require_head: bool = True) -> None:
    target = _receipt_target(receipt)
    rendered = render_documents(
        target, None if receipt["milestone"] is None else receipt["milestone"]["number"],
    )
    if (require_head and _head() != receipt["meta"]["commit"]) \
            or not _is_expected_meta_commit(
                receipt["start_commit"], receipt["meta"]["commit"], target, rendered,
            ):
        raise PlanError("complete receipt Git readback drifted")
    _assert_tracker(receipt["plan"]["repo"], target,
                    _expected_tracker(target, receipt), "after complete receipt")
    if target["mode"] == "release":
        _assert_milestone_membership(
            receipt["plan"]["repo"], target, receipt, "after complete receipt",
        )
    if not run_gates:
        return
    if target["mode"] == "product":
        _run_product_gates(receipt["plan"]["repo"], target)
    else:
        numbers_by_key = {row["key"]: row["number"] for row in receipt["issues"]}
        numbers = [numbers_by_key[key] for key in target["development_order"]]
        _run_release_gates(
            receipt["plan"]["repo"], target,
            receipt["milestone"]["number"], numbers,
        )
        _assert_milestone_membership(
            receipt["plan"]["repo"], target, receipt,
            "after complete receipt gates",
        )


def _verify_complete_chain(receipt: dict, path: Path) -> None:
    """Accept only the exact contiguous chain that followed this receipt."""
    plan = receipt["plan"]
    if _configured_repo() != plan["repo"]:
        raise PlanError("configured repository drift")
    index = next(
        index for index, row in enumerate(plan["projects"])
        if row["path"] == receipt["project"]
    )
    expected_head = receipt["meta"]["commit"]
    gap = False
    for target in plan["projects"][index + 1:]:
        later_path = receipt_path(receipt["plan_sha256"], target["path"])
        if not later_path.is_file():
            gap = True
            continue
        if gap:
            raise PlanError("later receipt exists after a missing project receipt")
        later = read_receipt(later_path)
        if later["plan_sha256"] != receipt["plan_sha256"] \
                or later["plan"] != plan \
                or later["state"] != "COMPLETE" \
                or later["start_commit"] != expected_head:
            raise PlanError(f"later project receipt is not a valid chain: {target['path']}")
        _verify_complete(later, later_path, run_gates=False, require_head=False)
        expected_head = later["meta"]["commit"]
    if _head() != expected_head:
        raise PlanError("complete receipt Git readback drifted")
    _verify_complete(receipt, path, require_head=False)


def _start_commit(plan: dict, plan_sha: str, target_index: int) -> str:
    expected = plan["base_commit"]
    for prior in plan["projects"][:target_index]:
        path = receipt_path(plan_sha, prior["path"])
        if not path.is_file():
            raise PlanError(f"project order requires completed receipt for {prior['path']}")
        receipt = read_receipt(path)
        if receipt["state"] != "COMPLETE" or receipt["start_commit"] != expected:
            raise PlanError(f"prior project receipt is not complete: {prior['path']}")
        _verify_complete(receipt, path, run_gates=False, require_head=False)
        expected = receipt["meta"]["commit"]
    return expected


def _preflight(plan: dict, plan_sha: str, target: dict, target_index: int) -> str:
    if _configured_repo() != plan["repo"]:
        raise PlanError("plan repository does not match aw.toml")
    start = _start_commit(plan, plan_sha, target_index)
    if _head() != start:
        raise PlanError("base commit drift")
    dirty = _dirty()
    if dirty:
        raise PlanError(f"working tree is dirty before apply: {dirty[0]}")
    if not (ROOT / target["path"]).is_dir():
        raise PlanError(f"project does not exist: {target['path']}")
    _check_documents_before(target)
    _assert_tracker(plan["repo"], target, _baseline_snapshot(target), "before apply")
    _assert_existing_milestone_membership(plan["repo"], target)
    _preview_documents(target, start, plan["repo"])
    return start


def cmd_validate(args: argparse.Namespace) -> int:
    plan, _sha256 = read_plan(args.plan, stdin_ok=True, seal_missing=True)
    print(canonical_bytes(plan).decode(), end="")
    return 0


def _report_receipt_handoff(path: Path) -> None:
    """Print the one deterministic Resume handoff after a partial run."""
    try:
        durable = read_receipt(path)
        state = durable["state"]
    except PlanError:
        state = "UNREADABLE"
    try:
        shown = str(path.relative_to(ROOT))
    except ValueError:
        shown = str(path)
    print(json.dumps({
        "next_command": (
            "uv run --project apps/aw aw release-plan resume "
            f"--receipt {shown}"
        ),
        "receipt": shown,
        "state": state,
    }, ensure_ascii=False, sort_keys=True), file=sys.stderr)


def _execute_with_handoff(receipt: dict, path: Path) -> int:
    try:
        return _execute(receipt, path)
    except Exception:
        _report_receipt_handoff(path)
        raise


def cmd_apply(args: argparse.Namespace) -> int:
    plan, sha = read_plan(args.plan, stdin_ok=False)
    if sha != args.approved_digest:
        raise PlanError("approved digest does not match canonical plan")
    targets = [(index, item) for index, item in enumerate(plan["projects"])
               if item["path"] == args.project]
    if len(targets) != 1:
        raise PlanError("--project must select exactly one project in the plan")
    index, target = targets[0]
    path = receipt_path(sha, target["path"])
    with _release_plan_lock():
        if path.exists():
            raise PlanError(
                f"receipt already exists; use resume --receipt {path.relative_to(ROOT)}"
            )
        start = _preflight(plan, sha, target, index)
        receipt = _new_receipt(plan, sha, target, start)
        _write_receipt(path, receipt)
        return _execute_with_handoff(receipt, path)


def cmd_resume(args: argparse.Namespace) -> int:
    path = Path(args.receipt)
    if not path.is_absolute():
        path = ROOT / path
    path = path.resolve()
    with _release_plan_lock():
        receipt = read_receipt(path)
        expected_path = receipt_path(
            receipt["plan_sha256"], receipt["project"],
        ).resolve()
        if path != expected_path:
            raise PlanError(f"receipt path must be {expected_path.relative_to(ROOT)}")
        if receipt["state"] == "COMPLETE":
            _verify_complete_chain(receipt, path)
            print(json.dumps({
                "receipt": str(path.relative_to(ROOT)), "state": "COMPLETE",
                "project": receipt["project"], "evidence": receipt["evidence"],
            }, ensure_ascii=False, sort_keys=True))
            return 0
        return _execute_with_handoff(receipt, path)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="release-plan.py", description=__doc__)
    sub = parser.add_subparsers(dest="verb", required=True)
    validate = sub.add_parser("validate")
    validate.add_argument("--plan", required=True)
    validate.set_defaults(func=cmd_validate)
    apply = sub.add_parser("apply")
    apply.add_argument("--plan", required=True)
    apply.add_argument("--project", required=True)
    apply.add_argument("--approved-digest", required=True)
    apply.set_defaults(func=cmd_apply)
    resume = sub.add_parser("resume")
    resume.add_argument("--receipt", required=True)
    resume.set_defaults(func=cmd_resume)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        return args.func(args)
    except (PlanError, workitem.GhError) as error:
        print(f"refused: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
