#!/usr/bin/env python3.12
"""Fast runtime fix loop for CPython replacement slices.

This is the development inner loop, not a replacement readiness gate. It keeps
runtime work out of the slow Cargo/full-suite path:

* select changed CPython fixtures and their lib clusters,
* optionally run one guarded non-incremental debug build,
* validate with sweep.py against target/debug/mamba,
* optionally run per-cluster lint, promotion inventory, and CPython oracle.

Use replacement_readiness.py only at milestone boundaries.
"""

from __future__ import annotations

import argparse
import fcntl
import os
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path


TOOLS_DIR = Path(__file__).resolve().parent
MAMBA_DIR = TOOLS_DIR.parents[3]
WORKSPACE_DIR = MAMBA_DIR.parents[1]
FIXTURES_DIR = MAMBA_DIR / "tests" / "cpython"
SWEEP = TOOLS_DIR / "sweep.py"
FIXTURE_LINT = TOOLS_DIR / "fixture_lint.py"
PROMOTION_GATE = TOOLS_DIR / "promotion_gate.py"
VERIFY_ORACLE = TOOLS_DIR / "verify_cpython_oracle.py"
BUILD_LOCK = Path(os.environ.get("MAMBA_FAST_LOOP_BUILD_LOCK", "/tmp/mamba-fast-loop-cargo.lock"))


@dataclass
class StepResult:
    name: str
    code: int
    seconds: float


def run(
    argv: list[str],
    *,
    name: str,
    cwd: Path = WORKSPACE_DIR,
    env: dict[str, str] | None = None,
    env_note: str | None = None,
) -> StepResult:
    print(f"\n== {name}")
    print("+ " + " ".join(argv))
    if env_note:
        print(f"  env: {env_note}")
    start = time.monotonic()
    proc = subprocess.run(argv, cwd=cwd, env=env)
    seconds = time.monotonic() - start
    print(f"== {name}: exit={proc.returncode} wall={seconds:.2f}s")
    return StepResult(name=name, code=proc.returncode, seconds=seconds)


def git_changed(paths: list[str]) -> list[str]:
    cmd = ["git", "diff", "--name-only", "HEAD", "--", *paths]
    proc = subprocess.run(cmd, cwd=WORKSPACE_DIR, text=True, capture_output=True)
    if proc.returncode != 0:
        raise SystemExit(proc.stderr.strip() or "git diff failed")
    return [line.strip() for line in proc.stdout.splitlines() if line.strip()]


def normalize_fixture_arg(raw: str) -> str:
    path = Path(raw)
    if path.is_absolute():
        return path.relative_to(FIXTURES_DIR).as_posix()
    if raw.startswith("projects/mamba/tests/cpython/"):
        return Path(raw).relative_to("projects/mamba/tests/cpython").as_posix()
    if raw.startswith("tests/cpython/"):
        return Path(raw).relative_to("tests/cpython").as_posix()
    return raw


def normalize_repo_path(raw: str) -> str:
    path = Path(raw)
    if path.is_absolute():
        return path.relative_to(WORKSPACE_DIR).as_posix()
    if raw.startswith(("src/", "tests/")):
        return (Path("projects/mamba") / raw).as_posix()
    return raw


def changed_fixtures() -> list[str]:
    fixtures: list[str] = []
    for rel in git_changed(["projects/mamba/tests/cpython"]):
        if not rel.endswith(".py"):
            continue
        fixtures.append(normalize_fixture_arg(rel))
    return sorted(set(fixtures))


def changed_rust(paths: list[str] | None) -> list[str]:
    roots = [normalize_repo_path(path) for path in paths] if paths else ["projects/mamba/src"]
    return sorted(rel for rel in git_changed(roots) if rel.endswith(".rs"))


def cluster_for_fixture(rel: str) -> str | None:
    parts = Path(rel).parts
    if len(parts) >= 3:
        return "/".join(parts[:3])
    if len(parts) >= 2:
        return "/".join(parts[:-1])
    return None


