#!/usr/bin/env python3
"""MVP release-gate command runner (closes #2821).

Parent: #2775 (MVP release blocking CI profiles).

Orchestrates the six release-blocking MVP profiles documented in
`projects/mamba/validation/mvp.toml`:

    smoke, correctness, performance, ecosystem, package_manager, mambalibs

Reads the per-profile manifests under `validation/profiles/*.toml`, runs
each profile's documented command in declaration order, captures exit
status + JSON output, and emits a release summary that matches the
schema locked by #2820 (`validation/schemas/release_summary.schema.json`).

Acceptance (issue #2821):

    1. Runner returns nonzero when any release-blocking profile fails.
       Exit code is the count of blocking failures (capped at 99), or 0
       on a clean run. Non-blocking ("report_only") profile failures do
       not affect the exit code, by design.
    2. Runner writes summaries to a predictable output directory.
       `--output-dir` defaults to `/tmp/mamba/release/`. The summary is
       written as `<release_id>.summary.json` inside that directory.
    3. Runner does not require network for default profiles. The per-
       profile manifests already declare `[policy].offline = true` /
       `[policy].network = "offline"` for the default required buckets;
       this script never adds environment variables or sockets of its
       own. The opt-in `--include-live-network` flag only flips the
       package-manager profile into its live-network bucket.

Operating modes
---------------

`--dry-run`
    Skip command execution entirely. Synthesize a "pass" outcome for
    every profile listed in the inventory, write the resulting summary
    to disk, and exit 0. Used by the schema test and by CI smoke checks
    to verify the runner's plumbing without booting cargo.

`--simulate <profile>=<status>` (repeatable)
    For each `<profile>=<status>` pair, override that profile's outcome
    to the given status (`pass`, `fail`, or `skip`). Implies
    `--dry-run`. Lets tests exercise the "nonzero on blocking failure"
    contract without actually breaking a real run.

`--profile <id>` (repeatable)
    Restrict the run to the listed profile ids. Other profiles are
    omitted from the summary.

`--include-live-network`
    Forward the package-manager profile's documented opt-in flag.
    Without this flag, only offline buckets are exercised.

`--release-id <id>`
    Override the synthesized release id. Default is
    `mamba-mvp-dryrun-<utc-yyyymmddTHHMMSSZ>`.

`--manifest <path>`
    Override the path to `validation/mvp.toml`. Default is resolved
    relative to this script (`../validation/mvp.toml`).

Exit codes
----------

    0   every blocking profile passed (or `--dry-run` requested).
    N   N blocking profiles failed (capped at 99).
    100 usage / argument error.
    101 manifest or per-profile TOML missing / unreadable.

The script is pure stdlib (Python 3.11+ for tomllib). It must not grow
runtime dependencies — the release gate has to be runnable on the
plainest CI image we ship.
"""

from __future__ import annotations

import argparse
import datetime as _dt
import json
import os
import subprocess
import sys
import tomllib
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


SCHEMA_VERSION = 1
DEFAULT_OUTPUT_DIR = "/tmp/mamba/release"

EXIT_USAGE = 100
EXIT_MANIFEST = 101
EXIT_FAILURE_CAP = 99

PROFILE_ORDER = (
    "smoke",
    "correctness",
    "performance",
    "ecosystem",
    "package_manager",
    "mambalibs",
)

OBJECTIVE_FOR_PROFILE = {p: p for p in PROFILE_ORDER}


@dataclass
class ProfileSpec:
    """One row from the inventory plus its per-profile manifest."""

    profile_id: str
    required: bool
    manifest_path: Path
    command: str = ""
    blocking: bool = True

    @property
    def objective(self) -> str:
        return OBJECTIVE_FOR_PROFILE.get(self.profile_id, self.profile_id)


@dataclass
class ProfileResult:
    profile_id: str
    command: str
    status: str
    blocking: bool
    counts: dict[str, int] = field(default_factory=dict)
    blockers: list[dict[str, Any]] = field(default_factory=list)

    def to_summary(self) -> dict[str, Any]:
        rolled = {"passed": 0, "failed": 0, "missing": 0}
        rolled.update(self.counts)
        return {
            "profile": self.profile_id,
            "command": self.command,
            "status": self.status,
            "blocking": self.blocking,
            "counts": rolled,
        }


def _die(code: int, msg: str) -> None:
    sys.stderr.write(f"release_gate: {msg}\n")
    sys.exit(code)


