# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "surface"
# case = "api_field_is_present"
# subject = "dataclasses.Field"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dataclasses.Field: api_field_is_present (surface)."""
import dataclasses

assert hasattr(dataclasses, "Field")
print("api_field_is_present OK")
