# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "statistics_filename_and_cumulative"
# subject = "tracemalloc.Snapshot"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: statistics('filename') collapses lines per file and cumulative=True counts a frame at every traceback depth (b.py cumulative is 98 B, count 5)"""
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

# Group by filename collapses every line in a file (lineno reported as 0).
by_file = snap1.statistics("filename")
b_file = [s for s in by_file if str(s.traceback) == "b.py:0"][0]
assert b_file.size == 66, "b.py filename total"

# Cumulative aggregation counts a frame at every traceback depth.
cumulative = snap1.statistics("filename", cumulative=True)
b_cum = [s for s in cumulative if str(s.traceback) == "b.py:0"][0]
assert b_cum.size == 98, "b.py cumulative size"
assert b_cum.count == 5, "b.py cumulative count"

print("statistics_filename_and_cumulative OK")
