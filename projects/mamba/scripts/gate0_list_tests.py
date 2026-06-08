#!/usr/bin/env python3
"""Gate 0 — `cargo test -p mamba -- --list` worker gate (closes #2534).

The first MVP test gate every worker must clear before claiming mamba
test readiness. Compiles the mamba test profile and enumerates the
discovered tests via `cargo test --list`, without running them.

Exit 0 only when the test profile compiles AND tests can be listed.
A compile failure here blocks every deeper gate (#2527 family).

Default mode prints a summary line to stdout (total/list status). The
`--json` mode emits a machine-readable payload with per-target counts.

Usage:
    python3 scripts/gate0_list_tests.py
    python3 scripts/gate0_list_tests.py --release
    python3 scripts/gate0_list_tests.py --json
    python3 scripts/gate0_list_tests.py --manifest /abs/path/Cargo.toml

Exit codes:
    0 — compile + list succeeded
    1 — cargo compile failed, or --list output unparseable
    2 — usage / argument error
    3 — cargo binary not on PATH
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable


# `cargo test --list` summary lines that we can deterministically count.
# The format is stable: `N tests, M benchmarks` per target executable.
LIST_SUMMARY_RE = re.compile(
    r"^\s*(\d+)\s+tests?,\s+(\d+)\s+benchmarks?\s*$",
    re.MULTILINE,
)

# Each compiled test/bench target prints a `Running ...` line before its
# own `--list` output. We use this to count distinct target binaries.
RUNNING_TARGET_RE = re.compile(
    r"^\s*Running\s+(?:tests/|benches/)?[^\s]+\s+\(([^)]+)\)\s*$",
    re.MULTILINE,
)


@dataclass
class GateResult:
    profile: str
    targets: int = 0
    tests: int = 0
    benches: int = 0
    exit_code: int = 0
    cargo_argv: list[str] = field(default_factory=list)


def _default_manifest() -> Path:
    return Path(__file__).resolve().parent.parent / "Cargo.toml"


def run_gate(manifest: Path, release: bool, extra: list[str]) -> GateResult:
    profile = "release" if release else "dev"
    argv = [
        "cargo",
        "test",
        "-p",
        "mamba",
        "--manifest-path",
        str(manifest),
    ]
    if release:
        argv.append("--release")
    argv += ["--", "--list"]
    argv += extra

    result = GateResult(profile=profile, cargo_argv=argv)

    # Capture both streams so we can count `Running` lines (cargo writes
    # those to stderr) together with `N tests, M benchmarks` summaries
    # (stdout). The combined buffer is also forwarded to sys.stderr below
    # so workers still see compile progress.
    proc = subprocess.run(
        argv,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
        env={**os.environ, "CARGO_TERM_COLOR": "never"},
    )
    combined = proc.stdout or ""
    sys.stderr.write(combined)
    result.exit_code = proc.returncode
    if proc.returncode != 0:
        return result

    stdout = combined
    result.targets = len(RUNNING_TARGET_RE.findall(stdout))
    tests = 0
    benches = 0
    matched = False
    for m in LIST_SUMMARY_RE.finditer(stdout):
        matched = True
        tests += int(m.group(1))
        benches += int(m.group(2))
    result.tests = tests
    result.benches = benches

    # cargo can succeed with no `--list` summary lines only if there were no
    # test targets at all — for mamba that should never happen, so treat it
    # as a gate failure.
    if not matched:
        sys.stderr.write(
            "gate0_list_tests: cargo exited 0 but produced no '--list' "
            "summary lines; aborting.\n"
        )
        result.exit_code = 1
    return result


def render_text(r: GateResult) -> str:
    if r.exit_code != 0:
        return (
            f"gate-0 FAIL (profile={r.profile}, exit={r.exit_code}). "
            "Compile failure blocks every deeper gate."
        )
    return (
        f"gate-0 OK (profile={r.profile}) — {r.targets} target(s), "
        f"{r.tests} test(s), {r.benches} bench(es) listed."
    )


def render_json(r: GateResult) -> str:
    payload = {
        "gate": "gate-0",
        "profile": r.profile,
        "ok": r.exit_code == 0,
        "exit_code": r.exit_code,
        "targets": r.targets,
        "tests": r.tests,
        "benches": r.benches,
        "cargo_argv": r.cargo_argv,
    }
    return json.dumps(payload, indent=2, sort_keys=True)


def main(argv: Iterable[str]) -> int:
    parser = argparse.ArgumentParser(
        description=(__doc__ or "").splitlines()[0] or "gate-0 list wrapper"
    )
    parser.add_argument(
        "--manifest",
        type=Path,
        default=_default_manifest(),
        help="path to projects/mamba/Cargo.toml (defaults to script's parent)",
    )
    parser.add_argument(
        "--release",
        action="store_true",
        help="use the release profile (matches the canonical MVP gate)",
    )
    parser.add_argument(
        "--json", action="store_true", help="emit machine-readable JSON"
    )
    parser.add_argument(
        "cargo_passthrough",
        nargs=argparse.REMAINDER,
        help="extra arguments forwarded to cargo test --list",
    )
    args = parser.parse_args(list(argv))

    if shutil.which("cargo") is None:
        sys.stderr.write("gate0_list_tests: cargo not found on PATH\n")
        return 3
    if not args.manifest.is_file():
        sys.stderr.write(
            f"gate0_list_tests: manifest not found: {args.manifest}\n"
        )
        return 2

    extra = [a for a in (args.cargo_passthrough or []) if a != "--"]
    result = run_gate(args.manifest, args.release, extra)

    out = render_json(result) if args.json else render_text(result)
    print(out)
    return 1 if result.exit_code != 0 else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
