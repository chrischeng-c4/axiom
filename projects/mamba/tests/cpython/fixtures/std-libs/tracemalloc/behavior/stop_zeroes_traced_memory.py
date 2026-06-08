# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "stop_zeroes_traced_memory"
# subject = "tracemalloc.stop"
# kind = "semantic"
# xfail = "mamba tracemalloc is a GC-counter shim; stop does not zero the traced-memory counters (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.stop: after stop() get_traced_memory() returns (0, 0) and get_tracemalloc_memory() reports non-negative tracer overhead"""
import tracemalloc

if tracemalloc.is_tracing():
    tracemalloc.stop()

tracemalloc.start(1)
_blob = b"y" * 200000  # noqa: F841 - allocate something to trace

# get_tracemalloc_memory reports overhead used by the tracer itself.
overhead = tracemalloc.get_tracemalloc_memory()
assert overhead >= 0, "tracer overhead non-negative"

# stop disables tracing and zeroes the traced-memory counters.
tracemalloc.stop()
assert tracemalloc.is_tracing() is False, "not tracing after stop"
assert tracemalloc.get_traced_memory() == (0, 0), "counters zero after stop"

print("stop_zeroes_traced_memory OK")
