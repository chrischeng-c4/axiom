# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "surface"
# case = "get_tracemalloc_memory_is_callable"
# subject = "tracemalloc.get_tracemalloc_memory"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tracemalloc.get_tracemalloc_memory: get_tracemalloc_memory_is_callable (surface)."""
import tracemalloc

assert callable(tracemalloc.get_tracemalloc_memory)
print("get_tracemalloc_memory_is_callable OK")
