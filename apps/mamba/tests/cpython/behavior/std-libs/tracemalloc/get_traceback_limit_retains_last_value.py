# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "get_traceback_limit_retains_last_value"
# subject = "tracemalloc.get_traceback_limit"
# kind = "semantic"
# xfail = "mamba tracemalloc is a GC-counter shim; start does not record the traceback limit, so get_traceback_limit does not reflect start(n) (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.get_traceback_limit: get_traceback_limit() defaults to 1 and after start(n)/stop retains the most recently configured limit n"""
import tracemalloc

if tracemalloc.is_tracing():
    tracemalloc.stop()

# Fresh in this process the limit defaults to 1.
assert tracemalloc.get_traceback_limit() == 1, "default limit is 1"

# start(n) sets the limit; it is observable while tracing.
tracemalloc.start(5)
assert tracemalloc.get_traceback_limit() == 5, "limit reflects start(5)"

# After stop the most recently configured limit is retained (not zeroed).
tracemalloc.stop()
assert tracemalloc.get_traceback_limit() == 5, "limit retained after stop"

print("get_traceback_limit_retains_last_value OK")
