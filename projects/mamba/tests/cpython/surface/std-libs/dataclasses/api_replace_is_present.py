# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "surface"
# case = "api_replace_is_present"
# subject = "dataclasses.replace"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dataclasses.replace: api_replace_is_present (surface)."""
import dataclasses

assert hasattr(dataclasses, "replace")
print("api_replace_is_present OK")
