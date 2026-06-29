#!/usr/bin/env python3.12
"""Single CPython replacement-readiness report.

This command is intentionally stricter than development conformance. It reports
the current replacement blockers across readiness dimensions and exits nonzero
until every dimension is proven green. Red/blocked output is valid evidence.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


TOOLS_DIR = Path(__file__).resolve().parent
MAMBA_DIR = TOOLS_DIR.parents[3]
WORKSPACE_DIR = MAMBA_DIR.parents[1]
PROMOTION_GATE = TOOLS_DIR / "promotion_gate.py"
DENOMINATOR_INVENTORY = MAMBA_DIR / "tools" / "cpython_regrtest_inventory.py"
STRICT_TYPE_ACCOUNTING = TOOLS_DIR / "strict_type_accounting.py"
GATE_CHECK = TOOLS_DIR / "gate_check.py"
PLATFORM_READINESS = TOOLS_DIR / "platform_readiness.py"
IMPORT_READINESS = TOOLS_DIR / "import_readiness.py"
THIRD_PARTY_READINESS = TOOLS_DIR / "third_party_readiness.py"
MAMBALIBS_READINESS = TOOLS_DIR / "mambalibs_readiness.py"
DEBUGGER_READINESS = TOOLS_DIR / "debugger_readiness.py"
CONCURRENCY_READINESS = TOOLS_DIR / "concurrency_readiness.py"

EXIT_NOT_READY = 70


@dataclass
class Dimension:
    id: str
    title: str
    status: str
    owner_issue: str
    summary: str
    counts: dict[str, Any] = field(default_factory=dict)
    evidence: list[str] = field(default_factory=list)
    blockers: list[dict[str, Any]] = field(default_factory=list)

    def to_json(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "title": self.title,
            "status": self.status,
            "owner_issue": self.owner_issue,
            "summary": self.summary,
            "counts": self.counts,
            "evidence": self.evidence,
            "blockers": self.blockers,
        }


def run_json(
    argv: list[str], *, accepted: set[int], cwd: Path | None = None
) -> tuple[int, dict[str, Any]]:
    proc = subprocess.run(argv, text=True, capture_output=True, cwd=cwd)
    if proc.returncode not in accepted:
        raise RuntimeError(
            f"{' '.join(argv)} exited {proc.returncode}\n"
            f"stdout={proc.stdout}\nstderr={proc.stderr}"
        )
    candidates = [
        line.strip()
        for line in proc.stdout.splitlines()
        if line.strip().startswith("{") and line.strip().endswith("}")
    ]
    stdout = candidates[-1] if candidates else proc.stdout
    try:
        return proc.returncode, json.loads(stdout)
    except json.JSONDecodeError as exc:
        raise RuntimeError(
            f"{' '.join(argv)} did not emit JSON: {exc}\nstdout={proc.stdout}"
        ) from exc


def promotion_dimension(show: int) -> Dimension:
    code, payload = run_json(
        [
            sys.executable,
            str(PROMOTION_GATE),
            "--profile",
            "replacement",
            "--json",
            "--show",
            str(show),
        ],
        accepted={0, EXIT_NOT_READY},
    )
    total = int(payload["promotion_debt_total"])
    status = "green" if code == 0 and total == 0 else "red"
    blockers = []
    for key in ("unowned_debt", "promotion_pending_debt", "xfail_debt", "skip_debt"):
        for item in payload.get(key, [])[:show]:
            blockers.append(
                {
                    "kind": item["kind"],
                    "path": item["path"],
                    "owner_refs": item["owner_refs"],
                    "reason": item["reason"],
                    "source": key,
                }
            )
    return Dimension(
        id="promotion_debt",
        title="No xfail/skip/optional promotion debt",
        status=status,
        owner_issue="#703",
        summary=(
            "replacement profile has no xfail/skip/optional debt"
            if status == "green"
            else f"replacement profile has {total} promotion blockers"
        ),
        counts={
            "scanned_files": payload["scanned_files"],
            "xfail": payload["xfail_count"],
            "skip": payload["skip_count"],
            "optional": payload["optional_count"],
            "total": total,
            "owned": payload["owned_count"],
            "unowned": payload["unowned_count"],
            "promotion_pending": payload["promotion_pending_count"],
            "parse_errors": payload["parse_error_count"],
        },
        evidence=[
            "python3.12 projects/mamba/tests/harness/cpython/tools/promotion_gate.py --profile replacement --json"
        ],
        blockers=blockers,
    )


def denominator_dimension(show: int) -> Dimension:
    _, payload = run_json(
        [
            sys.executable,
            str(DENOMINATOR_INVENTORY),
            "--json",
            "--top",
            str(show),
        ],
        accepted={0},
    )
    ownership = payload["denominator_ownership"]
    status = "green" if ownership["pass"] else "uncovered"
    blockers = [
        {
            "kind": "unowned_denominator_module",
            "module": item["module"],
            "key": item["key"],
            "static_test_defs": item["static_test_defs"],
        }
        for item in payload["top_no_fixture_lib_or_source_match"][:show]
    ]
    return Dimension(
        id="cpython_denominator",
        title="CPython 3.12 Lib/test denominator ownership",
        status=status,
        owner_issue="#706",
        summary=(
            "all CPython regrtest modules have fixture/source ownership"
            if status == "green"
            else f"{ownership['unowned_modules']} CPython regrtest modules lack fixture/source ownership"
        ),
        counts={
            "python": payload["python"],
            "test_py_files": payload["cpython_test_case_candidates"]["test_py_files"],
            "regrtest_modules": payload["cpython_regrtest_modules"],
            "static_test_defs_in_regrtest_modules": payload[
                "static_test_defs_in_regrtest_modules"
            ],
            "owned_modules": ownership["owned_modules"],
            "unowned_modules": ownership["unowned_modules"],
            "owned_static_test_defs": ownership["owned_static_test_defs"],
            "unowned_static_test_defs": ownership["unowned_static_test_defs"],
            "invalid_metadata_files": ownership["invalid_metadata_files"],
            "parse_errors": ownership["parse_errors"],
        },
        evidence=["python3.12 projects/mamba/tools/cpython_regrtest_inventory.py --json"],
        blockers=blockers,
    )


def strict_type_dimension(show: int, type_limit: int) -> Dimension:
    argv = [
        sys.executable,
        str(STRICT_TYPE_ACCOUNTING),
        "--json",
        "--show",
        str(show),
    ]
    if type_limit:
        argv.extend(["--limit", str(type_limit)])
    code, payload = run_json(argv, accepted={0, EXIT_NOT_READY})
    status = "green" if code == 0 and payload["ready"] else "red"
    typeshed = payload["typeshed"]
    enforcement = payload["enforcement"]
    soundness = payload["soundness"]
    divergences = payload["divergences"]
    blockers = payload.get("blockers", [])[:show]
    return Dimension(
        id="strict_type_accounting",
        title="Strict-type divergence and typeshed accounting",
        status=status,
        owner_issue="#704",
        summary=(
            "strict-type denominator, enforcement, soundness, and divergence accounting are green"
            if status == "green"
            else (
                "strict-type accounting is not replacement-ready: "
                f"enforced={enforcement['enforced']}/{enforcement['gradable']} "
                f"gradable, soundness={soundness['passed']}/{soundness['gradable']} "
                f"gradable, divergences valid={divergences['valid']}/{divergences['declared']}"
            )
        ),
        counts={
            "typeshed_rows": typeshed["rows"],
            "typeshed_enforceable": typeshed["enforceable"],
            "typeshed_unknown_skipped": typeshed["unknown_skipped"],
            "type_fixture_wall": typeshed["type_fixture_wall"],
            "type_measured": enforcement["measured"],
            "type_gradable": enforcement["gradable"],
            "type_enforced": enforcement["enforced"],
            "type_leaked": enforcement["leaked"],
            "type_ungradable": enforcement["ungradable"],
            "soundness_measured": soundness["measured"],
            "soundness_gradable": soundness["gradable"],
            "soundness_passed": soundness["passed"],
            "soundness_failed": soundness["failed"],
            "soundness_oracle_skip": soundness["oracle_skip"],
            "declared_divergences": divergences["declared"],
            "valid_divergences": divergences["valid"],
            "invalid_divergences": divergences["invalid"],
            "missing_divergence_owner": divergences["missing_owner"],
            "sampled": payload["sampled"],
        },
        evidence=[
            "python3.12 projects/mamba/tests/harness/cpython/tools/strict_type_accounting.py --json",
            "python3.12 projects/mamba/tests/harness/cpython/tools/fixture_lint.py --bucket type",
        ],
        blockers=blockers,
    )


def perf_dimension(show: int) -> Dimension:
    _, payload = run_json(
        ["cargo", "test", "-p", "mamba", "--test", "cpython_status", "--", "--json"],
        accepted={0},
        cwd=WORKSPACE_DIR,
    )
    perf = payload["perf"]
    blocker_counts = {
        "baseline_missing_rows": perf["baseline_missing_rows"],
        "baseline_stale_rows": perf["baseline_stale_rows"],
        "baseline_missing_cpu_rows": perf["baseline_missing_cpu_rows"],
        "baseline_missing_rss_rows": perf["baseline_missing_rss_rows"],
        "missing_fixture_count": perf["missing_fixture_count"],
        "missing_prereq_import_count": perf["missing_prereq_import_count"],
        "malformed_pin_count": perf["malformed_pin_count"],
        "query_error": 1 if perf["query_error"] else 0,
    }
    total_blockers = sum(int(value) for value in blocker_counts.values())
    status = (
        "green"
        if perf["pins"] > 0 and perf["baseline_db_exists"] and total_blockers == 0
        else "red"
    )

    blockers: list[dict[str, Any]] = []
    for source, kind in (
        ("missing_row_pins", "missing_baseline_row"),
        ("recordable_missing_row_pins", "recordable_missing_baseline_row"),
        ("stale_row_pins", "stale_baseline_row"),
        ("missing_cpu_pins", "missing_cpu_sample"),
        ("missing_rss_pins", "missing_rss_sample"),
        ("missing_fixture_pins", "missing_perf_fixture"),
        ("missing_prereq_import_pins", "missing_prereq_import"),
    ):
        for item in perf.get(source, [])[:show]:
            blockers.append({"kind": kind, "source": source, **item})
    for item in perf.get("malformed_pins", [])[:show]:
        blockers.append({"kind": "malformed_perf_pin", "detail": item})
    if perf["query_error"]:
        blockers.append(
            {"kind": "baseline_query_error", "detail": perf["query_error"]}
        )

    return Dimension(
        id="perf_rss_baselines",
        title="Performance and peak-RSS baseline gate",
        status=status,
        owner_issue="#707",
        summary=(
            "all perf pins have comparable CPython CPU and peak-RSS baseline rows"
            if status == "green"
            else (
                "perf/RSS baseline is not replacement-ready: "
                f"{perf['baseline_missing_rows']} missing rows, "
                f"{perf['baseline_missing_cpu_rows']} missing CPU rows, "
                f"{perf['baseline_missing_rss_rows']} missing RSS rows, "
                f"{perf['missing_prereq_import_count']} missing prereq imports"
            )
        ),
        counts={
            "pins": perf["pins"],
            "baseline_db_exists": perf["baseline_db_exists"],
            "baseline_rows": perf["baseline_rows"],
            "baseline_missing_rows": perf["baseline_missing_rows"],
            "baseline_recordable_missing_rows": perf["baseline_recordable_missing_rows"],
            "baseline_stale_rows": perf["baseline_stale_rows"],
            "baseline_missing_cpu_rows": perf["baseline_missing_cpu_rows"],
            "baseline_missing_rss_rows": perf["baseline_missing_rss_rows"],
            "missing_fixture_count": perf["missing_fixture_count"],
            "missing_prereq_import_count": perf["missing_prereq_import_count"],
            "malformed_pin_count": perf["malformed_pin_count"],
            "query_error": perf["query_error"],
        },
        evidence=[
            "cargo test -p mamba --test cpython_status -- --json",
            "python3.12 projects/mamba/tests/harness/cpython/tools/perf_baseline.py record --missing-only --ready-only --limit 10 --keep-going",
        ],
        blockers=blockers[:show],
    )


def safety_dimension(show: int) -> Dimension:
    code, payload = run_json(
        [sys.executable, str(GATE_CHECK), "--json"],
        accepted={0, 1},
    )
    criteria = payload["criteria"]
    failed = [item for item in criteria if not item["passed"]]
    d4 = next((item for item in criteria if item["id"] == "D4"), None)
    status = "green" if code == 0 and not failed else "red"
    blockers = [
        {
            "kind": "safety_gate_criterion_failed",
            "id": item["id"],
            "name": item["name"],
            "detail": item["detail"],
        }
        for item in failed[:show]
    ]
    return Dimension(
        id="safety_stability_security",
        title="Safety, stability, leak, crash, and secret-leak gates",
        status=status,
        owner_issue="#709",
        summary=(
            "safety/stability production gate is green"
            if status == "green"
            else (
                "safety/stability production gate is not replacement-ready: "
                f"{payload['met']}/{payload['total']} criteria met"
            )
        ),
        counts={
            "criteria_total": payload["total"],
            "criteria_met": payload["met"],
            "criteria_failed": len(failed),
            "d4_safety_passed": d4["passed"] if d4 else False,
            "d4_safety_detail": d4["detail"] if d4 else "missing D4 criterion",
        },
        evidence=[
            "python3.12 projects/mamba/tests/harness/cpython/tools/gate_check.py --json",
            "cargo test -p mamba -- --list",
            "python3.12 projects/mamba/tests/harness/cpython/tools/verify_cpython_oracle.py --help",
        ],
        blockers=blockers,
    )


def platform_dimension(show: int) -> Dimension:
    code, payload = run_json(
        [sys.executable, str(PLATFORM_READINESS), "--json", "--show", str(show)],
        accepted={0, EXIT_NOT_READY},
    )
    counts = payload["counts"]
    status = "green" if code == 0 and payload["ready"] else "red"
    return Dimension(
        id="platform_os_process_network_tls",
        title="Platform, OS, process, network, and TLS coverage",
        status=status,
        owner_issue="#710",
        summary=(
            "platform/OS/process/network/TLS readiness is green"
            if status == "green"
            else (
                "platform/OS/process/network/TLS readiness is not replacement-ready: "
                f"{counts['fixtures']} fixtures, {counts['unmeasured']} unmeasured, "
                f"{counts['promotion_pending']} promotion-pending, "
                f"{counts['runtime_failure_debt']} runtime-debt, "
                f"{counts['sandbox_denied']} sandbox-denied, "
                f"{counts['unsupported_platform']} unsupported-platform"
            )
        ),
        counts={
            "fixtures": counts["fixtures"],
            "target_libs": counts["target_libs"],
            "missing_target_libs": counts["missing_target_libs"],
            "pass_candidate": counts["pass_candidate"],
            "promotion_pending": counts["promotion_pending"],
            "runtime_failure_debt": counts["runtime_failure_debt"],
            "sandbox_denied": counts["sandbox_denied"],
            "unsupported_platform": counts["unsupported_platform"],
            "runtime_ok": counts["runtime_ok"],
            "runtime_fail": counts["runtime_fail"],
            "runtime_timeout": counts["runtime_timeout"],
            "runtime_crash": counts["runtime_crash"],
            "unmeasured": counts["unmeasured"],
            "unowned_gap_count": counts["unowned_gap_count"],
            "perf_pins": counts["perf_pins"],
            "malformed_perf_pins": counts["malformed_perf_pins"],
        },
        evidence=payload["evidence_commands"],
        blockers=payload["blockers"][:show],
    )


def import_dimension(show: int) -> Dimension:
    code, payload = run_json(
        [sys.executable, str(IMPORT_READINESS), "--json", "--show", str(show)],
        accepted={0, EXIT_NOT_READY},
    )
    counts = payload["counts"]
    status = "green" if code == 0 and payload["ready"] else "red"
    return Dimension(
        id="import_package_module_system",
        title="Import, package, and module-system semantics",
        status=status,
        owner_issue="#708",
        summary=(
            "import/package/module-system readiness is green"
            if status == "green"
            else (
                "import/package/module-system readiness is not replacement-ready: "
                f"{counts['fixtures']} fixtures, {counts['unmeasured']} unmeasured, "
                f"{counts['promotion_pending']} promotion-pending, "
                f"{counts['runtime_failure_debt']} runtime-debt, "
                f"{counts['legacy_regression_unaccounted']} legacy-regression, "
                f"{counts['missing_semantic_classes']} missing semantic classes"
            )
        ),
        counts={
            "fixtures": counts["fixtures"],
            "scopes": counts["scopes"],
            "missing_scopes": counts["missing_scopes"],
            "semantic_classes": counts["semantic_classes"],
            "missing_semantic_classes": counts["missing_semantic_classes"],
            "pass_candidate": counts["pass_candidate"],
            "promotion_pending": counts["promotion_pending"],
            "runtime_failure_debt": counts["runtime_failure_debt"],
            "legacy_regression_unaccounted": counts[
                "legacy_regression_unaccounted"
            ],
            "sandbox_denied": counts["sandbox_denied"],
            "unsupported_platform": counts["unsupported_platform"],
            "runtime_ok": counts["runtime_ok"],
            "runtime_fail": counts["runtime_fail"],
            "runtime_timeout": counts["runtime_timeout"],
            "runtime_crash": counts["runtime_crash"],
            "unmeasured": counts["unmeasured"],
            "unowned_gap_count": counts["unowned_gap_count"],
        },
        evidence=payload["evidence_commands"],
        blockers=payload["blockers"][:show],
    )


def third_party_dimension(show: int) -> Dimension:
    code, payload = run_json(
        [sys.executable, str(THIRD_PARTY_READINESS), "--json", "--show", str(show)],
        accepted={0, EXIT_NOT_READY},
    )
    counts = payload["counts"]
    status = "green" if code == 0 and payload["ready"] else "red"
    return Dimension(
        id="third_party_c_extension_strategy",
        title="Third-party and C-extension compatibility strategy",
        status=status,
        owner_issue="#711",
        summary=(
            "third-party package and C-extension readiness is green"
            if status == "green"
            else (
                "third-party/C-extension readiness is not replacement-ready: "
                f"{counts['packages']} packages, {counts['ready_packages']} ready, "
                f"{counts['unmeasured']} unmeasured fixture rows, "
                f"{counts['blocked_c_extension_packages']} mandatory C-extension packages, "
                f"{counts['blocked_optional_native_packages']} optional-native packages"
            )
        ),
        counts={
            "packages": counts["packages"],
            "fixtures": counts["fixtures"],
            "tiers": counts["tiers"],
            "missing_tiers": counts["missing_tiers"],
            "ready_packages": counts["ready_packages"],
            "not_ready_packages": counts["not_ready_packages"],
            "blocked_c_extension_packages": counts["blocked_c_extension_packages"],
            "blocked_optional_native_packages": counts[
                "blocked_optional_native_packages"
            ],
            "unclassified_packages": counts["unclassified_packages"],
            "mamba_native_replacement_packages": counts[
                "mamba_native_replacement_packages"
            ],
            "pure_python_packages": counts["pure_python_packages"],
            "optional_native_acceleration_packages": counts[
                "optional_native_acceleration_packages"
            ],
            "mandatory_c_extension_packages": counts[
                "mandatory_c_extension_packages"
            ],
            "managed_environment_gates": counts["managed_environment_gates"],
            "missing_managed_environment_gates": counts[
                "missing_managed_environment_gates"
            ],
            "schema_manifest_packages": counts["schema_manifest_packages"],
            "ecosystem_required_3p_packages": counts[
                "ecosystem_required_3p_packages"
            ],
            "provider_packages": counts["provider_packages"],
            "mambalibs_native_fixture_count": counts[
                "mambalibs_native_fixture_count"
            ],
            "mambalibs_native_import_gate_count": counts[
                "mambalibs_native_import_gate_count"
            ],
            "runtime_ok": counts["runtime_ok"],
            "runtime_fail": counts["runtime_fail"],
            "runtime_timeout": counts["runtime_timeout"],
            "runtime_crash": counts["runtime_crash"],
            "unmeasured": counts["unmeasured"],
            "stub_real_world_fixtures": counts["stub_real_world_fixtures"],
            "missing_import_smoke_packages": counts[
                "missing_import_smoke_packages"
            ],
            "missing_behavior_smoke_packages": counts[
                "missing_behavior_smoke_packages"
            ],
            "missing_install_run_manifest_packages": counts[
                "missing_install_run_manifest_packages"
            ],
            "unowned_gap_count": counts["unowned_gap_count"],
        },
        evidence=payload["evidence_commands"],
        blockers=payload["blockers"][:show],
    )


def mambalibs_dimension(show: int) -> Dimension:
    code, payload = run_json(
        [sys.executable, str(MAMBALIBS_READINESS), "--json", "--show", str(show)],
        accepted={0, EXIT_NOT_READY},
    )
    counts = payload["counts"]
    status = "green" if code == 0 and payload["ready"] else "red"
    return Dimension(
        id="mambalibs_native_kit",
        title="Mambalibs native-kit completeness",
        status=status,
        owner_issue="#714",
        summary=(
            "mambalibs native-kit readiness is green"
            if status == "green"
            else (
                "mambalibs/native-kit readiness is not replacement-ready: "
                f"{counts['native_kits']} native kits, {counts['pass']} pass, "
                f"{counts['fail']} fail, {counts['blocker']} blocked, "
                f"{counts['import_gate_manifests']} import gates"
            )
        ),
        counts={
            "native_kits": counts["native_kits"],
            "pass": counts["pass"],
            "fail": counts["fail"],
            "blocker": counts["blocker"],
            "fixture_manifests": counts["fixture_manifests"],
            "import_gate_manifests": counts["import_gate_manifests"],
            "support_status_pass": counts["support_status_pass"],
            "support_status_xfail": counts["support_status_xfail"],
            "support_status_blocker": counts["support_status_blocker"],
            "support_status_missing": counts["support_status_missing"],
            "registered_runtime_kits": counts["registered_runtime_kits"],
            "manifest_errors": counts["manifest_errors"],
            "register_errors": counts["register_errors"],
            "path_errors": counts["path_errors"],
            "blockers": counts["blockers"],
            "unowned_gap_count": counts["unowned_gap_count"],
        },
        evidence=payload["evidence_commands"],
        blockers=payload["blockers"][:show],
    )


def debugger_dimension(show: int) -> Dimension:
    code, payload = run_json(
        [sys.executable, str(DEBUGGER_READINESS), "--json", "--show", str(show)],
        accepted={0, EXIT_NOT_READY},
    )
    counts = payload["counts"]
    status = "green" if code == 0 and payload["ready"] else "red"
    return Dimension(
        id="debugger_introspection_profiling",
        title="Debugger, introspection, profiling, and tracing surfaces",
        status=status,
        owner_issue="#712",
        summary=(
            "debugger/introspection/profiling/tracing readiness is green"
            if status == "green"
            else (
                "debugger/introspection/profiling/tracing readiness is not replacement-ready: "
                f"{counts['fixtures']} fixtures, {counts['unmeasured']} unmeasured, "
                f"{counts['promotion_pending']} promotion-pending, "
                f"{counts['runtime_failure_debt']} runtime-debt, "
                f"{counts['missing_semantic_classes']} missing semantic classes"
            )
        ),
        counts={
            "fixtures": counts["fixtures"],
            "scopes": counts["scopes"],
            "target_libs": counts["target_libs"],
            "missing_target_libs": counts["missing_target_libs"],
            "semantic_classes": counts["semantic_classes"],
            "missing_semantic_classes": counts["missing_semantic_classes"],
            "parse_errors": counts["parse_errors"],
            "pass_candidate": counts["pass_candidate"],
            "promotion_pending": counts["promotion_pending"],
            "runtime_failure_debt": counts["runtime_failure_debt"],
            "sandbox_denied": counts["sandbox_denied"],
            "unsupported_platform": counts["unsupported_platform"],
            "metadata_error": counts["metadata_error"],
            "runtime_ok": counts["runtime_ok"],
            "runtime_fail": counts["runtime_fail"],
            "runtime_timeout": counts["runtime_timeout"],
            "runtime_crash": counts["runtime_crash"],
            "unmeasured": counts["unmeasured"],
            "unowned_gap_count": counts["unowned_gap_count"],
            "perf_pins": counts["perf_pins"],
            "malformed_perf_pins": counts["malformed_perf_pins"],
        },
        evidence=payload["evidence_commands"],
        blockers=payload["blockers"][:show],
    )


def concurrency_dimension(show: int) -> Dimension:
    code, payload = run_json(
        [sys.executable, str(CONCURRENCY_READINESS), "--json", "--show", str(show)],
        accepted={0, EXIT_NOT_READY},
    )
    counts = payload["counts"]
    status = "green" if code == 0 and payload["ready"] else "red"
    return Dimension(
        id="concurrency_free_threaded",
        title="Concurrency and free-threaded semantics",
        status=status,
        owner_issue="#713",
        summary=(
            "concurrency/free-threaded readiness is green"
            if status == "green"
            else (
                "concurrency/free-threaded readiness is not replacement-ready: "
                f"{counts['fixtures']} fixtures, {counts['unmeasured']} unmeasured, "
                f"{counts['promotion_pending']} promotion-pending, "
                f"{counts['runtime_failure_debt']} runtime-debt, "
                f"{counts['missing_semantic_classes']} missing semantic classes"
            )
        ),
        counts={
            "fixtures": counts["fixtures"],
            "scopes": counts["scopes"],
            "target_libs": counts["target_libs"],
            "missing_target_libs": counts["missing_target_libs"],
            "semantic_classes": counts["semantic_classes"],
            "missing_semantic_classes": counts["missing_semantic_classes"],
            "parse_errors": counts["parse_errors"],
            "pass_candidate": counts["pass_candidate"],
            "promotion_pending": counts["promotion_pending"],
            "runtime_failure_debt": counts["runtime_failure_debt"],
            "sandbox_denied": counts["sandbox_denied"],
            "unsupported_platform": counts["unsupported_platform"],
            "metadata_error": counts["metadata_error"],
            "runtime_ok": counts["runtime_ok"],
            "runtime_fail": counts["runtime_fail"],
            "runtime_timeout": counts["runtime_timeout"],
            "runtime_crash": counts["runtime_crash"],
            "unmeasured": counts["unmeasured"],
            "unowned_gap_count": counts["unowned_gap_count"],
            "perf_pins": counts["perf_pins"],
            "malformed_perf_pins": counts["malformed_perf_pins"],
        },
        evidence=payload["evidence_commands"],
        blockers=payload["blockers"][:show],
    )


def blocked_dimension(
    *,
    id: str,
    title: str,
    owner_issue: str,
    summary: str,
    evidence: list[str],
) -> Dimension:
    return Dimension(
        id=id,
        title=title,
        status="blocked",
        owner_issue=owner_issue,
        summary=summary,
        evidence=evidence,
        blockers=[
            {
                "kind": "readiness_dimension_not_integrated",
                "owner_issue": owner_issue,
                "reason": summary,
            }
        ],
    )


def dimensions(show: int, type_limit: int) -> list[Dimension]:
    return [
        denominator_dimension(show),
        promotion_dimension(show),
        strict_type_dimension(show, type_limit),
        perf_dimension(show),
        safety_dimension(show),
        platform_dimension(show),
        import_dimension(show),
        third_party_dimension(show),
        mambalibs_dimension(show),
        debugger_dimension(show),
        concurrency_dimension(show),
    ]


def build_report(show: int, type_limit: int = 0) -> dict[str, Any]:
    dims = dimensions(show, type_limit)
    counts = {
        "green": sum(1 for item in dims if item.status == "green"),
        "red": sum(1 for item in dims if item.status == "red"),
        "blocked": sum(1 for item in dims if item.status == "blocked"),
        "uncovered": sum(1 for item in dims if item.status == "uncovered"),
    }
    ready = counts["red"] == 0 and counts["blocked"] == 0 and counts["uncovered"] == 0
    return {
        "schema_version": 1,
        "profile": "replacement-readiness",
        "development_build_policy": "debug",
        "ready": ready,
        "status": "green" if ready else "red",
        "counts": counts,
        "dimensions": [item.to_json() for item in dims],
    }


def print_human(report: dict[str, Any]) -> None:
    print(f"replacement readiness: {report['status']}")
    print(f"  profile: {report['profile']}")
    print(f"  development build policy: {report['development_build_policy']}")
    counts = report["counts"]
    print(
        "  dimensions: "
        f"green={counts['green']} red={counts['red']} "
        f"blocked={counts['blocked']} uncovered={counts['uncovered']}"
    )
    for item in report["dimensions"]:
        print(f"- {item['status']}: {item['id']} ({item['owner_issue']})")
        print(f"  {item['summary']}")
        if item["counts"]:
            counts_text = ", ".join(
                f"{key}={value}" for key, value in sorted(item["counts"].items())
            )
            print(f"  counts: {counts_text}")
        for blocker in item["blockers"][:3]:
            label = blocker.get("path") or blocker.get("module") or blocker.get("kind")
            reason = blocker.get("reason") or blocker.get("kind")
            print(f"  blocker: {label} - {reason}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--show", type=int, default=10)
    parser.add_argument(
        "--type-limit",
        type=int,
        default=0,
        help="sample N strict-type fixtures for development validation; sampled reports stay red",
    )
    args = parser.parse_args(argv)

    report = build_report(args.show, args.type_limit)
    if args.json:
        print(json.dumps(report, sort_keys=True))
    else:
        print_human(report)
    return 0 if report["ready"] else EXIT_NOT_READY


if __name__ == "__main__":
    raise SystemExit(main())
