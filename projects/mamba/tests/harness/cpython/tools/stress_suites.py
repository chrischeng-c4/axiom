#!/usr/bin/env python3.12
"""Targeted stability suites for issues #1126 and #1125.

This tool keeps three TEST-ONLY surfaces runnable without a full corpus run:

* ``soak``              repeated process runs with a peak-RSS ceiling assert
* ``concurrency``       threaded stress fixtures with timeout/no-deadlock checks
* ``debug-assertions``  crash-corpus smoke for debug mamba builds

Every suite runs its fixture set directly, emits a small human summary by
default, and can return a machine-readable JSON envelope with ``--json``.
"""

from __future__ import annotations

import argparse
import json
import os
import resource
import shutil
import subprocess
import sys
import time
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parent))
import harness_lib  # noqa: E402


TOOLS_DIR = Path(__file__).resolve().parent
MAMBA_DIR = TOOLS_DIR.parents[3]
FIXTURES_DIR = MAMBA_DIR / "tests" / "cpython"

DEFAULT_SOAK_REPEAT = 8
DEFAULT_CONCURRENCY_REPEAT = 3
DEFAULT_SOAK_TIMEOUT = 15
DEFAULT_CONCURRENCY_TIMEOUT = 20
DEFAULT_DEBUG_TIMEOUT = 20
DEFAULT_SOAK_RSS_CEILING_KIB = 512 * 1024

SUITE_FIXTURES = {
    "soak": (
        "_regression/core/stability/heap_churn_soak.py",
    ),
    "concurrency": (
        "concurrency/atomicity/list/append_no_lost_updates.py",
        "concurrency/atomicity/dict/distinct_key_setitem_no_loss.py",
        "concurrency/safety/lock/counter_under_lock_is_exact.py",
        "concurrency/primitives/threading/barrier_rounds_complete_without_deadlock.py",
    ),
    "debug-assertions": (
        "_regression/core/compiler_resilience/debug_assertions_smoke_small.py",
        "_regression/core/compiler_resilience/deep_nesting.py",
        "_regression/core/compiler_resilience/oversized_int.py",
    ),
}

FULL_DEBUG_CORPUS_DIRS = (
    FIXTURES_DIR / "_regression" / "core" / "compiler_resilience",
    FIXTURES_DIR / "security" / "core" / "crashers",
)
HIDDEN_PROBE = "__probe-run"


@dataclass(frozen=True)
class FixtureMeta:
    rel: str
    xfail: str


def repo_rel(path: Path) -> str:
    return path.relative_to(FIXTURES_DIR).as_posix()


def fixture_path(rel: str) -> Path:
    path = FIXTURES_DIR / rel
    if not path.is_file():
        raise SystemExit(f"fixture not found: {rel}")
    return path


def default_mamba_bin() -> str:
    if override := os.environ.get("MAMBA_BIN", "").strip():
        return override
    for candidate in (
        MAMBA_DIR.parents[1] / "target" / "debug" / "mamba",
        MAMBA_DIR.parents[1] / "target" / "release" / "mamba",
    ):
        if candidate.is_file():
            return str(candidate)
    return shutil.which("mamba") or "mamba"


def default_python_bin() -> str:
    if override := os.environ.get("MAMBA_ORACLE_PYTHON", "").strip():
        return override
    oracle_env = FIXTURES_DIR / ".cache" / "oracle-env" / "bin" / "python3"
    if oracle_env.is_file():
        return str(oracle_env)
    return shutil.which("python3.12") or shutil.which("python3") or "python3"


def strip_comment_prefix(block: list[str]) -> str:
    lines: list[str] = []
    for raw in block:
        if raw.startswith("# "):
            lines.append(raw[2:])
        elif raw == "#":
            lines.append("")
        else:
            lines.append(raw.lstrip("#"))
    return "\n".join(lines)


def parse_tool_mamba(text: str) -> dict[str, Any]:
    lines = text.splitlines()
    inside_script = False
    capture = False
    block: list[str] = []
    for raw in lines:
        if raw.strip() == "# /// script":
            inside_script = True
            capture = False
            block = []
            continue
        if inside_script and raw.strip() == "# ///":
            if block:
                try:
                    parsed = tomllib.loads(strip_comment_prefix(block))
                except tomllib.TOMLDecodeError:
                    return {}
                tool = parsed.get("tool", {})
                mamba = tool.get("mamba", {})
                return mamba if isinstance(mamba, dict) else {}
            return {}
        if inside_script and raw.strip() == "# [tool.mamba]":
            capture = True
            block = [raw]
            continue
        if capture:
            if raw.startswith("#"):
                block.append(raw)
                continue
            capture = False
    return {}


def parse_comment_xfail(text: str) -> str:
    for raw in text.splitlines():
        line = raw.strip()
        if line.startswith("# mamba-xfail:"):
            return line.split(":", 1)[1].strip()
    return ""


