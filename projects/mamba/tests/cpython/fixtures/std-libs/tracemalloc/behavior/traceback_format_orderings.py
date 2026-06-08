# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "traceback_format_orderings"
# subject = "tracemalloc.Snapshot"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: Traceback.format() renders newest-frame-first by default, honours limit, and reverses under most_recent_first"""
import tracemalloc

RAW = [
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
]
snap = tracemalloc.Snapshot(RAW, 2)
trace = snap.traces[0]

# Traceback.format() renders newest-frame-first by default.
fmt = trace.traceback.format()
assert fmt == ['  File "b.py", line 4', '  File "a.py", line 2'], "format default"
assert trace.traceback.format(limit=1) == ['  File "a.py", line 2'], "format limit"
assert trace.traceback.format(most_recent_first=True) == [
    '  File "a.py", line 2',
    '  File "b.py", line 4',
], "format most_recent_first"

print("traceback_format_orderings OK")
