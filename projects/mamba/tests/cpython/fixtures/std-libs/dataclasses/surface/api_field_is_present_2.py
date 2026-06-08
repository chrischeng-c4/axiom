# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "surface"
# case = "api_field_is_present_2"
# subject = "dataclasses.field"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dataclasses.field: api_field_is_present_2 (surface)."""
import dataclasses

assert hasattr(dataclasses, "field")
print("api_field_is_present_2 OK")