def fixture_meta(rel: str) -> FixtureMeta:
    text = fixture_path(rel).read_text(encoding="utf-8")
    record = parse_tool_mamba(text)
    record_xfail = record.get("xfail", "")
    xfail = record_xfail.strip() if isinstance(record_xfail, str) else ""
    if not xfail:
        xfail = parse_comment_xfail(text)
    return FixtureMeta(rel=rel, xfail=xfail)


def lossy(data: str | bytes) -> str:
    if isinstance(data, str):
        return data
    return data.decode("utf-8", errors="replace")


def run_plain(argv: list[str], timeout: int) -> dict[str, Any]:
    start = time.monotonic()
    rc, out, err = harness_lib.run_fixture(argv, timeout, text=False)
    return {
        "returncode": rc,
        "stdout": lossy(out),
        "stderr": lossy(err),
        "elapsed_s": round(time.monotonic() - start, 4),
    }


def normalize_rss_kib(value: int) -> int:
    return value // 1024 if sys.platform == "darwin" else value


def run_measured(argv: list[str], timeout: int) -> dict[str, Any]:
    helper = [
        sys.executable,
        str(Path(__file__).resolve()),
        HIDDEN_PROBE,
        "--timeout",
        str(timeout),
        "--",
        *argv,
    ]
    proc = subprocess.run(
        helper,
        capture_output=True,
        text=True,
        timeout=timeout + 15,
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or "probe helper failed")
    return json.loads(proc.stdout)


def run_oracle(rel: str, python_bin: str, timeout: int) -> dict[str, Any]:
    return run_plain([python_bin, str(fixture_path(rel))], timeout)


def run_mamba(rel: str, mamba_bin: str, timeout: int, *, measured: bool) -> dict[str, Any]:
    argv = [mamba_bin, "run", str(fixture_path(rel))]
    return run_measured(argv, timeout) if measured else run_plain(argv, timeout)


def concurrency_verdict(stdout: str) -> tuple[bool, str]:
    for line in stdout.splitlines():
        if line.startswith("concurrency: PASS"):
            return True, "PASS"
        if line.startswith("concurrency: FAIL:"):
            return False, line[len("concurrency: FAIL:"):].strip()
    return False, "missing concurrency verdict line"


def looks_like_panic(stdout: str, stderr: str) -> bool:
    haystack = "\n".join((stdout, stderr)).lower()
    return (
        "panicked at" in haystack
        or "assertion failed:" in haystack
        or "thread 'main' panicked" in haystack
    )


def elapsed_total(results: list[dict[str, Any]]) -> float:
    return round(sum(item.get("elapsed_s", 0.0) for item in results), 4)


def suite_result(
    suite: str,
    cases: list[dict[str, Any]],
    *,
    extra: dict[str, Any] | None = None,
) -> dict[str, Any]:
    counts = {"PASS": 0, "XFAIL": 0, "FAIL": 0, "CRASH": 0}
    for case in cases:
        counts[case["status"]] = counts.get(case["status"], 0) + 1
    result = {
        "suite": suite,
        "cases": cases,
        "counts": counts,
        "ok": counts["FAIL"] == 0 and counts["CRASH"] == 0,
    }
    if extra:
        result.update(extra)
    return result


def run_soak_suite(args: argparse.Namespace) -> dict[str, Any]:
    cases: list[dict[str, Any]] = []
    for rel in args.fixtures:
        meta = fixture_meta(rel)
        oracle = run_oracle(rel, args.python, args.timeout)
        if oracle["returncode"] != 0:
            cases.append(
                {
                    "fixture": rel,
                    "status": "FAIL",
                    "detail": f"oracle rc={oracle['returncode']}",
                    "oracle": oracle,
                    "runs": [],
                }
            )
            continue
        runs = [run_mamba(rel, args.mamba_bin, args.timeout, measured=True) for _ in range(args.repeat)]
        nonzero = next((run for run in runs if run["returncode"] != 0), None)
        max_rss = max(run["peak_rss_kib"] for run in runs)
        if nonzero is not None:
            if nonzero["returncode"] is None or nonzero["returncode"] < 0 or looks_like_panic(
                nonzero["stdout"], nonzero["stderr"],
            ):
                status = "CRASH"
                detail = "crash/timeout during soak run"
            else:
                status = "XFAIL" if meta.xfail else "FAIL"
                detail = f"mamba rc={nonzero['returncode']}"
        elif max_rss > args.rss_ceiling_kib:
            status = "FAIL"
            detail = f"peak rss {max_rss} KiB > ceiling {args.rss_ceiling_kib} KiB"
        else:
            status = "PASS"
            detail = (
                f"repeat={args.repeat} max_rss={max_rss} KiB "
                f"ceiling={args.rss_ceiling_kib} KiB"
            )
        cases.append(
            {
                "fixture": rel,
                "status": status,
                "detail": detail,
                "xfail": meta.xfail,
                "oracle": oracle,
                "runs": runs,
                "max_rss_kib": max_rss,
                "elapsed_s": elapsed_total(runs),
            }
        )
    return suite_result(
        "soak",
        cases,
        extra={
            "repeat": args.repeat,
            "rss_ceiling_kib": args.rss_ceiling_kib,
            "mamba_bin": args.mamba_bin,
            "python": args.python,
        },
    )


