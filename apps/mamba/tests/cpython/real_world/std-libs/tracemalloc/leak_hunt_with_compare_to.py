# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "real_world"
# case = "leak_hunt_with_compare_to"
# subject = "tracemalloc.Snapshot"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: a memory-profiling workflow builds two synthetic snapshots and uses compare_to + statistics to surface the call site whose allocation grew the most, the way a leak hunt reports a top offender"""
import tracemalloc

# A leak hunt: take a baseline snapshot, run the suspect workload, take a
# second snapshot, then diff them to find the call site that grew the most.
# We build the two snapshots from raw traces so the scenario is deterministic
# (domain, size, traceback_frames, total_nframe). cache.py:42 is the leak.
baseline = tracemalloc.Snapshot(
    [
        (0, 1024, (("cache.py", 42), ("service.py", 10)), 2),
        (0, 256, (("router.py", 7), ("service.py", 10)), 2),
        (0, 64, (("util.py", 3),), 1),
    ],
    2,
)
after = tracemalloc.Snapshot(
    [
        # cache.py:42 ballooned across many more allocations (the leak).
        (0, 1024, (("cache.py", 42), ("service.py", 10)), 2),
        (0, 900000, (("cache.py", 42), ("service.py", 10)), 2),
        (0, 256, (("router.py", 7), ("service.py", 10)), 2),
        (0, 64, (("util.py", 3),), 1),
    ],
    2,
)

# Top offender by diff: compare_to sorts by descending size_diff.
diff = after.compare_to(baseline, "lineno")
top = diff[0]
assert str(top.traceback) == "cache.py:42", "leak surfaces at cache.py:42"
assert top.size_diff == 900000, "leak grew by the injected 900000 bytes"
assert top.count_diff == 1, "one extra allocation at the leak site"

# A simple report a profiler would print: top 3 growth sites, biggest first.
report = [(str(s.traceback), s.size_diff) for s in diff[:3]]
assert report[0] == ("cache.py:42", 900000), "report headline is the leak"
assert all(report[i][1] >= report[i + 1][1] for i in range(len(report) - 1)), (
    "report ordered by descending growth"
)

# The current (post-run) snapshot's own statistics agree the leak dominates.
by_line = after.statistics("lineno")
assert str(by_line[0].traceback) == "cache.py:42", "statistics agree on hotspot"
assert by_line[0].size == 901024, "hotspot total size = baseline + leak"

# compare_to is non-mutating: the baseline snapshot is untouched.
assert len(baseline.traces) == 3, "baseline snapshot intact after diff"

print("leak_hunt_with_compare_to OK")
