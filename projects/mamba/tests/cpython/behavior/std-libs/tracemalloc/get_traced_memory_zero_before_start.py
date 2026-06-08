# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "get_traced_memory_zero_before_start"
# subject = "tracemalloc.get_traced_memory"
# kind = "semantic"
# xfail = "mamba tracemalloc is a GC-counter shim; get_traced_memory does not report the (0, 0) not-tracing contract (#1261 long-tail stub batch)"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
"""tracemalloc.get_traced_memory: get_traced_memory() returns (0, 0) before start() rather than raising"""
import tracemalloc

if tracemalloc.is_tracing():
    tracemalloc.stop()

assert tracemalloc.get_traced_memory() == (0, 0), "zero before start"

print("get_traced_memory_zero_before_start OK")