def _load_inventory(manifest_path: Path) -> list[ProfileSpec]:
    if not manifest_path.is_file():
        _die(EXIT_MANIFEST, f"inventory not found: {manifest_path}")
    raw = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    profiles = raw.get("profiles") or {}
    specs: list[ProfileSpec] = []
    for pid in PROFILE_ORDER:
        entry = profiles.get(pid)
        if entry is None:
            continue
        required = bool(entry.get("required", False))
        rel = entry.get("manifest")
        if rel is None:
            continue
        manifest_file = (manifest_path.parent / rel).resolve()
        specs.append(
            ProfileSpec(
                profile_id=pid,
                required=required,
                manifest_path=manifest_file,
            )
        )
    if not specs:
        _die(EXIT_MANIFEST, f"no profile manifests resolved from {manifest_path}")
    return specs


def _load_profile_command(spec: ProfileSpec) -> None:
    if not spec.manifest_path.is_file():
        _die(EXIT_MANIFEST, f"profile manifest missing: {spec.manifest_path}")
    data = tomllib.loads(spec.manifest_path.read_text(encoding="utf-8"))
    spec.command = _extract_command(spec.profile_id, data)
    spec.blocking = spec.required


def _extract_command(profile_id: str, data: dict[str, Any]) -> str:
    if profile_id == "smoke":
        cmds = data.get("commands", {})
        first = cmds.get("compile_list", {})
        return first.get("command", "")
    if profile_id == "correctness":
        return "cargo test -p mamba --tests --quiet"
    if profile_id == "performance":
        return "cargo bench --bench cross_runtime -- --json"
    if profile_id == "ecosystem":
        return "cargo test -p mamba --test conformance_real_world"
    if profile_id == "package_manager":
        return "cargo test -p mamba --test pkgmgr_workflows"
    if profile_id == "mambalibs":
        return "cargo test -p mamba --test mambalibs_pipeline"
    return ""


def _parse_simulate(values: list[str]) -> dict[str, str]:
    out: dict[str, str] = {}
    for raw in values:
        if "=" not in raw:
            _die(EXIT_USAGE, f"--simulate value must be profile=status: {raw!r}")
        pid, status = raw.split("=", 1)
        pid = pid.strip()
        status = status.strip()
        if status not in ("pass", "fail", "skip"):
            _die(EXIT_USAGE, f"--simulate status must be pass/fail/skip: {status!r}")
        out[pid] = status
    return out


def _run_profile(spec: ProfileSpec, *, cwd: Path, env: dict[str, str]) -> ProfileResult:
    proc = subprocess.run(
        spec.command,
        shell=True,
        cwd=cwd,
        env=env,
        capture_output=True,
        text=True,
    )
    status = "pass" if proc.returncode == 0 else "fail"
    counts: dict[str, int] = {"passed": 0, "failed": 0, "missing": 0}
    blockers: list[dict[str, Any]] = []
    parsed = _maybe_parse_json(proc.stdout)
    if isinstance(parsed, dict):
        for key in ("passed", "failed", "missing", "skipped", "xfail", "stub", "import_pass", "silent_pass"):
            v = parsed.get(key)
            if isinstance(v, int) and v >= 0:
                counts[key] = v
    if status == "fail":
        counts["failed"] = max(counts.get("failed", 0), 1)
        blockers.append({
            "profile": spec.profile_id,
            "fixture_id": "<unknown>",
            "kind": "failed",
            "reason": (proc.stderr.strip().splitlines() or ["nonzero exit"])[-1],
            "tracking_issue": None,
        })
    return ProfileResult(
        profile_id=spec.profile_id,
        command=spec.command,
        status=status,
        blocking=spec.blocking,
        counts=counts,
        blockers=blockers,
    )


def _maybe_parse_json(text: str) -> Any:
    text = text.strip()
    if not text:
        return None
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        return None


def _synthesize_result(spec: ProfileSpec, status: str) -> ProfileResult:
    counts = {"passed": 1, "failed": 0, "missing": 0}
    blockers: list[dict[str, Any]] = []
    if status == "fail":
        counts = {"passed": 0, "failed": 1, "missing": 0}
        blockers.append({
            "profile": spec.profile_id,
            "fixture_id": "<dry-run>",
            "kind": "failed",
            "reason": "simulated failure (--simulate)",
            "tracking_issue": None,
        })
    elif status == "skip":
        counts = {"passed": 0, "failed": 0, "missing": 0, "skipped": 1}
    return ProfileResult(
        profile_id=spec.profile_id,
        command=spec.command,
        status=status,
        blocking=spec.blocking,
        counts=counts,
        blockers=blockers,
    )


