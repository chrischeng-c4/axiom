# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "surface"
# case = "filter_is_callable"
# subject = "tracemalloc.Filter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tracemalloc.Filter: filter_is_callable (surface)."""
import tracemalloc

assert callable(tracemalloc.Filter)
print("filter_is_callable OK")
