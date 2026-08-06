"""Black-box contract for legacy WI-type decoding baseline (#3303)."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, project_fixture, run_aw, show

CASE_ID = "work-item-planning-legacy-wi-type-decoding-baseline"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "legacy-wi-type-decoding-baseline"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-planning-legacy-wi-type-decoding-baseline"
)
ASSERTIONS = (
    "each of the four legacy bug/enhancement/refactor/test frontmatter-and-label values, injected "
    "the way an older tracker's on-disk record would carry them, decodes through the live wi show "
    "CLI as canonical type=change with its own type:<legacy> label preserved verbatim, is bucketed "
    "by wi list --type change and excluded from wi list --type epic, and leaves the on-disk tracker "
    "record byte-for-byte unmodified after every read -- proving the alias is decode-only and never "
    "a tracker mutation",
    "a genuinely unrecognized type value is rejected outright by wi show with an explicit "
    "unsupported-issue-type parse error, and a real canonically-created epic remains type=epic "
    "rather than being swept into change -- proving the alias table is the exact closed "
    "bug/enhancement/refactor/test set rather than a blanket string-to-change coercion, so the "
    "passing decode assertions above are not vacuous",
    "the shipped wi list --help text itself documents that the canonical `change` filter value also "
    "matches legacy non-epic labels, tying the observed decode-only-alias behavior to the CLI's own "
    "published contract rather than to undocumented internals",
)

_LEGACY_TYPES = ("bug", "enhancement", "refactor", "test")

_EPIC_BODY = (
    "## Requirements\n\n- R1: Demonstrate canonical epic type retention.\n\n"
    "## Verification Inventory\n\n"
    "| Requirement | Gate | Oracle | Depends On |\n"
    "|-------------|------|--------|------------|\n"
    "| R1 | `aw wi show <slug>` | show reports type=epic. | - |\n"
)


def _workspace_slug(root: Path) -> str:
    resolved = str(root.resolve())
    collapsed = re.sub(r"[^a-zA-Z0-9]+", "-", resolved)
    return collapsed.strip("-").lower()


def _issue_path(root: Path, slug: str) -> Path:
    return Path("/tmp/aw/workspaces") / _workspace_slug(root) / "issues" / "open" / f"{slug}.md"


def _write_legacy_issue(root: Path, legacy: str) -> tuple[str, Path, str]:
    slug = f"legacy-{legacy}-issue"
    path = _issue_path(root, slug)
    path.parent.mkdir(parents=True, exist_ok=True)
    raw = (
        f"---\ntype: {legacy}\ntitle: Legacy {legacy} issue\nstate: open\n"
        f"labels:\n  - type:{legacy}\n  - app:demo\n---\n\nlegacy body {legacy}\n"
    )
    path.write_text(raw, encoding="utf-8")
    return slug, path, raw


def verify() -> list[str]:
    with project_fixture() as root:
        # Cluster 1: decode-only alias, per legacy type, with a byte-for-byte
        # no-mutation check and list-bucket differentiation.
        for legacy in _LEGACY_TYPES:
            slug, path, raw_before = _write_legacy_issue(root, legacy)

            shown = show(root, slug)
            assert shown["type"] == "change", shown
            assert shown["labels"] == [f"type:{legacy}", "app:demo"], shown

            raw_after = path.read_text(encoding="utf-8")
            assert raw_after == raw_before, (raw_before, raw_after)

        listed_change = json.loads(
            run_aw(root, "wi", "list", "--project", "demo", "--type", "change", "--json").stdout
        )
        change_titles = {entry["title"] for entry in listed_change}
        for legacy in _LEGACY_TYPES:
            title = f"Legacy {legacy} issue"
            assert title in change_titles, change_titles
            (entry,) = [e for e in listed_change if e["title"] == title]
            assert entry["type"] == "change", entry

        listed_epic = json.loads(
            run_aw(root, "wi", "list", "--project", "demo", "--type", "epic", "--json").stdout
        )
        epic_titles = {entry["title"] for entry in listed_epic}
        for legacy in _LEGACY_TYPES:
            assert f"Legacy {legacy} issue" not in epic_titles, epic_titles

        # Cluster 2: a genuinely unrecognized type is rejected outright, and a
        # canonically created epic is never swept into change -- proving the
        # alias table is the exact closed legacy set, not a blanket coercion.
        bogus_slug = "bogus-type-issue"
        bogus_path = _issue_path(root, bogus_slug)
        bogus_path.write_text(
            "---\ntype: not_a_real_type\ntitle: Bogus type issue\nstate: open\n---\n\nbogus body\n",
            encoding="utf-8",
        )
        rejected = run_aw(root, "wi", "show", bogus_slug, expect_success=False)
        assert "unsupported issue type" in rejected.stderr, rejected.stderr

        real_epic = create(root, "Real canonical epic", "epic", "--body", _EPIC_BODY)
        real_epic_shown = show(root, real_epic["slug"])
        assert real_epic_shown["type"] == "epic", real_epic_shown

        # Cluster 3: the documented CLI surface names the alias behavior.
        help_text = run_aw(root, "wi", "list", "--help").stdout
        assert "legacy non-epic labels" in help_text, help_text

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
