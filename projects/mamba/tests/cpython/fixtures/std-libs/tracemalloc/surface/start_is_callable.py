# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "surface"
# case = "start_is_callable"
# subject = "tracemalloc.start"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tracemalloc.start: start_is_callable (surface)."""
import tracemalloc

assert callable(tracemalloc.start)
print("start_is_callable OK")
