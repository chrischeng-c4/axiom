# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "surface"
# case = "api_astuple_is_present"
# subject = "dataclasses.astuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dataclasses.astuple: api_astuple_is_present (surface)."""
import dataclasses

assert hasattr(dataclasses, "astuple")
print("api_astuple_is_present OK")
