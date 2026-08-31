#!/usr/bin/env python3
"""Milestone-native epic, release, and development-order facade.

A milestone is the one epic record for one project version. Its title is
`<project>@<major>.<minor>.<patch>`. Its description carries the observable
goal, a numbered list of assigned change issues, and the completion conditions.
GitHub's native issue milestone field is the only child-ownership relation.

Milestone references are explicit. Use `milestone:<number>` or the exact title.
A bare number is refused because issue and milestone numbers share no namespace.
"""

from __future__ import annotations

import argparse
import json
import re
import shlex
import sys
from dataclasses import dataclass
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import workitem  # noqa: E402
import wi_types  # noqa: E402


TITLE = re.compile(
    r"^(?P<project>[a-z0-9][a-z0-9-]*)@"
    r"(?P<major>0|[1-9][0-9]*)\."
    r"(?P<minor>0|[1-9][0-9]*)\."
    r"(?P<patch>0|[1-9][0-9]*)$"
)
MILESTONE_REF = re.compile(r"^milestone:(?P<number>[1-9][0-9]*)$")
ORDER_ROW = re.compile(
    r"^[ \t]*(?P<rank>[1-9][0-9]*)\.[ \t]+#(?P<iid>[1-9][0-9]*)[ \t]*$"
)
H2 = re.compile(r"^##[ \t]+(?P<name>.+?)[ \t]*$", re.M)
SECTIONS = ("Goal", "Development Order", "Acceptance")
DRAFT_LINE = "Pending: create and assign change issues, then replace this line with a numbered list."
SKELETON = f"""## Goal

<!-- State one observable outcome for this project version. -->

## Development Order

{DRAFT_LINE}

## Acceptance

<!-- List the exact evidence required before this version can close. -->
-
"""


@dataclass(frozen=True)
class ReleaseIdentity:
    project: str
    major: int
    minor: int
    patch: int

    @property
    def version(self) -> tuple[int, int, int]:
        return self.major, self.minor, self.patch

    @property
    def title(self) -> str:
        return f"{self.project}@{self.major}.{self.minor}.{self.patch}"


def release_identity(title: str) -> ReleaseIdentity | None:
    found = TITLE.fullmatch(title.strip())
    if not found:
        return None
    identity = ReleaseIdentity(
        found.group("project"),
        int(found.group("major")),
        int(found.group("minor")),
        int(found.group("patch")),
    )
    if identity.minor > 63 or identity.patch > 63:
        return None
    return identity


def split_description(text: str) -> tuple[dict[str, str], list[str]]:
    matches = list(H2.finditer(text))
    headings = [m.group("name").strip() for m in matches]
    errors: list[str] = []
    if headings != list(SECTIONS):
        errors.append(
            "description H2 headings must be exactly `## Goal`, `## Development Order`, "
            "and `## Acceptance`, in that order"
        )
    sections: dict[str, str] = {}
    for index, match in enumerate(matches):
        end = matches[index + 1].start() if index + 1 < len(matches) else len(text)
        sections[match.group("name").strip()] = text[match.end():end].strip()
    return sections, errors


def development_order(text: str) -> tuple[list[str], list[str]]:
    rows: list[tuple[int, str]] = []
    errors: list[str] = []
    for number, line in enumerate(text.splitlines(), 1):
        if not line.strip():
            continue
        found = ORDER_ROW.fullmatch(line)
        if found:
            rows.append((int(found.group("rank")), found.group("iid")))
        else:
            errors.append(
                "`## Development Order` line "
                f"{number} must be exactly `<rank>. #<iid>`"
            )
    ranks = [rank for rank, _ in rows]
    if ranks and ranks != list(range(1, len(rows) + 1)):
        errors.append("`## Development Order` ranks must start at 1 and be contiguous")
    numbers = [number for _, number in rows]
    duplicates = sorted({number for number in numbers if numbers.count(number) > 1}, key=int)
    if duplicates:
        errors.append(
            "`## Development Order` lists duplicate issue(s): "
            + ", ".join("#" + number for number in duplicates)
        )
    return numbers, errors


