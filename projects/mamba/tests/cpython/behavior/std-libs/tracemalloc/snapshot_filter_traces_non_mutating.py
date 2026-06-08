# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "snapshot_filter_traces_non_mutating"
# subject = "tracemalloc.Snapshot"
# kind = "semantic"
# xfail = "mamba does not implement the tracemalloc.Snapshot class (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.Snapshot: filter_traces with an exclusive Filter drops matching traces into a new snapshot and leaves the original intact; a DomainFilter keeps only the requested domain"""
import tracemalloc

RAW = [
    (0, 10, (("a.py", 2), ("b.py", 4)), 3),
    (1, 2, (("a.py", 5), ("b.py", 4)), 3),
    (2, 66, (("b.py", 1),), 1),
    (3, 7, (("<unknown>", 0),), 1),
]
snap = tracemalloc.Snapshot(RAW, 2)

# filter_traces with an exclusive Filter drops every frame matching b.py.
excl = tracemalloc.Filter(False, "b.py")
filtered = snap.filter_traces((excl,))
assert len(filtered.traces) == 3, "exclusive filter drops b.py-only trace"
# The original snapshot is untouched (filter_traces is non-mutating).
assert len(snap.traces) == 4, "original snapshot intact"

# DomainFilter keeps only the requested domain.
dom = tracemalloc.DomainFilter(True, domain=3)
only3 = snap.filter_traces((dom,))
assert len(only3.traces) == 1, "domain filter keeps one"

print("snapshot_filter_traces_non_mutating OK")
