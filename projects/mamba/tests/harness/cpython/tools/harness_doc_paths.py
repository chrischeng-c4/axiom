#!/usr/bin/env python3.12
"""Validate CPython harness documentation paths for #716.

The checker is intentionally narrow: it scans the agent-facing docs and issue
templates that tell future agents which CPython harness commands to run. It
rejects retired path families while allowing historical notes in generated
fixtures and manifests to remain untouched.
"""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any


TOOLS_DIR = Path(__file__).resolve().parent
MAMBA_DIR = TOOLS_DIR.parents[3]
REPO_ROOT = MAMBA_DIR.parents[1]

DOC_PATHS = (
    "projects/mamba/README.md",
    "projects/mamba/tests/README.md",
    "projects/mamba/tests/PRODUCTION-GATE.md",
    "projects/mamba/tests/harness/cpython/conventions/FIXTURE-LAYOUT.md",
    "projects/mamba/tests/harness/cpython/conventions/REAL-WORLD-CONVENTION.md",
    ".github/ISSUE_TEMPLATE/axis1-seed.md",
    "projects/mamba/issue-loop.md",
)

REQUIRED_PATHS = (
    "projects/mamba/tests/PRODUCTION-GATE.md",
    "projects/mamba/tests/cpython",
    "projects/mamba/tests/cpython/type",
    "projects/mamba/tests/harness/cpython",
    "projects/mamba/tests/harness/cpython/conventions/FIXTURE-LAYOUT.md",
    "projects/mamba/tests/harness/cpython/tools/wall_status.py",
    "projects/mamba/tests/harness/cpython/tools/gate_check.py",
    "projects/mamba/tests/harness/cpython/tools/verify_cpython_oracle.py",
    "projects/mamba/tests/harness/cpython/tools/fixture_lint.py",
    "projects/mamba/tools/fixture_gen.py",
)

FORBIDDEN_PATTERNS = (
    (r"\bprojects/mamba/PRODUCTION-GATE\.md\b", "production_gate_moved_under_tests"),
    (r"\b(?:projects/mamba/)?tests/cpython/fixtures\b", "retired_cpython_fixtures_root"),
    (r"\b(?:projects/mamba/)?tests/cpython/tools\b", "retired_cpython_tools_root"),
    (r"\b(?:projects/mamba/)?tests/cpython/conventions\b", "retired_cpython_conventions_root"),
    (r"\b(?:projects/mamba/)?tests/cpython/type-strict\b", "retired_type_strict_root"),
    (r"\bprojects/mamba/tests/fixtures/cpython_lib_test\b", "retired_axis1_seed_root"),
    (r"--test\s+cpython_lib_test_runner\b", "renamed_axis1_test_binary"),
    (
        r"--test\s+conformance_cpython_lib_test\s+--release\b",
        "release_axis1_dev_command",
    ),
    (r"\bcpython_lib_test_baseline\.toml\b", "retired_axis1_baseline_file"),
    (r"\bcpython_lib_test_allowlist\.toml\b", "retired_axis1_allowlist_file"),
    (
        r"python3(?:\.\d+)?\s+tools/(?:fixture_lint|verify_cpython_oracle|gate_check|wall_status)\.py\b",
        "ambiguous_harness_tool_command",
    ),
)

PYTHON_COMMAND_RE = re.compile(
    r"python3(?:\.\d+)?\s+"
    r"(?P<path>(?:projects/mamba/)?(?:tests/harness/cpython/tools|tools)/[A-Za-z0-9_./-]+\.py)"
)


@dataclass(frozen=True)
class Finding:
    path: str
    line: int
    kind: str
    text: str

    def to_json(self) -> dict[str, Any]:
        return {
            "path": self.path,
            "line": self.line,
            "kind": self.kind,
            "text": self.text,
        }


def repo_path(path: str) -> Path:
    return REPO_ROOT / path


def resolve_command_path(raw: str) -> Path:
    if raw.startswith("projects/mamba/"):
        return REPO_ROOT / raw
    if raw.startswith("tests/") or raw.startswith("tools/"):
        return MAMBA_DIR / raw
    return REPO_ROOT / raw


def scan_docs(show: int) -> dict[str, Any]:
    missing_docs = [path for path in DOC_PATHS if not repo_path(path).exists()]
    missing_required = [path for path in REQUIRED_PATHS if not repo_path(path).exists()]
    findings: list[Finding] = []
    commands_checked: dict[str, int] = {}
    missing_command_paths: list[str] = []

    compiled = [(re.compile(pattern), kind) for pattern, kind in FORBIDDEN_PATTERNS]
    for rel in DOC_PATHS:
        path = repo_path(rel)
        if not path.exists():
            continue
        for line_no, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            for pattern, kind in compiled:
                if pattern.search(line):
                    findings.append(Finding(rel, line_no, kind, line.strip()))
            for match in PYTHON_COMMAND_RE.finditer(line):
                raw = match.group("path")
                commands_checked[raw] = commands_checked.get(raw, 0) + 1
                if not resolve_command_path(raw).exists():
                    missing_command_paths.append(raw)

    ready = not missing_docs and not missing_required and not findings and not missing_command_paths
    return {
        "schema_version": 1,
        "owner_issue": "#716",
        "ready": ready,
        "status": "green" if ready else "red",
        "counts": {
            "docs_scanned": len(DOC_PATHS),
            "required_paths": len(REQUIRED_PATHS),
            "missing_docs": len(missing_docs),
            "missing_required_paths": len(missing_required),
            "forbidden_references": len(findings),
            "python_commands_checked": sum(commands_checked.values()),
            "missing_python_command_paths": len(missing_command_paths),
        },
        "docs": DOC_PATHS,
        "required_paths": REQUIRED_PATHS,
        "missing_docs": missing_docs,
        "missing_required_paths": missing_required,
        "forbidden_references": [finding.to_json() for finding in findings[:show]],
        "python_commands_checked": dict(sorted(commands_checked.items())),
        "missing_python_command_paths": sorted(set(missing_command_paths)),
        "evidence_commands": [
            "python3.12 projects/mamba/tests/harness/cpython/tools/harness_doc_paths.py --json",
            "python3.12 projects/mamba/tests/harness/cpython/tools/wall_status.py --help",
            "python3.12 projects/mamba/tests/harness/cpython/tools/gate_check.py --help",
            "cargo test -p mamba --test schema_gates harness_doc_paths_gate_716 -- --nocapture",
        ],
    }


def print_human(report: dict[str, Any]) -> None:
    counts = report["counts"]
    print(f"harness doc paths: {report['status']}")
    print(
        "  docs={docs_scanned} required={required_paths} "
        "forbidden={forbidden_references} missing_cmds={missing_python_command_paths}".format(
            **counts
        )
    )
    for finding in report["forbidden_references"]:
        print(f"- {finding['path']}:{finding['line']} {finding['kind']}: {finding['text']}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--show", type=int, default=20)
    args = parser.parse_args(argv)

    report = scan_docs(args.show)
    if args.json:
        print(json.dumps(report, sort_keys=True))
    else:
        print_human(report)
    return 0 if report["ready"] else 70


if __name__ == "__main__":
    raise SystemExit(main())
