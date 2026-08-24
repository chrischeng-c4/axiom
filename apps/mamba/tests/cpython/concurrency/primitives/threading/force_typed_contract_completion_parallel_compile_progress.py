import json
import os
import resource
import sys
import time
import hashlib
import subprocess
import concurrent.futures
import multiprocessing
from pathlib import Path


def drop_leading_comments(content: str) -> str:
    lines = content.splitlines(keepends=True)
    idx = 0
    while idx < len(lines) and lines[idx].startswith("# "):
        idx += 1
    return "".join(lines[idx:])


def _run_phase_worker(
    mamba_bin: str,
    fixture_list: list,
    work_list: list,
    num_workers: int,
    queue: multiprocessing.Queue,
):
    # Warmup phase inside the phase child process
    for fix_path in fixture_list:
        subprocess.run([mamba_bin, "check", fix_path], capture_output=True)

    def run_unit(path_str: str) -> int:
        res = subprocess.run(
            [mamba_bin, "check", path_str], capture_output=True
        )
        return res.returncode

    usage_start = resource.getrusage(resource.RUSAGE_CHILDREN)
    t_start = time.perf_counter()

    with concurrent.futures.ThreadPoolExecutor(
        max_workers=num_workers
    ) as executor:
        results = list(executor.map(run_unit, work_list))

    t_end = time.perf_counter()
    usage_end = resource.getrusage(resource.RUSAGE_CHILDREN)

    nonzero_exit_count = sum(1 for rc in results if rc != 0)
    wall_seconds = t_end - t_start
    cpu_seconds = (usage_end.ru_utime - usage_start.ru_utime) + (
        usage_end.ru_stime - usage_start.ru_stime
    )
    cpu_per_wall = (
        cpu_seconds / wall_seconds if wall_seconds > 0 else 0.0
    )
    peak_rss_bytes = usage_end.ru_maxrss

    row = {
        "workers": num_workers,
        "wall_seconds": wall_seconds,
        "cpu_seconds": cpu_seconds,
        "cpu_per_wall": cpu_per_wall,
        "peak_rss_bytes": peak_rss_bytes,
        "units_completed": len(results),
        "nonzero_exit_count": nonzero_exit_count,
    }
    queue.put(row)


