# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "start_enables_tracing_and_sets_limit"
# subject = "tracemalloc.start"
# kind = "semantic"
# xfail = "mamba tracemalloc is a GC-counter shim; start/stop do not toggle is_tracing or record the traceback limit (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.start: start(1) makes is_tracing() True and get_traceback_limit() == 1; stop() makes is_tracing() False"""
import tracemalloc

if tracemalloc.is_tracing():
    tracemalloc.stop()

# start(nframe) enables tracing and sets the traceback depth.
tracemalloc.start(1)
assert tracemalloc.is_tracing() is True, "tracing after start"
assert tracemalloc.get_traceback_limit() == 1, "traceback limit"

# stop disables tracing.
tracemalloc.stop()
assert tracemalloc.is_tracing() is False, "not tracing after stop"

print("start_enables_tracing_and_sets_limit OK")