def run_concurrency_suite(args: argparse.Namespace) -> dict[str, Any]:
    cases: list[dict[str, Any]] = []
    for rel in args.fixtures:
        meta = fixture_meta(rel)
        oracle = run_oracle(rel, args.python, args.timeout)
        if oracle["returncode"] != 0:
            cases.append(
                {
                    "fixture": rel,
                    "status": "FAIL",
                    "detail": f"oracle rc={oracle['returncode']}",
                    "oracle": oracle,
                    "runs": [],
                }
            )
            continue
        runs = [run_mamba(rel, args.mamba_bin, args.timeout, measured=False) for _ in range(args.repeat)]
        failed: dict[str, Any] | None = None
        for run in runs:
            ok, detail = concurrency_verdict(run["stdout"])
            run["concurrency_ok"] = ok
            run["concurrency_detail"] = detail
            if run["returncode"] != 0 or not ok:
                failed = run
                break
        if failed is None:
            status = "PASS"
            detail = f"repeat={args.repeat} no lost updates or deadlock"
        elif failed["returncode"] is None or failed["returncode"] < 0 or looks_like_panic(
            failed["stdout"], failed["stderr"],
        ):
            status = "CRASH"
            detail = "crash/timeout during concurrency stress"
        elif meta.xfail:
            status = "XFAIL"
            detail = meta.xfail
        else:
            status = "FAIL"
            if failed["returncode"] is None:
                detail = "timeout"
            elif failed["returncode"] != 0:
                detail = f"mamba rc={failed['returncode']}"
            else:
                detail = failed["concurrency_detail"]
        cases.append(
            {
                "fixture": rel,
                "status": status,
                "detail": detail,
                "xfail": meta.xfail,
                "oracle": oracle,
                "runs": runs,
                "elapsed_s": elapsed_total(runs),
            }
        )
    return suite_result(
        "concurrency",
        cases,
        extra={
            "repeat": args.repeat,
            "mamba_bin": args.mamba_bin,
            "python": args.python,
        },
    )


def debug_fixture_list(args: argparse.Namespace) -> list[str]:
    rels = list(args.fixtures)
    if args.full_crash_corpus:
        rels = []
        for root in FULL_DEBUG_CORPUS_DIRS:
            rels.extend(repo_rel(path) for path in sorted(root.glob("*.py")))
    return rels


def run_debug_suite(args: argparse.Namespace) -> dict[str, Any]:
    rels = debug_fixture_list(args)
    cases: list[dict[str, Any]] = []
    for rel in rels:
        meta = fixture_meta(rel)
        oracle = run_oracle(rel, args.python, args.timeout)
        if oracle["returncode"] != 0:
            cases.append(
                {
                    "fixture": rel,
                    "status": "FAIL",
                    "detail": f"oracle rc={oracle['returncode']}",
                    "oracle": oracle,
                    "run": None,
                }
            )
            continue
        run = run_mamba(rel, args.mamba_bin, args.timeout, measured=True)
        if run["returncode"] is None:
            status = "CRASH"
            detail = "timeout"
        elif run["returncode"] < 0:
            status = "CRASH"
            detail = f"signal {-run['returncode']}"
        elif looks_like_panic(run["stdout"], run["stderr"]):
            status = "CRASH"
            detail = "panic/assertion output detected"
        elif run["returncode"] == 0:
            status = "PASS"
            detail = "clean execution"
        elif meta.xfail:
            status = "XFAIL"
            detail = meta.xfail
        else:
            status = "FAIL"
            detail = f"mamba rc={run['returncode']}"
        cases.append(
            {
                "fixture": rel,
                "status": status,
                "detail": detail,
                "xfail": meta.xfail,
                "oracle": oracle,
                "run": run,
                "elapsed_s": run["elapsed_s"],
                "peak_rss_kib": run["peak_rss_kib"],
            }
        )
    return suite_result(
        "debug-assertions",
        cases,
        extra={
            "full_crash_corpus": args.full_crash_corpus,
            "crash_corpus_roots": [repo_rel(path) for path in FULL_DEBUG_CORPUS_DIRS],
            "mamba_bin": args.mamba_bin,
            "python": args.python,
        },
    )


