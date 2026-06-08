# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "compare_to_statistic_diff"
# subject = "tracemalloc.Snapshot"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: compare_to yields StatisticDiff entries with size/count deltas (a.py:5 grew by 5000 B, count +1)"""
import tracemalloc

RAW1 = [
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (1, 2, (("a.py", 5), ("b.py", 4)), 3),
    (2, 66, (("b.py", 1),), 1),
    (3, 7, (("<unknown>", 0),), 1),
]
RAW2 = [
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (2, 2, (("a.py", 5), ("b.py", 4)), 3),
    (2, 5000, (("a.py", 5), ("b.py", 4)), 3),
    (4, 400, (("c.py", 578),), 1),
]
snap1 = tracemalloc.Snapshot(RAW1, 2)
snap2 = tracemalloc.Snapshot(RAW2, 2)

# compare_to yields StatisticDiff entries with size/count deltas.
diff = snap2.compare_to(snap1, "lineno")
grew = diff[0]
assert str(grew.traceback) == "a.py:5", "biggest growth is a.py:5"
assert grew.size == 5002, "a.py:5 new size"
assert grew.size_diff == 5000, "a.py:5 size delta"
assert grew.count_diff == 1, "a.py:5 count delta"

print("compare_to_statistic_diff OK")
