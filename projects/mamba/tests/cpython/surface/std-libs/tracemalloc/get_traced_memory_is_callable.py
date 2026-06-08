# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "surface"
# case = "get_traced_memory_is_callable"
# subject = "tracemalloc.get_traced_memory"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tracemalloc.get_traced_memory: get_traced_memory_is_callable (surface)."""
import tracemalloc

assert callable(tracemalloc.get_traced_memory)
print("get_traced_memory_is_callable OK")