def _build_summary(
    results: list[ProfileResult],
    *,
    release_id: str,
    summary_path: Path,
    cpython: str,
    mamba_edition: str,
) -> dict[str, Any]:
    release_required = [r.profile_id for r in results if r.blocking]
    report_only = [r.profile_id for r in results if not r.blocking]
    blocking_failures = sum(1 for r in results if r.blocking and r.status == "fail")
    blockers_by_objective: dict[str, list[dict[str, Any]]] = {}
    for r in results:
        if not r.blockers:
            continue
        objective = OBJECTIVE_FOR_PROFILE.get(r.profile_id, r.profile_id)
        blockers_by_objective.setdefault(objective, []).extend(r.blockers)

    return {
        "schema_version": SCHEMA_VERSION,
        "release_id": release_id,
        "generated_at": _utcnow_iso(),
        "runtime_identity": {
            "cpython": cpython,
            "mamba_edition": mamba_edition,
        },
        "overall": {
            "pass": blocking_failures == 0,
            "blocking_failure_count": blocking_failures,
            "release_required_profiles": release_required,
            "report_only_profiles": report_only,
        },
        "profiles": {r.profile_id: r.to_summary() for r in results},
        "blockers_by_objective": blockers_by_objective,
        "artifacts": {
            "summary_path": str(summary_path),
            "logs_path": None,
            "baseline_path": None,
            "lockfile_path": None,
            "environment_path": None,
            "project_path": None,
        },
    }


def _utcnow_iso() -> str:
    return _dt.datetime.now(_dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def _default_release_id() -> str:
    return "mamba-mvp-dryrun-" + _dt.datetime.now(_dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def _parse_args(argv: list[str]) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        prog="release_gate",
        description="MVP release-gate orchestrator (#2821).",
    )
    p.add_argument("--manifest", type=Path, default=None,
                   help="path to validation/mvp.toml (default: relative to this script)")
    p.add_argument("--output-dir", type=Path, default=Path(DEFAULT_OUTPUT_DIR),
                   help=f"directory where summary JSON is written (default: {DEFAULT_OUTPUT_DIR})")
    p.add_argument("--release-id", default=None,
                   help="stable identifier for this run (default: synthesized)")
    p.add_argument("--profile", action="append", default=[],
                   help="restrict run to this profile id; repeatable")
    p.add_argument("--dry-run", action="store_true",
                   help="skip command execution; synthesize a pass for every profile")
    p.add_argument("--simulate", action="append", default=[],
                   metavar="PROFILE=STATUS",
                   help="override a profile's outcome (pass/fail/skip); implies --dry-run")
    p.add_argument("--include-live-network", action="store_true",
                   help="enable package-manager live-network workflows (opt-in)")
    p.add_argument("--cpython", default="3.12",
                   help="runtime_identity.cpython value to record (default: 3.12)")
    p.add_argument("--mamba-edition", default="py312",
                   help="runtime_identity.mamba_edition value to record (default: py312)")
    return p.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    ns = _parse_args(list(sys.argv[1:] if argv is None else argv))

    script_dir = Path(__file__).resolve().parent
    project_root = script_dir.parent
    manifest_path = ns.manifest or (project_root / "validation" / "mvp.toml")

    specs = _load_inventory(manifest_path.resolve())
    for spec in specs:
        _load_profile_command(spec)

    if ns.profile:
        keep = set(ns.profile)
        specs = [s for s in specs if s.profile_id in keep]
        if not specs:
            _die(EXIT_USAGE, f"no profiles matched --profile filter: {ns.profile}")

    simulate = _parse_simulate(ns.simulate)
    dry_run = ns.dry_run or bool(simulate)

    release_id = ns.release_id or _default_release_id()
    out_dir = ns.output_dir.resolve()
    out_dir.mkdir(parents=True, exist_ok=True)
    summary_path = out_dir / f"{release_id}.summary.json"

    if dry_run:
        results = [_synthesize_result(s, simulate.get(s.profile_id, "pass")) for s in specs]
    else:
        env = dict(os.environ)
        if ns.include_live_network:
            env["MAMBA_RELEASE_GATE_LIVE_NETWORK"] = "1"
        results = [_run_profile(s, cwd=project_root, env=env) for s in specs]

    summary = _build_summary(
        results,
        release_id=release_id,
        summary_path=summary_path,
        cpython=ns.cpython,
        mamba_edition=ns.mamba_edition,
    )
    summary_path.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")

    blocking_failures = summary["overall"]["blocking_failure_count"]
    return min(blocking_failures, EXIT_FAILURE_CAP)


if __name__ == "__main__":
    sys.exit(main())