def validate_description(text: str, *, allow_draft: bool = False) -> list[str]:
    sections, errors = split_description(text)
    goal = sections.get("Goal", "")
    visible_goal = re.sub(r"\s+", "", goal)
    if not goal or "<!--" in goal or len(visible_goal) < 12:
        errors.append("`## Goal` must state a substantive observable outcome")

    order_text = sections.get("Development Order", "")
    draft = order_text.strip() == DRAFT_LINE
    order, order_errors = ([], []) if draft else development_order(order_text)
    errors.extend(order_errors)
    if not order and not (allow_draft and draft):
        errors.append(
            "`## Development Order` must list every assigned issue as `1. #<iid>`; "
            "draft text is accepted only with `--draft`"
        )

    acceptance = sections.get("Acceptance", "")
    if not re.search(r"^[ \t]*[-*][ \t]+\S", acceptance, re.M) or "<!--" in acceptance:
        errors.append("`## Acceptance` must contain at least one concrete list item")
    return errors


def is_draft_description(text: str) -> bool:
    sections, errors = split_description(text)
    return not errors and sections.get("Development Order", "").strip() == DRAFT_LINE


def _pages(raw: str) -> list[dict]:
    loaded = json.loads(raw)
    if loaded and isinstance(loaded[0], list):
        return [row for page in loaded for row in page]
    return loaded


def list_milestones(repo: str, state: str = "all") -> list[dict]:
    raw = workitem.gh(
        "api",
        "--paginate",
        "--slurp",
        f"repos/{repo}/milestones?state={state}&per_page=100",
    )
    return _pages(raw)


def duplicate_title(title: str, repo: str, *, excluding: int | None = None) -> dict | None:
    for milestone in list_milestones(repo, "all"):
        if milestone.get("title") == title and milestone.get("number") != excluding:
            return milestone
    return None


def resolve_milestone(ref: str, repo: str) -> dict:
    ref = ref.strip()
    match = MILESTONE_REF.fullmatch(ref)
    if match:
        return json.loads(workitem.gh("api", f"repos/{repo}/milestones/{match.group('number')}"))
    if ref.isdigit():
        raise workitem.GhError(
            f"bare `{ref}` is ambiguous; use `milestone:{ref}` or an exact `<project>@<version>` title"
        )
    matches = [milestone for milestone in list_milestones(repo) if milestone["title"] == ref]
    if len(matches) != 1:
        raise workitem.GhError(
            f"expected exactly one milestone titled `{ref}` in {repo}; found {len(matches)}"
        )
    return matches[0]


def milestone_issues(milestone: dict, repo: str) -> list[dict]:
    raw = workitem.gh(
        "api",
        "--paginate",
        "--slurp",
        f"repos/{repo}/issues?milestone={milestone['number']}&state=all&per_page=100",
    )
    issues = []
    for issue in _pages(raw):
        if "pull_request" in issue:
            continue
        issues.append({
            "number": issue["number"],
            "title": issue["title"],
            "state": issue["state"].upper(),
            "labels": [label["name"] for label in issue.get("labels", [])],
            "url": issue["html_url"],
        })
    return issues


def order_payload(milestone: dict, issues: list[dict], *, open_only: bool = False) -> dict:
    description = milestone.get("description") or ""
    errors = validate_description(description)
    sections, _ = split_description(description)
    declared, order_errors = development_order(sections.get("Development Order", ""))
    errors.extend(error for error in order_errors if error not in errors)

    identity = release_identity(milestone.get("title") or "")
    if identity is None:
        errors.append("milestone title must be `<project>@<major>.<minor>.<patch>` with minor and patch 0..63")
        expected_label = None
    else:
        expected_label = workitem.project_label(identity.project)

    by_number = {str(issue["number"]): issue for issue in issues}
    declared_set = set(declared)
    assigned_set = set(by_number)
    missing = sorted(assigned_set - declared_set, key=int)
    foreign = sorted(declared_set - assigned_set, key=int)
    if missing:
        errors.append("assigned issue(s) missing from development order: " + ", ".join("#" + n for n in missing))
    if foreign:
        errors.append("development order names issue(s) not assigned to this milestone: " + ", ".join("#" + n for n in foreign))

    for issue in issues:
        labels = issue.get("labels", [])
        try:
            wi_types.delivery_type(labels, subject=f"#{issue['number']}")
        except wi_types.TypeError as exc:
            errors.append(str(exc))
        project_labels = sorted(
            label for label in labels if label.startswith(("app:", "lib:"))
        )
        if expected_label and project_labels != [expected_label]:
            rendered = ", ".join(project_labels) or "<none>"
            errors.append(
                f"#{issue['number']} needs exactly project label `{expected_label}`; "
                f"found {rendered}"
            )

    rows = []
    for number in declared:
        issue = by_number.get(number)
        if issue is None:
            continue
        if open_only and issue["state"] == "CLOSED":
            continue
        rows.append({
            "number": issue["number"],
            "title": issue["title"],
            "state": issue["state"],
            "url": issue["url"],
        })
    return {
        "milestone": milestone["number"],
        "title": milestone["title"],
        "state": milestone["state"].upper(),
        "orderable": not errors,
        "errors": errors,
        "order": rows,
    }


