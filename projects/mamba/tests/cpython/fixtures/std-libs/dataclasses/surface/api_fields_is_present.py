# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "surface"
# case = "api_fields_is_present"
# subject = "dataclasses.fields"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dataclasses.fields: api_fields_is_present (surface)."""
import dataclasses

assert hasattr(dataclasses, "fields")
print("api_fields_is_present OK")
