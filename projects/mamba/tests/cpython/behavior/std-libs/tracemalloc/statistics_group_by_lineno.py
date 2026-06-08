# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "statistics_group_by_lineno"
# subject = "tracemalloc.Snapshot"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: statistics('lineno') sorts groups by descending size and aggregates same-line traces (a.py:2 across 3 traces is 30 B, count 3)"""
import tracemalloc

RAW1 = [
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (1, 2, (("a.py", 5), ("b.py", 4)), 3),
    (2, 66, (("b.py", 1),), 1),
    (3, 7, (("<unknown>", 0),), 1),
]
snap1 = tracemalloc.Snapshot(RAW1, 2)

# Group by line number: results sorted by descending size.
by_line = snap1.statistics("lineno")
top = by_line[0]
assert str(top.traceback) == "b.py:1", "top group is b.py:1"
assert top.size == 66, "top group size"
assert top.count == 1, "top group count"
# Same line aggregated across traces: a.py:2 appears in 3 traces, 30 B.
a2 = [s for s in by_line if str(s.traceback) == "a.py:2"][0]
assert (a2.size, a2.count) == (30, 3), "a.py:2 aggregate"

print("statistics_group_by_lineno OK")