def _api(argv: list[str], dry_run: bool) -> str:
    if dry_run:
        print("[dry-run] " + shlex.join(["gh", *argv]))
        return ""
    return workitem.gh(*argv)


def _description(path: str) -> str:
    return Path(path).read_text(encoding="utf-8")


def cmd_skeleton(_args) -> int:
    print(SKELETON, end="")
    return 0


def cmd_validate(args) -> int:
    if args.description_file:
        description = _description(args.description_file)
        title = args.title
        subject = args.description_file
    else:
        milestone = resolve_milestone(args.ref, args.repo)
        description = milestone.get("description") or ""
        title = milestone["title"]
        subject = f"milestone:{milestone['number']}"
    errors = validate_description(description, allow_draft=args.draft)
    if release_identity(title) is None:
        errors.insert(0, "title must be `<project>@<major>.<minor>.<patch>` with minor and patch 0..63")
    if args.json:
        print(json.dumps({"subject": subject, "valid": not errors, "errors": errors}, indent=2))
    elif errors:
        print(f"{subject}: INVALID ({len(errors)} error(s))")
        for error in errors:
            print(f"  - {error}")
    else:
        print(f"{subject}: valid")
    return 1 if errors else 0


def cmd_show(args) -> int:
    milestone = resolve_milestone(args.ref, args.repo)
    issues = milestone_issues(milestone, args.repo)
    payload = dict(milestone)
    payload["issues"] = issues
    if args.json:
        print(json.dumps(payload, indent=2))
    else:
        print(f"milestone:{milestone['number']} [{milestone['state'].upper()}] {milestone['title']}")
        print(f"  issues: {len(issues)} ({sum(i['state'] == 'OPEN' for i in issues)} open)")
        print()
        print(milestone.get("description") or "")
    return 0


def cmd_children(args) -> int:
    milestone = resolve_milestone(args.ref, args.repo)
    issues = milestone_issues(milestone, args.repo)
    if args.json:
        print(json.dumps(issues, indent=2))
    else:
        print(f"milestone:{milestone['number']} {milestone['title']}: {len(issues)} issue(s)")
        for issue in issues:
            print(f"  {'*' if issue['state'] == 'OPEN' else ' '} #{issue['number']} {issue['state']:6} {issue['title']}")
    return 0


def cmd_order(args) -> int:
    milestone = resolve_milestone(args.ref, args.repo)
    payload = order_payload(milestone, milestone_issues(milestone, args.repo), open_only=args.open_only)
    if args.json:
        print(json.dumps(payload, indent=2))
    else:
        print(f"milestone:{payload['milestone']} {payload['title']}")
        for index, issue in enumerate(payload["order"], 1):
            print(f"  {index}. #{issue['number']} [{issue['state']}] {issue['title']}")
        for error in payload["errors"]:
            print(f"  ! {error}")
    return 0 if payload["orderable"] else 1


def cmd_reconcile(args) -> int:
    return cmd_order(argparse.Namespace(
        ref=args.ref, repo=args.repo, open_only=False, json=args.json
    ))


