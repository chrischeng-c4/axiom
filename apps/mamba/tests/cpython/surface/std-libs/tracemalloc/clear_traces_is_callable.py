# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "surface"
# case = "clear_traces_is_callable"
# subject = "tracemalloc.clear_traces"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tracemalloc.clear_traces: clear_traces_is_callable (surface)."""
import tracemalloc

assert callable(tracemalloc.clear_traces)
print("clear_traces_is_callable OK")
