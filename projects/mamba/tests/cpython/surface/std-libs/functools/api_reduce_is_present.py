# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "api_reduce_is_present"
# subject = "functools.reduce"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""functools.reduce: api_reduce_is_present (surface)."""
import functools

assert hasattr(functools, "reduce")
print("api_reduce_is_present OK")