def cmd_next(args) -> int:
    """Print the only open queue head and its next required delivery action."""
    target = resolve_milestone(args.ref, args.repo)
    issues = milestone_issues(target, args.repo)
    payload = order_payload(target, issues)
    if payload["errors"]:
        for error in payload["errors"]:
            print(f"  ! {error}")
        return 1
    for rank, row in enumerate(payload["order"], 1):
        if row["state"] != "OPEN":
            continue
        issue = workitem.fetch_issue(str(row["number"]), args.repo)
        try:
            kind = wi_types.delivery_type(issue.get("labels", []), subject=f"#{issue['number']}")
        except wi_types.TypeError as exc:
            print(f"  ! {exc}")
            return 1
        flow = wi_types.flow_for(kind)
        rows = workitem.lifecycle_rows(issue.get("body") or "")
        required = wi_types.required_legs(kind)
        next_phase = next((leg for leg in required if leg not in rows), "close")
        answer = {
            "milestone": target["number"],
            "rank": rank,
            "iid": issue["number"],
            "type": kind,
            "flow": flow,
            "next_phase": next_phase,
        }
        if args.json:
            print(json.dumps(answer, indent=2))
        else:
            print(f"milestone:{target['number']} queue head {rank}. #{issue['number']}")
            print(f"  type: {kind}")
            print(f"  flow: {flow}")
            print(f"  next.phase: {next_phase}")
        return 0
    answer = {"milestone": target["number"], "queue": "empty", "next_phase": "close"}
    print(json.dumps(answer, indent=2) if args.json else f"milestone:{target['number']} queue empty; next.phase: close")
    return 0


def cmd_versions(args) -> int:
    rows = []
    for milestone in list_milestones(args.repo, args.state):
        identity = release_identity(milestone["title"])
        if identity and (not args.project or identity.project == args.project):
            rows.append((identity.project, identity.version, milestone))
    rows.sort(key=lambda row: (row[0], row[1]))
    payload = [row[2] for row in rows]
    if args.json:
        print(json.dumps(payload, indent=2))
    else:
        for _, _, milestone in rows:
            print(f"milestone:{milestone['number']} {milestone['state'].upper():6} {milestone['title']}")
    return 0


def cmd_create(args) -> int:
    description = _description(args.description_file)
    errors = validate_description(description, allow_draft=args.draft)
    if release_identity(args.title) is None:
        errors.insert(0, "title must be `<project>@<major>.<minor>.<patch>` with minor and patch 0..63")
    if not args.draft:
        errors.append("Milestone creation must use `--draft`; finalize order after assigning changes")
    elif not is_draft_description(description):
        errors.append("`--draft` requires the skeleton's exact Development Order draft line")
    if errors:
        for error in errors:
            print(f"  - {error}")
        return 1
    duplicate = duplicate_title(args.title, args.repo)
    if duplicate:
        raise workitem.GhError(
            f"release identity `{args.title}` already exists as milestone:{duplicate['number']}"
        )
    argv = ["api", "--method", "POST", f"repos/{args.repo}/milestones",
            "-f", f"title={args.title}", "-f", "state=open", "-f", f"description={description}"]
    if args.due_on:
        argv += ["-f", f"due_on={args.due_on}"]
    out = _api(argv, args.dry_run)
    if out:
        created = json.loads(out)
        print(f"created milestone:{created['number']} {created['title']}")
    return 0


def cmd_update(args) -> int:
    milestone = resolve_milestone(args.ref, args.repo)
    issues = milestone_issues(milestone, args.repo)
    title = args.title or milestone["title"]
    description = _description(args.description_file) if args.description_file else (milestone.get("description") or "")
    errors = validate_description(description, allow_draft=args.draft)
    if release_identity(title) is None:
        errors.insert(0, "title must be `<project>@<major>.<minor>.<patch>` with minor and patch 0..63")
    if args.draft:
        if issues:
            errors.append("`--draft` is refused after any issue is assigned")
        if not is_draft_description(description):
            errors.append("`--draft` requires the skeleton's exact Development Order draft line")
    else:
        candidate = dict(milestone, title=title, description=description)
        payload = order_payload(candidate, issues)
        errors.extend(error for error in payload["errors"] if error not in errors)
    if args.title:
        duplicate = duplicate_title(title, args.repo, excluding=milestone["number"])
        if duplicate:
            errors.append(
                f"release identity `{title}` already exists as milestone:{duplicate['number']}"
            )
    if errors:
        for error in errors:
            print(f"  - {error}")
        return 1
    fields = []
    if args.title:
        fields += ["-f", f"title={args.title}"]
    if args.description_file:
        fields += ["-f", f"description={description}"]
    if args.due_on:
        fields += ["-f", f"due_on={args.due_on}"]
    if args.clear_due_on:
        fields += ["-f", "due_on="]
    if not fields:
        raise workitem.GhError("update needs --title, --description-file, --due-on, or --clear-due-on")
    _api(["api", "--method", "PATCH", f"repos/{args.repo}/milestones/{milestone['number']}", *fields], args.dry_run)
    if not args.dry_run:
        print(f"updated milestone:{milestone['number']}")
    return 0


