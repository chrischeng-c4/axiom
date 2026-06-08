# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "api_singledispatch_is_present"
# subject = "functools.singledispatch"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""functools.singledispatch: api_singledispatch_is_present (surface)."""
import functools

assert hasattr(functools, "singledispatch")
print("api_singledispatch_is_present OK")