def fixture_is_file(rel: str) -> bool:
    return (FIXTURES_DIR / rel).is_file()


def fixture_is_dir(rel: str) -> bool:
    return (FIXTURES_DIR / rel).is_dir()


def selected_paths(args: argparse.Namespace) -> tuple[list[str], list[str]]:
    exact: list[str] = []
    if args.changed:
        exact.extend(changed_fixtures())

    clusters: list[str] = []
    for item in (normalize_fixture_arg(item).rstrip("/") for item in args.paths):
        if fixture_is_file(item):
            exact.append(item)
        elif fixture_is_dir(item):
            clusters.append(item)

    exact = sorted(dict.fromkeys(item for item in exact if fixture_is_file(item)))
    if args.cluster:
        clusters.extend(cluster for item in exact if (cluster := cluster_for_fixture(item)))
    clusters.extend(normalize_fixture_arg(item).rstrip("/") for item in args.lib)
    clusters = sorted(dict.fromkeys(item for item in clusters if (FIXTURES_DIR / item).exists()))
    return exact, clusters


def mamba_bin(args: argparse.Namespace) -> Path:
    if args.mamba_bin:
        return Path(args.mamba_bin)
    if os.environ.get("MAMBA_BIN"):
        return Path(os.environ["MAMBA_BIN"])
    if args.target_dir:
        return Path(args.target_dir) / "debug" / "mamba"
    return WORKSPACE_DIR / "target" / "debug" / "mamba"


def stale_rust_files(binary: Path, rust_files: list[str]) -> list[str]:
    if not rust_files:
        return []
    if not binary.exists():
        return rust_files
    bin_mtime = binary.stat().st_mtime
    stale: list[str] = []
    for rel in rust_files:
        path = WORKSPACE_DIR / rel
        if path.exists() and path.stat().st_mtime > bin_mtime:
            stale.append(rel)
    return stale


def build_once(args: argparse.Namespace) -> StepResult:
    BUILD_LOCK.parent.mkdir(parents=True, exist_ok=True)
    with BUILD_LOCK.open("w") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            print(f"another runtime_fast_loop build owns {BUILD_LOCK}", file=sys.stderr)
            return StepResult("cargo build", 75, 0.0)
        lock.write(f"pid={os.getpid()}\n")
        lock.flush()
        env = dict(os.environ)
        notes: list[str] = []
        if not args.cargo_incremental:
            env["CARGO_INCREMENTAL"] = "0"
            notes.append("CARGO_INCREMENTAL=0")
        if args.target_dir:
            env["CARGO_TARGET_DIR"] = args.target_dir
            notes.append(f"CARGO_TARGET_DIR={args.target_dir}")
        if args.sccache and not env.get("RUSTC_WRAPPER"):
            if sccache := shutil.which("sccache"):
                env["RUSTC_WRAPPER"] = sccache
                notes.append(f"RUSTC_WRAPPER={sccache}")
        return run(
            ["cargo", "build", "-p", "mamba", "--bin", "mamba"],
            name="cargo build",
            env=env,
            env_note=", ".join(notes) if notes else None,
        )


