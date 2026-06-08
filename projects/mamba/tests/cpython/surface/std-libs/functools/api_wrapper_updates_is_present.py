# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "api_wrapper_updates_is_present"
# subject = "functools.WRAPPER_UPDATES"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""functools.WRAPPER_UPDATES: api_wrapper_updates_is_present (surface)."""
import functools

assert hasattr(functools, "WRAPPER_UPDATES")
print("api_wrapper_updates_is_present OK")