def cmd_close(args) -> int:
    milestone = resolve_milestone(args.ref, args.repo)
    issues = milestone_issues(milestone, args.repo)
    payload = order_payload(milestone, issues)
    open_issues = [issue for issue in issues if issue["state"] == "OPEN"]
    if payload["errors"] or open_issues:
        for error in payload["errors"]:
            print(f"  - {error}", file=sys.stderr)
        if open_issues:
            print("  - open assigned issue(s): " + ", ".join(f"#{i['number']}" for i in open_issues), file=sys.stderr)
        return 1
    _api(["api", "--method", "PATCH", f"repos/{args.repo}/milestones/{milestone['number']}",
          "-f", "state=closed"], args.dry_run)
    if not args.dry_run:
        after = resolve_milestone(f"milestone:{milestone['number']}", args.repo)
        if after.get("state", "").lower() != "closed":
            raise workitem.GhError(f"close readback expected milestone:{milestone['number']} CLOSED")
        print(f"closed milestone:{milestone['number']} {milestone['title']}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="milestone.py", description=__doc__)
    parser.add_argument("--repo", help="owner/name; defaults to aw.toml's issue platform")
    sub = parser.add_subparsers(dest="verb", required=True)

    p = sub.add_parser("skeleton")
    p.set_defaults(func=cmd_skeleton)

    p = sub.add_parser("validate")
    p.add_argument("ref", nargs="?")
    p.add_argument("--description-file")
    p.add_argument("--title")
    p.add_argument("--draft", action="store_true")
    p.add_argument("--json", action="store_true")
    p.set_defaults(func=cmd_validate)

    for verb, func in (("show", cmd_show), ("children", cmd_children), ("reconcile", cmd_reconcile)):
        p = sub.add_parser(verb)
        p.add_argument("ref")
        p.add_argument("--json", action="store_true")
        p.set_defaults(func=func)

    p = sub.add_parser("order")
    p.add_argument("ref")
    p.add_argument("--open-only", action="store_true")
    p.add_argument("--json", action="store_true")
    p.set_defaults(func=cmd_order)

    p = sub.add_parser("next")
    p.add_argument("ref")
    p.add_argument("--json", action="store_true")
    p.set_defaults(func=cmd_next)

    p = sub.add_parser("versions")
    p.add_argument("--project")
    p.add_argument("--state", choices=("open", "closed", "all"), default="open")
    p.add_argument("--json", action="store_true")
    p.set_defaults(func=cmd_versions)

    p = sub.add_parser("create")
    p.add_argument("--title", required=True)
    p.add_argument("--description-file", required=True)
    p.add_argument("--due-on")
    p.add_argument("--draft", action="store_true")
    p.add_argument("--dry-run", action="store_true")
    p.set_defaults(func=cmd_create)

    p = sub.add_parser("update")
    p.add_argument("ref")
    p.add_argument("--title")
    p.add_argument("--description-file")
    p.add_argument("--due-on")
    p.add_argument("--clear-due-on", action="store_true")
    p.add_argument("--draft", action="store_true")
    p.add_argument("--dry-run", action="store_true")
    p.set_defaults(func=cmd_update)

    p = sub.add_parser("close")
    p.add_argument("ref")
    p.add_argument("--dry-run", action="store_true")
    p.set_defaults(func=cmd_close)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        local_file_validation = args.verb == "validate" and args.description_file
        if args.verb != "skeleton" and not local_file_validation and not args.repo:
            args.repo = workitem.default_repo()
        if args.verb == "validate":
            if bool(args.ref) == bool(args.description_file):
                raise workitem.GhError("validate needs exactly one of REF or --description-file")
            if args.description_file and not args.title:
                raise workitem.GhError("file validation also needs --title")
        return args.func(args)
    except (workitem.GhError, json.JSONDecodeError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
