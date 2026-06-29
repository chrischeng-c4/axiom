#!/usr/bin/env python3.12
"""CPython replacement promotion gate.

This is a readiness gate, not a fixture runner. It scans the CPython fixture
tree for xfail, skip, optional, and promotion-pending debt that must be gone
before mamba can claim a replacement profile. Red output is intentional
evidence: this gate should fail until every debt item is fixed, promoted, or
tracked with an owner issue and acceptance path.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import tomllib
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


TOOLS_DIR = Path(__file__).resolve().parent
MAMBA_DIR = TOOLS_DIR.parents[3]
DEFAULT_ROOT = TOOLS_DIR.parents[2] / "cpython"
DEFAULT_MANIFEST = MAMBA_DIR / "ecosystem_fixture_manifest.toml"

EXIT_DEBT = 70
EXIT_PARSE_ERROR = 71

ISSUE_REF_RE = re.compile(
    r"(?:\bWI\s*)?#\d+\b|\b(?:issue|tracker|GH)[-: ]#?\d+\b",
    re.IGNORECASE,
)
PROMOTION_PENDING_RE = re.compile(
    r"\bpromotion pending\b|auto-(?:ported|extracted)\s+CPython\s+test|CPython 3\.12 seed",
    re.IGNORECASE,
)
SKIP_PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    ("mamba-skip-directive", re.compile(r"^\s*#\s*mamba-skip\s*:")),
    ("pytest-skip-marker", re.compile(r"^\s*@pytest\.mark\.skip(?:if)?\b")),
    ("pytest-skip-call", re.compile(r"\bpytest\.skip\s*\(")),
    ("unittest-skip-marker", re.compile(r"^\s*@(?:unittest\.)?skip(?:If|Unless)?\b")),
    ("unittest-skip-call", re.compile(r"\b(?:self\.)?skipTest\s*\(")),
    ("unittest-skip-exception", re.compile(r"\bSkipTest\b")),
)


@dataclass(frozen=True)
class FixtureAxes:
    dimension: str
    bucket: str
    lib: str
    case: str
    subject: str = ""
    source: str = ""
    status: str = ""

    def to_json(self) -> dict[str, str]:
        return {
            "dimension": self.dimension,
            "bucket": self.bucket,
            "lib": self.lib,
            "case": self.case,
            "subject": self.subject,
            "source": self.source,
            "status": self.status,
        }


@dataclass
class ManifestOutcome:
    fixture_id: str
    expected_outcome: str
    blocker: str


@dataclass
class Debt:
    kind: str
    path: str
    reason: str
    axes: FixtureAxes
    sources: list[str] = field(default_factory=list)
    owner_refs: list[str] = field(default_factory=list)
    promotion_pending: bool = False

    @property
    def owned(self) -> bool:
        return bool(self.owner_refs)

    def merge(self, reason: str, source: str) -> None:
        if reason and reason not in self.reason:
            self.reason = f"{self.reason}; {reason}" if self.reason else reason
        self.sources = sorted(set([*self.sources, source]))
        self.owner_refs = sorted(set([*self.owner_refs, *issue_refs(reason)]))
        self.promotion_pending = self.promotion_pending or is_promotion_pending(reason)

    def to_json(self) -> dict[str, Any]:
        return {
            "kind": self.kind,
            "path": self.path,
            "reason": self.reason,
            "axis": self.axes.to_json(),
            "sources": self.sources,
            "owner_refs": self.owner_refs,
            "owned": self.owned,
            "promotion_pending": self.promotion_pending,
        }


@dataclass
class ScanResult:
    scanned_files: int = 0
    debts: dict[tuple[str, str], Debt] = field(default_factory=dict)
    parse_errors: list[dict[str, str]] = field(default_factory=list)

    def add_debt(
        self,
        *,
        kind: str,
        path: str,
        reason: str,
        axes: FixtureAxes,
        source: str,
    ) -> None:
        key = (kind, path)
        existing = self.debts.get(key)
        if existing is not None:
            existing.merge(reason, source)
            return
        self.debts[key] = Debt(
            kind=kind,
            path=path,
            reason=reason,
            axes=axes,
            sources=[source],
            owner_refs=issue_refs(reason),
            promotion_pending=is_promotion_pending(reason),
        )

    def debt_items(self, kind: str | None = None) -> list[Debt]:
        items = sorted(self.debts.values(), key=lambda item: (item.kind, item.path))
        if kind is None:
            return items
        return [item for item in items if item.kind == kind]

    def axis_breakdown(self) -> dict[str, dict[str, dict[str, int]]]:
        out: dict[str, dict[str, dict[str, int]]] = {
            "dimension": {},
            "bucket": {},
            "lib": {},
            "dimension_bucket_lib": {},
        }

        def bump(section: str, key: str, kind: str) -> None:
            bucket = out[section].setdefault(key, {"total": 0})
            bucket["total"] += 1
            bucket[kind] = bucket.get(kind, 0) + 1

        for item in self.debt_items():
            bump("dimension", item.axes.dimension, item.kind)
            bump("bucket", item.axes.bucket, item.kind)
            bump("lib", f"{item.axes.bucket}/{item.axes.lib}", item.kind)
            bump(
                "dimension_bucket_lib",
                f"{item.axes.dimension}/{item.axes.bucket}/{item.axes.lib}",
                item.kind,
            )
        return {section: dict(sorted(values.items())) for section, values in out.items()}

    def to_json(
        self,
        profile: str,
        failed: bool,
        detail_limit: int,
        include_all_details: bool,
    ) -> dict[str, Any]:
        def details(items: list[Debt]) -> list[dict[str, Any]]:
            selected = items if include_all_details else items[:detail_limit]
            return [item.to_json() for item in selected]

        def maybe_sample(items: list[Debt]) -> list[dict[str, Any]]:
            return details(items)

        all_debt = self.debt_items()
        unowned = [item for item in all_debt if not item.owned]
        promotion_pending = [item for item in all_debt if item.promotion_pending]
        parse_errors = (
            self.parse_errors
            if include_all_details
            else self.parse_errors[:detail_limit]
        )
        xfail_debt = self.debt_items("xfail")
        skip_debt = self.debt_items("skip")
        optional_debt = self.debt_items("optional")
        return {
            "profile": profile,
            "failed": failed,
            "scanned_files": self.scanned_files,
            "xfail_count": len(xfail_debt),
            "skip_count": len(skip_debt),
            "optional_count": len(optional_debt),
            "promotion_debt_total": len(all_debt),
            "parse_error_count": len(self.parse_errors),
            "owned_count": len(all_debt) - len(unowned),
            "unowned_count": len(unowned),
            "promotion_pending_count": len(promotion_pending),
            "detail_limit": detail_limit,
            "details_truncated": not include_all_details
            and (
                len(all_debt) > detail_limit
                or len(self.parse_errors) > detail_limit
            ),
            "debt_by_axis": self.axis_breakdown(),
            "xfail_debt": maybe_sample(xfail_debt),
            "skip_debt": maybe_sample(skip_debt),
            "optional_debt": maybe_sample(optional_debt),
            "unowned_debt": maybe_sample(unowned),
            "promotion_pending_debt": maybe_sample(promotion_pending),
            "parse_errors": parse_errors,
        }


def issue_refs(text: str) -> list[str]:
    return sorted(set(match.group(0).strip() for match in ISSUE_REF_RE.finditer(text)))


def is_promotion_pending(text: str) -> bool:
    return bool(PROMOTION_PENDING_RE.search(text))


def rel_path(path: Path, root: Path) -> str:
    try:
        rel = path.relative_to(root)
    except ValueError:
        rel = path
    return rel.as_posix()


def py_files(root: Path) -> list[Path]:
    out: list[Path] = []
    for path in sorted(root.rglob("*.py")):
        parts = set(path.relative_to(root).parts)
        if ".cache" in parts or "_invalid" in parts:
            continue
        if path.name.endswith("_stub.py"):
            continue
        out.append(path)
    return out


def extract_script_toml(text: str) -> str | None:
    lines: list[str] = []
    in_block = False
    for raw in text.splitlines():
        stripped = raw.strip()
        if stripped.startswith("#") and "/// script" in stripped:
            in_block = True
            continue
        if in_block and stripped.startswith("#") and stripped.endswith("///"):
            return "\n".join(lines)
        if in_block:
            if raw.startswith("# "):
                lines.append(raw[2:])
            elif raw == "#":
                lines.append("")
            else:
                return None
    return None


def tool_mamba(text: str) -> tuple[dict[str, Any] | None, str | None]:
    if "[tool.mamba]" not in text:
        return None, None
    block = extract_script_toml(text)
    if block is None:
        return None, "cannot extract PEP 723 script block"
    try:
        parsed = tomllib.loads(block)
    except tomllib.TOMLDecodeError as exc:
        return None, str(exc)
    try:
        meta = parsed["tool"]["mamba"]
    except KeyError:
        return None, "missing [tool.mamba]"
    if not isinstance(meta, dict):
        return None, "[tool.mamba] is not a table"
    return meta, None


def nonempty_str(value: Any) -> str | None:
    if isinstance(value, str) and value.strip():
        return value.strip()
    return None


def meta_str(meta: dict[str, Any] | None, key: str) -> str:
    if meta is None:
        return ""
    value = meta.get(key)
    return value if isinstance(value, str) else ""


def axes_for(path: Path, root: Path, meta: dict[str, Any] | None) -> FixtureAxes:
    rel = path.relative_to(root)
    parts = rel.parts
    if meta is not None:
        return FixtureAxes(
            dimension=meta_str(meta, "dimension") or path_axis(parts, 0),
            bucket=meta_str(meta, "bucket") or path_axis(parts, 1),
            lib=meta_str(meta, "lib") or path_axis(parts, 2),
            case=meta_str(meta, "case") or path.stem,
            subject=meta_str(meta, "subject"),
            source=meta_str(meta, "source"),
            status=meta_str(meta, "status"),
        )
    if parts and parts[0] == "_regression":
        return FixtureAxes(
            dimension="_regression",
            bucket=path_axis(parts, 1),
            lib=path_axis(parts, 2),
            case=path.stem,
        )
    return FixtureAxes(
        dimension=path_axis(parts, 0),
        bucket=path_axis(parts, 1),
        lib=path_axis(parts, 2),
        case=path.stem,
    )


def path_axis(parts: tuple[str, ...], index: int) -> str:
    return parts[index] if len(parts) > index else "<unknown>"


def legacy_xfail_reason(text: str) -> str | None:
    for raw in text.splitlines():
        stripped = raw.strip()
        if stripped.startswith("# mamba-xfail:"):
            return stripped.removeprefix("# mamba-xfail:").strip() or "mamba-xfail"
    return None


def skip_reasons(text: str) -> list[str]:
    reasons: list[str] = []
    for raw in text.splitlines():
        for name, pattern in SKIP_PATTERNS:
            if pattern.search(raw):
                reasons.append(name)
    return sorted(set(reasons))


def load_manifest(path: Path | None) -> dict[str, ManifestOutcome]:
    if path is None or not path.exists():
        return {}
    parsed = tomllib.loads(path.read_text(encoding="utf-8"))
    raw_fixtures = parsed.get("fixtures", {})
    if not isinstance(raw_fixtures, dict):
        return {}
    out: dict[str, ManifestOutcome] = {}
    for fixture_id, entry in raw_fixtures.items():
        if not isinstance(entry, dict):
            continue
        relpath = entry.get("relpath")
        expected_outcome = entry.get("expected_outcome", "pass")
        blocker = entry.get("blocker", "")
        if not isinstance(relpath, str) or not isinstance(expected_outcome, str):
            continue
        out[relpath] = ManifestOutcome(
            fixture_id=str(fixture_id),
            expected_outcome=expected_outcome,
            blocker=blocker if isinstance(blocker, str) else "",
        )
    return out


def is_real_world_optional(rel: str, manifest: dict[str, ManifestOutcome]) -> bool:
    parts = Path(rel).parts
    if "real_world" not in parts:
        return False
    return rel not in manifest


def scan(root: Path, manifest_path: Path | None) -> ScanResult:
    result = ScanResult()
    root = root.resolve()
    manifest = load_manifest(manifest_path)
    for path in py_files(root):
        result.scanned_files += 1
        text = path.read_text(encoding="utf-8", errors="replace")
        rel = rel_path(path, root)
        meta, parse_error = tool_mamba(text)
        axes = axes_for(path, root, meta)
        if parse_error is not None:
            result.parse_errors.append({"path": rel, "error": parse_error})
            continue

        if meta is not None:
            if reason := nonempty_str(meta.get("xfail")):
                result.add_debt(
                    kind="xfail",
                    path=rel,
                    reason=reason,
                    axes=axes,
                    source="tool.mamba.xfail",
                )
            if reason := nonempty_str(meta.get("skip")):
                result.add_debt(
                    kind="skip",
                    path=rel,
                    reason=reason,
                    axes=axes,
                    source="tool.mamba.skip",
                )
        if reason := legacy_xfail_reason(text):
            result.add_debt(
                kind="xfail",
                path=rel,
                reason=reason,
                axes=axes,
                source="mamba-xfail-directive",
            )

        if rel in manifest and manifest[rel].expected_outcome != "pass":
            outcome = manifest[rel]
            result.add_debt(
                kind="skip" if outcome.expected_outcome == "skip" else "xfail",
                path=rel,
                reason=outcome.blocker
                or f"ecosystem fixture expected_outcome={outcome.expected_outcome}",
                axes=axes,
                source=f"ecosystem_manifest.{outcome.fixture_id}.{outcome.expected_outcome}",
            )
        elif is_real_world_optional(rel, manifest):
            result.add_debt(
                kind="optional",
                path=rel,
                reason=(
                    "real-world fixture is outside ecosystem_fixture_manifest; "
                    "development runner treats failures as optional"
                ),
                axes=axes,
                source="ecosystem_manifest.optional_absent",
            )

        reasons = skip_reasons(text)
        if reasons:
            result.add_debt(
                kind="skip",
                path=rel,
                reason=", ".join(reasons),
                axes=axes,
                source="runtime-skip-syntax",
            )
    return result


def print_breakdown(title: str, values: dict[str, dict[str, int]], limit: int) -> None:
    if not values:
        return
    print(f"{title}:")
    ranked = sorted(values.items(), key=lambda item: (-item[1]["total"], item[0]))
    for key, counts in ranked[:limit]:
        counts_text = ", ".join(
            f"{name}={value}" for name, value in sorted(counts.items())
        )
        print(f"  {key}: {counts_text}")
    if len(ranked) > limit:
        print(f"  ... {len(ranked) - limit} more")


def print_human(payload: dict[str, Any], show: int) -> None:
    print(f"promotion gate profile: {payload['profile']}")
    print(f"  scanned files: {payload['scanned_files']}")
    print(f"  xfail debt: {payload['xfail_count']}")
    print(f"  skip debt: {payload['skip_count']}")
    print(f"  optional debt: {payload['optional_count']}")
    print(f"  parse errors: {payload['parse_error_count']}")
    print(f"  total promotion debt: {payload['promotion_debt_total']}")
    print(f"  owned debt: {payload['owned_count']}")
    print(f"  unowned debt: {payload['unowned_count']}")
    print(f"  promotion-pending debt: {payload['promotion_pending_count']}")
    by_axis = payload["debt_by_axis"]
    print_breakdown("debt_by_dimension", by_axis["dimension"], show)
    print_breakdown("debt_by_bucket", by_axis["bucket"], show)
    print_breakdown("debt_by_lib", by_axis["lib"], show)
    for key in (
        "parse_errors",
        "xfail_debt",
        "skip_debt",
        "optional_debt",
        "unowned_debt",
        "promotion_pending_debt",
    ):
        items = payload[key]
        if not items:
            continue
        print(f"{key}:")
        for item in items[:show]:
            if key == "parse_errors":
                print(f"  {item['path']}: {item['error']}")
            else:
                refs = ",".join(item["owner_refs"]) or "unowned"
                axis = item["axis"]
                axis_text = (
                    f"{axis['dimension']}/{axis['bucket']}/{axis['lib']}"
                )
                print(f"  {item['path']} [{axis_text}; {refs}]: {item['reason']}")
        if len(items) > show:
            print(f"  ... {len(items) - show} more")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", default=str(DEFAULT_ROOT))
    parser.add_argument(
        "--manifest",
        default=str(DEFAULT_MANIFEST),
        help="ecosystem real-world fixture manifest; use '' to disable",
    )
    parser.add_argument(
        "--profile",
        choices=("inventory", "replacement"),
        default="replacement",
        help="replacement fails on any xfail/skip/optional debt; inventory only reports",
    )
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--show", type=int, default=20)
    parser.add_argument(
        "--json-all-details",
        action="store_true",
        help="emit every debt entry in JSON instead of the compact sample",
    )
    args = parser.parse_args(argv)

    manifest = Path(args.manifest) if args.manifest else None
    result = scan(Path(args.root), manifest)
    parse_failed = bool(result.parse_errors)
    debt_failed = args.profile == "replacement" and bool(result.debts)
    failed = parse_failed or debt_failed
    payload = result.to_json(args.profile, failed, args.show, args.json_all_details)

    if args.json:
        print(json.dumps(payload, sort_keys=True))
    else:
        print_human(payload, args.show)

    if parse_failed:
        return EXIT_PARSE_ERROR
    if debt_failed:
        return EXIT_DEBT
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
