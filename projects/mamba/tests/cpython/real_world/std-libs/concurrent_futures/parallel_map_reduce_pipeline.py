# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures"
# dimension = "real_world"
# case = "parallel_map_reduce_pipeline"
# subject = "concurrent.futures.ThreadPoolExecutor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""concurrent.futures.ThreadPoolExecutor: a downstream batch job fans a CPU-light transform over a work list with ThreadPoolExecutor (submit + as_completed + map), aggregates the per-item results into a deterministic reduced total, and shows the pool drains cleanly on context exit"""
import concurrent.futures

# A deterministic CPU-light transform standing in for a per-record job.
def transform(n):
    return n * n + 1


work = list(range(64))
expected_total = sum(transform(n) for n in work)

# --- Stage 1: fan out with submit(), fan in with as_completed(), reduce. ----
with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
    pending = {pool.submit(transform, n): n for n in work}
    by_input = {}
    for fut in concurrent.futures.as_completed(pending, timeout=30):
        n = pending[fut]
        by_input[n] = fut.result()

assert len(by_input) == len(work), f"every item produced a result: {len(by_input)}"
submit_total = sum(by_input.values())
assert submit_total == expected_total, f"submit/as_completed reduce = {submit_total}, expected {expected_total}"
# Pool drained on context exit: a fresh executor is needed for stage 2.

# --- Stage 2: the same transform via Executor.map preserves input order. ----
with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
    mapped = list(pool.map(transform, work, timeout=30))

assert mapped == [transform(n) for n in work], "map preserves input order"
assert sum(mapped) == expected_total, "map reduce matches the submit reduce"

# --- Stage 3: both fan-out strategies agree item-for-item. ------------------
for n in work:
    assert by_input[n] == mapped[n], f"submit vs map disagree at {n}: {by_input[n]} != {mapped[n]}"

print("parallel_map_reduce_pipeline OK:", submit_total)
