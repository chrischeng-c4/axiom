#!/usr/bin/env python3
"""MVP smoke test inventory summary for mamba (closes #2537).

Produces a cheap, static inventory of the mamba MVP test surface. Does not
compile or run any tests; counts come from source-tree inspection so the
command runs in well under a second on a clean checkout.

Outputs four categories of Rust tests (normal, ignored, feature-gated,
external-tool) plus datatest fixture counts (conformance, CPython compat,
CPython Lib/test seed, real-world) and bench fixture counts.

Usage:
    python3 scripts/inventory_summary.py
    python3 scripts/inventory_summary.py --json
    python3 scripts/inventory_summary.py --root /abs/path/to/projects/mamba

Exit codes:
    0 — inventory printed
    1 — inventory could not be computed (missing dirs, unreadable files)
    2 — usage / parse error
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Iterable

# Tokens recognised as "external tool" invocations in test bodies. Any test
# file that calls into one of these is bucketed separately so MVP triage can
# tell which gate failures are environmental (Python on PATH, etc.) versus
# pure-Rust regressions.
EXTERNAL_TOOL_TOKENS = (
    'Command::new("python3"',
    'Command::new("python"',
    'Command::new("cargo"',
    'Command::new("git"',
    'PYTHON3_CMD',
    'pyo3',
)


@dataclass
class RustTestCounts:
    normal: int = 0
    ignored: int = 0
    feature_gated: int = 0
    external_tool: int = 0
    files: int = 0

    @property
    def total(self) -> int:
        return self.normal + self.ignored


@dataclass
class FixtureCounts:
    conformance_py: int = 0
    cpython_compat_py: int = 0
    cpython_lib_test_seed: int = 0
    real_world_py: int = 0


@dataclass
class BenchCounts:
    rust_bench_files: int = 0
    third_party_bench_files: int = 0
    python_bench_fixtures: int = 0


@dataclass
class Inventory:
    rust_tests: RustTestCounts = field(default_factory=RustTestCounts)
    fixtures: FixtureCounts = field(default_factory=FixtureCounts)
    benches: BenchCounts = field(default_factory=BenchCounts)


# `#[test]` and `#[ignore]` recognition is tolerant of inner attributes,
# whitespace, and conditional cfg wrappers. The split is line-anchored so we
# can scan once and accumulate counts deterministically.
TEST_ATTR_RE = re.compile(r"^\s*#\[\s*test\s*\]\s*$")
IGNORE_ATTR_RE = re.compile(r"^\s*#\[\s*ignore")
CFG_FEATURE_RE = re.compile(r"#\[\s*cfg\([^)]*feature\s*=")


def count_rust_tests(tests_dir: Path) -> RustTestCounts:
    counts = RustTestCounts()
    rs_files = sorted(p for p in tests_dir.glob("*.rs") if p.is_file())
    counts.files = len(rs_files)
    for path in rs_files:
        text = path.read_text(encoding="utf-8", errors="replace")
        lines = text.splitlines()
        file_has_feature_cfg = bool(CFG_FEATURE_RE.search(text))
        file_has_external_tool = any(tok in text for tok in EXTERNAL_TOOL_TOKENS)
        for idx, line in enumerate(lines):
            if not TEST_ATTR_RE.match(line):
                continue
            # Look at adjacent attribute lines (before and after #[test]) up
            # to the function signature; #[ignore] is conventionally placed
            # right next to #[test] in either order.
            ignored = False
            for direction in (-1, 1):
                j = idx + direction
                while 0 <= j < len(lines):
                    sib = lines[j].strip()
                    if not sib or sib.startswith("//"):
                        j += direction
                        continue
                    if not sib.startswith("#["):
                        break
                    if IGNORE_ATTR_RE.match(sib):
                        ignored = True
                        break
                    j += direction
                if ignored:
                    break
            if ignored:
                counts.ignored += 1
            else:
                counts.normal += 1
            if file_has_feature_cfg:
                counts.feature_gated += 1
            if file_has_external_tool:
                counts.external_tool += 1
    return counts


def _count_py(root: Path, pattern: str = "*.py") -> int:
    if not root.is_dir():
        return 0
    return sum(1 for _ in root.rglob(pattern))


def count_fixtures(fixtures_dir: Path) -> FixtureCounts:
    counts = FixtureCounts()
    conformance = fixtures_dir / "conformance"
    cpython = fixtures_dir / "cpython"
    cpython_seed = fixtures_dir / "cpython_lib_test" / "seed"

    counts.conformance_py = _count_py(conformance)
    counts.cpython_compat_py = _count_py(cpython)
    counts.cpython_lib_test_seed = sum(
        1 for p in cpython_seed.glob("*.py") if p.is_file()
    ) if cpython_seed.is_dir() else 0

    if conformance.is_dir():
        counts.real_world_py = sum(
            1 for p in conformance.rglob("real_world/*.py") if p.is_file()
        )
    return counts


def count_benches(benches_dir: Path) -> BenchCounts:
    counts = BenchCounts()
    if not benches_dir.is_dir():
        return counts
    counts.rust_bench_files = sum(
        1 for p in benches_dir.glob("*.rs") if p.is_file()
    )
    third_party = benches_dir / "3p"
    if third_party.is_dir():
        counts.third_party_bench_files = sum(
            1 for p in third_party.rglob("*.rs") if p.is_file()
        )
        counts.python_bench_fixtures = sum(
            1 for p in third_party.rglob("*.py") if p.is_file()
        )
    return counts


def compute(root: Path) -> Inventory:
    tests_dir = root / "tests"
    fixtures_dir = tests_dir / "fixtures"
    benches_dir = root / "benches"

    missing: list[str] = []
    for required in (tests_dir, fixtures_dir):
        if not required.is_dir():
            missing.append(str(required))
    if missing:
        raise FileNotFoundError(
            "missing required mamba directories: " + ", ".join(missing)
        )

    return Inventory(
        rust_tests=count_rust_tests(tests_dir),
        fixtures=count_fixtures(fixtures_dir),
        benches=count_benches(benches_dir),
    )


def render_text(inv: Inventory) -> str:
    rt = inv.rust_tests
    fx = inv.fixtures
    bn = inv.benches
    lines = [
        "mamba MVP smoke test inventory",
        "==============================",
        "Rust tests:",
        f"  test files:    {rt.files}",
        f"  normal:        {rt.normal}",
        f"  ignored:       {rt.ignored}",
        f"  feature-gated: {rt.feature_gated}",
        f"  external-tool: {rt.external_tool}",
        f"  total:         {rt.total}",
        "",
        "Datatest fixtures:",
        f"  conformance (.py):       {fx.conformance_py}",
        f"  cpython compat (.py):    {fx.cpython_compat_py}",
        f"  cpython Lib/test seed:   {fx.cpython_lib_test_seed}",
        f"  real-world:              {fx.real_world_py}",
        "",
        "Bench fixtures:",
        f"  benches/*.rs:            {bn.rust_bench_files}",
        f"  benches/3p/*.rs:         {bn.third_party_bench_files}",
        f"  benches/3p/*.py:         {bn.python_bench_fixtures}",
    ]
    return "\n".join(lines)


def render_json(inv: Inventory) -> str:
    payload = asdict(inv)
    payload["rust_tests"]["total"] = inv.rust_tests.total
    return json.dumps(payload, indent=2, sort_keys=True)


def _default_root() -> Path:
    return Path(__file__).resolve().parent.parent


def main(argv: Iterable[str]) -> int:
    parser = argparse.ArgumentParser(
        description=(__doc__ or "").splitlines()[0] or "mamba MVP inventory"
    )
    parser.add_argument(
        "--root",
        type=Path,
        default=_default_root(),
        help="path to projects/mamba (defaults to script's parent)",
    )
    parser.add_argument(
        "--json", action="store_true", help="emit machine-readable JSON"
    )
    args = parser.parse_args(list(argv))

    try:
        inv = compute(args.root)
    except FileNotFoundError as e:
        print(f"inventory_summary: {e}", file=sys.stderr)
        return 1
    except OSError as e:
        print(f"inventory_summary: I/O failure: {e}", file=sys.stderr)
        return 1

    out = render_json(inv) if args.json else render_text(inv)
    print(out)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
