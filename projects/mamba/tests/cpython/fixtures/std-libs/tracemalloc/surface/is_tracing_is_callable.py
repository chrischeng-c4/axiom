# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "surface"
# case = "is_tracing_is_callable"
# subject = "tracemalloc.is_tracing"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""tracemalloc.is_tracing: is_tracing_is_callable (surface)."""
import tracemalloc

assert callable(tracemalloc.is_tracing)
print("is_tracing_is_callable OK")
