# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "surface"
# case = "get_object_traceback_is_callable"
# subject = "tracemalloc.get_object_traceback"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tracemalloc.get_object_traceback: get_object_traceback_is_callable (surface)."""
import tracemalloc

assert callable(tracemalloc.get_object_traceback)
print("get_object_traceback_is_callable OK")
