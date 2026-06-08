# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "api_have_threads_is_present"
# subject = "decimal.HAVE_THREADS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""decimal.HAVE_THREADS: api_have_threads_is_present (surface)."""
import decimal

assert hasattr(decimal, "HAVE_THREADS")
print("api_have_threads_is_present OK")
