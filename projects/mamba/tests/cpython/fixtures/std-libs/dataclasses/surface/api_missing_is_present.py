# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "surface"
# case = "api_missing_is_present"
# subject = "dataclasses.MISSING"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dataclasses.MISSING: api_missing_is_present (surface)."""
import dataclasses

assert hasattr(dataclasses, "MISSING")
print("api_missing_is_present OK")
