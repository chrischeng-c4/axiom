# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "api_cached_property_is_present"
# subject = "functools.cached_property"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""functools.cached_property: api_cached_property_is_present (surface)."""
import functools

assert hasattr(functools, "cached_property")
print("api_cached_property_is_present OK")
