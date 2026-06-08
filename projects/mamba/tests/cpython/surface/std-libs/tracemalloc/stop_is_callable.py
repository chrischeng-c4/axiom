# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "surface"
# case = "stop_is_callable"
# subject = "tracemalloc.stop"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tracemalloc.stop: stop_is_callable (surface)."""
import tracemalloc

assert callable(tracemalloc.stop)
print("stop_is_callable OK")
