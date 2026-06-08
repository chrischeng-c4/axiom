# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "surface"
# case = "get_traceback_limit_is_callable"
# subject = "tracemalloc.get_traceback_limit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tracemalloc.get_traceback_limit: get_traceback_limit_is_callable (surface)."""
import tracemalloc

assert callable(tracemalloc.get_traceback_limit)
print("get_traceback_limit_is_callable OK")