def main():
    repo_root = Path.cwd()

    # 1. Read baseline provenance record
    prov_path = (
        repo_root
        / "apps/mamba/external-contracts/evidence/mamba-t1-force-typed-baseline-provenance.json"
    )
    if not prov_path.exists():
        sys.stderr.write(f"Provenance record not found: {prov_path}\n")
        sys.exit(1)

    with open(prov_path, "r", encoding="utf-8") as f:
        prov_data = json.load(f)

    baseline_sha = prov_data["baseline_sha"]
    current_sha = prov_data["current_sha"]
    work_item = prov_data.get("work_item", 2011)
    claim = prov_data.get("claim", "force-typed-contract-completion")

    # Assert expected sha values from contract specification
    expected_baseline_sha = "8eb4ccfe4456a6398bb109388a4af0c7c626c148"
    expected_current_sha = "03e8c5216126fa1fd2fed4447a207cb53ee62414"
    if baseline_sha != expected_baseline_sha:
        sys.stderr.write(
            f"Provenance baseline_sha mismatch: got {baseline_sha}, expected {expected_baseline_sha}\n"
        )
        sys.exit(1)
    if current_sha != expected_current_sha:
        sys.stderr.write(
            f"Provenance current_sha mismatch: got {current_sha}, expected {expected_current_sha}\n"
        )
        sys.exit(1)

    # 2. Read cases.jsonl to discover the 7 families and their expected template digests
    cases_path = (
        repo_root
        / "apps/mamba/tests/governance/gates/t1_implicit_any_ingress_matrix/cases.jsonl"
    )
    if not cases_path.exists():
        sys.stderr.write(f"Cases inventory file not found: {cases_path}\n")
        sys.exit(1)

    family_cases = {}
    with open(cases_path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            entry = json.loads(line)
            fixture_path_str = entry["fixture_or_probe_path"]
            family = Path(fixture_path_str).stem
            expected_digest = entry["paired_template_sha256"]
            family_cases[family] = expected_digest

    if len(family_cases) != 7:
        sys.stderr.write(
            f"Expected 7 families in cases.jsonl, found {len(family_cases)}\n"
        )
        sys.exit(1)

    # Recompute and verify paired template sha256 digests across all 7 families
    verified_template_digests = {}
    for family, expected_digest in family_cases.items():
        implicit_path = (
            repo_root
            / "apps/mamba/tests/cpython/_regression/core/typecheck/implicit_any_ingress"
            / f"{family}.py"
        )
        explicit_path = (
            repo_root
            / "apps/mamba/tests/cpython/_regression/core/typecheck/explicit_any_acceptance"
            / f"{family}.py"
        )

        if not implicit_path.exists() or not explicit_path.exists():
            sys.stderr.write(
                f"Fixture missing for family {family}: {implicit_path} or {explicit_path}\n"
            )
            sys.exit(1)

        with open(implicit_path, "r", encoding="utf-8") as f:
            implicit_content = f.read()

        with open(explicit_path, "r", encoding="utf-8") as f:
            explicit_content = f.read()

        implicit_norm = drop_leading_comments(implicit_content)
        explicit_norm = drop_leading_comments(explicit_content)

        # Apply explicit member normalization transform
        explicit_norm = explicit_norm.replace(": Any = ", " = ").replace(
            " -> Any:", ":"
        )

        if implicit_norm != explicit_norm:
            sys.stderr.write(
                f"Normalized fixture mismatch for family '{family}'\n"
            )
            sys.exit(1)

        computed_digest = hashlib.sha256(
            implicit_norm.encode("utf-8")
        ).hexdigest()
        if computed_digest != expected_digest:
            sys.stderr.write(
                f"Digest mismatch for family '{family}': computed {computed_digest}, expected {expected_digest}\n"
            )
            sys.exit(1)

        verified_template_digests[family] = computed_digest

    # 3. CPU Count Check
    logical_cpus = os.cpu_count() or 0
    if logical_cpus < 4:
        err_doc = {
            "probe_id": "force_typed_contract_completion_parallel_compile_progress",
            "claim": claim,
            "work_item": work_item,
            "error": "INSUFFICIENT_CPU",
            "logical_cpus": logical_cpus,
            "required_cpus": 4,
            "result": "fail",
        }
        print(json.dumps(err_doc, indent=2))
        sys.exit(1)

    # 4. Binary Check & Work List Construction
    mamba_bin = repo_root / "target/release/mamba"
    if not mamba_bin.exists():
        sys.stderr.write(f"Release binary not found: {mamba_bin}\n")
        sys.exit(1)

    fixture_list = []
    for family in family_cases.keys():
        fixture_list.append(
            f"apps/mamba/tests/cpython/_regression/core/typecheck/implicit_any_ingress/{family}.py"
        )
        fixture_list.append(
            f"apps/mamba/tests/cpython/_regression/core/typecheck/explicit_any_acceptance/{family}.py"
        )

    # 8 repetitions of the 14 fixtures = 112 work units
    repeats = 8
    work_list = fixture_list * repeats

    def execute_phase(num_workers: int):
        ctx = multiprocessing.get_context("spawn")
        queue = ctx.Queue()
        proc = ctx.Process(
            target=_run_phase_worker,
            args=(str(mamba_bin), fixture_list, work_list, num_workers, queue),
        )
        proc.start()
        row = queue.get()
        proc.join()
        return row

    # Phase 1: Serial (1 worker)
    serial_row = execute_phase(1)

    # Phase 2: Parallel (4 workers)
    parallel_row = execute_phase(4)

    # 5. Threshold Calculations & Assertions
    wall_speedup = (
        serial_row["wall_seconds"] / parallel_row["wall_seconds"]
        if parallel_row["wall_seconds"] > 0
        else 0.0
    )
    cpu_utilization = parallel_row["cpu_per_wall"]
    rss_budget_bytes = 1.25 * serial_row["peak_rss_bytes"] + 16777216

    wall_speedup_pass = wall_speedup >= 1.50
    cpu_utilization_pass = cpu_utilization >= 1.50
    rss_budget_pass = parallel_row["peak_rss_bytes"] <= rss_budget_bytes

    failed_thresholds = []
    if not wall_speedup_pass:
        failed_thresholds.append(
            f"wall_speedup ({wall_speedup:.3f} < 1.50)"
        )
    if not cpu_utilization_pass:
        failed_thresholds.append(
            f"cpu_utilization ({cpu_utilization:.3f} < 1.50)"
        )
    if not rss_budget_pass:
        failed_thresholds.append(
            f"peak_rss_bytes ({parallel_row['peak_rss_bytes']} > {rss_budget_bytes:.0f})"
        )

    overall_pass = len(failed_thresholds) == 0

    output_doc = {
        "probe_id": "force_typed_contract_completion_parallel_compile_progress",
        "claim": claim,
        "work_item": work_item,
        "baseline_sha": baseline_sha,
        "current_sha": current_sha,
        "platform": "aarch64-apple-darwin",
        "peak_rss_measurement": "max_single_child_rss_per_phase",
        "logical_cpus": logical_cpus,
        "paired_template_sha256": verified_template_digests,
        "family_count": len(verified_template_digests),
        "work_units": len(work_list),
        "rows": [serial_row, parallel_row],
        "wall_speedup": wall_speedup,
        "rss_budget_bytes": rss_budget_bytes,
        "wall_speedup_pass": wall_speedup_pass,
        "cpu_utilization_pass": cpu_utilization_pass,
        "rss_budget_pass": rss_budget_pass,
        "result": "pass" if overall_pass else "fail",
        "failed_thresholds": failed_thresholds,
    }

    print(json.dumps(output_doc, indent=2))

    if not overall_pass:
        sys.exit(1)


if __name__ == "__main__":
    main()
