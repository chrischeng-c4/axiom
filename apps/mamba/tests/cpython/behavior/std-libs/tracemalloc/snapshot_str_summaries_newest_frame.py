# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "snapshot_str_summaries_newest_frame"
# subject = "tracemalloc.Snapshot"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: str() of a trace, its traceback, and its newest frame all summarise the most-recent frame as 'b.py:4'"""
import tracemalloc

RAW = [
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
]
snap = tracemalloc.Snapshot(RAW, 2)
trace = snap.traces[0]

# str() of trace / traceback / frame all summarise the most-recent frame.
assert str(trace) == "b.py:4: 10 B", "trace str"
assert str(trace.traceback) == "b.py:4", "traceback str"
assert str(trace.traceback[0]) == "b.py:4", "frame str"

print("snapshot_str_summaries_newest_frame OK")
