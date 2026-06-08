# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "api_total_ordering_is_present"
# subject = "functools.total_ordering"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""functools.total_ordering: api_total_ordering_is_present (surface)."""
import functools

assert hasattr(functools, "total_ordering")
print("api_total_ordering_is_present OK")
