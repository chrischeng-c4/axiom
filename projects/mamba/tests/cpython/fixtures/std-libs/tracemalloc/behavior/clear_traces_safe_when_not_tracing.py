# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "clear_traces_safe_when_not_tracing"
# subject = "tracemalloc.clear_traces"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.clear_traces: clear_traces() is a safe no-op whether or not tracing is active and leaves is_tracing() False"""
import tracemalloc

if tracemalloc.is_tracing():
    tracemalloc.stop()

# clear_traces is callable whether or not tracing was ever started.
tracemalloc.clear_traces()
assert tracemalloc.is_tracing() is False, "not tracing after clear while stopped"

tracemalloc.start()
tracemalloc.clear_traces()
assert tracemalloc.is_tracing() is True, "still tracing after clear while started"
tracemalloc.stop()

print("clear_traces_safe_when_not_tracing OK")
