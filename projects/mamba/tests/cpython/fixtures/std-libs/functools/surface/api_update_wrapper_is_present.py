# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "api_update_wrapper_is_present"
# subject = "functools.update_wrapper"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""functools.update_wrapper: api_update_wrapper_is_present (surface)."""
import functools

assert hasattr(functools, "update_wrapper")
print("api_update_wrapper_is_present OK")
