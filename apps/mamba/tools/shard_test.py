#!/usr/bin/env python3
"""shard_test.py — Process-level Sharded Test Runner for Mamba integration tests (r3).

Bypasses JIT_LOCK process-global serialization by running K concurrent worker
processes that pull from a shared module-granular work-queue (dynamic LPT
scheduling), instead of a static hand-weighted bin-packing. Each worker keeps
grabbing the next queue item and invoking the pre-built test binary directly
(no `cargo test` wrapper) until the queue is drained, which self-balances
load across shards regardless of per-module cost variance.

File-collision-class flakiness (PID-named cwd scratch files colliding across
composition) is expected to be handled at the source by the harness
cwd-sandbox (tests/cpython_ported/harness.rs, #2529 r3) rather than by this
runner; the serial-retry triage below remains as a defense-in-depth net and
honest-accounting mechanism.

Usage:
    python3 tools/shard_test.py [--shards K] [--filter PREFIX] [--verbose]
"""

from __future__ import annotations

import argparse
import collections
import json
import math
import queue
import re
import shutil
import subprocess
import sys
import threading
import time
from pathlib import Path

TOOLS_DIR = Path(__file__).resolve().parent
MAMBA_DIR = TOOLS_DIR.parent
TARGET_SHARD_DIR = MAMBA_DIR / "target" / "shard"
MODULE_TIMES_PATH = TARGET_SHARD_DIR / "module_times.json"

# Auto-split threshold (seconds): a module whose *previous* measured wall
# exceeded this is split into N test-disjoint pieces this round (N sized so
# each piece's estimated wall lands near SPLIT_THRESHOLD_S), so the
# work-queue can schedule the pieces onto separate workers. Pieces are
# assigned by round-robin (index modulo N) over the *sorted* test-name list
# rather than contiguous blocks: slow tests within a module are often
# alphabetically clustered (e.g. a family of `test_gc*` names), and a
# contiguous midpoint split can concentrate that whole cluster into a single
# half, producing a badly imbalanced split (observed: a 276-test module's
# naive 2-way contiguous split landed 39.5s/213.6s instead of ~2x ~130s).
# Round-robin interleaving spreads any such cluster evenly across all N
# pieces regardless of where it falls alphabetically.
SPLIT_THRESHOLD_S = 60.0

# Modules with fewer than this many tests are batched together into a single
# queue item (deterministically, in LPT sort order) to amortize the
# per-invocation process-startup cost across many small modules instead of
# paying it once per tiny module.
BATCH_MIN_TESTS = 20
BATCH_TARGET_TESTS = 20


def cleanup_artifact_files() -> int:
    """Remove leftover @mamba_test_* artifact files from MAMBA_DIR. Returns count removed."""
    removed = 0
    for p in MAMBA_DIR.glob("@mamba_test_*"):
        try:
            if p.is_file() or p.is_symlink():
                p.unlink()
                removed += 1
            elif p.is_dir():
                shutil.rmtree(p)
                removed += 1
        except Exception:
            pass
    return removed


def parse_summary_line(content: str) -> tuple[int, int, int, float] | None:
    """Parse `test result: <status>. X passed; Y failed; Z ignored... finished in Ws` from test output."""
    m = re.search(
        r"test result:\s*(?:ok|FAILED)\.\s*(\d+)\s*passed;\s*(\d+)\s*failed;\s*(\d+)\s*ignored;.*?finished in\s*([\d\.]+)s",
        content,
    )
    if not m:
        return None
    passed = int(m.group(1))
    failed = int(m.group(2))
    ignored = int(m.group(3))
    wall_time = float(m.group(4))
    return (passed, failed, ignored, wall_time)


def extract_failed_test_paths(log_content: str) -> list[str]:
    """Extract test function paths from `failures:` block or `---- <test> stdout ----` lines."""
    failed_tests: list[str] = []
    in_failures_block = False
    for line in log_content.splitlines():
        line_s = line.strip()
        if line_s == "failures:":
            in_failures_block = True
            continue
        if in_failures_block:
            if not line_s or line_s.startswith("test result:"):
                in_failures_block = False
            elif line_s.startswith("cpython_ported::"):
                if line_s not in failed_tests:
                    failed_tests.append(line_s)
        elif line_s.startswith("---- cpython_ported::") and line_s.endswith(" stdout ----"):
            test_path = line_s[5:-12].strip()
            if test_path not in failed_tests:
                failed_tests.append(test_path)
    return failed_tests


