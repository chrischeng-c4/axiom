#!/usr/bin/env python3
"""Run the CPython side of the conformance fixture tree.

This is a fixture-authoring gate, not a mamba runtime gate. It proves the
runtime-compatible CPython fixtures are valid under CPython before mamba uses
them as an oracle:

* runtime fixtures must exit 0 under CPython
* if a CPython golden exists, stdout must match it exactly
* `bench` fixtures are skipped here and covered by perf_baseline.py
* `# RUN:` pipeline/parser fixtures are skipped here and covered by harness/
* `--ready-only` skips third-party fixtures whose imports are unavailable
  locally, matching the perf baseline ready-only workflow

No pass/fail results are stored. The command exits non-zero on the first set of
fixture defects it finds.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import os
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path


TOOLS_DIR = Path(__file__).resolve().parent
CPYTHON_DIR = TOOLS_DIR.parents[2] / "cpython"  # tests/cpython (fixtures + .cache)
MAMBA_DIR = CPYTHON_DIR.parent.parent
FIXTURES_ROOT = CPYTHON_DIR / "fixtures"


@dataclass(frozen=True)
class CaseResult:
    path: Path
    status: str
    reason: str = ""
    stdout: str = ""
    stderr: str = ""


def is_type_strict(path: Path, text: str) -> bool:
    return "type-strict" in path.parts or "# mamba-strict-type:" in text


def has_pipeline_run_directive(text: str) -> bool:
    return any(line.lstrip().startswith("# RUN:") for line in text.splitlines())


def first_import_module(text: str) -> str | None:
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith(("import ", "from ")):
            break
    else:
        return None

    if line.startswith("import "):
        rest = line.removeprefix("import ")
        module = (
            rest.split("#", 1)[0]
            .split(",", 1)[0]
            .split(" as ", 1)[0]
            .strip()
        )
        return module or None

    if line.startswith("from "):
        rest = line.removeprefix("from ")
        module = rest.split(" import ", 1)[0].strip()
        if module and not module.startswith("."):
            return module
    return None


def third_party_prereq_import(path: Path, text: str) -> str | None:
    if "3rd-libs" not in path.parts:
        return None
    return first_import_module(text)


def is_bench(path: Path) -> bool:
    return "bench" in path.parts


def expected_path_for(path: Path, text: str) -> Path | None:
    if is_type_strict(path, text):
        candidate = path.with_suffix("").with_suffix(".cpython.expected")
        return candidate if candidate.exists() else None
    candidate = path.with_suffix(".expected")
    return candidate if candidate.exists() else None


def discover(root: Path, bucket: str | None, lib: str | None) -> list[Path]:
    cases: list[Path] = []
    for path in sorted(root.rglob("*.py")):
        rel = path.relative_to(root)
        if path.name.endswith("_stub.py"):
            continue
        if "_invalid" in rel.parts:
            continue
        if bucket and (len(rel.parts) < 1 or rel.parts[0] != bucket):
            continue
        if lib and (len(rel.parts) < 2 or rel.parts[1] != lib):
            continue
        cases.append(path)
    return cases


def missing_python_modules(modules: set[str], python: str) -> set[str]:
    if not modules:
        return set()

    script = """
import importlib
import sys

for module in sys.argv[1:]:
    try:
        importlib.import_module(module)
    except Exception:
        print(module)