def lint_args_for_cluster(cluster: str) -> list[str] | None:
    parts = cluster.split("/")
    if len(parts) < 3:
        return None
    dimension, bucket, lib = parts[:3]
    if dimension == "type":
        return ["--bucket", "type", "--lib", lib]
    return ["--bucket", bucket, "--lib", lib]


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="*", help="fixtures or directories relative to tests/cpython")
    parser.add_argument("--changed", action="store_true", help="include changed CPython fixtures")
    parser.add_argument("--lib", action="append", default=[], help="add a fixture lib cluster, e.g. behavior/std-libs/ast")
    parser.add_argument("--no-cluster", dest="cluster", action="store_false", help="do not add whole lib clusters for selected fixtures")
    parser.set_defaults(cluster=True)
    parser.add_argument("--build", action="store_true", help="run one guarded cargo build before validation")
    parser.add_argument(
        "--cargo-incremental",
        action="store_true",
        help="allow Cargo incremental during --build; default disables it to avoid stalled dev-loop rebuilds",
    )
    parser.add_argument("--target-dir", help="CARGO_TARGET_DIR for the optional build and default mamba-bin")
    parser.add_argument("--sccache", action="store_true", help="enable sccache for the optional build when it is usable on this host")
    parser.add_argument("--watch-rust", action="append", help="changed Rust roots that must be reflected in mamba_bin before validating")
    parser.add_argument("--allow-stale", action="store_true", help="allow validating with a binary older than changed Rust files")
    parser.add_argument("--mamba-bin", help="mamba binary to validate; defaults to target/debug/mamba or MAMBA_BIN")
    parser.add_argument("--jobs", type=int, default=min(8, os.cpu_count() or 4))
    parser.add_argument("--timeout", type=int, default=10)
    parser.add_argument("--lint", action="store_true", help="run fixture_lint for selected clusters")
    parser.add_argument("--promotion", action="store_true", help="run per-cluster promotion inventory")
    parser.add_argument("--oracle", action="store_true", help="run CPython oracle for selected clusters")
    args = parser.parse_args(argv)

    exact, clusters = selected_paths(args)
    if not exact and not clusters:
        parser.error("nothing selected; pass fixtures, --changed, or --lib")

    results: list[StepResult] = []
    if args.build:
        results.append(build_once(args))
        if results[-1].code != 0:
            return results[-1].code

    binary = mamba_bin(args)
    rust_delta = changed_rust(args.watch_rust)
    stale = stale_rust_files(binary, rust_delta)
    if stale and not args.allow_stale:
        print(
            f"{binary} is older than {len(stale)} changed Rust file(s); "
            "rerun with --build or --allow-stale.",
            file=sys.stderr,
        )
        for rel in stale[:20]:
            print(f"  stale: {rel}", file=sys.stderr)
        if len(stale) > 20:
            print(f"  ... {len(stale) - 20} more", file=sys.stderr)
        return 65

    env = dict(os.environ, MAMBA_BIN=str(binary))
    if exact:
        results.append(run(
            [sys.executable, str(SWEEP), *exact, "--jobs", str(args.jobs), "--timeout", str(args.timeout)],
            name=f"sweep exact ({len(exact)})",
            env=env,
        ))
    if clusters:
        results.append(run(
            [sys.executable, str(SWEEP), *clusters, "--jobs", str(args.jobs), "--timeout", str(args.timeout)],
            name=f"sweep clusters ({len(clusters)})",
            env=env,
        ))

    for cluster in clusters:
        if args.lint and (lint_args := lint_args_for_cluster(cluster)):
            results.append(run([sys.executable, str(FIXTURE_LINT), *lint_args], name=f"fixture_lint {cluster}"))
        if args.promotion:
            results.append(run([
                sys.executable,
                str(PROMOTION_GATE),
                "--root",
                str(FIXTURES_DIR / cluster),
                "--manifest",
                "",
                "--profile",
                "inventory",
                "--show",
                "5",
            ], name=f"promotion inventory {cluster}"))
        if args.oracle:
            results.append(run([
                sys.executable,
                str(VERIFY_ORACLE),
                "--root",
                str(FIXTURES_DIR / cluster),
                "--ready-only",
                "--jobs",
                str(args.jobs),
                "--progress-every",
                "0",
            ], name=f"cpython oracle {cluster}"))

    print("\nfast-loop summary:")
    for result in results:
        print(f"  {result.name}: exit={result.code} wall={result.seconds:.2f}s")
    return 1 if any(result.code != 0 for result in results) else 0


if __name__ == "__main__":
    raise SystemExit(main())