def build_and_locate_binary(verbose: bool) -> str | None:
    """Pre-build the test binary once and return its path via cargo's own
    machine-readable build output (`--message-format=json`), avoiding any
    guesswork about the hashed binary filename. Returns None on build failure."""
    if verbose:
        print("[shard_runner] Pre-building test binary (`cargo test --no-run`)...")
    build_cmd = [
        "cargo",
        "test",
        "-p",
        "mamba",
        "--test",
        "cpython_ported_integration",
        "--no-run",
        "--message-format=json",
    ]
    proc = subprocess.run(build_cmd, cwd=MAMBA_DIR, capture_output=True, text=True)
    if proc.returncode != 0:
        print("Error: cargo test --no-run failed:", file=sys.stderr)
        print(proc.stderr, file=sys.stderr)
        return None

    binary_path = None
    for line in proc.stdout.splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue
        if msg.get("reason") != "compiler-artifact":
            continue
        target = msg.get("target", {})
        if target.get("name") != "cpython_ported_integration":
            continue
        if not msg.get("profile", {}).get("test"):
            continue
        executable = msg.get("executable")
        if executable:
            binary_path = executable

    if not binary_path:
        print("Error: could not locate cpython_ported_integration test binary in build output", file=sys.stderr)
        return None
    return binary_path


def list_tests(binary_path: str, filter_prefix: str | None) -> list[str]:
    list_proc = subprocess.run(
        [binary_path, "--list"], cwd=MAMBA_DIR, capture_output=True, text=True
    )
    if list_proc.returncode != 0:
        print("Error: test binary --list failed:", file=sys.stderr)
        print(list_proc.stderr, file=sys.stderr)
        return []

    test_paths = []
    for line in list_proc.stdout.splitlines():
        line_s = line.strip()
        if line_s.endswith(": test"):
            tp = line_s[:-6].strip()
            if filter_prefix:
                if filter_prefix in tp:
                    test_paths.append(tp)
            else:
                test_paths.append(tp)
    return test_paths


def load_prev_module_times() -> dict:
    try:
        with open(MODULE_TIMES_PATH, encoding="utf-8") as f:
            data = json.load(f)
        if isinstance(data, dict):
            return data
    except Exception:
        pass
    return {}


def build_work_queue(
    mod_test_paths: dict[str, list[str]], prev_times: dict
) -> list[dict]:
    """Build the ordered list of work items:

    - LPT heuristic: iterate modules sorted by (-test_count, name) descending
      so the biggest jobs enter the queue first (dynamic list scheduling: a
      plain FIFO queue drained by K free workers implements LPT online).
    - Auto-split: a module whose previous measured wall exceeded
      SPLIT_THRESHOLD_S is split into N test-disjoint pieces (N = ceil(prev
      wall / SPLIT_THRESHOLD_S), at least 2), each becoming its own queue
      item with an `--exact` test-name filter list, so the pieces can land
      on separate workers. Pieces are assigned round-robin over the sorted
      test-name list (not contiguous blocks) so an alphabetically-clustered
      family of slow tests doesn't concentrate into a single piece.
    - Batching: modules with fewer than BATCH_MIN_TESTS tests are combined
      (in the same deterministic sort order) into shared queue items so the
      per-invocation binary-startup cost isn't paid once per tiny module.
    """
    mods_sorted = sorted(mod_test_paths.items(), key=lambda kv: (-len(kv[1]), kv[0]))

    work_items: list[dict] = []
    pending_small: list[tuple[str, list[str]]] = []

    def flush_pending_small() -> None:
        if not pending_small:
            return
        total = sum(len(tps) for _, tps in pending_small)
        members = [m for m, _ in pending_small]
        name = f"batch[{members[0]}..{members[-1]}]+{len(members)}mods"
        filters = [f"{m}::" for m in members]
        work_items.append(
            {
                "name": name,
                "filters": filters,
                "exact": False,
                "tests": total,
                "kind": "batch",
                "members": members,
            }
        )
        pending_small.clear()

    for mod, tps in mods_sorted:
        cnt = len(tps)
        prev = prev_times.get(mod)
        if prev and isinstance(prev, dict) and prev.get("wall_s", 0.0) > SPLIT_THRESHOLD_S and cnt >= 2:
            prev_wall = prev.get("wall_s", 0.0)
            n_pieces = max(2, min(cnt, math.ceil(prev_wall / SPLIT_THRESHOLD_S)))
            sorted_tps = sorted(tps)
            pieces: list[list[str]] = [[] for _ in range(n_pieces)]
            for i, tp in enumerate(sorted_tps):
                pieces[i % n_pieces].append(tp)
            for i, piece in enumerate(pieces):
                if not piece:
                    continue
                work_items.append(
                    {
                        "name": f"{mod}::__split{i}__",
                        "filters": piece,
                        "exact": True,
                        "tests": len(piece),
                        "kind": "split",
                        "parent": mod,
                    }
                )
        elif cnt < BATCH_MIN_TESTS:
            pending_small.append((mod, tps))
            if sum(len(t) for _, t in pending_small) >= BATCH_TARGET_TESTS:
                flush_pending_small()
        else:
            work_items.append(
                {
                    "name": mod,
                    "filters": [f"{mod}::"],
                    "exact": False,
                    "tests": cnt,
                    "kind": "module",
                }
            )

    flush_pending_small()

    for i, item in enumerate(work_items):
        item["idx"] = i

    return work_items


