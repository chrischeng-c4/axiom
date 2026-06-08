# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "api_partialmethod_is_present"
# subject = "functools.partialmethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""functools.partialmethod: api_partialmethod_is_present (surface)."""
import functools

assert hasattr(functools, "partialmethod")
print("api_partialmethod_is_present OK")
