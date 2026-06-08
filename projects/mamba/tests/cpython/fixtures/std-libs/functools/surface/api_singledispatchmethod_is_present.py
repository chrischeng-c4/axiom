# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "api_singledispatchmethod_is_present"
# subject = "functools.singledispatchmethod"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""functools.singledispatchmethod: api_singledispatchmethod_is_present (surface)."""
import functools

assert hasattr(functools, "singledispatchmethod")
print("api_singledispatchmethod_is_present OK")