def sanitize_for_filename(name: str) -> str:
    safe = re.sub(r"[^A-Za-z0-9_.-]+", "_", name)
    return safe[:120]


def run_shard_runner(
    shards: int | None = None,
    filter_prefix: str | None = None,
    verbose: bool = False,
) -> int:
    import os

    start_time = time.time()

    # Pre-cleanup leftover artifact files
    removed = cleanup_artifact_files()
    if verbose and removed:
        print(f"[shard_runner] Pre-cleaned {removed} leftover @mamba_test_* artifact(s).")

    if shards is None or shards <= 0:
        cpu_cnt = os.cpu_count() or 4
        shards = max(1, cpu_cnt - 2)

    # Step 1: Pre-build cargo test binary once, and locate its exact path via
    # cargo's own JSON build output (avoids re-invoking `cargo test` per item,
    # which cuts ~1.5s of cargo-wrapper overhead off every single work item).
    binary_path = build_and_locate_binary(verbose)
    if binary_path is None:
        return 1

    # Step 2: Enumerate test paths directly from the binary (`--list`).
    if verbose:
        print("[shard_runner] Enumerating tests via test binary `--list`...")
    test_paths = list_tests(binary_path, filter_prefix)

    if not test_paths:
        print(
            f"Error: No tests matched filter criteria (filter={filter_prefix!r})",
            file=sys.stderr,
        )
        return 1

    # Step 3: Group test paths by leaf module (module-atomic).
    mod_test_paths: dict[str, list[str]] = collections.defaultdict(list)
    for tp in test_paths:
        mod = tp.rsplit("::", 1)[0]
        mod_test_paths[mod].append(tp)

    # Step 4: Build the module-granular work queue (LPT order + auto-split +
    # small-module batching), seeded from the previous round's measured
    # per-module wall times (if any).
    prev_times = load_prev_module_times()
    work_items = build_work_queue(mod_test_paths, prev_times)

    actual_k = min(shards, len(work_items))
    TARGET_SHARD_DIR.mkdir(parents=True, exist_ok=True)

    if verbose:
        n_split = sum(1 for it in work_items if it["kind"] == "split")
        n_batch = sum(1 for it in work_items if it["kind"] == "batch")
        n_solo = sum(1 for it in work_items if it["kind"] == "module")
        print(
            f"[shard_runner] Work queue: {len(work_items)} items "
            f"({n_solo} solo modules, {n_batch} batches, {n_split} auto-split halves), "
            f"{len(test_paths)} tests, K={actual_k} workers."
        )

    # Step 5: K worker threads drain a shared FIFO queue (list-scheduling ==
    # dynamic LPT since items are queued largest-first).
    work_queue: "queue.Queue[dict]" = queue.Queue()
    for item in work_items:
        work_queue.put(item)

    results: list[dict] = []
    failed_test_candidates: list[str] = []
    results_lock = threading.Lock()
    worker_totals = [
        {"tests": 0, "passed": 0, "failed": 0, "ignored": 0, "wall": 0.0, "items": 0}
        for _ in range(actual_k)
    ]

    def worker_loop(worker_id: int) -> None:
        while True:
            try:
                item = work_queue.get_nowait()
            except queue.Empty:
                return
            log_file = (
                TARGET_SHARD_DIR
                / f"item_{item['idx']:04d}_{sanitize_for_filename(item['name'])}.log"
            )
            # --test-threads=1: JIT_LOCK (src/codegen/cranelift/jit.rs) is a
            # process-global mutex serializing all JIT compile+execute within
            # one process, so libtest's default internal thread pool (usually
            # num_cpus) buys this subprocess no real parallelism -- only one
            # of its threads can ever be doing JIT work at a time, the rest
            # just block on the mutex. Left at the default, K=8 concurrent
            # worker subprocesses each spinning up ~num_cpus internal threads
            # oversubscribes a 10-core box (~80 OS threads for 10 cores),
            # which adds scheduling jitter and was observed pushing
            # borderline-timing tests (e.g. set_methods::test_gc, which runs
            # solo in ~9.1-9.3s against a fixed 10s harness timeout) over
            # their deadline under composition. Pinning to 1 thread per
            # subprocess keeps total concurrency at exactly K (the intended
            # process-level parallelism axis) with no logical throughput
            # loss, since the mutex already serialized the useful work.
            cmd = [binary_path, "--test-threads=1"]
            if item["exact"]:
                cmd.append("--exact")
            cmd.extend(item["filters"])

            t0 = time.time()
            with open(log_file, "w", encoding="utf-8") as f_out:
                proc = subprocess.run(
                    cmd, cwd=MAMBA_DIR, stdout=f_out, stderr=subprocess.STDOUT
                )
            elapsed = time.time() - t0

            log_content = log_file.read_text(encoding="utf-8", errors="ignore")
            parsed = parse_summary_line(log_content)

            item_failed_candidates: list[str] = []
            if proc.returncode != 0 or parsed is None or parsed[1] > 0:
                item_failed_candidates = extract_failed_test_paths(log_content)

            with results_lock:
                results.append(
                    {
                        "item": item,
                        "worker_id": worker_id,
                        "exit_code": proc.returncode,
                        "parsed": parsed,
                        "elapsed": elapsed,
                        "log_path": log_file,
                    }
                )
                if item_failed_candidates:
                    failed_test_candidates.extend(item_failed_candidates)
                wt = worker_totals[worker_id]
                wt["items"] += 1
                wt["wall"] += elapsed
                if parsed:
                    p, fl, ig, _ = parsed
                    wt["tests"] += p + fl + ig
                    wt["passed"] += p
                    wt["failed"] += fl
                    wt["ignored"] += ig
                else:
                    wt["tests"] += item["tests"]

    threads = [
        threading.Thread(target=worker_loop, args=(w,), daemon=True) for w in range(actual_k)
    ]
    if verbose:
        print(f"[shard_runner] Launching {actual_k} worker threads over a {len(work_items)}-item queue...")
    for t in threads:
        t.start()
    for t in threads:
        t.join()

    total_runner_wall = time.time() - start_time

    # Step 6: Print per-worker table.
    print(f"\nWork-Queue Test Runner Results (K={actual_k}, items={len(work_items)}):")
    print(f"{'Worker':<8} {'Items':<7} {'Tests':<8} {'Passed':<8} {'Failed':<8} {'Ignored':<8} {'Worker Wall':<12}")
    print("-" * 75)

    tot_passed = tot_failed = tot_ignored = 0
    max_item_wall = 0.0
    max_item_name = ""
    has_run_failure = False

    for w, wt in enumerate(worker_totals):
        tot_passed += wt["passed"]
        tot_failed += wt["failed"]
        tot_ignored += wt["ignored"]
        print(
            f"Worker {w:<2} {wt['items']:<7} {wt['tests']:<8} {wt['passed']:<8} "
            f"{wt['failed']:<8} {wt['ignored']:<8} {wt['wall']:.1f}s"
        )

    print("-" * 75)

    module_times: dict[str, dict] = {}
    split_halves: dict[str, list[float]] = collections.defaultdict(list)

    for res in results:
        item = res["item"]
        parsed = res["parsed"]
        if parsed is None or res["exit_code"] != 0 or parsed[1] > 0:
            has_run_failure = True
        if res["elapsed"] > max_item_wall:
            max_item_wall = res["elapsed"]
            max_item_name = item["name"]

        if item["kind"] == "split":
            split_halves[item["parent"]].append(res["elapsed"])
        elif item["kind"] == "module":
            module_times[item["name"]] = {"wall_s": res["elapsed"], "tests": item["tests"]}
        elif item["kind"] == "batch":
            module_times[item["name"]] = {"wall_s": res["elapsed"], "tests": item["tests"]}

    for parent, walls in split_halves.items():
        parent_tests = len(mod_test_paths.get(parent, []))
        # sum, not max: the parent's recorded cost must be the module's TOTAL
        # work so next round's ceil(total/threshold) piece count is a fixed
        # point. max() made piece count oscillate (3 pieces -> fast pieces ->
        # 2 pieces -> slow pieces -> 3 ...), bouncing wall 160s<->182s.
        module_times[parent] = {"wall_s": sum(walls), "tests": parent_tests}

    if verbose:
        print(f"\nSlowest items (top 10):")
        for res in sorted(results, key=lambda r: -r["elapsed"])[:10]:
            print(f"  {res['elapsed']:7.1f}s  {res['item']['name']}")

    # Step 7: Serial Retry Triage for composition sensitivity if any failures occurred.
    comp_sensitive_tests: list[str] = []
    true_failed_tests: list[str] = []

    if failed_test_candidates:
        print("\n[shard_runner] Performing serial retry triage on failing test candidates...")
        for ft in sorted(set(failed_test_candidates)):
            retry_cmd = [binary_path, ft, "--exact"]
            retry_proc = subprocess.run(retry_cmd, cwd=MAMBA_DIR, capture_output=True, text=True)
            retry_parsed = parse_summary_line(retry_proc.stdout)
            if retry_proc.returncode == 0 and retry_parsed and retry_parsed[1] == 0:
                comp_sensitive_tests.append(ft)
            else:
                true_failed_tests.append(ft)

        if comp_sensitive_tests:
            print("\nComposition-Sensitive Tests Table (file-I/O fixture collisions):")
            print("-" * 75)
            for ft in comp_sensitive_tests:
                print(f"  [COMPOSITION-SENSITIVE] {ft}")
            print("-" * 75)

        if true_failed_tests:
            print("\nTrue Failure Tests Table:")
            print("-" * 75)
            for ft in true_failed_tests:
                print(f"  [TRUE-FAILURE] {ft}")
            print("-" * 75)

    # Step 8: Persist per-module wall times for next round's auto-split decisions.
    # MERGE into the existing file rather than overwrite: a filtered run (e.g.
    # `--filter harness::`) only measures the modules it ran, and clobbering
    # the full DB with that subset erases the timings the next full run needs
    # for auto-split (observed: a 1-item RG run reset the DB, the next full
    # run scheduled set_methods unsplit at 400s and exposed 19 new
    # composition-sensitive victims).
    try:
        merged: dict = {}
        if MODULE_TIMES_PATH.exists():
            try:
                with open(MODULE_TIMES_PATH, "r", encoding="utf-8") as f:
                    merged = json.load(f)
            except Exception:
                merged = {}
        merged.update(module_times)
        with open(MODULE_TIMES_PATH, "w", encoding="utf-8") as f:
            json.dump(merged, f, indent=2, sort_keys=True)
    except Exception as e:
        print(f"[shard_runner] Warning: failed to write {MODULE_TIMES_PATH}: {e}", file=sys.stderr)

    # Clean up any leftover artifact files created during the run (best-effort
    # defense; the harness cwd-sandbox should make this a no-op in practice).
    cleanup_artifact_files()

    reconciled = (tot_passed + tot_failed + tot_ignored) == len(test_paths)
    overall_status = "FAILED" if (has_run_failure or tot_failed > 0 or comp_sensitive_tests) else "ok"
    print(
        f"test result: {overall_status}. {tot_passed} passed; {tot_failed} failed; {tot_ignored} ignored; "
        f"finished in {total_runner_wall:.2f}s (max item wall: {max_item_wall:.1f}s [{max_item_name}], "
        f"K={actual_k}, items={len(work_items)}, reconciled={reconciled})\n"
    )

    return 1 if (has_run_failure or tot_failed > 0 or comp_sensitive_tests) else 0


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Process-level work-queue sharded test runner for cpython_ported_integration"
    )
    parser.add_argument("--shards", "-k", type=int, default=None, help="Number of worker threads/processes")
    parser.add_argument("--filter", "-f", type=str, default=None, help="Filter test path substring")
    parser.add_argument("-v", "--verbose", action="store_true", help="Print verbose details")

    args = parser.parse_args()
    sys.exit(run_shard_runner(shards=args.shards, filter_prefix=args.filter, verbose=args.verbose))


if __name__ == "__main__":
    main()
