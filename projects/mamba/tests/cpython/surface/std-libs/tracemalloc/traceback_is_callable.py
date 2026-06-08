# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "surface"
# case = "traceback_is_callable"
# subject = "tracemalloc.Traceback"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tracemalloc.Traceback: traceback_is_callable (surface)."""
import tracemalloc

assert callable(tracemalloc.Traceback)
print("traceback_is_callable OK")
