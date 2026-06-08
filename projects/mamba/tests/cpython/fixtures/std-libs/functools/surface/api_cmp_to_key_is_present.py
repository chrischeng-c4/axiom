# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "surface"
# case = "api_cmp_to_key_is_present"
# subject = "functools.cmp_to_key"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""functools.cmp_to_key: api_cmp_to_key_is_present (surface)."""
import functools

assert hasattr(functools, "cmp_to_key")
print("api_cmp_to_key_is_present OK")
