# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "traced_memory_positive_peak_ge_current"
# subject = "tracemalloc.get_traced_memory"
# kind = "semantic"
# xfail = "mamba tracemalloc is a GC-counter shim; get_traced_memory does not track per-allocation current/peak sizes (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.get_traced_memory: while tracing a large allocation get_traced_memory() reports current > 0 with peak >= current, and reset_peak keeps peak >= current"""
import tracemalloc

if tracemalloc.is_tracing():
    tracemalloc.stop()

tracemalloc.start(1)
tracemalloc.clear_traces()
blob = b"x" * 200000  # noqa: F841 - kept alive to count toward traced memory

# get_traced_memory reports a non-trivial current size and peak >= current.
current, peak = tracemalloc.get_traced_memory()
assert current > 0, "current traced memory positive"
assert peak >= current, "peak at least current"

# reset_peak keeps peak >= current but does not lose live allocations.
tracemalloc.reset_peak()
cur2, peak2 = tracemalloc.get_traced_memory()
assert peak2 >= cur2, "peak still >= current after reset"

tracemalloc.stop()

print("traced_memory_positive_peak_ge_current OK")