def print_human(result: dict[str, Any]) -> None:
    print(f"== {result['suite']}")
    for case in result["cases"]:
        detail = case.get("detail", "")
        print(f"{case['status']:>6}  {case['fixture']}  {detail}")
    counts = result["counts"]
    print(
        "summary:"
        f" PASS={counts.get('PASS', 0)}"
        f" XFAIL={counts.get('XFAIL', 0)}"
        f" FAIL={counts.get('FAIL', 0)}"
        f" CRASH={counts.get('CRASH', 0)}"
    )


def parse_args(argv: list[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run targeted soak/concurrency/debug-assertions test slices.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    list_parser = subparsers.add_parser("list", help="list default fixture sets")
    list_parser.add_argument("--json", action="store_true", help="print JSON")

    def add_common(sub: argparse.ArgumentParser, *, default_suite: str) -> None:
        sub.add_argument(
            "--fixture",
            dest="fixtures",
            action="append",
            default=None,
            help="relative fixture path; repeat to override the default set",
        )
        sub.add_argument("--mamba-bin", default=default_mamba_bin(), help="mamba binary to run")
        sub.add_argument("--python", default=default_python_bin(), help="oracle CPython binary")
        sub.add_argument("--timeout", type=int, default=DEFAULT_SOAK_TIMEOUT, help="per-run timeout in seconds")
        sub.add_argument("--json", action="store_true", help="emit machine-readable JSON")
        sub.set_defaults(default_suite=default_suite)

    soak = subparsers.add_parser("soak", help="repeat a small soak probe with RSS ceiling asserts")
    add_common(soak, default_suite="soak")
    soak.set_defaults(timeout=DEFAULT_SOAK_TIMEOUT)
    soak.add_argument("--repeat", type=int, default=DEFAULT_SOAK_REPEAT, help="repeat each fixture N times")
    soak.add_argument(
        "--rss-ceiling-kib",
        type=int,
        default=DEFAULT_SOAK_RSS_CEILING_KIB,
        help="maximum allowed peak RSS per run in KiB",
    )

    conc = subparsers.add_parser("concurrency", help="run threaded stress fixtures with timeout checks")
    add_common(conc, default_suite="concurrency")
    conc.set_defaults(timeout=DEFAULT_CONCURRENCY_TIMEOUT)
    conc.add_argument("--repeat", type=int, default=DEFAULT_CONCURRENCY_REPEAT, help="repeat each fixture N times")

    debug = subparsers.add_parser(
        "debug-assertions",
        help="run the crash-corpus smoke lane against a debug mamba binary",
    )
    add_common(debug, default_suite="debug-assertions")
    debug.set_defaults(timeout=DEFAULT_DEBUG_TIMEOUT)
    debug.add_argument(
        "--full-crash-corpus",
        action="store_true",
        help="replace the small default set with every fixture in compiler_resilience and security/core/crashers",
    )

    hidden = subparsers.add_parser(HIDDEN_PROBE)
    hidden.add_argument("--timeout", type=int, required=True)
    hidden.add_argument("argv", nargs=argparse.REMAINDER)

    args = parser.parse_args(argv)
    if hasattr(args, "default_suite"):
        args.fixtures = args.fixtures or list(SUITE_FIXTURES[args.default_suite])
    return args


def handle_probe(args: argparse.Namespace) -> int:
    argv = list(args.argv)
    if argv and argv[0] == "--":
        argv = argv[1:]
    start = time.monotonic()
    rc, out, err = harness_lib.run_fixture(argv, args.timeout, text=False)
    peak_rss = normalize_rss_kib(resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss)
    payload = {
        "returncode": rc,
        "stdout": lossy(out),
        "stderr": lossy(err),
        "elapsed_s": round(time.monotonic() - start, 4),
        "peak_rss_kib": peak_rss,
    }
    print(json.dumps(payload))
    return 0


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    if args.command == HIDDEN_PROBE:
        return handle_probe(args)
    if args.command == "list":
        payload = {
            name: list(fixtures)
            for name, fixtures in SUITE_FIXTURES.items()
        }
        payload["debug-assertions-full-crash-corpus-roots"] = [
            repo_rel(path) for path in FULL_DEBUG_CORPUS_DIRS
        ]
        if args.json:
            print(json.dumps(payload, indent=2, sort_keys=True))
        else:
            for name, fixtures in payload.items():
                print(f"{name}:")
                for rel in fixtures:
                    print(f"  - {rel}")
        return 0

    if args.command == "soak":
        result = run_soak_suite(args)
    elif args.command == "concurrency":
        result = run_concurrency_suite(args)
    elif args.command == "debug-assertions":
        result = run_debug_suite(args)
    else:  # pragma: no cover - argparse guards this
        raise SystemExit(f"unknown command: {args.command}")

    if args.json:
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        print_human(result)
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
