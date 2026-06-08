# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "snapshot_trace_fields_and_order"
# subject = "tracemalloc.Snapshot"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: a Snapshot built from raw traces exposes traceback_limit, trace count, per-trace size, and a traceback indexed newest-frame-first"""
import tracemalloc

# Raw traces: (domain, size, traceback_frames, total_nframe). A Snapshot can be
# built directly without live tracing, which makes the post-processing API
# deterministic.
RAW = [
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (1, 2, (("a.py", 5), ("b.py", 4)), 3),
    (2, 66, (("b.py", 1),), 1),
    (3, 7, (("<unknown>", 0),), 1),
]
snap = tracemalloc.Snapshot(RAW, 2)

# traceback_limit and number of recorded traces.
assert snap.traceback_limit == 2, "traceback_limit"
assert len(snap.traces) == 4, "trace count"

# A single trace exposes size + a traceback indexed most-recent-first
# (index 0 is the innermost / most-recent frame).
trace = snap.traces[0]
assert trace.size == 10, "trace.size"
assert trace.traceback.total_nframe == 3, "total_nframe"
assert len(trace.traceback) == 2, "frames in traceback"
assert trace.traceback[0].filename == "b.py", "newest frame filename"
assert trace.traceback[0].lineno == 4, "newest frame lineno"
assert trace.traceback[1].filename == "a.py", "older frame filename"

# Slicing a Traces sequence returns a plain tuple of traces.
assert snap.traces[:2] == (snap.traces[0], snap.traces[1]), "traces slice"

print("snapshot_trace_fields_and_order OK")
