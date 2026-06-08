# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "traced_object_has_traceback_then_cleared"
# subject = "tracemalloc.get_object_traceback"
# kind = "semantic"
# xfail = "mamba tracemalloc is a GC-counter shim; get_object_traceback does not record per-allocation tracebacks (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.get_object_traceback: a freshly allocated object has a recorded traceback while tracing, and clear_traces() drops it back to None"""
import tracemalloc

if tracemalloc.is_tracing():
    tracemalloc.stop()

tracemalloc.start(1)

# A freshly allocated object has a recorded traceback.
tracemalloc.clear_traces()
blob = b"x" * 200000
tb = tracemalloc.get_object_traceback(blob)
assert tb is not None, "object has traceback while tracing"

# clear_traces forgets recorded allocations.
tracemalloc.clear_traces()
assert tracemalloc.get_object_traceback(blob) is None, "traceback cleared"

tracemalloc.stop()

print("traced_object_has_traceback_then_cleared OK")