"""
    result = subprocess.run(
        [python, "-c", script, *sorted(modules)],
        text=True,
        capture_output=True,
    )
    if result.returncode != 0:
        return set(modules)
    return set(result.stdout.splitlines())


def third_party_prereqs(cases: list[Path]) -> dict[Path, str]:
    prereqs: dict[Path, str] = {}
    for path in cases:
        if "3rd-libs" not in path.parts:
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        if module := third_party_prereq_import(path, text):
            prereqs[path] = module
    return prereqs


def run_one(
    path: Path,
    python: str,
    timeout: float,
    ready_only: bool,
    missing_prereqs: set[str],
) -> CaseResult:
    text = path.read_text(encoding="utf-8", errors="replace")
    if is_bench(path):
        return CaseResult(path, "skip", "bench/perf-baseline-owned")
    if has_pipeline_run_directive(text):
        return CaseResult(path, "skip", "pipeline-run-directive")
    if ready_only:
        prereq = third_party_prereq_import(path, text)
        if prereq in missing_prereqs:
            return CaseResult(path, "skip", "missing-prereq-import")

    expected_path = expected_path_for(path, text)
    try:
        with tempfile.TemporaryDirectory(prefix="mamba-cpython-oracle-") as tmp:
            env = dict(os.environ)
            env["TMPDIR"] = tmp
            env["TEMP"] = tmp
            env["TMP"] = tmp
            result = subprocess.run(
                [python, str(path.resolve())],
                cwd=tmp,
                text=True,
                capture_output=True,
                timeout=timeout,
                env=env,
            )
    except subprocess.TimeoutExpired as exc:
        return CaseResult(
            path,
            "fail",
            f"timeout after {timeout:g}s",
            exc.stdout or "",
            exc.stderr or "",
        )

    if result.returncode != 0:
        return CaseResult(
            path,
            "fail",
            f"CPython exit {result.returncode}",
            result.stdout,
            result.stderr,
        )

    if expected_path is not None:
        expected = expected_path.read_text(encoding="utf-8", errors="replace")
        if result.stdout != expected:
            return CaseResult(
                path,
                "fail",
                f"stdout mismatch against {expected_path.relative_to(MAMBA_DIR)}",
                result.stdout,
                result.stderr,
            )

    return CaseResult(path, "pass")


def result_sample(result: CaseResult) -> str:
    rel = result.path.relative_to(MAMBA_DIR)
    lines = [f"FAIL {rel}: {result.reason}"]
    if result.stdout:
        lines.append("  stdout:")
        lines.extend(f"    {line}" for line in result.stdout.rstrip().splitlines()[:12])
    if result.stderr:
        lines.append("  stderr:")
        lines.extend(f"    {line}" for line in result.stderr.rstrip().splitlines()[:12])
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", default=str(FIXTURES_ROOT))
    parser.add_argument("--python", default=os.environ.get("PYTHON", sys.executable))
    parser.add_argument("--bucket")
    parser.add_argument("--lib")
    parser.add_argument("--jobs", type=int, default=max((os.cpu_count() or 2) // 2, 1))
    parser.add_argument("--timeout", type=float, default=30.0)
    parser.add_argument("--max-failures", type=int, default=40)
    parser.add_argument(
        "--ready-only",
        action="store_true",
        help="skip third-party fixtures whose import prereqs are missing locally",
    )
    parser.add_argument(
        "--progress-every",
        type=int,
        default=0,
        help="print progress after every N completed fixtures; 0 disables progress",
    )
    args = parser.parse_args(argv)

    root = Path(args.root).resolve()
    if os.sep in args.python:
        python_path = Path(args.python)
        python = str(python_path if python_path.is_absolute() else Path.cwd() / python_path)
    else:
        python = args.python
    cases = discover(root, args.bucket, args.lib)
    prereqs = third_party_prereqs(cases) if args.ready_only else {}
    missing_prereqs = missing_python_modules(set(prereqs.values()), python)
    passes = 0
    skips: dict[str, int] = {}
    failures: list[CaseResult] = []
    completed = 0

    with concurrent.futures.ThreadPoolExecutor(max_workers=max(args.jobs, 1)) as pool:
        futures = [
            pool.submit(run_one, path, python, args.timeout, args.ready_only, missing_prereqs)
            for path in cases
        ]
        for future in concurrent.futures.as_completed(futures):
            result = future.result()
            completed += 1
            if result.status == "pass":
                passes += 1
            elif result.status == "skip":
                skips[result.reason] = skips.get(result.reason, 0) + 1
            else:
                failures.append(result)
            if args.progress_every and completed % args.progress_every == 0:
                print(
                    "progress: "
                    f"{completed}/{len(cases)} "
                    f"pass={passes} "
                    f"skip={sum(skips.values())} "
                    f"fail={len(failures)}",
                    flush=True,
                )

    print(f"total fixtures: {len(cases)}")
    print(f"  cpython runtime pass: {passes}")
    for reason in sorted(skips):
        print(f"  skipped {reason}: {skips[reason]}")
    print(f"  failures: {len(failures)}")

    if failures:
        for result in failures[: args.max_failures]:
            print("")
            print(result_sample(result))
        if len(failures) > args.max_failures:
            print(f"\n... {len(failures) - args.max_failures} more failures omitted")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
